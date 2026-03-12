use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const EC_PATH: &str = "/sys/kernel/debug/ec/ec0/io";
const OFFSET: u64 = 8;
const PID_FILE: &str = "/tmp/victus-rgb.pid";

fn ensure_root() {
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("Run with sudo.");
        std::process::exit(1);
    }
}

fn ensure_ec_access() {
    if !Path::new("/sys/kernel/debug").exists() {
        let _ = Command::new("mount")
            .args(["-t", "debugfs", "none", "/sys/kernel/debug"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    if !Path::new(EC_PATH).exists() {
        let _ = Command::new("modprobe")
            .args(["ec_sys", "write_support=1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn write_rgb(r: u8, g: u8, b: u8) {
    let mut f = OpenOptions::new()
        .write(true)
        .open(EC_PATH)
        .expect("Failed to open EC");

    f.seek(SeekFrom::Start(OFFSET)).unwrap();
    f.write_all(&[r, g, b]).unwrap();
}

fn read_rgb() {
    let mut f = OpenOptions::new().read(true).open(EC_PATH).unwrap();
    let mut buf = [0u8; 3];

    f.seek(SeekFrom::Start(OFFSET)).unwrap();
    f.read_exact(&mut buf).unwrap();

    println!("Current RGB: {} {} {}", buf[0], buf[1], buf[2]);
}

fn kill_previous() {
    if let Ok(mut f) = File::open(PID_FILE) {
        let mut pid = String::new();

        if f.read_to_string(&mut pid).is_ok() {
            let pid = pid.trim();

            if !pid.is_empty() {
                let _ = Command::new("kill")
                    .args(["-TERM", pid])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                thread::sleep(Duration::from_millis(100));

                let _ = Command::new("kill")
                    .args(["-KILL", pid])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
    }

    let _ = std::fs::remove_file(PID_FILE);
}

fn save_pid(pid: u32) {
    if let Ok(mut f) = File::create(PID_FILE) {
        let _ = write!(f, "{}", pid);
    }
}

fn spawn_background(args: Vec<String>) {
    kill_previous();

    let exe = env::current_exe().unwrap();

    let child = Command::new(exe)
        .args(&args)
        .arg("--worker")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn failed");

    save_pid(child.id());

    println!("Effect started in background.");

    std::process::exit(0);
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;

    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn rainbow() {
    let mut hue = 0.0;

    loop {
        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);

        write_rgb(r, g, b);

        hue += 0.003;

        if hue > 1.0 {
            hue = 0.0;
        }

        thread::sleep(Duration::from_millis(20));
    }
}

fn breathe(color: (u8, u8, u8)) {
    loop {
        for i in 0..100 {
            let v = i as f32 / 100.0;

            write_rgb(
                (color.0 as f32 * v) as u8,
                (color.1 as f32 * v) as u8,
                (color.2 as f32 * v) as u8,
            );

            thread::sleep(Duration::from_millis(15));
        }

        for i in (0..100).rev() {
            let v = i as f32 / 100.0;

            write_rgb(
                (color.0 as f32 * v) as u8,
                (color.1 as f32 * v) as u8,
                (color.2 as f32 * v) as u8,
            );

            thread::sleep(Duration::from_millis(15));
        }
    }
}

fn alternate(c1: (u8, u8, u8), c2: (u8, u8, u8)) {
    loop {
        write_rgb(c1.0, c1.1, c1.2);
        thread::sleep(Duration::from_millis(500));

        write_rgb(c2.0, c2.1, c2.2);
        thread::sleep(Duration::from_millis(500));
    }
}

fn fade(c1: (u8, u8, u8), c2: (u8, u8, u8)) {
    loop {
        for i in 0..100 {
            let t = i as f32 / 100.0;

            let r = (c1.0 as f32 + (c2.0 as f32 - c1.0 as f32) * t) as u8;
            let g = (c1.1 as f32 + (c2.1 as f32 - c1.1 as f32) * t) as u8;
            let b = (c1.2 as f32 + (c2.2 as f32 - c1.2 as f32) * t) as u8;

            write_rgb(r, g, b);

            thread::sleep(Duration::from_millis(20));
        }

        for i in (0..100).rev() {
            let t = i as f32 / 100.0;

            let r = (c1.0 as f32 + (c2.0 as f32 - c1.0 as f32) * t) as u8;
            let g = (c1.1 as f32 + (c2.1 as f32 - c1.1 as f32) * t) as u8;
            let b = (c1.2 as f32 + (c2.2 as f32 - c1.2 as f32) * t) as u8;

            write_rgb(r, g, b);

            thread::sleep(Duration::from_millis(20));
        }
    }
}

fn preset(name: &str) -> Option<(u8, u8, u8)> {
    match name {
        "red" => Some((255, 0, 0)),
        "green" => Some((0, 255, 0)),
        "blue" => Some((0, 0, 255)),
        "yellow" => Some((255, 255, 0)),
        "cyan" => Some((0, 255, 255)),
        "purple" => Some((255, 0, 255)),
        "neon-purple" => Some((100, 12, 223)),
        "white" => Some((255, 255, 255)),
        "off" => Some((0, 0, 0)),
        _ => None,
    }
}

fn main() {
    ensure_root();
    ensure_ec_access();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: victus-rgb <command>");
        return;
    }

    let worker = args.contains(&"--worker".to_string());

    match args[1].as_str() {
        "current" => read_rgb(),

        "stop" => kill_previous(),

        "rainbow" => {
            if !worker {
                spawn_background(vec!["rainbow".into()]);
            }
            rainbow();
        }

        "breathe" => {
            let c = preset(&args[2]).unwrap();

            if !worker {
                spawn_background(vec!["breathe".into(), args[2].clone()]);
            }

            breathe(c);
        }

        "alternate" => {
            let c1 = preset(&args[2]).unwrap();
            let c2 = preset(&args[3]).unwrap();

            if !worker {
                spawn_background(vec![
                    "alternate".into(),
                    args[2].clone(),
                    args[3].clone(),
                ]);
            }

            alternate(c1, c2);
        }

        "fade" => {
            let c1 = preset(&args[2]).unwrap();
            let c2 = preset(&args[3]).unwrap();

            if !worker {
                spawn_background(vec![
                    "fade".into(),
                    args[2].clone(),
                    args[3].clone(),
                ]);
            }

            fade(c1, c2);
        }

        color => {
            if let Some((r, g, b)) = preset(color) {
                kill_previous();
                write_rgb(r, g, b);
            }
        }
    }
}
