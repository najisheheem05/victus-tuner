use std::thread;
use std::time::Duration;

use crate::color::hsv_to_rgb;
use crate::ec::write_rgb;

pub fn rainbow() {
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
