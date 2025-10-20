use figlet_rs::FIGfont;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

fn test_font(name: &str, path: &str) -> bool {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let font = FIGfont::from_file(path).ok()?;
        font.convert("TEST").map(|f| f.to_string())
    }));

    matches!(result, Ok(Some(_)))
}

fn scan_figlet_font_dirs() -> Vec<String> {
    let mut dirs = Vec::new();
    for d in [
        "/opt/homebrew/share/figlet/fonts",
        "/usr/local/share/figlet",
        "/usr/share/figlet",
        "/usr/share/figlet/fonts",
    ] {
        if Path::new(d).is_dir() {
            dirs.push(d.to_string());
        }
    }
    dirs
}

fn discover_all_fonts() -> Vec<(String, String)> {
    let mut fonts = Vec::new();

    for dir in scan_figlet_font_dirs() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "flf" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let path_str = path.to_string_lossy().to_string();
                            fonts.push((stem.to_string(), path_str));
                        }
                    }
                }
            }
        }
    }

    fonts.sort_by(|a, b| a.0.cmp(&b.0));
    fonts
}

fn main() {
    // Set a custom panic hook to prevent panics from bubbling up
    std::panic::set_hook(Box::new(|_| {
        // Silent panic handling - we'll catch these with catch_unwind
    }));

    println!("\nDiscovering all FIGlet fonts...\n");
    let fonts = discover_all_fonts();
    println!("Found {} font files\n", fonts.len());

    println!("Testing fonts with figlet-rs:\n");
    let mut working = Vec::new();
    let mut broken = Vec::new();

    for (i, (name, path)) in fonts.iter().enumerate() {
        print!("\r[{}/{}] Testing {:30} ", i + 1, fonts.len(), name);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        if test_font(name, path) {
            working.push(name.clone());
        } else {
            broken.push(name.clone());
        }
    }

    // Restore default panic hook
    let _ = std::panic::take_hook();

    println!("\n\n=== RESULTS ===");
    println!("Working fonts: {} / {}", working.len(), fonts.len());
    println!("Broken fonts: {}\n", broken.len());

    println!("=== WORKING FONTS ===");
    for (i, name) in working.iter().enumerate() {
        if i > 0 && i % 10 == 0 {
            println!();
        }
        print!("{:20}", name);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }
    println!("\n");

    // Print Rust array format for easy copy-paste into code
    println!("=== FOR CODE (const SAFE_FONTS) ===");
    println!("const SAFE_FONTS: &[&str] = &[");
    for (i, name) in working.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        if i % 8 == 0 {
            print!("\n    ");
        }
        print!("\"{}\"", name);
    }
    println!("\n];");

    if !broken.is_empty() {
        println!("\n=== BROKEN FONTS ({}) ===", broken.len());
        for name in broken.iter() {
            println!("  - {}", name);
        }
    }
}
