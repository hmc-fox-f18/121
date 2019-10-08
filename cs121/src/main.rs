extern crate ws;
extern crate chrono;

use ws::listen;
use chrono::Utc;

fn main() {
    listen("127.0.0.1:3012", |out| {
        move |msg| {
            let local_time = Utc::now();
            println!("{:?}: {}", local_time, msg);
            Ok(())
      }
    }).unwrap()
}
