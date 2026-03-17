use crate::{
        domain::UserStore,
        handle_login, handle_login_or_signup, handle_logout, handle_signup, handle_verify_2fa,
        handle_verify_token,
        utils::tracing::{make_span_with_request_id, on_request, on_response},
        AppState,
};
use axum::{
        routing::MethodRouter,
        routing::{get, post},
        Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub fn app_routes(app_state: AppState, cors: CorsLayer, asset_dir: MethodRouter) -> Router {
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
                .layer(TraceLayer::new_for_http()
                        .make_span_with(make_span_with_request_id)
                        .on_request(on_request)
                        .on_response(on_response))
}
