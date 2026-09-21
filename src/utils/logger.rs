use std::env;
use std::sync::Once;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static INIT: Once = Once::new();
static INIT_TEST_ENV: Once = Once::new();

pub fn init_tracing_subscriber() {
    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    });
}

pub fn init_test_env() {
    INIT_TEST_ENV.call_once(|| {
        dotenvy::from_filename(".env.test").ok();
    });
    init_tracing_subscriber();
}
