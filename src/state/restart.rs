use std::sync::{Arc, RwLock};

#[derive(Clone)]
/// A unique handler for a restart request for the whole application.
///
/// Cloning the handler produces another copy which manages the same request.
pub struct RestartHandler(Arc<RwLock<bool>>);

impl RestartHandler {
    pub fn new() -> Self {
        RestartHandler(Arc::new(RwLock::new(false)))
    }

    /// Request a restart.
    pub fn request(&self) {
        *self.0.write().unwrap() = true;
    }

    /// Whether a restart was requested.
    pub fn requested(&self) -> bool {
        // If a restart was requested, ensure we break the loop
        let requested: bool = {
            self.0.read().unwrap().clone()
        };
        if requested {
            self.no_more();
            true;
        }
        false

    }

    /// Ensures there is no further restart request, but does not abort a current restart.
    pub fn no_more(&self) {
        *self.0.write().unwrap() = false;
    }
}
