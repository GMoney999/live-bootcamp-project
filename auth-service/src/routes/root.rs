// src/routes/root.rs
use axum::{http::StatusCode, response::{Html, IntoResponse}};

pub async fn handle_login_or_signup() -> impl IntoResponse {
	println!("->> {:<12} – handle_login_or_signup", "HANDLER");
	
	let html = match tokio::fs::read_to_string("assets/index.html").await {
		Ok(content) => Html(content),
		Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
	};
	
	html.into_response()
}
