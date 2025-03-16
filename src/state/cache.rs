use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

/// If you want your state to be regenerated dynamically, for example in a [`CachedState`],
/// it should implement the ReloadableState trait. For this purpose, all the information
/// needed to rebuild the state should be owned by the structure when it is first created.
pub trait ReloadableState {
    /// Reload struct state
    fn reload_state(&mut self);
}

/// An application State that is cached for a certain number of seconds. Every `ttl` seconds,
/// it is reloaded. It is assumed here that regenerating the state will not take more than ttl
/// seconds. The cached state struct also needs to implement [`ReloadableState`].
/// It is also assumed, for the moment, that reloading the state cannot fail, or will just not
/// update the state.
pub struct CachedState<T: ReloadableState> {
    state: Arc<RwLock<T>>,
}

impl<T: ReloadableState + std::marker::Send + std::marker::Sync + 'static> CachedState<T> {
    /// Generate a new CachedState for ttl seconds, from an existing ReloadableState
    pub fn new(state: T, ttl: usize) -> CachedState<T> {
        let wrapped_state = Arc::new(RwLock::new(state));
        let cloned_state = wrapped_state.clone();

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(ttl as u64));
            let mut state = cloned_state.write().unwrap();
            state.reload_state();
        });

        CachedState {
            state: wrapped_state,
        }
    }

    /// Return a new copy of the state
    pub fn state(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.state.read().unwrap()
    }
}

impl<T: std::fmt::Display + ReloadableState> std::fmt::Display for CachedState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.state.read().unwrap().fmt(f)
    }
}
