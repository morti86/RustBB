use axum::{
    Router,
    routing::get,
};
use crate::{AppState, utils::file_upload};

pub fn file_upload_handler() -> Router<AppState> {
    Router::new()
        .route("/{image_id}", get(file_upload::serve_avatar))
        .route("/locales", get(file_upload::list_locales))
}

