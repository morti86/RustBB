use serde::{Serialize, Deserialize};
use wasm_bindgen::JsValue;
use chrono::{DateTime, Utc};
use serde_wasm_bindgen::from_value;

use crate::dto::{FilterUserDto, LoginUserDto, RegisterUserDto, UnbanUserDto, UserData, UserListResponseDto, UserLoginResponseDto, WarnUserDto};

use crate::bind::{get, post, put, set_cookie};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserSession {
    pub user_id: String,
    pub username: String,
    pub login_time: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl From<JsValue> for UserSession {
    fn from(value: JsValue) -> Self {
        let r = from_value(value);
        r.expect( "FE001: can't convert UserSession" )
    }
}

pub async fn me() -> Result<FilterUserDto, JsValue> {
    let user = get("/users/me").await?;
    let filter_user = FilterUserDto::from(user);
    Ok(filter_user)
}

pub async fn user(user_id: &str) -> Result<UserData, JsValue> {
    let addr = format!("/users/user/{}", user_id);
    let user = get(&addr).await?;
    let body = from_value(user)?;
    Ok(body)
}

pub async fn warn_user(user_id: &str, comment: Option<&str>, warned_by: &str, ban_len: Option<i32>) -> Result<(), JsValue> {
    let req = WarnUserDto {
        uuid: user_id.to_string(),
        comment: comment.map(|c: &str| c.to_string()),
        warned_by: warned_by.to_string(),
        banned: ban_len,
    };
    let body = serde_json::to_string(&req)
        .expect("sj");

    let _ = put("/users/warn", JsValue::from_str(&body)).await?;

    Ok(())
}

pub async fn unban_user(user_id: &str) -> Result<(), JsValue> {
    let req = UnbanUserDto {
        uuid: user_id.to_string(),
    };
    let body = serde_json::to_string(&req)
        .expect("sj");

    let _ = put("/users/unban", JsValue::from_str(&body)).await?;

    Ok(())
}

pub async fn update_user(user: &UserData) -> Result<(), JsValue> {
    let addr = format!("/users/user/{}", user.id);
    let body = serde_json::to_string(user)
        .expect("UU");
    post(&addr, JsValue::from_str(&body)).await?;
    Ok(())
}

pub async fn login(username: &str, password: &str) -> Result<UserLoginResponseDto, JsValue> {
    let request = LoginUserDto {
        username: username.to_string(),
        password: password.to_string(),
    };

    let body = serde_json::to_string(&request)
        .expect("SJ");

    let response = post("/auth/login", JsValue::from_str(&body)).await?;
    let resp = UserLoginResponseDto::from(response);

    let token = resp.token.clone();
    set_cookie("token", &token);
    Ok(resp)
}

pub async fn logout() -> Result<(), JsValue> {
    let body = String::from("{}");
    post("/auth/logout", JsValue::from_str(&body)).await?;

    Ok(())
}

pub async fn register(username: &str, email: &str, password: &str, password_confirm: &str) -> Result<(), JsValue> {
    let dto = RegisterUserDto {
        name: username.to_string(),
        email: email.to_string(),
        password: password.to_string(),
        password_confirm: password_confirm.to_string(),
    };
    let body = serde_json::to_string(&dto)
        .expect("SJ");

    let _res = post("/auth/register", JsValue::from_str(&body)).await?;
    
    Ok(())
}

pub async fn user_list(page: Option<usize>, limit: Option<usize>) -> Result<UserListResponseDto, JsValue> {
    let mut addr = "/users/list".to_string();
    let mut params = String::new();
    if let Some(page) = page {
        params.push_str(&format!("&page={}",page));
    }
    if let Some(limit) = limit {
        params.push_str(&format!("&limit={}", limit));
    }
    if !params.is_empty() {
        addr.push('?');
        addr.push_str(&params);
    }
    let users = get(&addr).await?;
    Ok(UserListResponseDto::from(users))
}
