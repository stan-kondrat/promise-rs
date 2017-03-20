# Rust Promise library


Under the hood, executor spawn new thread at each time when `Promise::new(executor)` invoked.


## Usage

```rust
extern crate promise;
use promise::Promise;

fn main() {
  let mut promise = Promise::new(|resolve, reject| {
    // do something
    let result: Option<String> = Some("resolve result".to_string());
    resolve(result);
  });

  promise
    .then(|value| { /* on fulfilled */ None }, |reason| { /* on rejected */ None })
    .catch(|reason| { /* on rejected: */ None });
}
```

## Motivation

Best way to begin learning a new language is start write own library. As I came from front-end world, will create yet another Promise library for Rust.


## Links

1. [JavaScript Promise Syntax](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise)
2. [Zero-cost futures and streams in Rust](https://github.com/alexcrichton/futures-rs)
3. [Asynchronous promises in rust using threads](https://github.com/bcndanos/asynchronous)
