use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::{env, fs};

use crate::ec::write_rgb;

const FPS_DELAY: u64 = 33; // ~30 FPS
const SMOOTHING: f32 = 0.45;

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

/// Convert RGB (0-255) to HSV (hue 0-360, sat 0-1, val 0-1).
fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

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

    let sat = if max == 0.0 { 0.0 } else { delta / max };

    (hue, sat, max)
}

/// Convert HSV (hue 0-360, sat 0-1, val 0-1) to RGB (0-255).
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h as u32 {
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

/// Boost color vibrance by increasing saturation and brightness in HSV space.
/// Returns HSV so the caller can smooth in HSV space.
fn vibrant(r: u8, g: u8, b: u8, level: f32) -> (f32, f32, f32) {
    let (h, s, v) = rgb_to_hsv(r, g, b);
    if level <= 0.0 {
        return (h, s, v);
    }

    let sat_boost = 1.0 + 0.6 * level;
    let val_boost = 1.0 + 0.2 * level;

    let s = (s * sat_boost).min(1.0);
    let v = (v * val_boost).min(1.0);

    (h, s, v)
}

pub fn ambient(vibrancy: f32) {
    let (wayland_display, xdg_runtime_dir) = detect_wayland_env().unwrap_or_else(|| {
        eprintln!("Error: Could not detect Wayland display.");
        eprintln!("Try running with: sudo -E victuner rgb ambient");
        std::process::exit(1);
    });

    eprintln!(
        "Ambient mode: display={}/{}, vibrancy={:.1}",
        xdg_runtime_dir, wayland_display, vibrancy
    );

    let mut prev_hsv = (0.0f32, 0.0f32, 0.0f32);

    loop {
        if let Some((r, g, b)) = get_screen_color(&wayland_display, &xdg_runtime_dir) {
            let (vh, vs, vv) = vibrant(r, g, b, vibrancy);

            // Circular hue interpolation (shortest arc on the 360° wheel)
            let mut dh = vh - prev_hsv.0;
            if dh > 180.0 {
                dh -= 360.0;
            }
            if dh < -180.0 { 
                dh += 360.0;
            }
            let sh = (prev_hsv.0 + dh * (1.0 - SMOOTHING) + 360.0) % 360.0;
            let ss = prev_hsv.1 * SMOOTHING + vs * (1.0 - SMOOTHING);
            let sv = prev_hsv.2 * SMOOTHING + vv * (1.0 - SMOOTHING);

            let (sr, sg, sb) = hsv_to_rgb(sh, ss, sv);
            write_rgb(sr, sg, sb);
            prev_hsv = (sh, ss, sv);
        }

        thread::sleep(Duration::from_millis(FPS_DELAY));
    }
}
