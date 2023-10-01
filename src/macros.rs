macro_rules! debugln {
    ($($args:expr),*) => (
        #[cfg(debug_assertions)]
        {
            eprint!("[DEBUG] ");
            eprintln!($($args),*);
        }
    )
}

macro_rules! parse_err {
    ($($args:expr),*) => (
        ParseError::from(format!($($args),*))
    )
}

pub(crate) use debugln;
pub(crate) use parse_err;
