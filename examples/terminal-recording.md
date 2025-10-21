---
title: Terminal Recording Demo
author: Presenterm Examples
---

# Terminal Recording in Presentations

Showcase live terminal sessions directly in your slides using asciinema recordings.

<!-- end_slide -->

## What is Asciinema?

Asciinema is a lightweight, text-based terminal recording format that:

- Records terminal sessions with exact timing
- Produces small file sizes (just text and ANSI codes)
- Preserves colors and formatting
- Can be played back in real-time

<!-- end_slide -->

## Live Shell Session

Here's a recording of an actual shell session that waits for you to start it:

```asciinema +start:wait +play:once
recording-demo.cast
```

**Press space to play!**

<!-- end_slide -->

## Playback Controls

Control how recordings play with attributes:

**Auto-play with looping:**
```cast +start:auto +play:loop
recording-demo.cast
```

- `+start:wait` - Wait for keypress (default)
- `+start:auto` - Auto-start when slide appears
- `+play:once` - Play once and stop (default)
- `+play:loop` - Loop continuously

<!-- end_slide -->

## Creating Your Own Recordings

To create asciinema recordings:

1. Install asciinema: `brew install asciinema`
2. Record a session: `asciinema rec demo.cast`
3. Perform your commands
4. Press `Ctrl+D` to stop recording
5. Embed in your presentation with ` ```asciinema `

<!-- end_slide -->

## Use Cases

Perfect for:

- **Live coding demos** - Show actual terminal workflows
- **CLI tool tutorials** - Demonstrate command usage
- **DevOps workflows** - Display deployment processes
- **Debugging sessions** - Share troubleshooting steps

<!-- end_slide -->

## Benefits Over Static Code

Why use asciinema instead of code blocks?

✓ Shows real timing and interaction
✓ Captures actual command output
✓ Demonstrates real-world workflows
✓ More engaging than static text
✓ Smaller than video files

<!-- end_slide -->

## Best Practices

Tips for great terminal recordings:

1. Keep recordings short (30-60 seconds)
2. Use clear, readable terminal themes
3. Increase font size for visibility
4. Plan your commands ahead
5. Edit out mistakes (use `asciinema cut`)

<!-- end_slide -->

## Advanced: Animation Control

Banner and ASCII features also support animation:

```banner:slant
+animate:rainbow
DEMO
```

Combine with terminal recordings for dynamic presentations!

<!-- end_slide -->

## Summary

Asciinema recordings bring your terminal sessions to life:

- Easy to create and embed
- Small file size
- Perfect for technical presentations
- Supported in presenterm with `asciinema` or `cast` code blocks

<!-- end_slide -->

## Try It Yourself!

Create your first recording:

```bash
# Record a session
asciinema rec my-demo.cast

# Play it back
asciinema play my-demo.cast

# Embed in presenterm (use path relative to .md file)
# ```asciinema
# my-demo.cast
# ```
```

**Note**: The path in the code block should be relative to the markdown file's location, not your current working directory.

Happy presenting!
