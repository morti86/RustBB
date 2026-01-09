use std::sync::Arc;

use axum::{Extension, Json, Router, extract::Query, http::{HeaderMap, StatusCode, header}, middleware::from_fn, response::{IntoResponse, Redirect}, routing::{get, post}};
use axum_extra::extract::cookie::Cookie;
use chrono::{Utc, Duration};
use time::OffsetDateTime;
use tracing::error;
use validator::Validate;

use crate::{AppState, db::user::UserExt, dto::{Response, user}, error::{ForumError, ForumResult}, mail::mails::{send_forgot_password_email, send_verification_email, send_welcome_email}, middleware::JWTAuthMiddeware, utils::{password, token}};

pub fn auth_handler() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout).layer(from_fn(crate::middleware::auth)))
        .route("/verify", get(verify_email))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}

pub async fn register(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<user::RegisterUserDto>
) -> ForumResult<impl IntoResponse> {
    body.validate()?;

    let verification_token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);

    let hash_password = password::hash(&body.password)?;

    let result = app_state.db_client
        .add_user(&body.name, &body.email, &hash_password, &verification_token, expires_at)
        .await;

    match result {
        Ok(_) => {
            let verify = app_state.env.email_verification;
            if verify {
                let send_email_result = send_verification_email(&body.email, &body.name, &verification_token).await;

                if let Err(e) = send_email_result {
                    error!("Failed to send verification email: {}", e);
                }
            } else {
                let _ = app_state.db_client.verifed_token(verification_token.as_str());
            }

            Ok((StatusCode::CREATED, Json(Response {
                status: "success",
                message: format!("{}{}", "Registration successful!", if verify { "Please check your email to verify your account." } else { "" })
            })))
        },
        Err(e) => Err(ForumError::ServerError(e.to_string()))
    }
}

pub async fn login(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<user::LoginUserDto>
) -> ForumResult<impl IntoResponse> {
    body.validate()?;

    let result = app_state.db_client
        .get_user(None, Some(&body.username), None)
        .await?;

    let user = result.ok_or(
        ForumError::Unauthorized
    )?;

    if user.is_banned() {
        return Err(ForumError::Banned);
    }

    let password_matched = password::compare(&body.password, &user.password)?;

    if password_matched {
        let token = token::create_token(
            &user.id, 
            &app_state.env.jwt_secret.as_bytes(), 
            app_state.env.jwt_maxage
        )?;

        let cookie_duration = time::Duration::minutes(app_state.env.jwt_maxage * 60);
        let cookie = Cookie::build(("token", token.clone()))
            .path("/")
            .max_age(cookie_duration)
            .http_only(true)
            .build();

        let response = axum::response::Json(user::UserLoginResponseDto {
            role: user.role,
            status: "success".to_string(),
            token: String::new(),
        });

        let mut headers = HeaderMap::new();

        headers.append(
            header::SET_COOKIE,
            cookie.to_string().parse().unwrap(), 
        );

        let mut response = response.into_response();
        response.headers_mut().extend(headers);

        Ok(response)
    } else {
        Err(ForumError::Auth(String::from("")))
    }
}

pub async fn logout(
    Extension(_app_state): Extension<Arc<AppState>>,
    Extension(_user): Extension<JWTAuthMiddeware>,
) -> ForumResult<impl IntoResponse> {

    let cookie = Cookie::build(("token", String::new()))
        .path("/")
        .expires(OffsetDateTime::now_utc())
        .http_only(true)
        .build();
    let response = axum::response::Json(user::UserLoginResponseDto {
            role: crate::models::UserRole::User,
            status: "success".to_string(),
            token: String::new(),
        });

    let mut headers = HeaderMap::new();
    headers.append(
            header::SET_COOKIE,
            cookie.to_string().parse().unwrap(), 
        );
    let mut response = response.into_response();
    response.headers_mut().extend(headers);

    Ok(response)
}

pub async fn verify_email(
    Query(query_params): Query<user::VerifyEmailQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>
) -> ForumResult<impl IntoResponse> {
    query_params.validate()?;

    let result = app_state.db_client
        .get_user(None, None, Some(&query_params.token))
        .await?;

    let user = result.ok_or( ForumError::Token(String::from("Invalid")) )?;

    if let Some(expires_at) = user.token_expires_at {
        if Utc::now() > expires_at {
            return Err(ForumError::Token(String::from("Verification token expired")) )?;
        }
    } else {
        return Err(ForumError::Token(String::from("Verification token invalid")) )?;
    }

    app_state.db_client.verifed_token(&query_params.token).await?;

    let send_welcome_email_result = send_welcome_email(&user.email, &user.name).await;

    if let Err(e) = send_welcome_email_result {
        eprintln!("Failed to send welcome email: {}", e);
    }

    let token = token::create_token(
        &user.id, 
        app_state.env.jwt_secret.as_bytes(),
        app_state.env.jwt_maxage 
    )?;

    let cookie_duration = time::Duration::minutes(app_state.env.jwt_maxage * 60);
    let cookie = Cookie::build(("token", token.clone()))
        .path("/")
        .max_age(cookie_duration)
        .http_only(true)
        .build();

    let mut headers = HeaderMap::new();

    headers.append(
        header::SET_COOKIE,
        cookie.to_string().parse().unwrap() 
    );

    let frontend_url = format!("{}/settings", app_state.env.host_url);

    let redirect = Redirect::to(&frontend_url);

    let mut response = redirect.into_response();

    response.headers_mut().extend(headers);

    Ok(response)
}

pub async fn forgot_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<user::ForgotPasswordRequestDto>
) -> ForumResult<impl IntoResponse> {
    body.validate()?;

    let result = app_state.db_client
        .get_user(None, Some(&body.email), None)
        .await?;

    let user = result.ok_or( ForumError::Token(String::from("Invalid")) )?;

    let verification_token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(30);

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.db_client
        .add_verifed_token(user_id, &verification_token, expires_at)
        .await?;

    let reset_link = format!("{}/reset-password?token={}", app_state.env.host_url, &verification_token);

    let email_sent = send_forgot_password_email(&user.email, &reset_link, &user.name).await;

    if let Err(e) = email_sent {
        tracing::error!("Failed to send forgot password email: {}", e);
        return Err(ForumError::ServerError("Failed to send email".to_string()));
    }

    let response = Response {
        message: "Password reset link has been sent to your email.".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub async fn reset_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<user::ResetPasswordRequestDto>
) -> ForumResult<impl IntoResponse> {
    body.validate()?;

    let result = app_state.db_client
        .get_user(None, None, Some(&body.token))
        .await?;
    let user = result.ok_or( ForumError::Token(String::from("Invalid")) )?;

    if let Some(expires_at) = user.token_expires_at {
        if Utc::now() > expires_at {
            return Err(ForumError::Token(String::from("Verification token expired")) )?;
        }
    } else {
        return Err(ForumError::Token(String::from("Verification token invalid")) )?;
    }

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let hash_password = password::hash(&body.new_password)?;

    app_state.db_client
        .update_user_password(user_id.clone(), hash_password.as_str())
        .await?;

    app_state.db_client
        .verifed_token(&body.token)
        .await?;

    let response = Response {
        message: "Password has been successfully reset.".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

