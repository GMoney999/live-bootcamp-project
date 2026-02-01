pub mod auth;
pub mod constants;

use axum::routing::{get_service, MethodRouter};
use tower_http::services::{ServeDir, ServeFile};

pub fn fetch_assets() -> MethodRouter {
        get_service(ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html")))
}
