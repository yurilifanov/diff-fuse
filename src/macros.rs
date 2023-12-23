macro_rules! srcloc {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

macro_rules! logfmt {
    ($type:expr, $($args:expr),*) => {
        format!(
            "[{:?}][{}][{}] {}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::new(0, 0))
                .as_micros(),
            $type,
            crate::macros::srcloc!(),
            format!($($args),*)
        )
    }
}

macro_rules! debugln {
    ($($args:expr),*) => (
        #[cfg(debug_assertions)]
        {
            eprintln!("{}", crate::macros::logfmt!("DEBUG", $($args),*));
        }
    )
}

macro_rules! warnln {
    ($($args:expr),*) => (
        eprintln!("{}", crate::macros::logfmt!("WARNING", $($args),*));
    )
}

macro_rules! parse_err {
    ($($args:expr),*) => (
        ParseErr::from(format!($($args),*))
    )
}

macro_rules! merge_err {
    ($($args:expr),*) => (
        MergeErr::from(format!($($args),*))
    )
}

pub(crate) use debugln;
pub(crate) use logfmt;
pub(crate) use merge_err;
pub(crate) use parse_err;
pub(crate) use srcloc;
pub(crate) use warnln;
