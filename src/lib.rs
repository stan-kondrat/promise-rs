use std::thread;
use std::sync::{Arc, Mutex};

pub enum State {
    PENDING,
    FULFILLED,
    REJECTED,
}

pub struct Handler {
    pub resolve: bool,
    pub handler: Box<Fn() + Send>,
}

pub struct Promise {
    pub handlers: Arc<Mutex<Option<Vec<Handler>>>>,
}

impl Promise {
    pub fn new<F>(executor: F) -> Promise
        where F: Send + 'static + Fn(&Fn(), &Fn()) {
        let handlers = Arc::new(Mutex::new(Some(Vec::new())));
        let handlers_cloned1 = handlers.clone();
        let handlers_cloned2 = handlers.clone();
        thread::spawn(move || {
            let resolve = move || {
                for handler in handlers_cloned1.lock().unwrap().take().unwrap().into_iter() {
                    let handler: Handler = handler;
                    if handler.resolve {
                        (handler.handler)();
                    }
                }
            };
            let reject = move || {
                for handler in handlers_cloned2.lock().unwrap().take().unwrap().into_iter() {
                    let handler: Handler = handler;
                    if !handler.resolve {
                        (handler.handler)();
                    }
                }
            };
            executor(&resolve, &reject);
        });

        Promise { handlers: handlers }
    }


    pub fn then<F1, F2>(&mut self, on_fulfilled: F1, on_rejected: F2) -> &mut Promise
        where F1: Send + 'static + Fn(), F2: Send + 'static + Fn() {
        let handler1 = Handler{ resolve: true, handler: Box::new(on_fulfilled) };
        let handler2 = Handler{ resolve: false, handler: Box::new(on_rejected) };
        self.handlers.lock().unwrap().as_mut().unwrap().push(handler1);
        self.handlers.lock().unwrap().as_mut().unwrap().push(handler2);
        self
    }

    pub fn catch<F>(&mut self, on_rejected: F) -> &mut Promise
        where F: Send + 'static + Fn() {
        let handler = Handler{ resolve: false, handler: Box::new(on_rejected) };
        self.handlers.lock().unwrap().as_mut().unwrap().push(handler);
        self
    }
}
