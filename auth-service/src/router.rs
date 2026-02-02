use crate::{
        domain::UserStore, handle_login, handle_login_or_signup, handle_logout, handle_signup,
        handle_verify_2fa, handle_verify_token, AppState,
};
use axum::{
        routing::MethodRouter,
        routing::{get, post},
        Router,
};
use tower_http::cors::CorsLayer;

pub fn app_routes<T>(app_state: AppState<T>, cors: CorsLayer, asset_dir: MethodRouter) -> Router
where
        T: UserStore + 'static,
{
        Router::new()
                .fallback_service(asset_dir)
                .route("/", get(handle_login_or_signup))
                .route("/signup", post(handle_signup))
                .route("/login", post(handle_login))
                .route("/logout", post(handle_logout))
                .route("/verify-2fa", post(handle_verify_2fa))
                .route("/verify-token", post(handle_verify_token))
                .with_state(app_state)
                .layer(cors)
}
