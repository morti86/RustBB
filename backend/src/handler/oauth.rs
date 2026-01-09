use crate::{AppState, models::{User, UserRole}, utils::token::create_token};

use async_trait::async_trait;
use axum::{
    Json, Router, extract::{Query, State}, http::StatusCode, response::{IntoResponse, Redirect},
    routing::get
};
use axum_extra::{extract::cookie::PrivateCookieJar};
use http::{HeaderMap, header::SET_COOKIE};
use serde::Deserialize;
use crate::{error::{ForumError, ForumResult}, 
    dto::Response,
    oauth::{FacebookUserInfo, GoogleUserInfo, DiscordUserInfo}};
use axum_extra::extract::cookie::Cookie;

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    state: String,
}

#[async_trait]
pub trait OauthExt {
    async fn find_or_create_user_from_google(&self, user_info: &GoogleUserInfo) -> ForumResult<User>;
    async fn find_or_create_user_from_facebook(&self, user_info: &FacebookUserInfo) -> ForumResult<User>;
    async fn find_or_create_user_from_discord(&self, user_info: &DiscordUserInfo) -> ForumResult<User>;
}

pub fn auth_router() -> Router<AppState> { 
    Router::new()
        .route("/google", get(google_auth_start))
        .route("/google/callback", get(google_auth_callback))
        .route("/facebook", get(facebook_auth_start))
        .route("/facebook/callback", get(facebook_auth_callback))
        .route("/discord", get(discord_auth_start))
        .route("/discord/callback", get(discord_auth_callback))
}

#[async_trait]
impl OauthExt for crate::db::DBClient {

    async fn find_or_create_user_from_google(
        &self,
        user_info: &GoogleUserInfo)
        -> ForumResult<User> {
        // First, try to find user by oauth_uid
        let existing_user = sqlx::query_as!(User, 
            r#"
            SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                   role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
            FROM forum.users
            WHERE oauth_provider = 'google' AND oauth_uid = $1
            "#, user_info.sub)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(user) = existing_user {
            return Ok(user);
        }

        // If not found, try to find by email
        let existing_user_by_email = sqlx::query_as!(User, 
            r#"
            SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                   role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
            FROM forum.users WHERE email = $1
            "#, user_info.email)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(_user) = existing_user_by_email {
            // Update existing user with OAuth info
            let updated_user = sqlx::query_as!(User, 
                r#"
                UPDATE forum.users
                SET oauth_provider = 'google', oauth_uid = $1
                WHERE email = $2
                RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                   role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
                "#, user_info.sub, user_info.email)
                .fetch_one(&self.pool)
                .await?;

            return Ok(updated_user);
        }

        // Create new user
        let username = user_info.name.clone().unwrap_or_else(|| {
            user_info.email.split('@').next().unwrap_or("user").to_string()
        });

        let new_user = sqlx::query_as!(User, 
            r#"
            INSERT INTO forum.users (name, email, oauth_provider, oauth_uid, role)
            VALUES ($1, $2, 'google', $3, $4)
            RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                   role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
            "#, username, user_info.email, user_info.sub, UserRole::User as UserRole)
            .fetch_one(&self.pool)
            .await?;

        Ok(new_user)
}

async fn find_or_create_user_from_facebook(
    &self,
    user_info: &FacebookUserInfo,
) -> ForumResult<User> {
    // First, try to find user by oauth_uid
    let existing_user = sqlx::query_as!(User, 
        r#"
        SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                   role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
        FROM forum.users
        WHERE oauth_provider = 'facebook' AND oauth_uid = $1
        "#, user_info.id)
        .fetch_optional(&self.pool)
        .await?;

    if let Some(user) = existing_user {
        return Ok(user);
    }

    // If not found, try to find by email
    if let Some(email) = &user_info.email {
        let existing_user_by_email = sqlx::query_as!(User,
            r#"
            SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                   role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
            FROM forum.users WHERE email = $1
            "#, email)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(user) = existing_user_by_email {
            // Update existing user with OAuth info
            let updated_user = sqlx::query_as!(User,
                r#"
                UPDATE forum.users
                SET oauth_provider = 'facebook', oauth_uid = $1
                WHERE id = $2
                RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                   role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
                "#, user_info.id, user.id)
                .fetch_one(&self.pool)
                .await?;

            return Ok(updated_user);
        }
    }

    // Create new user
    let email = user_info.email.clone().unwrap_or_else(|| {
        String::from("")
    });

    let new_user = sqlx::query_as!(User,
        r#"
        INSERT INTO forum.users (name, email, oauth_provider, oauth_uid)
        VALUES ($1, $2, 'facebook', $3)
        RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                   role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
        "#, user_info.name, email, user_info.id)
        .fetch_one(&self.pool)
        .await?;

    Ok(new_user)
}

async fn find_or_create_user_from_discord(
    &self,
    user_info: &DiscordUserInfo,
) -> ForumResult<User> {
    // First, try to find user by oauth_uid
    let existing_user = sqlx::query_as!(User, 
        r#"
        SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
            role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
        FROM forum.users
        WHERE oauth_provider = 'discord' AND oauth_uid = $1
        "#, user_info.id)
        .fetch_optional(&self.pool)
        .await?;

    if let Some(user) = existing_user {
        return Ok(user);
    }

    // If not found, try to find by email
    if let Some(email) = &user_info.email {
        let existing_user_by_email = sqlx::query_as!(User, 
            r#"
            SELECT id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
            role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
            FROM forum.users WHERE email = $1
            "#, email)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(user) = existing_user_by_email {
            // Update existing user with OAuth info
            let updated_user = sqlx::query_as!(User, 
                r#"
                UPDATE forum.users
                SET oauth_provider = 'discrd', oauth_uid = $1
                WHERE id = $2
                RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
                    role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
                "#, user_info.id, user.id)
                .fetch_one(&self.pool)
                .await?;

            return Ok(updated_user);
        }
    }

    // Create new user
    let email = user_info.email.clone().unwrap_or_else(|| {
        format!("Discord / {}", user_info.global_name)
    });

    let new_user = sqlx::query_as!(User, 
        r#"
        INSERT INTO forum.users (name, email, oauth_provider, oauth_uid, role)
        VALUES ($1, $2, 'discord', $3, $4)
        RETURNING id, name, email, password, verified, created_at, updated_at, verification_token, token_expires_at, 
            role as "role: UserRole", description, avatar, facebook, discord, x_id, banned_until, last_online, oauth_access_token, oauth_refresh_token, oauth_provider, oauth_uid
        "#, user_info.username, email, user_info.id, UserRole::User as UserRole)
        .fetch_one(&self.pool)
        .await?;

    Ok(new_user)
}
}

pub async fn google_auth_start(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> impl IntoResponse {
    match state.oauth_service.get_google_auth_url() {
        Some((auth_url, csrf_token, pkce_verifier)) => {

            // Store CSRF token and PKCE verifier in session cookies
            let csrf_token_secret = csrf_token.secret();
            let jar = jar
                .add(axum_extra::extract::cookie::Cookie::new(
                    format!("csrf_token_{}", csrf_token_secret),
                    csrf_token_secret.to_string(),
                ))
                .add(axum_extra::extract::cookie::Cookie::new(
                    format!("pkce_verifier_{}", csrf_token_secret),
                    pkce_verifier.secret().to_string(),
                ));

            // Return redirect with cookies
            (jar, Redirect::to(auth_url.as_str())).into_response()
        }
        None => {
            (StatusCode::NOT_IMPLEMENTED,"Google OAuth not configured".to_string()).into_response()
        }
    }
}

pub async fn discord_auth_start(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> impl IntoResponse {
    match state.oauth_service.get_discord_auth_url() {
        Some((auth_url, csrf_token)) => {

            // Store CSRF token verifier in session cookies
            let csrf_token_secret = csrf_token.secret();
            let jar = jar
                .add(Cookie::new(
                    format!("csrf_token_{}", csrf_token_secret),
                    csrf_token_secret.to_string(),
                ));

            // Return redirect with cookies
            (jar, Redirect::to(auth_url.as_str())).into_response()
        }
        None => {
            (StatusCode::NOT_IMPLEMENTED,"Discord OAuth not configured".to_string()).into_response()
        }
    }
}

#[axum::debug_handler]
pub async fn google_auth_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthCallbackQuery>,
) -> ForumResult<Json<Response>> {
    // Retrieve stored CSRF token and PKCE verifier from session cookies
    let csrf_token_cookie_name = format!("csrf_token_{}", query.state);
    let pkce_verifier_cookie_name = format!("pkce_verifier_{}", query.state);

    let csrf_token_cookie = jar.get(&csrf_token_cookie_name)
        .ok_or_else(|| ForumError::from((StatusCode::BAD_REQUEST, "CSRF token not found in session")))?;

    let pkce_verifier_cookie = jar.get(&pkce_verifier_cookie_name)
        .ok_or_else(|| ForumError::from((StatusCode::BAD_REQUEST, "PKCE verifier not found in session")))?;

    // Validate CSRF token
    if csrf_token_cookie.value() != query.state {
        return Err((StatusCode::BAD_REQUEST, "CSRF token mismatch").into());
    }

    // Create PKCE verifier from stored value
    let pkce_verifier = oauth2::PkceCodeVerifier::new(pkce_verifier_cookie.value().to_string());

    // Exchange authorization code for token
    match state.oauth_service.exchange_google_code(query.code, pkce_verifier).await {
        Ok(user_info) => {
            // Find or create user in database

            let user = state.db_client.find_or_create_user_from_google(&user_info).await?;

            let secret = env!("JWT_SECRET_KEY");
            let expires_in_seconds = env!("JWT_MAXAGE").parse::<i64>().unwrap_or(60);
            let token = create_token(&user.id, secret.as_bytes(), expires_in_seconds)?;
            // Generate JWT token

            let cookie_duration = time::Duration::minutes(state.env.jwt_maxage * 60);
            let cookie = Cookie::build(("token", token.clone()))
                .path("/")
                .max_age(cookie_duration)
                .http_only(true)
                .build();
            let response = axum::response::Json(crate::dto::user::UserLoginResponseDto {
                role: user.role,
                status: "success".to_string(),
                token,
            });
            let mut headers = HeaderMap::new();

            headers.append(
                SET_COOKIE,
                cookie.to_string().parse().unwrap(), 
            );

            let mut response = response.into_response();
            response.headers_mut().extend(headers);


            Ok(Json(Response { status: "success", message: user.id.to_string() }))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string()).into()),
    }
}

pub async fn facebook_auth_start(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> impl IntoResponse {
    match state.oauth_service.get_facebook_auth_url() {
        Some((auth_url, csrf_token)) => {
            // Store CSRF token in session cookie
            let csrf_token_secret = csrf_token.secret();
            let jar = jar.add(Cookie::new(
                format!("csrf_token_{}", csrf_token_secret),
                csrf_token_secret.to_string(),
            ));

            // Return redirect with cookies
            (jar, Redirect::to(auth_url.as_str())).into_response()
        }
        None => (
            StatusCode::NOT_IMPLEMENTED,
            "Facebook OAuth not configured".to_string(),
        ).into_response(),
    }
}

#[axum::debug_handler]
pub async fn facebook_auth_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthCallbackQuery>,
) -> ForumResult<Json<Response>> {
    // Retrieve stored CSRF token from session cookies
    let csrf_token_cookie_name = format!("csrf_token_{}", query.state);

    let csrf_token_cookie = jar.get(&csrf_token_cookie_name)
        .ok_or_else(|| ForumError::from((StatusCode::BAD_REQUEST, "CSRF token not found in session")))?;

    // Validate CSRF token
    if csrf_token_cookie.value() != query.state {
        return Err((StatusCode::BAD_REQUEST, "CSRF token mismatch").into());
    }

    match state.oauth_service.exchange_facebook_code(query.code).await {
        Ok(user_info) => {
            // Find or create user in database
            let user = state.db_client.find_or_create_user_from_facebook(&user_info).await?;

            // Generate JWT token
            let secret = env!("JWT_SECRET_KEY");
            let expires_in_seconds = env!("JWT_MAXAGE").parse::<i64>().unwrap_or(60);
            let token = create_token(&user.id, secret.as_bytes(), expires_in_seconds)?;

            let cookie_duration = time::Duration::minutes(state.env.jwt_maxage * 60);
            let cookie = Cookie::build(("token", token.clone()))
                .path("/")
                .max_age(cookie_duration)
                .http_only(true)
                .build();
            let response = axum::response::Json(crate::dto::user::UserLoginResponseDto {
                role: user.role,
                status: "success".to_string(),
                token,
            });
            let mut headers = HeaderMap::new();

            headers.append(
                SET_COOKIE,
                cookie.to_string().parse().unwrap(), 
            );

            let mut response = response.into_response();
            response.headers_mut().extend(headers);

            Ok(Json(Response { status: "success", message: user.id.to_string() }))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string()).into()),
    }
}

#[axum::debug_handler]
pub async fn discord_auth_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthCallbackQuery>,
) -> ForumResult<Json<Response>> {
    // Retrieve stored CSRF token from session cookies
    let csrf_token_cookie_name = format!("csrf_token_{}", query.state);

    let csrf_token_cookie = jar.get(&csrf_token_cookie_name)
        .ok_or_else(|| ForumError::from((StatusCode::BAD_REQUEST, "CSRF token not found in session")))?;

    // Validate CSRF token
    if csrf_token_cookie.value() != query.state {
        return Err((StatusCode::BAD_REQUEST, "CSRF token mismatch").into());
    }

    match state.oauth_service.exchange_discord_code(query.code).await {
        Ok(user_info) => {
            // Find or create user in database
            let user = state.db_client.find_or_create_user_from_discord(&user_info).await?;

            // Generate JWT token
            let secret = env!("JWT_SECRET_KEY");
            let expires_in_seconds = env!("JWT_MAXAGE").parse::<i64>().unwrap_or(60);
            let token = create_token(&user.id, secret.as_bytes(), expires_in_seconds)?;

            let cookie_duration = time::Duration::minutes(state.env.jwt_maxage * 60);
            let cookie = Cookie::build(("token", token.clone()))
                .path("/")
                .max_age(cookie_duration)
                .http_only(true)
                .build();
            let response = axum::response::Json(crate::dto::user::UserLoginResponseDto {
                role: user.role,
                status: "success".to_string(),
                token,
            });
            let mut headers = HeaderMap::new();

            headers.append(
                SET_COOKIE,
                cookie.to_string().parse().unwrap(), 
            );

            let mut response = response.into_response();
            response.headers_mut().extend(headers);

            Ok(Json(Response { status: "success", message: user.id.to_string() }))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string()).into()),
    }
}
