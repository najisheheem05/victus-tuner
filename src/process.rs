use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const PID_FILE: &str = "/tmp/victus-rgb.pid";

pub fn kill_previous() {
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

pub fn spawn_background(args: Vec<String>) {
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
