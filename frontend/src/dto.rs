use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use wasm_bindgen::{JsValue, UnwrapThrowExt};
use serde_wasm_bindgen::from_value;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Resp {
    pub status: String,
    pub message: String,
}

impl From<JsValue> for Resp {
    fn from(value: JsValue) -> Self {
        from_value(value)
            .unwrap_throw()
    }
}

// ----- Auth -----

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RegisterUserDto {
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "passwordConfirm")]
    pub password_confirm: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoginUserDto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginResponseDto {
    pub status: String,
    pub token: String,
    pub role: String,
}

impl From<JsValue> for UserLoginResponseDto {
    fn from(value: JsValue) -> Self {
        from_value(value)
            .unwrap_throw()
    }
}

// ----- User -----

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
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

#[derive(Deserialize)]
struct TmpUserDto {
    user: FilterUserDto,
}

impl From<JsValue> for FilterUserDto {
    fn from(value: JsValue) -> Self {
        let t: TmpUserDto = from_value(value)
            .unwrap_throw();
        t.user
        
    }
}

impl FilterUserDto {
    pub fn is_admin(&self) -> bool {
        self.role.to_lowercase().eq("admin")
    }

    pub fn is_mod(&self) -> bool {
        self.role.to_lowercase().eq("mod")
    }

    pub fn avatar(&self) -> String {
        self.avatar.clone()
            .unwrap_or(String::from("default.png"))
    }

    pub fn description(&self) -> String {
        self.description.clone()
            .unwrap_or(String::from("-"))
    }

    pub fn facebook(&self) -> String {
        self.facebook.clone().unwrap_or_default()
    }

    pub fn discord(&self) -> String {
        self.discord.clone().unwrap_or_default()
    }

    pub fn x_id(&self) -> String {
        self.x_id.clone().unwrap_or_default()
    }

    pub fn is_banned(&self) -> bool {
        if let Some(b) = self.banned_until.as_ref() {
            b.gt(&Utc::now())
        } else {
            false
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WarnUserDto {
    pub uuid: String,
    pub comment: Option<String>,
    pub warned_by: String,
    pub banned: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnbanUserDto {
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct UserData {
    pub id: String,
    pub name: String,
    pub email: String,
    pub verified: bool,
    pub role: String,
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

impl UserData {
    pub fn is_admin(&self) -> bool {
        self.role.to_lowercase().eq("admin")
    }

    pub fn is_mod(&self) -> bool {
        self.role.to_lowercase().eq("mod")
    }

    pub fn avatar(&self) -> String {
        self.avatar.clone()
            .unwrap_or(String::from("default.png"))
    }

    pub fn description(&self) -> String {
        self.description.clone()
            .unwrap_or(String::from("-"))
    }

    pub fn facebook(&self) -> String {
        self.facebook.clone().unwrap_or_default()
    }

    pub fn discord(&self) -> String {
        self.discord.clone().unwrap_or_default()
    }

    pub fn x_id(&self) -> String {
        self.x_id.clone().unwrap_or_default()
    }

    pub fn is_banned(&self) -> bool {
        self.banned
            .unwrap_or(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListResponseDto {
    pub status: String,
    pub users: Vec<FilterUserDto>,
    pub results: i64,
}

impl From<JsValue> for UserListResponseDto {
    fn from(value: JsValue) -> Self {
        from_value(value)
            .unwrap_throw()
    }
}


// ----- Forum -----

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Section {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub new_posts: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetSectionsResponseDto {
    pub sections: Vec<Section>,
}

impl From<JsValue> for GetSectionsResponseDto {
    fn from(value: JsValue) -> Self {
        from_value(value)
            .unwrap_throw()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ThreadListItemDto { 
    pub id: i64,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub author: String,
    pub author_name: String,
    pub section_id: i64,
    pub locked: bool,
    pub sticky: bool,

}

#[derive(Serialize, Deserialize)]
pub struct GetSectionResponseDto {
    pub threads: Vec<ThreadListItemDto>,
}

impl From<JsValue> for GetSectionResponseDto {
    fn from(value: JsValue) -> Self {
        from_value(value)
            .unwrap_throw()
    }
}
        
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CreateThreadDto {
    pub title: String,
    pub content: String,
    pub section: i64,
    pub hash_tags: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DeleteThreadDto {
    pub thread_id: i64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UpdateThreadDto {
    pub thread_id: i64,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LockThreadDto {
    pub thread_id: i64,
    pub locked: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CreateSectionDto {
    pub name: String,
    pub description: String,
    pub allowed_for: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DeleteSectionDto {
    s_id: i64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GetThreadDto {
    pub thread_id: i64,
    pub page: i32,
    pub limit: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UpdatePostDto {
    pub post_id: i64,
    pub content: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DeletePostDto {
    pub post_id: i64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GetThreadsDto {
    pub page: Option<i32>,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct ReplyThreadDto {
    pub post_id: Option<i64>,
    pub t_id: i64,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetThreadResponseDto {
    pub info: Thread,
    pub posts: Vec<Post>,
}

impl From<JsValue> for GetThreadResponseDto {
    fn from(value: JsValue) -> Self {
        from_value(value)
            .unwrap_throw()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Post {
    pub id: i64,
    pub content: String,
    pub author: Option<String>,
    pub author_name: Option<String>,
    pub topic: i64,
    pub comments: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub likes: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Thread {
    pub id: i64,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub author: String,
    pub section: i64,
    pub locked: bool,
    pub sticky: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct ImageUploadResponse {
    pub success: bool,
    pub message: String,
    pub avatar_url: String,
    pub filename: String,
}

impl From<JsValue> for ImageUploadResponse {
    fn from(value: JsValue) -> Self {
        from_value(value)
            .unwrap_throw()
    }
}

