use serde::{Serialize, Deserialize};
use wasm_bindgen::JsValue;
use web_sys::{Headers, RequestInit};
use chrono::{DateTime, Utc};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::UnwrapThrowExt;

use crate::{bind::{delete, get, post, put}, c_log, dto::{CreateSectionDto, CreateThreadDto, DeletePostDto, GetSectionResponseDto, GetSectionsResponseDto, GetThreadResponseDto, ReplyThreadDto, Section, ThreadListItemDto, UpdatePostDto, UpdateThreadDto}};

pub async fn get_sections() -> Result<Vec<Section>, JsValue> {
    let sections = get("/forum/list").await?;
    let response = GetSectionsResponseDto::from(sections);
    Ok(response.sections)
}

pub async fn get_topics(section_id: i64, page: Option<i32>, limit: Option<usize>) -> Result<Vec<ThreadListItemDto>, JsValue> {    
    
    let mut addr = format!("/forum/section/{}",section_id);
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
    let threads = get(&addr).await?;
    let response = GetSectionResponseDto::from(threads);
    Ok(response.threads)
}

pub async fn get_thread(thread_id: i64, page: Option<i32>, limit: Option<usize>) -> Result<GetThreadResponseDto, JsValue> {
    let mut addr = format!("/forum/threads/{}",thread_id);
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
    let thread = get(&addr).await?;
    let response = GetThreadResponseDto::from(thread);
    Ok(response)
}

pub async fn add_post(thread_id: i64, content: &str) -> Result<(), JsValue> {
    let addr = format!("/forum/post/new");
    let rt = ReplyThreadDto { post_id: None, t_id: thread_id, content: content.to_string() };
    let body = serde_json::to_string(&rt)
        .unwrap_throw();
    post(&addr, JsValue::from_str(body.as_str())).await?;
    Ok(())
}

pub async fn edit_post(post_id: i64, content: &str) -> Result<(), JsValue> {
    let dto = UpdatePostDto {
        post_id,
        content: content.to_string(),
    };
    let body = serde_json::to_string(&dto)
        .unwrap_throw();

    put("/forum/post", JsValue::from_str(body.as_str())).await?;
    Ok(())
}

pub async fn edit_thread(thread_id: i64, title: &str, content: &str) -> Result<(), JsValue> {
    let dto = UpdateThreadDto {
        thread_id,
        title: title.to_string(),
        content: content.to_string(),
    };
    let body = serde_json::to_string(&dto)
        .unwrap_throw();

    put("/forum/threads", JsValue::from_str(body.as_str())).await?;
    Ok(())
}

pub async fn new_thread(title: &str, content: &str, section: i64, hash_tags: Vec<String>) -> Result<(), JsValue> {
    let dto = CreateThreadDto {
        title: title.to_string(),
        content: content.to_string(),
        section,
        hash_tags,
    };
    let body = serde_json::to_string(&dto)
        .unwrap_throw();

    post("/forum/threads/new", JsValue::from_str(body.as_str())).await?;

    Ok(())
}

pub async fn delete_post(post_id: i64) -> Result<(), JsValue> {
    let dto = DeletePostDto {
        post_id
    };
    let body = serde_json::to_string(&dto)
        .unwrap_throw();
    delete("/forum/post", JsValue::from_str(body.as_str())).await?;
    Ok(())

}

pub async fn create_section(dto: &CreateSectionDto) -> Result<(), JsValue> {
    //let dto = CreateSectionDto {
    //    name: name.to_string(),
    //    description: description.to_string(),
    //    allowed_for: allowed_for.to_vec(),
    //};
    let body = serde_json::to_string(dto)
        .unwrap_throw();

    put("/forum/section/add", JsValue::from_str(body.as_str())).await?;
    Ok(())
}
