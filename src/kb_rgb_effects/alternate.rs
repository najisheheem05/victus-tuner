use std::thread;
use std::time::Duration;

use crate::ec::write_rgb;

pub fn alternate(c1: (u8, u8, u8), c2: (u8, u8, u8)) {
    loop {
        write_rgb(c1.0, c1.1, c1.2);
        thread::sleep(Duration::from_millis(500));

        write_rgb(c2.0, c2.1, c2.2);
        thread::sleep(Duration::from_millis(500));
    }
}
