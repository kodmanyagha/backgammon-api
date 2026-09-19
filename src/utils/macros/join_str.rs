//#[macro_export]
macro_rules! join_str {
    () => {
        String::new()
    };
    ($($arg:expr),*) => {{
        [$($arg.to_string()),*].join("")
    }};
}

pub(crate) use join_str;
