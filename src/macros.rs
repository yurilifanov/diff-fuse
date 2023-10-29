macro_rules! debugln {
    ($($args:expr),*) => (
        #[cfg(debug_assertions)]
        {
            eprint!("[DEBUG] ");
            eprintln!($($args),*);
        }
    )
}

macro_rules! warnln {
    ($($args:expr),*) => (
        eprint!("[WARNING] ");
        eprintln!($($args),*);
    )
}

macro_rules! parse_err {
    ($($args:expr),*) => (
        ParseError::from(format!($($args),*))
    )
}

macro_rules! merge_err {
    ($($args:expr),*) => (
        MergeError::from(format!($($args),*))
    )
}

pub(crate) use debugln;
pub(crate) use merge_err;
pub(crate) use parse_err;
pub(crate) use warnln;
