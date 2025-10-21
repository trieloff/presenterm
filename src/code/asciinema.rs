use crate::markdown::text::{WeightedLine, WeightedText};
use crate::markdown::text_style::TextStyle;
use crate::render::operation::{AsRenderOperations, BlockLine, Pollable, PollableState, RenderAsync, RenderAsyncStartPolicy, RenderOperation};
use crate::render::properties::WindowSize;
use crate::theme::Alignment;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use vte::{Parser, Perform};

/// A single frame event in an asciinema recording
#[derive(Debug, Clone, Deserialize)]
struct CastEvent {
    /// Timestamp in seconds
    #[serde(rename = "0")]
    time: f64,
    /// Event type (usually "o" for output)
    #[serde(rename = "1")]
    event_type: String,
    /// Terminal output data
    #[serde(rename = "2")]
    data: String,
}

/// Asciinema cast file header
#[derive(Debug, Deserialize)]
struct CastHeader {
    version: u32,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    timestamp: Option<u64>,
}

/// A parsed asciinema recording
#[derive(Debug, Clone)]
pub(crate) struct AsciinemaRecording {
    /// Recording events
    events: Vec<CastEvent>,
    /// Terminal width from header
    width: u32,
    /// Terminal height from header
    height: u32,
}

/// Errors that can occur when parsing asciinema recordings
#[derive(thiserror::Error, Debug)]
pub enum AsciinemaError {
    #[error("failed to parse cast file: {0}")]
    ParseError(String),

    #[error("invalid cast file format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl AsciinemaRecording {
    /// Parse an asciinema cast file from its contents
    pub(crate) fn from_cast(content: &str) -> Result<Self, AsciinemaError> {
        let mut lines = content.lines();

        // First line should be the header
        let header_line = lines.next()
            .ok_or_else(|| AsciinemaError::InvalidFormat("empty cast file".to_string()))?;

        let header: CastHeader = serde_json::from_str(header_line)
            .map_err(|e| AsciinemaError::ParseError(format!("invalid header: {}", e)))?;

        if header.version != 2 {
            return Err(AsciinemaError::InvalidFormat(format!(
                "unsupported cast version: {}",
                header.version
            )));
        }

        let width = header.width.unwrap_or(80);
        let height = header.height.unwrap_or(24);

        // Parse events
        let mut events = Vec::new();
        for (idx, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let event: CastEvent = serde_json::from_str(line)
                .map_err(|e| AsciinemaError::ParseError(format!("line {}: {}", idx + 2, e)))?;

            // Only process output events
            if event.event_type == "o" {
                events.push(event);
            }
        }

        Ok(Self {
            events,
            width,
            height,
        })
    }

    /// Get the total duration of the recording in seconds
    pub(crate) fn duration(&self) -> f64 {
        self.events.last().map(|e| e.time).unwrap_or(0.0)
    }

    /// Get the terminal width
    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    /// Get the frame at a specific timestamp
    fn get_frame_at(&self, timestamp: f64) -> String {
        let mut output = String::new();

        // Collect all events up to this timestamp
        for event in &self.events {
            if event.time <= timestamp {
                output.push_str(&event.data);
            } else {
                break;
            }
        }

        output
    }
}

/// VTE performer that builds terminal screen buffer
struct TerminalScreen {
    /// Current screen buffer (rows)
    lines: Vec<String>,
    /// Current cursor position (row, col)
    cursor: (usize, usize),
    /// Terminal dimensions
    width: usize,
    height: usize,
}

impl TerminalScreen {
    fn new(width: usize, height: usize) -> Self {
        let lines = vec![String::new(); height];
        Self {
            lines,
            cursor: (0, 0),
            width,
            height,
        }
    }

    fn put_char(&mut self, c: char) {
        let (row, col) = self.cursor;
        if row >= self.height {
            return;
        }

        // Ensure the line is long enough
        while self.lines[row].len() <= col {
            self.lines[row].push(' ');
        }

        // Replace character at cursor position
        if self.lines[row].chars().nth(col).is_some() {
            let before: String = self.lines[row].chars().take(col).collect();
            let after: String = self.lines[row].chars().skip(col + 1).collect();
            self.lines[row] = format!("{}{}{}", before, c, after);
        }

        // Move cursor forward
        self.cursor.1 = (self.cursor.1 + 1).min(self.width - 1);
    }

    fn get_lines(&self) -> Vec<String> {
        self.lines.clone()
    }
}

impl Perform for TerminalScreen {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                // Line feed
                self.cursor.0 = (self.cursor.0 + 1).min(self.height - 1);
            }
            b'\r' => {
                // Carriage return
                self.cursor.1 = 0;
            }
            b'\x08' => {
                // Backspace
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn csi_dispatch(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

/// Asciinema player that renders recording frames
#[derive(Debug)]
pub(crate) struct AsciinemaPlayer {
    /// The recording to play
    recording: AsciinemaRecording,
    /// Block length for alignment
    block_length: u16,
    /// Alignment settings
    alignment: Alignment,
    /// Font size
    font_size: u8,
    /// Playback state
    state: Arc<Mutex<PlaybackState>>,
    /// Whether to loop the animation
    loop_playback: bool,
    /// Speed multiplier (1.0 = normal speed)
    speed: f32,
    /// Start policy for playback
    start_policy: RenderAsyncStartPolicy,
}

#[derive(Debug)]
struct PlaybackState {
    /// Playback start time
    start_time: Option<Instant>,
    /// Current playback time in seconds
    current_time: f64,
    /// Whether playback has completed
    completed: bool,
}

impl AsciinemaPlayer {
    pub(crate) fn new(
        recording: AsciinemaRecording,
        block_length: u16,
        alignment: Alignment,
        font_size: u8,
        loop_playback: bool,
        speed: f32,
        start_policy: RenderAsyncStartPolicy,
    ) -> Self {
        Self {
            recording,
            block_length,
            alignment,
            font_size,
            state: Arc::new(Mutex::new(PlaybackState {
                start_time: None,
                current_time: 0.0,
                completed: false,
            })),
            loop_playback,
            speed: speed.max(0.1), // Minimum speed to avoid division by zero
            start_policy,
        }
    }

    fn render_frame(&self, timestamp: f64) -> Vec<RenderOperation> {
        let raw_output = self.recording.get_frame_at(timestamp);

        // Parse the terminal output using VTE
        let mut screen = TerminalScreen::new(
            self.recording.width as usize,
            self.recording.height as usize,
        );
        let mut parser = Parser::new();

        let bytes: Vec<u8> = raw_output.bytes().collect();
        parser.advance(&mut screen, &bytes);

        let lines = screen.get_lines();
        let text_style = TextStyle::default().size(self.font_size);

        let mut operations = Vec::new();

        // Add top border
        let border_char = "─";
        let border_text = border_char.repeat(self.recording.width as usize);
        let border_line = WeightedLine::from(vec![
            crate::markdown::elements::Text::new(&format!("┌{}┐", border_text), text_style)
        ]);
        operations.push(RenderOperation::RenderBlockLine(BlockLine {
            prefix: WeightedText::from(""),
            right_padding_length: 0,
            repeat_prefix_on_wrap: false,
            text: border_line,
            block_length: self.block_length,
            alignment: self.alignment,
            block_color: None,
        }));
        operations.push(RenderOperation::RenderLineBreak);

        // Render terminal content with side borders
        for line in lines {
            // Create weighted line from the terminal output with side borders
            let framed_line = format!("│{}│", line);
            let weighted_line = WeightedLine::from(vec![
                crate::markdown::elements::Text::new(&framed_line, text_style)
            ]);

            operations.push(RenderOperation::RenderBlockLine(BlockLine {
                prefix: WeightedText::from(""),
                right_padding_length: 0,
                repeat_prefix_on_wrap: false,
                text: weighted_line,
                block_length: self.block_length,
                alignment: self.alignment,
                block_color: None,
            }));
            operations.push(RenderOperation::RenderLineBreak);
        }

        // Add bottom border
        let bottom_border = WeightedLine::from(vec![
            crate::markdown::elements::Text::new(&format!("└{}┘", border_text), text_style)
        ]);
        operations.push(RenderOperation::RenderBlockLine(BlockLine {
            prefix: WeightedText::from(""),
            right_padding_length: 0,
            repeat_prefix_on_wrap: false,
            text: bottom_border,
            block_length: self.block_length,
            alignment: self.alignment,
            block_color: None,
        }));
        operations.push(RenderOperation::RenderLineBreak);

        operations
    }
}

impl AsRenderOperations for AsciinemaPlayer {
    fn as_render_operations(&self, _: &WindowSize) -> Vec<RenderOperation> {
        let state = self.state.lock().unwrap();
        self.render_frame(state.current_time)
    }
}

impl RenderAsync for AsciinemaPlayer {
    fn pollable(&self) -> Box<dyn Pollable> {
        Box::new(AsciinemaPlaybackPollable {
            state: self.state.clone(),
            duration: self.recording.duration(),
            loop_playback: self.loop_playback,
            speed: self.speed,
        })
    }

    fn start_policy(&self) -> RenderAsyncStartPolicy {
        self.start_policy
    }
}

struct AsciinemaPlaybackPollable {
    state: Arc<Mutex<PlaybackState>>,
    duration: f64,
    loop_playback: bool,
    speed: f32,
}

impl Pollable for AsciinemaPlaybackPollable {
    fn poll(&mut self) -> PollableState {
        let mut state = self.state.lock().unwrap();

        // Initialize start time on first poll
        if state.start_time.is_none() {
            state.start_time = Some(Instant::now());
            state.current_time = 0.0;
            return PollableState::Modified;
        }

        let elapsed = state.start_time.unwrap().elapsed().as_secs_f64();
        let playback_time = elapsed * self.speed as f64;

        if playback_time >= self.duration {
            if self.loop_playback {
                // Loop: reset to beginning
                state.start_time = Some(Instant::now());
                state.current_time = 0.0;
                return PollableState::Modified;
            } else {
                // Complete: freeze on last frame
                if !state.completed {
                    state.current_time = self.duration;
                    state.completed = true;
                    return PollableState::Done;
                }
                return PollableState::Unmodified;
            }
        }

        state.current_time = playback_time;
        PollableState::Modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_cast() {
        let cast_content = r#"{"version": 2, "width": 80, "height": 24}
[0.0, "o", "Hello"]
[1.0, "o", " World"]
"#;
        let recording = AsciinemaRecording::from_cast(cast_content).expect("failed to parse");
        assert_eq!(recording.width, 80);
        assert_eq!(recording.height, 24);
        assert_eq!(recording.events.len(), 2);
        assert_eq!(recording.duration(), 1.0);
    }
}
