extern crate promise;
use promise::Promise;

use std::thread;
use std::time::Duration;

fn main() {
    println!("1");
    let mut promise = Promise::new(|resolve, reject| {
                                        thread::sleep(Duration::from_millis(100));
                                        println!("3");
                                        resolve();
                                        // reject();
                                        println!("5");
                                    });
    promise.then(|| println!("4 then resolve"), || println!("4 then reject"));
    promise.catch(|| println!("catch"));

    println!("2");

    thread::sleep(Duration::from_millis(200));
}
