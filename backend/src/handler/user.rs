use std::sync::Arc;

use axum::{Extension, Json, Router, extract::{Path, Query}, middleware::{self, from_fn}, response::IntoResponse, routing::{get, post, put}};
use axum::extract::Multipart;
use validator::Validate;
use crate::{AppState, error::ForumError, middleware::{auth, is_banned}};
use crate::{db::user::UserExt,
    models::UserRole,
    dto::user,
    error::ForumResult,
    middleware::{role_check, JWTAuthMiddeware},
    utils::password,
    utils::file_upload,
};
use tracing::error;

pub fn user_handler() -> Router<AppState> {
    let admin_mod_only = middleware::from_fn(|state, req, next|
                    role_check(state, req, next, vec![UserRole::Admin, UserRole::Mod]) );

    Router::new()
        .route("/me", get(get_me)
            .layer(from_fn(is_banned))
            .layer(from_fn(auth)) )
        .route("/user/{uuid}", get(get_user_data))
        .route("/user/{uuid}", post(update_user_data).layer(from_fn(auth)))
        .route("/list", get(get_users))
        .route("/{user_id}/posts", get(user_posts))
        .route("/{user_id}/threads", get(user_threads))
        .route("/{user_id}/warnings", get(user_warnings).layer(from_fn(auth)) )
        .route("/message", post(send_pm).layer(from_fn(auth)) )
        .route("/unban", put(unban_user)
            .layer(admin_mod_only.clone()) 
            .layer(from_fn(auth))
            )
        .route("/warn", put(warn_user)
            .layer(admin_mod_only)
            .layer(from_fn(auth))
        )
        .route("/pms", get(get_pms).layer(from_fn(auth)) )
        .route("/avatar", post(upload_avatar))
}

pub async fn get_me(
    Extension(_app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>
) -> ForumResult<impl IntoResponse> {

    let filtered_user = user::FilterUserDto::filter_user(&user.user);

    let response_data = user::FilteredUserData {
            status: "success".to_string(),
            user: filtered_user,
    };

    Ok(Json(response_data))
}

pub async fn get_users(
    Query(query_params): Query<user::RequestQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>
) -> ForumResult<impl IntoResponse> {

    query_params.validate()
        .map_err(|e| {
            error!("Validation error: {}", e);
            ForumError::BadRequest 
        })?;

    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(10);
    
    let users = app_state.db_client
        .get_users(page as u32, limit)
        .await
        .map_err(|e| ForumError::ServerError(e.to_string()) )?;

    let user_count = app_state.db_client
        .get_user_count()
        .await
        .map_err(|e| ForumError::ServerError(e.to_string()) )?;

    let response = user::UserListResponseDto {
        status: "success".to_string(),
        users: user::FilterUserDto::filter_users(&users),
        results: user_count,
    };

    Ok(Json(response))
}

pub async fn get_user_data(
    Path(uuid) : Path<uuid::Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> ForumResult<impl IntoResponse> {
    let user = app_state.db_client.get_user_d(uuid)
        .await
        .map_err(|e| ForumError::ServerError(e.to_string()) )?;
    let response = user.unwrap_or_else(|| crate::dto::user::UserData { name: "Deleted User".to_string(), ..Default::default()});

    Ok(Json(response))


}
    
pub async fn update_user_name(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<user::NameUpdateDto>,
) -> ForumResult<impl IntoResponse> {
    body.validate()
       .map_err(|_e| ForumError::BadRequest )?;

    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.update_session(&user_id)?;
    app_state.db_client.
        update_user_name(user_id, &body.name)
        .await?;

    //let filtered_user = user::FilterUserDto::filter_user(&result);

    let response = user::Response {
        status: "success",
        message: "name changed".to_string(),
    };

    Ok(Json(response))
}

pub async fn update_user_role(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<user::RoleUpdateDto>,
) -> ForumResult<impl IntoResponse> {
    body.validate()
        .map_err(|_e| ForumError::BadRequest )?;

    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.update_session(&user_id)?;
    app_state.db_client
        .update_user_role(user_id, body.role)
        .await?;

    let response = user::Response {
        status: "success",
        message: "role changed".to_string(),
    };

    Ok(Json(response))
}

pub async fn update_user_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<user::UserPasswordUpdateDto>,
) -> ForumResult<impl IntoResponse> {
    body.validate()?;

    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.update_session(&user_id)?;
    let result = app_state.db_client
        .get_user(Some(user_id), None, None)
        .await?;

    let user = result.ok_or(ForumError::InvalidToken )?;

    let password_match = password::compare(&body.old_password, &user.password)
            .map_err(|e| ForumError::ServerError(e.to_string()))?;

    if !password_match {
        return Err( ForumError::OldPassword );
    }

    let hash_password = password::hash(&body.new_password)
        .map_err(|e| ForumError::ServerError(e.to_string()))?;

    app_state.db_client
        .update_user_password(user_id, hash_password.as_str())
        .await?;

    let response = user::Response {
        message: "Password updated Successfully".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub async fn update_user_data(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<user::FilterUserDto>,
) -> ForumResult<impl IntoResponse> {
    body.validate()?;

    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string())?;
    let body_uid = uuid::Uuid::parse_str(&body.id)?;

    app_state.update_session(&user_id)?;
    let result = app_state.db_client
        .get_user(Some(user_id), None, None)
        .await?;

    let _ = result.ok_or(ForumError::InvalidToken )?;
    if user_id != body_uid 
        && user.role != UserRole::Admin
        && user.role != UserRole::Mod {
            return Err(ForumError::Forbidden);
    }

    let user_role = UserRole::from_str(&body.role)?;

    app_state.db_client.update_user_data(user_id, 
        &body.name, 
        &body.email, 
        user_role, 
        body.description.as_deref(), 
        body.avatar.as_deref(),
        body.facebook.as_deref(),
        body.discord.as_deref(),
        body.x_id.as_deref(),
        ).await?;

    let response = user::Response {
        message: "User updated Successfully".to_string(),
        status: "success",
    };

    Ok(Json(response))

}

pub async fn warn_user(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<user::WarnUserDto>,
) -> ForumResult<impl IntoResponse> {

    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.update_session(&user_id)?;
    app_state.db_client
        .warn_user(body.uuid, body.comment.as_deref(), user_id, body.banned)
        .await?;

    let response = user::Response {
        message: "User warned".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub async fn unban_user(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<user::UnbanUserDto>,
) -> ForumResult<impl IntoResponse> {
    app_state.db_client
        .unban_user(body.uuid)
        .await?;

    let response = user::Response {
        message: "User unbanned".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub async fn user_posts(
    Path(user_id) : Path<uuid::Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> ForumResult<impl IntoResponse> {
    let result = app_state.db_client
        .get_user_posts(Some(user_id), None)
        .await?;

    let response = user::UserPostsResponseDto {
        posts: result,
    };

    Ok(Json(response))
}

pub async fn user_threads(
    Path(user_id) : Path<uuid::Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> ForumResult<impl IntoResponse> {
    let threads = app_state.db_client
        .get_user_threads(Some(user_id), None)
        .await?;

    let response = user::UserThreadsResponseDto {
        threads,
    };

    Ok(Json(response))
}

pub async fn user_warnings(
    Path(user_id) : Path<uuid::Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> ForumResult<impl IntoResponse> {
    let warnings = app_state.db_client
        .get_user_warnings(user_id, None)
        .await?;

    let response = user::UserWarningsResponseDto {
        warnings,
    };

    Ok(Json(response))
}

pub async fn send_pm(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<user::SendPmDto>,
) -> ForumResult<impl IntoResponse> {
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.db_client.send_pm(user_id, body.recipient_id, body.content.as_str())
        .await?;

    let response = user::Response {
        message: "Private message sent".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub async fn get_pms(
    Path((page,limit)) : Path<(u32,usize)>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> ForumResult<impl IntoResponse> {
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.update_session(&user_id)?;
    let pms = app_state.db_client.get_pms(user_id, page, limit)
        .await?;

    let response = user::UserPmsResponseDto { pms };

    Ok(Json(response))
}

/// Upload user avatar
/// POST /users/avatar
/// Requires authentication
/// Multipart form with "avatar" field containing image file
pub async fn upload_avatar(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    multipart: Multipart,
) -> ForumResult<impl IntoResponse> {
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.update_session(&user_id)?;
    // Get current user to check existing avatar
    let current_user = app_state.db_client.get_user(Some(user_id), None, None)
        .await?
        .ok_or(ForumError::Unauthorized)?;

    // Save uploaded image
    let filename = file_upload::save_uploaded_image(multipart, &app_state.env, user_id).await?;

    // Delete old avatar if it exists and is not default
    file_upload::delete_old_avatar(current_user.avatar.as_deref(), &app_state.env).await?;

    // Update user avatar in database
    app_state.db_client.update_user_avatar(user_id, Some(&filename))
        .await
        .map_err(|e| ForumError::ServerError(e.to_string()))?;

    // Generate full URL for the avatar
    let avatar_url = file_upload::get_avatar_url(&app_state.env, &filename);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Avatar uploaded successfully",
        "avatar_url": avatar_url,
        "filename": filename
    })))
}

