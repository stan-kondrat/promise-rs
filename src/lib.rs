use std::thread;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Debug, Clone)]
pub enum State {
    PENDING,
    FULFILLED,
    REJECTED,
}

pub struct Handler {
    pub resolve: bool,
    pub handler: Box<Fn(Option<String>) -> Option<String> + Send>,
}

pub struct Promise {
    pub value: Arc<Mutex<Option<String>>>,
    pub state: Arc<Mutex<Option<State>>>,
    pub handlers: Arc<Mutex<Option<Vec<Handler>>>>,
    pub thread: std::thread::JoinHandle<()>,
}

impl Promise {
    pub fn new<F>(executor: F) -> Promise
        where F: Send + 'static + Fn(&Fn(Option<String>), &Fn(Option<String>)) {

        let result = Arc::new(Mutex::new(None));
        let result_resolve = result.clone();
        let result_reject = result.clone();

        let state = Arc::new(Mutex::new(Some(State::PENDING)));
        let state_resolve = state.clone();
        let state_reject = state.clone();

        let handlers = Arc::new(Mutex::new(Some(Vec::new())));
        let handlers_resolve = handlers.clone();
        let handlers_reject = handlers.clone();

        let thread = thread::spawn(move || {
            let resolve = move |value| {
                let mut prev_value: Option<String> = value;
                for handler in handlers_resolve.lock().unwrap().take().unwrap().into_iter() {
                    let handler: Handler = handler;
                    if handler.resolve == true {
                        prev_value = (handler.handler)(prev_value.clone());
                    }
                }
                let mut result_resolve = result_resolve.lock().unwrap();
                *result_resolve = prev_value;
                let mut state_guard = state_resolve.lock().unwrap();
                let mut state = state_guard.as_mut().unwrap();
                *state = State::FULFILLED;
            };
            let reject = move |reason| {
                let mut prev_reason: Option<String> = reason;
                for handler in handlers_reject.lock().unwrap().take().unwrap().into_iter() {
                    let handler: Handler = handler;
                    if handler.resolve == false {
                        prev_reason = (handler.handler)(prev_reason.clone());
                    }
                }
                let mut result_reject = result_reject.lock().unwrap();
                *result_reject = prev_reason;
                let mut state_guard = state_reject.lock().unwrap();
                let mut state = state_guard.as_mut().unwrap();
                *state = State::REJECTED;
            };
            executor(&resolve, &reject);
        });

        Promise { handlers: handlers, state: state, value: result, thread:thread }
    }

    pub fn then<F1, F2>(&mut self, on_fulfilled: F1, on_rejected: F2) -> &mut Promise
        where F1: Send + 'static + Fn(Option<String>) -> Option<String>,
              F2: Send + 'static + Fn(Option<String>) -> Option<String> {

        let state = self.state.lock().unwrap().clone().unwrap();
        match state {
            State::FULFILLED => {
                let result_resolve = self.value.clone();
                let mut value = result_resolve.lock().unwrap();
                let prev_value = value.clone();
                *value = (on_fulfilled)(prev_value);
            },
            State::REJECTED => {
                let result_reject = self.value.clone();
                let mut reason = result_reject.lock().unwrap();
                let prev_reason = reason.clone();
                *reason = (on_rejected)(prev_reason);
            },
            State::PENDING => {
                let handler_fulfilled = Handler{ resolve: true, handler: Box::new(on_fulfilled) };
                let handler_rejected = Handler{ resolve: false, handler: Box::new(on_rejected) };
                self.handlers.lock().unwrap().as_mut().unwrap().push(handler_fulfilled);
                self.handlers.lock().unwrap().as_mut().unwrap().push(handler_rejected);
            },
        }
        self
    }

    pub fn catch<F>(&mut self, on_rejected: F) -> &mut Promise
        where F: Send + 'static + Fn(Option<String>) -> Option<String> {
        let state = self.state.lock().unwrap().clone().unwrap();
        match state {
            State::FULFILLED => {},
            State::REJECTED => {
                let result_reject = self.value.clone();
                let mut reason = result_reject.lock().unwrap();
                let prev_reason = reason.clone();
                *reason = (on_rejected)(prev_reason);
            },
            State::PENDING => {
                let handler = Handler{ resolve: false, handler: Box::new(on_rejected) };
                self.handlers.lock().unwrap().as_mut().unwrap().push(handler);
            },
        }
        self
    }

    /// The Promise::await() method is used to wait for a Promise fulfilled.
    pub fn await(self) {
        let _ = self.thread.join();
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

    /// The Promise::all(vec![p1, p2, p3]) method returns a single Promise that
    /// resolves when all of the promises in the vector argument have resolved,
    /// or rejects with the reason of the first promise that rejects.
    /// Resoved results joined with delimeter, by default semicolon;
    ///
    /// # Examples
    ///
    /// ```
    /// use promise::Promise;
    /// let promise1 = Promise::resolve(Some("resolve 1 result".to_string()));
    /// let promise2 = Promise::resolve(Some("reject 2 result".to_string()));
    /// let promise3 = Promise::resolve(Some("resolve 3 result".to_string()));
    ///
    /// let promise4 = Promise::new(|resolve, reject| {
    ///     std::thread::sleep(std::time::Duration::from_millis(10));
    ///     resolve(Some("resolve 3 result".to_string()));
    /// });
    ///
    /// let mut promise_all = Promise::all(vec![promise1, promise2, promise3, promise4]);
    ///
    /// promise_all.then(|value| {
    ///     println!("promise_all: {:?}", value);
    ///     None
    /// }, |_| None );
    /// ```
    pub fn all(promises: Vec<Promise>) -> Promise {
        Promise::all_ex(promises, ";")
    }
    pub fn all_ex(promises: Vec<Promise>, delimeter: &str) -> Promise {
        let mut rejected = false;
        #[derive(Debug)]
        // let mut pending_promises_threads: Vec<Arc<Mutex<Option<std::thread::JoinHandle<()>>>>> = vec![];
        let mut resoved_results: Vec<String> = vec![];
        let mut first_reject_reason = String::new();
        for promise in promises.into_iter() {
            let _ = promise.thread.join();
            let state = promise.state.lock().unwrap().clone().unwrap();
            let value = promise.value.lock().unwrap().clone().unwrap_or(String::new());
            match state {
                State::FULFILLED => {
                    resoved_results.push(value);
                },
                State::REJECTED => {
                    rejected = true;
                    first_reject_reason = value;
                },
                State::PENDING => {

                }
            }
        }
        if rejected {
            return Promise::reject(Some(first_reject_reason));
        } else {
            return Promise::resolve(Some(resoved_results.join(delimeter)));
        }
    }
}
