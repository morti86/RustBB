use std::{collections::HashMap, rc::Rc};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FileReader, FormData, Headers, HtmlElement, Request, RequestInit, Response};
use crate::{ADDR, dto::ImageUploadResponse};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);

    #[wasm_bindgen(js_name = "fetch")]
    fn fetch_with_request(request: &web_sys::Request) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = console)]
    fn time(_: &str);

    #[wasm_bindgen(js_namespace = console, js_name = timeEnd)]
    fn time_end(_: &str);

    pub fn alert(s: &str);
}

#[macro_export]
macro_rules! c_log {
    ($($arg:tt)*) => {
        $crate::bind::log(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! c_error {
    ($($arg:tt)*) => {
        $crate::bind::error(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! c_alert {
    ($($arg:tt)*) => {
        $crate::bind::alert(&format!($($arg)*));
    };
}

#[wasm_bindgen]
pub async fn get(url: &str) -> Result<JsValue, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");

    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.set_credentials(web_sys::RequestCredentials::Include);
    
    let url = format!("{}{}", ADDR, url);
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| JsValue::from_str(&format!("Request creation failed: {:?}", e)))?;
    let window = web_sys::window()
        .expect("FE002: no window ^o.O^");
    let promise = window.fetch_with_request(&request);
    let response = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let response: Response = response.into();

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let json = wasm_bindgen_futures::JsFuture::from(response.json()?).await?;
    Ok(json)
}

#[wasm_bindgen]
pub async fn post(url: &str, body: JsValue) -> Result<JsValue, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("POST");

    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    opts.set_credentials(web_sys::RequestCredentials::Include);
    opts.set_headers(&headers.into());
    opts.set_body(&body);

    c_log!("001");
    let url = format!("{}{}", ADDR, url);
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| JsValue::from_str(&format!("Request creation failed: {:?}", e)))?;

    let window = web_sys::window()
        .expect("FE002: no window ^o.O^");
    let promise = window.fetch_with_request(&request);
    let response = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let response: Response = response.into();

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    let json = wasm_bindgen_futures::JsFuture::from(response.json()?).await?;
    Ok(json)
}

#[wasm_bindgen]
pub async fn put(url: &str, body: JsValue) -> Result<(), JsValue> {
    let opts = RequestInit::new();
    opts.set_method("PUT");

    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;

    opts.set_credentials(web_sys::RequestCredentials::Include);
    opts.set_headers(&headers.into());
    opts.set_body(&body);

    let url = format!("{}{}", ADDR, url);
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| JsValue::from_str(&format!("Request creation failed: {:?}", e)))?;

    let window = web_sys::window()
        .expect("FE002: no window ^o.O^");
    let promise = window.fetch_with_request(&request);
    let response = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let response: Response = response.into();

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    Ok(())
}

#[wasm_bindgen]
pub async fn delete(url: &str, body: JsValue) -> Result<(), JsValue> {
    let opts = RequestInit::new();
    opts.set_method("DELETE");

    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;

    opts.set_credentials(web_sys::RequestCredentials::Include);
    opts.set_headers(&headers.into());
    opts.set_body(&body);

    let url = format!("{}{}", ADDR, url);
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| JsValue::from_str(&format!("Request creation failed: {:?}", e)))?;

    let window = web_sys::window()
        .expect("FE002: no window ^o.O^");
    let promise = window.fetch_with_request(&request);
    let response = wasm_bindgen_futures::JsFuture::from(promise).await?;
    let response: Response = response.into();

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    Ok(())

}

fn parse_cookies() -> Option<HashMap<String, String>> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let html = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
    let cookies =  html.cookie()
        .expect("failed to get cookie");
    
    let mut cookie_map = HashMap::new();
    for cookie in cookies.split(';') {
        let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
        if parts.len() == 2 {
            cookie_map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    
    Some(cookie_map)
}

pub fn set_cookie(key: &str, value: &str) {
    let window = web_sys::window()
        .expect("set_cookie -> window fail");
    let document = window.document()
        .expect("set_cookie -> doc fail");
    let html = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
    let value = format!("{}={}", key, value);
    html.set_cookie(&value)
        .expect("failed to set cookie");
}

pub fn del_cookie(key: &str) {
    let window = web_sys::window()
        .expect("set_cookie -> window fail");
    let document = window.document()
        .expect("set_cookie -> doc fail");
    let html = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
    let value = format!("{}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;", key);
    html.set_cookie(&value)
        .expect("failed del cookie");
    window.location().reload()
        .expect("reload failed in del_cookie");
}

pub fn get_jwt_from_cookie(cookie_name: &str) -> Option<String> {
    parse_cookies()
        .and_then(|cookies| cookies.get(cookie_name).cloned())
}

fn create_element(tag: &str) -> Result<HtmlElement, JsValue> {
    let window = web_sys::window()
        .expect("ce: expect window");
    let document = window.document()
        .expect("ce: expect doc");
    let element = document.create_element(tag)?;
    Ok(element.dyn_into::<HtmlElement>().unwrap())
}

#[wasm_bindgen]
pub async fn read_file_as_bytes(file: &File) -> Result<Vec<u8>, JsValue> {
    let file_reader = Rc::new(FileReader::new()?);
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let f_c = file_reader.clone();
        let resolve_clone = resolve.clone();
        let reject_clone = reject.clone();
        
        let on_load = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let result = f_c.result();
            resolve_clone.call1(&JsValue::NULL, &result.unwrap()).unwrap();
        }) as Box<dyn FnMut(web_sys::Event)>);
        
        file_reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
        on_load.forget();
        
        let on_error = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            reject_clone.call0(&JsValue::NULL).unwrap();
        }) as Box<dyn FnMut(web_sys::Event)>);
        
        file_reader.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();
    });
    
    let _ = file_reader.read_as_array_buffer(file);
    let array_buffer = wasm_bindgen_futures::JsFuture::from(promise).await?;
    
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let mut bytes = vec![0; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);
    
    Ok(bytes)
}

pub async fn upload_file_with_fetch(url: &str, file: &File) -> Result<ImageUploadResponse, JsValue> {
    let addr = format!("{}{}", ADDR, url);
    let window = web_sys::window().unwrap();

    let form_data = FormData::new()?;
    form_data.append_with_blob("file", &file)?;

    // Create fetch request
    let opts = RequestInit::new();
    opts.set_method("POST");
    // Don't set Content-Type header - FormData will set it automatically with boundary
    // Only set Authorization header
    let headers = Headers::new()?;
    opts.set_credentials(web_sys::RequestCredentials::Include);
    opts.set_headers(&headers);
    opts.set_body(&form_data);

    let request = Request::new_with_str_and_init(&addr, &opts)?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: web_sys::Response = resp_value.dyn_into()?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let json = wasm_bindgen_futures::JsFuture::from(response.json()?).await?;

    Ok(ImageUploadResponse::from(json))
}
