use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::user::me;

#[component]
pub fn OAuthCallback() -> Html {
    let ctx = use_context::<crate::UserContext>()
        .unwrap_throw();

    let navigator = use_navigator().expect("navigator missing!");
    let error = use_state(|| None::<String>);

    {
        let error = error.clone();
        let navigator = navigator.clone();
        let ctx = ctx.clone();

        use_effect(move || {

                // Fetch and store user data
                let error_clone = error.clone();
                let navigator_clone = navigator.clone();
                let ctx_clone = ctx.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    match me().await {
                        Ok(user_data) => {
                            ctx_clone.dispatch(Some(user_data.clone()));
                            navigator_clone.push(&crate::Route::Content);
                        }
                        Err(e) => {
                            let error_msg = e.as_string().unwrap_or_else(|| "Failed to fetch user data".to_string());
                            error_clone.set(Some(error_msg));
                        }
                    }
                });
           });
    }

    html! {
        <div id="oauth-callback" class="flex items-center justify-center min-h-screen">
            if let Some(err) = (*error).as_ref() {
                <div class="error">{err}</div>
            } else {
                <div>{"Processing OAuth callback..."}</div>
            }
        </div>
    }
}

fn extract_token_from_url(search: &str, hash: &str) -> Option<String> {
    // Try to extract token from query parameters (e.g., ?token=...)
    if !search.is_empty() {
        let params = web_sys::UrlSearchParams::new_with_str(search).ok()?;
        if let Some(token) = params.get("token") {
            if !token.is_empty() {
                return Some(token);
            }
        }
        // Also check for other common parameter names
        if let Some(token) = params.get("access_token") {
            if !token.is_empty() {
                return Some(token);
            }
        }
        if let Some(token) = params.get("jwt") {
            if !token.is_empty() {
                return Some(token);
            }
        }
    }

    // Try to extract token from fragment (e.g., #token=...)
    if !hash.is_empty() {
        // Remove the leading '#'
        let hash_str = if hash.starts_with('#') { &hash[1..] } else { hash };
        let params = web_sys::UrlSearchParams::new_with_str(hash_str).ok()?;
        if let Some(token) = params.get("token") {
            if !token.is_empty() {
                return Some(token);
            }
        }
        if let Some(token) = params.get("access_token") {
            if !token.is_empty() {
                return Some(token);
            }
        }
        if let Some(token) = params.get("jwt") {
            if !token.is_empty() {
                return Some(token);
            }
        }
    }

    None
}

