macro_rules! parse_str {
    ($input:expr, $target_type:ty) => {
        $input
            .parse::<$target_type>()
            .map_err(|_| crate::utils::consts::errors::PARSE_ERROR)
    };
}

pub(crate) use parse_str;
