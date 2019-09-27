extern crate ws;

use ws::listen;
use std::time::{Duration, Instant};

fn main() {
    let start = Instant::now();
    listen("127.0.0.1:3012", |out| {
        move |msg| {
            let duration = start.elapsed();
            println!("{:?}: {}", duration, msg);
            Ok(())
      }
    }).unwrap()
}
