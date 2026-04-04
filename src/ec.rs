use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub const EC_PATH: &str = "/sys/kernel/debug/ec/ec0/io";
pub const OFFSET: u64 = 8;

pub fn ensure_root() {
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("Run with sudo.");
        std::process::exit(1);
    }
}

pub fn ensure_ec_access() {
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

pub fn write_rgb(r: u8, g: u8, b: u8) {
    let mut f = OpenOptions::new()
        .write(true)
        .open(EC_PATH)
        .expect("Failed to open EC");

    f.seek(SeekFrom::Start(OFFSET)).unwrap();
    f.write_all(&[r, g, b]).unwrap();
}

pub fn read_rgb() {
    let mut f = OpenOptions::new().read(true).open(EC_PATH).unwrap();
    let mut buf = [0u8; 3];

    f.seek(SeekFrom::Start(OFFSET)).unwrap();
    f.read_exact(&mut buf).unwrap();

    println!("Current RGB: {} {} {}", buf[0], buf[1], buf[2]);
}
