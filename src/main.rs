extern crate promise;
use promise::Promise;

use std::thread;
use std::time::Duration;

fn main() {
    println!("1");
    let mut promise = Promise::new(|resolve| {
                                        thread::sleep(Duration::from_millis(100));
                                        println!("3");
                                        resolve();
                                        println!("6");
                                    });
    promise.then(|| println!("4"));
    promise.then(|| println!("5"));
    
    println!("2");

    thread::sleep(Duration::from_millis(200));
}
