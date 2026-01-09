use std::sync::Arc;

use axum::{Extension, Json, Router, extract::{Multipart, Path, Query}, middleware::from_fn, response::IntoResponse, routing::{delete, get, post, put}};
use validator::Validate;
use crate::{AppState, dto::{Response, forum::ActiveUsersDto}, error::ForumResult, middleware::is_banned, utils::file_upload};
use crate::{db::forum::ForumExt,
    db::user::UserExt,
    models::UserRole,
    dto::forum,
    error::ForumError,
    middleware::{role_check, JWTAuthMiddeware, auth},
};

pub fn forum_handler() -> Router<AppState> {
    let admin_mod_only = from_fn(|state, req, next| 
        role_check(state, req, next, vec![UserRole::Admin, UserRole::Mod]) );
    let admin_only = from_fn(|state, req, next| 
        role_check(state, req, next, vec![UserRole::Admin]) );

    Router::new()
        .route("/list", get(get_sections))
        .route("/section/{s_id}", get(get_threads))
        .route("/section/add", put(add_section)
            .layer(admin_only.clone())
            .layer(from_fn(auth))
            )
        .route("/threads/new", post(create_thread)
            .layer(from_fn(is_banned))
            .layer(from_fn(auth)) 
            )
        .route("/threads", delete(delete_thread).layer(admin_mod_only.clone()) )
        .route("/threads", put(update_thread)
            .layer(from_fn(auth))
            )
        .route("/threads/{thread_id}", get(get_thread))
        .route("/post/new", 
            post(reply_thread)
                .layer(from_fn(is_banned))
                .layer(from_fn(auth))
            )
        .route("/threads/lock", put(lock_thread)
            .layer(admin_mod_only.clone()) 
            )
        .route("/post", put(update_post)
            .layer(from_fn(is_banned))
            .layer(from_fn(auth))
            )
        .route("/post", delete(delete_post)
            .layer(from_fn(is_banned))
            .layer(from_fn(auth))
            )
        .route("/active", get(list_active))
        .route("/upload_image", post(upload_image)
            .layer(from_fn(is_banned))
            .layer(from_fn(auth))
            )



}

pub async fn create_thread(Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<forum::CreateThreadDto>,
) -> ForumResult<impl IntoResponse> {
    body.validate()?;
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();
    app_state.update_session(&user_id)?;
    app_state.db_client.update_user_activity(user_id).await?;
    let hash_tags = body.hash_tags;
    app_state.db_client.create_thread(user_id, body.section, body.title.as_str(), body.content.as_str(), &hash_tags )
        .await
        ?;

    let response = forum::Response {
        status: "success",
        message: "thread created".to_string(),
    };

    Ok(Json(response))

}

pub async fn delete_thread(Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<forum::DeleteThreadDto>,
) -> ForumResult<impl IntoResponse> {
    body.validate()?;
    let user_id = user.user.id;
    app_state.update_session(&user_id)?;
    app_state.db_client.update_user_activity(user_id).await?;
    app_state.db_client.delete_thread(body.thread_id)
        .await
        ?;

    let response = forum::Response {
        status: "success",
        message: "thread deleted".to_string(),
    };

    Ok(Json(response))
}


pub async fn update_thread(Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<forum::UpdateThreadDto>,
) -> ForumResult<impl IntoResponse> {

    body.validate()?;
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();
    app_state.update_session(&user_id)?;
    app_state.db_client.update_user_activity(user_id).await?;
    let user_role = user.role;

    if user_role != UserRole::Admin && 
        user_role != UserRole::Mod && 
        app_state.db_client.get_thread_author(body.thread_id as i32).await? != user_id{
        return Err(ForumError::Unauthorized);
    }

    app_state.db_client
        .update_thread(body.thread_id, body.title.as_str(), body.content.as_str() )
        .await?;

    let response = forum::Response {
        status: "success",
        message: "thread updated".to_string(),
    };

    Ok(Json(response))

}

pub async fn get_thread(
    Path(thread_id) : Path<i64>,
    Query(query_params): Query<forum::GetThreadsDto>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> ForumResult<impl IntoResponse> {


    let thread = app_state.db_client
        .get_thread_info(thread_id as i32)
        .await?;

    let posts = app_state.db_client
        .get_thread(thread_id,query_params.page.unwrap_or(1),query_params.limit.unwrap_or(10))
        .await?;

    let response = forum::GetThreadResponseDto {
        info: thread,
        posts,
    };

    Ok(Json(response))
}

pub async fn lock_thread(Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<forum::LockThreadDto>,
) -> ForumResult<impl IntoResponse> {
    app_state.db_client.lock_thread(body.thread_id, body.locked)
        .await
        ?;

    let response = forum::Response {
        status: "success",
        message: "thread updated".to_string(),
    };

    Ok(Json(response))
}

pub async fn get_sections(
    Extension(app_state): Extension<Arc<AppState>>,
    user: Option<Extension<JWTAuthMiddeware>>,
) -> ForumResult<impl IntoResponse> {
    let user_id = user.map(|u| u.user.id);

    if let Some(user_id) = user_id {
        app_state.update_session(&user_id)?;
        app_state.db_client.update_user_activity(user_id).await?;
    }

    let sections = app_state.db_client.get_sections(user_id).await?;

    let response = forum::GetSectionsResponseDto { sections };

    Ok(Json(response))
}

pub async fn add_section(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(_user): Extension<JWTAuthMiddeware>,
    Json(body): Json<forum::CreateSectionDto>,
) -> ForumResult<impl IntoResponse> {
    
    app_state.db_client.create_section(&body.name, &body.description, &body.allowed_for).await?;

    let r = Response {
        status: "success",
        message: "Section created".to_string(),
    };

    Ok(Json(r))
}

pub async fn get_threads(
    Path(thread_id) : Path<i64>,
    Query(query_params): Query<forum::GetThreadsDto>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> ForumResult<impl IntoResponse> {

    let threads = app_state.db_client.get_section(thread_id, query_params.page.unwrap_or(1), query_params.limit.unwrap_or(10))
        .await
        ?;

    let response = forum::GetSectionResponseDto { threads };

    Ok(Json(response))
}

pub async fn reply_thread(Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<forum::ReplyThreadDto>,
) -> ForumResult<impl IntoResponse> {
    body.validate()?;
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.update_session(&user_id)?;
    app_state.db_client.update_user_activity(user_id).await?;
    app_state.db_client.add_post(user_id, body.t_id, body.content.as_str(), body.post_id)
        .await
        ?;
    let response = forum::Response {
        status: "success",
        message: "post added".to_string(),
    };

    Ok(Json(response))

}

pub async fn update_post(Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<forum::UpdatePostDto>,
) -> ForumResult<impl IntoResponse> {
    body.validate()?;
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string())?;
    let user_role = user.role;

    app_state.update_session(&user_id)?;
    app_state.db_client.update_user_activity(user_id).await?;
    if user_role != UserRole::Admin && 
        user_role != UserRole::Mod && 
        let Some(author_id) = app_state.db_client.get_post_author(body.post_id).await? &&
        author_id != user_id  {
            return Err(ForumError::Unauthorized);
    }

    app_state.db_client.update_post(body.post_id, body.content.as_str())
        .await
        ?;

    let response = forum::Response {
        status: "success",
        message: "post updated".to_string(),
    };

    Ok(Json(response))

}

pub async fn delete_post(Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    Json(body): Json<forum::DeletePostDto>,
    ) -> ForumResult<impl IntoResponse> {
    body.validate()?;
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();
    let user_role = user.role;

    app_state.update_session(&user_id)?;
    app_state.db_client.update_user_activity(user_id).await?;
    if user_role != UserRole::Admin && 
        user_role != UserRole::Mod && 
        let Some(author_id) = app_state.db_client.get_post_author(body.post_id).await? &&
        author_id != user_id  {
            return Err(ForumError::Unauthorized);
    } 

    if app_state.db_client.posts_since(body.post_id).await? > 0 && user.role == UserRole::User {
        return Err(ForumError::Forum("Cannot delete posts that have answers".to_string()));
    }

    app_state.db_client.delete_post(body.post_id)
        .await?;

    let response = forum::Response {
        status: "success",
        message: "post updated".to_string(),
    };

    Ok(Json(response))

}

pub async fn list_active(Extension(app_state): Extension<Arc<AppState>>) -> ForumResult<impl IntoResponse> {
    let active = app_state.list_active();
    Ok(Json(
        ActiveUsersDto { count: active.len(), users: active }
    ))
}
/// Upload image for posts
/// POST /forum/upload_image
/// Requires authentication
/// Multipart form with "avatar" field containing image file
pub async fn upload_image(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddeware>,
    multipart: Multipart,
) -> ForumResult<impl IntoResponse> {
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.update_session(&user_id)?;
    app_state.db_client.update_user_activity(user_id).await?;
    
    // Save uploaded image
    let filename = file_upload::save_uploaded_image(multipart, &app_state.env, user_id).await?;

    // Generate full URL for the avatar
    let avatar_url = file_upload::get_avatar_url(&app_state.env, &filename);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Avatar uploaded successfully",
        "avatar_url": avatar_url,
        "filename": filename
    })))
}

