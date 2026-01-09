use core::str;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::models::{User, UserRole};

pub fn validate_password(s: &str) -> Result<(), ValidationError> {
    let mut r: u16 = 0;
    let mut s_it = s.chars();
    while let Some(c) = s_it.next() {
        r |= match c {
            ':'..='@' => 1,
            '!'..='/' => 2,
            'a'..='z' => 4,
            'A'..='Z' => 8,
            '0'..='9' => 16,
            _ => 0,
        };
        if r>=29 && s.chars().count()>8 {
            return Ok(());
        }
    }
    Err(ValidationError::new("A password must be at least 8 characters long, contain lowercase, uppercase characters, numbers and special characters"))
}

// ----- ----- Requests ----- -----

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct RegisterUserDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Email is invalid")
    )]
    pub email: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,

    #[validate(
        length(min = 1, message = "Confirm Password is required"),
        must_match(other = "password", message="passwords do not match")
    )]
    #[serde(rename = "passwordConfirm")]
    pub password_confirm: String,
}

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoginUserDto {
    pub username: String,
    #[validate(length(min = 3))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct RequestQueryDto {
    #[validate(range(min = 1))]
    pub page: Option<usize>,
    #[validate(range(min = 1, max = 50))]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct FilterUserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(rename = "bannedUntil")]
    pub banned_until: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub facebook: Option<String>,
    pub discord: Option<String>,
    pub x_id: Option<String>,
}

impl FilterUserDto {
    pub fn filter_user(user: &User) -> Self {
        FilterUserDto {
            id: user.id.to_string(),
            name: user.name.to_owned(),
            email: user.email.to_owned(),
            verified: user.verified,
            role: user.role.to_str().to_string(),
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at,
            banned_until: user.banned_until,
            description: user.description.to_owned(),
            avatar: user.avatar.to_owned(),
            facebook: user.facebook.to_owned(),
            discord: user.discord.to_owned(),
            x_id: user.x_id.to_owned(),
        }
    }

    pub fn filter_users(user: &[User]) -> Vec<FilterUserDto> {
        user.iter().map(FilterUserDto::filter_user).collect()
    }

    pub fn is_banned(&self) -> bool {
        match self.banned_until.as_ref() {
            Some(b) => b.gt(&Utc::now()),
            None => false,
        }
    }
}

#[derive(Validate, Debug, Clone, Serialize, Deserialize)]
pub struct SearchUsersDto {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    #[validate(range(min = 1))]
    pub page: u32,
    #[validate(range(min = 1))]
    pub limit: usize,
}

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
pub struct NameUpdateDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
}

#[derive(Validate, Debug, Serialize, Deserialize)]
pub struct AddUserDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(email(message = "Email is invalid"))]
    pub email: String,
    #[validate(custom(function = "validate_password"))]
    pub password: String,
    pub password_confirm: String,
}

#[derive(Validate, Debug, Serialize, Deserialize)]
pub struct SaveUserDto {
    pub user: FilterUserDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RoleUpdateDto {
    pub role: UserRole,
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct UserPasswordUpdateDto {
    #[validate(custom(function = "validate_password"))]
    pub new_password: String,

    #[validate(
        length(min = 6, message = "new password confirm must be at least 6 characters"),
        must_match(other = "new_password", message="new passwords do not match")
    )]
    pub new_password_confirm: String,

    #[validate(
        length(min = 6, message = "Old password must be at least 6 characters")
    )]
    pub old_password: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct VerifyEmailQueryDto {
    #[validate(length(min = 1, message = "Token is required."),)]
    pub token: String,
}

#[derive(Deserialize, Serialize, Validate, Debug, Clone)]
pub struct ForgotPasswordRequestDto {
    #[validate(length(min = 1, message = "Email is required"), email(message = "Email is invalid"))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct ResetPasswordRequestDto {
    #[validate(length(min = 1, message = "Token is required."),)]
    pub token: String,

    #[validate(
        length(min = 6, message = "new password must be at least 6 characters")
    )]
    pub new_password: String,

    #[validate(
        length(min = 6, message = "new password confirm must be at least 6 characters"),
        must_match(other = "new_password", message="new passwords do not match")
    )]
    pub new_password_confirm: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct RecentlyOnlineDto {
    pub since: DateTime<Utc>,
    pub page: u32,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct WarnUserDto {
    pub uuid: uuid::Uuid,
    pub comment: Option<String>,
    pub warned_by: uuid::Uuid,
    pub banned: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct UnbanUserDto {
    pub uuid: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SendPmDto {
    pub recipient_id: uuid::Uuid,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct GetUserPmsDto {
    pub page: u32,
    pub limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct AvatarUpdateDto {
    #[validate(length(min = 1, message = "Avatar URL is required"))]
    pub avatar_url: String,
}

// ----- ----- Responses ----- -----

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserData {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub verified: bool,
    pub role: UserRole,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub facebook: Option<String>,
    pub discord: Option<String>,
    pub x_id: Option<String>,
    pub last_online: Option<DateTime<Utc>>,
    pub banned: Option<bool>,
    pub posts_n: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilteredUserData {
    pub status: String,
    pub user: FilterUserDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponseDto {
    pub status: String,
    pub data: UserData,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListResponseDto {
    pub status: String,
    pub users: Vec<FilterUserDto>,
    pub results: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginResponseDto {
    pub status: String,
    pub token: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPostsResponseDto {
    pub posts: Vec<crate::models::Post>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserThreadsResponseDto {
    pub threads: Vec<crate::models::Thread>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWarningsResponseDto {
    pub warnings: Vec<crate::models::UserWarning>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPmsResponseDto {
    pub pms: Vec<crate::models::PrivateMessage>,
}
