use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::{env, fs};

use crate::ec::write_rgb;

const FPS_DELAY: u64 = 80; // ~12 FPS
const SMOOTHING: f32 = 0.7;

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

/// Capture dominant screen color using grim + ImageMagick (piped, no temp files).
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
        .args(["-", "-resize", "1x1!", "txt:-"])
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
                .args(["-", "-resize", "1x1!", "txt:-"])
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

pub fn ambient() {
    let (wayland_display, xdg_runtime_dir) = detect_wayland_env().unwrap_or_else(|| {
        eprintln!("Error: Could not detect Wayland display.");
        eprintln!("Try running with: sudo -E victuner rgb ambient");
        std::process::exit(1);
    });

    eprintln!(
        "Ambient mode: using {}/{}",
        xdg_runtime_dir, wayland_display
    );

    let mut prev = (0u8, 0u8, 0u8);

    loop {
        if let Some((r, g, b)) = get_screen_color(&wayland_display, &xdg_runtime_dir) {
            let (vr, vg, vb) = vibrant(r, g, b);

            let sr = (prev.0 as f32 * SMOOTHING + vr as f32 * (1.0 - SMOOTHING)) as u8;
            let sg = (prev.1 as f32 * SMOOTHING + vg as f32 * (1.0 - SMOOTHING)) as u8;
            let sb = (prev.2 as f32 * SMOOTHING + vb as f32 * (1.0 - SMOOTHING)) as u8;

            write_rgb(sr, sg, sb);
            prev = (sr, sg, sb);
        }

        thread::sleep(Duration::from_millis(FPS_DELAY));
    }
}

