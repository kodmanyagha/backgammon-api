macro_rules! time_diff_start {
    () => {
        tokio::time::Instant::now()
    };
}

macro_rules! time_diff_log {
    ($time_diff:expr, $title:expr) => {{
        tracing::info!(
            "{} seconds: {:?}",
            $title,
            $time_diff.elapsed().as_secs_f32()
        );
    }};
}

pub(crate) use time_diff_log;
pub(crate) use time_diff_start;
