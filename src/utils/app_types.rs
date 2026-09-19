use std::sync::Arc;

use axum::response::Html;

pub type HtmlResult = axum::response::Result<Html<String>, anyhow::Error>;

pub type ArcStr = Arc<str>;
