use std::{thread, time};
use std::sync::{Arc, Mutex};


pub struct Handler {
    pub resolve: bool,
    pub handler: Box<Fn(Option<String>) -> Option<String> + Send>,
}

pub struct Promise {
    pub handlers: Arc<Mutex<Option<Vec<Handler>>>>,
}

impl Promise {
    pub fn new<F>(executor: F) -> Promise
        where F: Send + 'static + Fn(&Fn(Option<String>), &Fn(Option<String>)) {
        let handlers = Arc::new(Mutex::new(Some(Vec::new())));
        let handlers_cloned1 = handlers.clone();
        let handlers_cloned2 = handlers.clone();
        thread::spawn(move || {
            thread::park_timeout(time::Duration::from_millis(1));
            let resolve = move |value| {
                let mut prev_value: Option<String> = value;
                for handler in handlers_cloned1.lock().unwrap().take().unwrap().into_iter() {
                    let handler: Handler = handler;
                    if handler.resolve == true {
                        prev_value = (handler.handler)(prev_value.clone());
                    }
                }
            };
            let reject = move |reason| {
                let mut prev_reason: Option<String> = reason;
                for handler in handlers_cloned2.lock().unwrap().take().unwrap().into_iter() {
                    let handler: Handler = handler;
                    if handler.resolve == false {
                        prev_reason = (handler.handler)(prev_reason.clone());
                    }
                }
            };
            executor(&resolve, &reject);
        });

        Promise { handlers: handlers }
    }

    pub fn then<F1, F2>(&mut self, on_fulfilled: F1, on_rejected: F2) -> &mut Promise
        where F1: Send + 'static + Fn(Option<String>) -> Option<String>,
              F2: Send + 'static + Fn(Option<String>) -> Option<String> {
        let handler1 = Handler{ resolve: true, handler: Box::new(on_fulfilled) };
        let handler2 = Handler{ resolve: false, handler: Box::new(on_rejected) };
        self.handlers.lock().unwrap().as_mut().unwrap().push(handler1);
        self.handlers.lock().unwrap().as_mut().unwrap().push(handler2);
        self
    }

    pub fn catch<F>(&mut self, on_rejected: F) -> &mut Promise
        where F: Send + 'static + Fn(Option<String>) -> Option<String> {
        let handler = Handler{ resolve: false, handler: Box::new(on_rejected) };
        self.handlers.lock().unwrap().as_mut().unwrap().push(handler);
        self
    }

    /// The Promise::resolve(value) method returns a Promise object that is resolved with the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use promise::Promise;
    /// let mut promise = Promise::resolve(Some("resolve result".to_string()));
    /// promise.then(|value| {
    ///     println!("value: {:?}", value);
    ///     None
    /// }, |x| x);
    /// ```
    pub fn resolve(value: Option<String>) -> Promise {
        Promise::new(move |resolve, _| {
            resolve(value.clone());
        })
    }

    /// The Promise::reject(reason) method returns a Promise object that is rejected with the given reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use promise::Promise;
    /// let mut promise = Promise::reject(Some("reject result".to_string()));
    /// promise.catch(|reason| {
    ///     println!("reason: {:?}", reason);
    ///     None
    /// });
    /// ```
    pub fn reject(reason: Option<String>) -> Promise {
        Promise::new(move |_, reject| {
            reject(reason.clone());
        })
    }
}
