pub fn is_debug() -> bool {
    thread_local! {
        static DEBUG: bool = std::env::var("PHYTO_DEBUG").is_ok();
    }
    DEBUG.with(|&v| v)
}

macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::debug::is_debug() {
            eprintln!("[phyto-fsm] {}", format!($($arg)*));
        }
    };
}

pub(crate) use debug;
