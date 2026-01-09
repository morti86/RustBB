use std::sync::Arc;

use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::IntoResponse,
    Extension
};

use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    db::user::UserExt,
    error::{ForumError, ForumResult},
    models::{User, UserRole},
    utils::token,
    AppState
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddeware {
    pub user: User,
}

pub async fn auth(
    cookie_jar: CookieJar,
    app_state: Extension<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> ForumResult<impl IntoResponse> {
    let cookies = cookie_jar
            .get("token")
            .map(|cookie| cookie.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(header::AUTHORIZATION)
                    .and_then(|auth_header| auth_header.to_str().ok())
                    .and_then(|auth_value| {
                        if auth_value.starts_with("Bearer ") {
                            Some(auth_value[7..].to_owned())
                        } else {
                            None
                        }
                    })  
            });

    let token = cookies.ok_or(ForumError::Unauthorized)?;

    let token_details = 
        match token::decode_token(token, app_state.env.jwt_secret.as_bytes()) {
            Ok(token_details) => token_details,
            Err(_) => {
                return Err(ForumError::InvalidToken);
            }
        };

    let user_id = uuid::Uuid::parse_str(&token_details.to_string())
            .map_err(|_| {
                ForumError::InvalidToken
            })?;

    let user = app_state.db_client.get_user(Some(user_id), None, None)
            .await
            .map_err(|_| {
                ForumError::NoSuchUser(user_id.to_string())
            })?;

    let user = user.ok_or(ForumError::NoSuchUser(user_id.to_string()))?;

    req.extensions_mut().insert(JWTAuthMiddeware {
        user: user.clone(),
    });

    Ok(next.run(req).await)

}


pub async fn role_check(
    Extension(_app_state): Extension<Arc<AppState>>,
    req: Request,
    next: Next,
    required_roles: Vec<UserRole>,
) -> ForumResult<impl IntoResponse> {
    let user = req
            .extensions()
            .get::<JWTAuthMiddeware>()
            .ok_or(ForumError::Unauthorized)?;

    if !required_roles.contains(&user.user.role) {
        return Err(ForumError::Forbidden)
    }

    Ok(next.run(req).await)
}

pub async fn is_banned(
    Extension(_app_state): Extension<Arc<AppState>>,
    req: Request,
    next: Next) -> ForumResult<impl IntoResponse> {
    let user = req
        .extensions()
        .get::<JWTAuthMiddeware>()
        .ok_or(ForumError::Unauthorized)?;
    if user.user.is_banned() {
        Err(ForumError::Forbidden)
    } else {
        Ok(next.run(req).await)
    }
}
