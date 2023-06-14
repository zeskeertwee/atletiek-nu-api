use poll_promise::Promise;

pub enum AsyncResource<R: Send + Clone + 'static> {
    Idle,
    Pending(Promise<anyhow::Result<R>>),
    Finished(R),
    Error(String),
}

impl<R: Send + Clone + 'static> Default for AsyncResource<R> {
    fn default() -> Self {
        Self::Idle
    }
}

impl<R: Send + Clone + 'static> AsyncResource<R> {
    pub fn new<T: FnOnce() -> anyhow::Result<R> + Send + 'static>(task: T) -> Self {
        Self::Pending(Promise::spawn_thread("AsyncResource-thread", task))
    }

    pub fn poll(&mut self) -> Option<R> {
        match self {
            Self::Pending(p) => match p.ready() {
                Some(v) => match v {
                    Ok(r) => {
                        let res = Some(r.to_owned());
                        *self = Self::Finished(r.to_owned());
                        return res;
                    }
                    Err(e) => *self = Self::Error(e.to_string()),
                },
                None => (),
            },
            _ => (),
        }

        None
    }

    pub fn is_pending(&self) -> bool {
        match self {
            Self::Pending(_) => true,
            _ => false,
        }
    }
}
