/// Application state.
#[derive(Debug, Default)]
pub struct App {
    pub running: bool,
    pub counter: u64,
}

impl App {
    /// Create a new [`App`] in a running state.
    pub fn new() -> Self {
        Self {
            running: true,
            counter: 0,
        }
    }

    /// Signal the application to stop.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Increment the counter by one.
    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    /// Decrement the counter by one (floors at zero).
    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_app_is_running() {
        let app = App::new();
        assert!(app.running);
        assert_eq!(app.counter, 0);
    }

    #[test]
    fn quit_sets_running_false() {
        let mut app = App::new();
        app.quit();
        assert!(!app.running);
    }

    #[test]
    fn counter_increment_and_decrement() {
        let mut app = App::new();
        app.increment_counter();
        app.increment_counter();
        assert_eq!(app.counter, 2);
        app.decrement_counter();
        assert_eq!(app.counter, 1);
    }

    #[test]
    fn counter_does_not_underflow() {
        let mut app = App::new();
        app.decrement_counter();
        assert_eq!(app.counter, 0);
    }
}
