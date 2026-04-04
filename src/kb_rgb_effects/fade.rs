use std::thread;
use std::time::Duration;

use crate::ec::write_rgb;

pub fn fade(c1: (u8, u8, u8), c2: (u8, u8, u8)) {
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
