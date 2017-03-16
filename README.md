# Rust Promise library


Under the hood, executor spawn new thread at each time when `Promise::new(executor)` invoked.


## Usage


```
extern crate promise;
use promise::Promise;

fn main() {
  let mut promise = Promise::new(|resolve| {
    // do something
    resolve();
  });

  promise.then(|| { /* on fulfilled */ });
  promise.then(|| { /* on fulfilled */ });
}
```

## Motivation

Best way to begin learning a new language is start write own library. As I came from front-end world, will create yet another Promise library for Rust.
