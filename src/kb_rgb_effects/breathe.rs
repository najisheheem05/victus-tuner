use std::thread;
use std::time::Duration;

use crate::ec::write_rgb;

pub fn breathe(color: (u8, u8, u8)) {
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
