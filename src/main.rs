mod color;
mod ec;
mod kb_rgb_effects;
mod process;

use std::env;

use color::preset;
use ec::{ensure_ec_access, ensure_root, read_rgb, write_rgb};
use kb_rgb_effects::{alternate, breathe, fade, rainbow};
use process::{kill_previous, spawn_background};

fn main() {
    ensure_root();
    ensure_ec_access();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: victus-rgb <command>");
        return;
    }

    let worker = args.contains(&"--worker".to_string());

    // Direct R G B values
    if args.len() == 4 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            args[1].parse::<u8>(),
            args[2].parse::<u8>(),
            args[3].parse::<u8>(),
        ) {
            kill_previous();
            write_rgb(r, g, b);
            return;
        }
    }

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
                spawn_background(vec!["fade".into(), args[2].clone(), args[3].clone()]);
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
a