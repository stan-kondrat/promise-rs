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
        where F: Send + 'static + Fn(&Fn()) {
        let handlers = Arc::new(Mutex::new(Some(Vec::new())));
        let handlers_cloned = handlers.clone();
        thread::spawn(move || {
            executor(& move || {
                for handler in handlers_cloned.lock().unwrap().take().unwrap().into_iter() {
                    let handler: Handler = handler;
                    (handler.handler)();
                }
            });
        });

        Promise { handlers: handlers }
    }

    pub fn then<F>(&mut self, on_fulfilled: F) -> &mut Promise
        where F: Send + 'static + Fn() {
        let handler = Handler{ resolve: true, handler: Box::new(on_fulfilled) };
        self.handlers.lock().unwrap().as_mut().unwrap().push(handler);
        self
    }
}
