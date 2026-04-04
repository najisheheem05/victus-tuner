mod color;
mod ec;
mod kb_rgb_effects;
mod process;

use std::env;

use color::preset;
use ec::{ensure_ec_access, ensure_root, read_rgb, write_rgb};
use kb_rgb_effects::{alternate, breathe, fade, rainbow};
use process::{kill_previous, spawn_background};

fn print_usage() {
    println!("Usage: victuner <module> <command> [args...]");
    println!();
    println!("Modules:");
    println!("  rgb    Keyboard RGB lighting control");
    println!();
    println!("RGB commands:");
    println!("  victuner rgb <color>              Set a preset color (red, green, blue, yellow, cyan, purple, neon-purple, white, off)");
    println!("  victuner rgb <R> <G> <B>          Set custom RGB values (0-255)");
    println!("  victuner rgb current              Show current RGB values");
    println!("  victuner rgb stop                 Stop any running effect");
    println!("  victuner rgb rainbow              Rainbow cycle effect");
    println!("  victuner rgb breathe <color>      Breathing effect with a preset color");
    println!("  victuner rgb alternate <c1> <c2>  Alternate between two preset colors");
    println!("  victuner rgb fade <c1> <c2>       Smooth fade between two preset colors");
}

fn handle_rgb(args: &[String], worker: bool) {
    if args.is_empty() {
        print_usage();
        return;
    }

    // Direct R G B values
    if args.len() == 3 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            args[0].parse::<u8>(),
            args[1].parse::<u8>(),
            args[2].parse::<u8>(),
        ) {
            kill_previous();
            write_rgb(r, g, b);
            return;
        }
    }

    match args[0].as_str() {
        "current" => read_rgb(),

        "stop" => kill_previous(),

        "rainbow" => {
            if !worker {
                spawn_background(vec!["rgb".into(), "rainbow".into()]);
            }
            rainbow();
        }

        "breathe" => {
            let c = preset(&args[1]).unwrap();
            if !worker {
                spawn_background(vec!["rgb".into(), "breathe".into(), args[1].clone()]);
            }
            breathe(c);
        }

        "alternate" => {
            let c1 = preset(&args[1]).unwrap();
            let c2 = preset(&args[2]).unwrap();
            if !worker {
                spawn_background(vec![
                    "rgb".into(),
                    "alternate".into(),
                    args[1].clone(),
                    args[2].clone(),
                ]);
            }
            alternate(c1, c2);
        }

        "fade" => {
            let c1 = preset(&args[1]).unwrap();
            let c2 = preset(&args[2]).unwrap();
            if !worker {
                spawn_background(vec![
                    "rgb".into(),
                    "fade".into(),
                    args[1].clone(),
                    args[2].clone(),
                ]);
            }
            fade(c1, c2);
        }

        color => {
            if let Some((r, g, b)) = preset(color) {
                kill_previous();
                write_rgb(r, g, b);
            } else {
                eprintln!("Unknown RGB command: {}", color);
                print_usage();
            }
        }
    }
}

fn main() {
    ensure_root();
    ensure_ec_access();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let worker = args.contains(&"--worker".to_string());

    match args[1].as_str() {
        "rgb" => handle_rgb(&args[2..], worker),

        _ => {
            eprintln!("Unknown module: {}", args[1]);
            print_usage();
        }
    }
}