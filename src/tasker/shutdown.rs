#[cfg(unix)]
mod platform {
    use nix::sys::signal::{SigSet, SIGINT, SIGQUIT, SIGTERM};

    pub struct ShutdownSignal(SigSet);

    impl ShutdownSignal {
        pub fn new() -> ShutdownSignal {
            let mut mask = SigSet::empty();

            mask.add(SIGINT);
            mask.add(SIGQUIT);
            mask.add(SIGTERM);
            mask.thread_block().unwrap();

            ShutdownSignal(mask)            
        }

        pub fn at_exit<F: FnOnce(usize)>(&self, handle: F) {
            let signal = self.0.wait().unwrap();
            handle(signal as usize);
        }
    }
}

pub use platform::ShutdownSignal;
