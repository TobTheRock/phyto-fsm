pub fn is_debug() -> bool {
    static DEBUG: std::sync::LazyLock<bool> =
        std::sync::LazyLock::new(|| std::env::var("PHYTO_DEBUG").is_ok());
    *DEBUG
}

macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::debug::is_debug() {
            eprintln!("[phyto-fsm] {}", format!($($arg)*));
        }
    };
}

pub(crate) use debug;
