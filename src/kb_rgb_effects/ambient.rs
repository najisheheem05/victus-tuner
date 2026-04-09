use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::{env, fs};

use crate::ec::write_rgb;

const FPS_DELAY: u64 = 80; // ~12 FPS
const SMOOTHING: f32 = 0.3;

// ── Palette config ──────────────────────────────────────────────────────────

/// Default palette colors (used when no config file exists).
const DEFAULT_PALETTE: &[(u8, u8, u8, &str)] = &[
    (255,   0,   0, "Red"),
    (255, 128,   0, "Orange"),
    (255, 255,   0, "Yellow"),
    (128, 255,   0, "Yellow-green"),
    (  0, 255,   0, "Green"),
    (  0, 255, 128, "Green-cyan"),
    (  0, 255, 255, "Cyan"),
    (  0, 128, 255, "Sky blue"),
    (  0,   0, 255, "Blue"),
    (100,  12, 223, "Purple"),
    (255,   0, 255, "Magenta"),
    (255,   0, 128, "Pink"),
];

/// A single palette entry: RGB + human-readable label.
#[derive(Clone)]
struct PaletteEntry {
    r: u8,
    g: u8,
    b: u8,
    label: String,
}

/// Config file path: ~/.config/victuner/ambient_palette.conf
/// When running under sudo we look up the real user's home.
fn palette_config_path() -> PathBuf {
    // Try the real (non-root) user's home first
    let home = env::var("SUDO_USER")
        .ok()
        .and_then(|user| {
            let output = Command::new("getent")
                .args(["passwd", &user])
                .output()
                .ok()?;
            let line = String::from_utf8_lossy(&output.stdout).to_string();
            line.split(':').nth(5).map(|s| s.to_string())
        })
        .or_else(|| env::var("HOME").ok())
        .unwrap_or_else(|| "/root".to_string());

    PathBuf::from(home)
        .join(".config")
        .join("victuner")
        .join("ambient_palette.conf")
}

/// Parse a single line: `R,G,B # Label` or `R,G,B`
fn parse_palette_line(line: &str) -> Option<PaletteEntry> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let (rgb_part, label_part) = if let Some(idx) = line.find('#') {
        (&line[..idx], line[idx + 1..].trim())
    } else {
        (line, "")
    };

    let parts: Vec<&str> = rgb_part.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return None;
    }

    let r: u8 = parts[0].parse().ok()?;
    let g: u8 = parts[1].parse().ok()?;
    let b: u8 = parts[2].parse().ok()?;

    let label = if label_part.is_empty() {
        format!("({},{},{})", r, g, b)
    } else {
        label_part.to_string()
    };

    Some(PaletteEntry { r, g, b, label })
}

/// Load the palette from the config file, falling back to defaults.
fn load_palette() -> Vec<PaletteEntry> {
    let path = palette_config_path();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            let entries: Vec<PaletteEntry> = contents
                .lines()
                .filter_map(parse_palette_line)
                .collect();
            if !entries.is_empty() {
                return entries;
            }
        }
    }

    // Write defaults and return them
    let palette: Vec<PaletteEntry> = DEFAULT_PALETTE
        .iter()
        .map(|(r, g, b, label)| PaletteEntry {
            r: *r,
            g: *g,
            b: *b,
            label: label.to_string(),
        })
        .collect();
    let _ = save_palette(&palette);
    palette
}

/// Persist the palette to the config file.
fn save_palette(palette: &[PaletteEntry]) -> std::io::Result<()> {
    let path = palette_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents: String = palette
        .iter()
        .map(|e| format!("{},{},{} # {}", e.r, e.g, e.b, e.label))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&path, contents + "\n")
}

/// Find the nearest palette color using Euclidean distance in RGB space.
fn snap_to_palette(r: u8, g: u8, b: u8, palette: &[PaletteEntry]) -> (u8, u8, u8) {
    let mut best = (r, g, b);
    let mut best_dist = u32::MAX;

    for entry in palette {
        let dr = r as i32 - entry.r as i32;
        let dg = g as i32 - entry.g as i32;
        let db = b as i32 - entry.b as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;

        if dist < best_dist {
            best_dist = dist;
            best = (entry.r, entry.g, entry.b);
        }
    }

    best
}

// ── Palette management (CLI) ────────────────────────────────────────────────

/// `victuner rgb ambient-palette [list|add|remove|edit]`
pub fn ambient_palette(args: &[String]) {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("list");

    match sub {
        "list" => palette_list(),
        "add" => palette_add(&args[1..]),
        "remove" => palette_remove(&args[1..]),
        "edit" => palette_edit(&args[1..]),
        "reset" => palette_reset(),
        _ => {
            eprintln!("Unknown palette command: {}", sub);
            palette_usage();
        }
    }
}

fn palette_usage() {
    println!("Usage: victuner rgb ambient-palette <command>");
    println!();
    println!("Commands:");
    println!("  list                           Show all palette colors");
    println!("  add <R> <G> <B> [label]        Add a new color");
    println!("  remove <index>                 Remove color at index (1-based)");
    println!("  edit <index> <R> <G> <B> [label]  Edit color at index (1-based)");
    println!("  reset                          Reset to default palette");
}

fn palette_list() {
    let palette = load_palette();
    println!("Ambient palette ({} colors):", palette.len());
    println!("  Config: {}", palette_config_path().display());
    println!();
    for (i, e) in palette.iter().enumerate() {
        println!(
            "  {:>2}. ({:>3},{:>3},{:>3})  {}",
            i + 1,
            e.r,
            e.g,
            e.b,
            e.label
        );
    }
}

fn palette_add(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: victuner rgb ambient-palette add <R> <G> <B> [label]");
        return;
    }

    let r: u8 = match args[0].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid R value: {}", args[0]);
            return;
        }
    };
    let g: u8 = match args[1].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid G value: {}", args[1]);
            return;
        }
    };
    let b: u8 = match args[2].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid B value: {}", args[2]);
            return;
        }
    };

    let label = if args.len() > 3 {
        args[3..].join(" ")
    } else {
        format!("({},{},{})", r, g, b)
    };

    let mut palette = load_palette();
    palette.push(PaletteEntry { r, g, b, label: label.clone() });

    match save_palette(&palette) {
        Ok(_) => println!("Added ({},{},{}) \"{}\" → palette now has {} colors", r, g, b, label, palette.len()),
        Err(e) => eprintln!("Failed to save palette: {}", e),
    }
}

fn palette_remove(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: victuner rgb ambient-palette remove <index>");
        eprintln!("Use 'ambient-palette list' to see indices");
        return;
    }

    let idx: usize = match args[0].parse::<usize>() {
        Ok(v) if v >= 1 => v,
        _ => {
            eprintln!("Invalid index: {} (must be 1-based)", args[0]);
            return;
        }
    };

    let mut palette = load_palette();
    if idx > palette.len() {
        eprintln!("Index {} out of range (palette has {} colors)", idx, palette.len());
        return;
    }

    let removed = palette.remove(idx - 1);
    match save_palette(&palette) {
        Ok(_) => println!(
            "Removed #{}: ({},{},{}) \"{}\" → palette now has {} colors",
            idx, removed.r, removed.g, removed.b, removed.label, palette.len()
        ),
        Err(e) => eprintln!("Failed to save palette: {}", e),
    }
}

fn palette_edit(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Usage: victuner rgb ambient-palette edit <index> <R> <G> <B> [label]");
        return;
    }

    let idx: usize = match args[0].parse::<usize>() {
        Ok(v) if v >= 1 => v,
        _ => {
            eprintln!("Invalid index: {} (must be 1-based)", args[0]);
            return;
        }
    };

    let r: u8 = match args[1].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid R value: {}", args[1]);
            return;
        }
    };
    let g: u8 = match args[2].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid G value: {}", args[2]);
            return;
        }
    };
    let b: u8 = match args[3].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid B value: {}", args[3]);
            return;
        }
    };

    let mut palette = load_palette();
    if idx > palette.len() {
        eprintln!("Index {} out of range (palette has {} colors)", idx, palette.len());
        return;
    }

    let label = if args.len() > 4 {
        args[4..].join(" ")
    } else {
        palette[idx - 1].label.clone()
    };

    let old = &palette[idx - 1];
    println!(
        "Editing #{}: ({},{},{}) \"{}\" → ({},{},{}) \"{}\"",
        idx, old.r, old.g, old.b, old.label, r, g, b, label
    );

    palette[idx - 1] = PaletteEntry { r, g, b, label };

    match save_palette(&palette) {
        Ok(_) => println!("Saved."),
        Err(e) => eprintln!("Failed to save palette: {}", e),
    }
}

fn palette_reset() {
    let palette: Vec<PaletteEntry> = DEFAULT_PALETTE
        .iter()
        .map(|(r, g, b, label)| PaletteEntry {
            r: *r,
            g: *g,
            b: *b,
            label: label.to_string(),
        })
        .collect();

    match save_palette(&palette) {
        Ok(_) => {
            println!("Palette reset to defaults ({} colors).", palette.len());
            palette_list();
        }
        Err(e) => eprintln!("Failed to save palette: {}", e),
    }
}

// ── Screen capture & color processing ───────────────────────────────────────

/// Parse "srgb(r,g,b)" from ImageMagick output.
/// Handles both integer values (IMv6: `srgb(42,39,46)`)
/// and percentage values (IMv7: `srgb(16.39%,15.43%,17.88%)`).
fn parse_color(output: &str) -> Option<(u8, u8, u8)> {
    let start = output.find("srgb(")?;
    let end = output[start..].find(')')?;
    let values = &output[start + 5..start + end];

    let parts: Vec<u8> = values
        .split(',')
        .filter_map(|x| {
            let s = x.trim();
            if let Some(pct) = s.strip_suffix('%') {
                // Percentage value (IMv7): convert from 0-100% to 0-255
                let pct: f32 = pct.parse().ok()?;
                Some((pct * 255.0 / 100.0).round() as u8)
            } else {
                // Integer value (IMv6): use directly
                s.parse().ok()
            }
        })
        .collect();

    if parts.len() == 3 {
        Some((parts[0], parts[1], parts[2]))
    } else {
        None
    }
}

/// Detect the Wayland runtime dir and display name.
/// When run under sudo, env vars are stripped, so we probe
/// the real user's XDG_RUNTIME_DIR for a wayland socket.
fn detect_wayland_env() -> Option<(String, String)> {
    // If vars are already set (e.g. `sudo -E`), use them directly
    if let (Ok(display), Ok(runtime)) =
        (env::var("WAYLAND_DISPLAY"), env::var("XDG_RUNTIME_DIR"))
    {
        let socket = format!("{}/{}", runtime, display);
        if Path::new(&socket).exists() {
            return Some((display, runtime));
        }
    }

    // Probe common runtime dirs for the real (non-root) user
    let uid = env::var("SUDO_UID")
        .ok()
        .or_else(|| env::var("PKEXEC_UID").ok())
        .unwrap_or_else(|| "1000".to_string());

    let runtime_dir = format!("/run/user/{}", uid);

    if let Ok(entries) = fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("wayland-") && !name.ends_with(".lock") {
                return Some((name, runtime_dir));
            }
        }
    }

    None
}

/// Capture most dominant screen color using grim + ImageMagick color quantization (piped, no temp files).
fn get_screen_color(wayland_display: &str, xdg_runtime_dir: &str) -> Option<(u8, u8, u8)> {
    let grim = Command::new("grim")
        .arg("-")
        .env("WAYLAND_DISPLAY", wayland_display)
        .env("XDG_RUNTIME_DIR", xdg_runtime_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    // Try "magick" (IMv7) first, fall back to "convert" (IMv6)
    let grim_stdout = grim.stdout?;

    let child = Command::new("magick")
        .args(["-", "-resize", "100x100!", "-colors", "1", "-depth", "8", "txt:-"])
        .stdin(grim_stdout)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .or_else(|_| {
            // magick not found, retry pipeline with convert
            let grim2 = Command::new("grim")
                .arg("-")
                .env("WAYLAND_DISPLAY", wayland_display)
                .env("XDG_RUNTIME_DIR", xdg_runtime_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?;
            Command::new("convert")
                .args(["-", "-resize", "100x100!", "-colors", "1", "-depth", "8", "txt:-"])
                .stdin(grim2.stdout.unwrap())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
        })
        .ok()?;

    let output = child.wait_with_output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);

    parse_color(&text)
}

/// Boost color vibrance by increasing saturation and brightness in HSV space.
/// This makes muted screen colors appear vivid on the keyboard.
fn vibrant(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    const SAT_BOOST: f32 = 1.6;
    const VAL_BOOST: f32 = 1.2;

    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    // Hue (0-360)
    let hue = if delta == 0.0 {
        0.0
    } else if max == rf {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if max == gf {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };
    let hue = if hue < 0.0 { hue + 360.0 } else { hue };

    // Boost saturation and value, clamped to [0, 1]
    let sat = if max == 0.0 { 0.0 } else { (delta / max) * SAT_BOOST };
    let sat = sat.min(1.0);
    let val = (max * VAL_BOOST).min(1.0);

    // HSV → RGB
    let c = val * sat;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = val - c;

    let (r1, g1, b1) = match hue as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

// ── Main ambient loop ───────────────────────────────────────────────────────

pub fn ambient() {
    let (wayland_display, xdg_runtime_dir) = detect_wayland_env().unwrap_or_else(|| {
        eprintln!("Error: Could not detect Wayland display.");
        eprintln!("Try running with: sudo -E victuner rgb ambient");
        std::process::exit(1);
    });

    let palette = load_palette();

    eprintln!(
        "Ambient mode: using {}/{} ({} palette colors)",
        xdg_runtime_dir, wayland_display, palette.len()
    );

    let mut prev = (0u8, 0u8, 0u8);

    loop {
        if let Some((r, g, b)) = get_screen_color(&wayland_display, &xdg_runtime_dir) {
            let (vr, vg, vb) = vibrant(r, g, b);

            // Snap to the nearest palette color
            let (pr, pg, pb) = snap_to_palette(vr, vg, vb, &palette);

            let sr = (prev.0 as f32 * SMOOTHING + pr as f32 * (1.0 - SMOOTHING)) as u8;
            let sg = (prev.1 as f32 * SMOOTHING + pg as f32 * (1.0 - SMOOTHING)) as u8;
            let sb = (prev.2 as f32 * SMOOTHING + pb as f32 * (1.0 - SMOOTHING)) as u8;

            write_rgb(sr, sg, sb);
            prev = (sr, sg, sb);
        }

        thread::sleep(Duration::from_millis(FPS_DELAY));
    }
}
