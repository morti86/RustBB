use wasm_bindgen::UnwrapThrowExt;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::{c_log, user::{login, me}};

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub on_oauth_start: Callback<()>,
}

#[component]
pub fn Login(props: &Props) -> Html {
    let ctx = use_context::<crate::UserContext>()
        .unwrap_throw();

    let navigator = use_navigator().expect("navigator missing!");
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error = use_state(|| None::<String>);

    let on_google_click = {
        let on_login_start = props.on_oauth_start.clone();
        Callback::<MouseEvent>::from(move |_| {
            on_login_start.emit(());
            let _ = web_sys::window()
                .unwrap()
                .location()
                .set_href("/auth/google"); // Matches your Axum route
        })
    };

    let on_facebook_click = {
        let on_login_start = props.on_oauth_start.clone();
        Callback::<MouseEvent>::from(move |_| {
            on_login_start.emit(());
            let _ = web_sys::window()
                .unwrap()
                .location()
                .set_href("/auth/facebook");
        })
    };

    let on_discord_click = {
        let on_login_start = props.on_oauth_start.clone();
        Callback::<MouseEvent>::from(move |_| {
            on_login_start.emit(());
            let _ = web_sys::window()
                .unwrap()
                .location()
                .set_href("/auth/discord");
        })
    };

    let c_c = ctx.clone();
    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let ctx = c_c.clone();
            let current_username = (*username).clone();
            let current_password = (*password).clone();

            if current_username.is_empty() || current_password.is_empty() {
                error.set(Some("Both fields are required".to_string()));
                return;
            }

            // Clear previous errors
            error.set(None);

            // Create a callback for the async login operation
            let username_clone = current_username.clone();
            let password_clone = current_password.clone();
            let error_clone = error.clone();
            let navigator_clone = navigator.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match login(&username_clone, &password_clone).await {
                    Ok(_res) => {
                        // Set token to state
                        
                        // Fetch and store user data
                        match me().await {
                            Ok(user_data) => {
                                c_log!("Me returned data");
                                ctx.dispatch(Some(user_data.clone()));
			    }
                            Err(e) => {
                                let error_msg = e.as_string().unwrap_or_else(|| "Failed to fetch user data".to_string());
                                error_clone.set(Some(error_msg));
                                return;
                            }
                        }
                        navigator_clone.push(&crate::Route::Content);
                    }
                    Err(e) => {
                        // Show error
                        let error_msg = e.as_string().unwrap_or_else(|| "Login failed".to_string());
                        error_clone.set(Some(error_msg));
                    }
                }
            });
        })
    };

    let on_username_input = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_password_input = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };


    let c_c = ctx.is_some();
    match c_c {
        true => {
            let c_c = ctx.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                if c_c.is_none() {
                    match me().await {
                        Ok(u) => {
                            c_log!("Token fine");
                            c_c.dispatch(Some(u));
                        }
                        Err(_e) => {
                            c_log!("Clearing token");
                            c_c.dispatch(None);
                        }
                    }
                }
            });

            let name = ctx.user.as_ref()
                .map(|u| u.name.clone())
                .unwrap_or(String::from("no ctx!!!"));
            return html! {
                <div id="user_data">{"welcome back "}<span class="user-name">{name}</span>
                </div>
            }
        }
        false => {
            html! {
                <div id="login-form" class="flex items-center justify-center flex-auto">
                    <form id="login" onsubmit={on_submit}>
                        if let Some(err) = (*error).as_ref() {
                            <div class="error">{err}</div>
                        }
                        <label for="username">{"username:"}</label><br/>
                        <input type="text"
                            id="username"
                            class="text-violet-700 bg-pink-100"
                            required=true
                            oninput={on_username_input}
                            value={(*username).clone()}
                            /><br/>
                        <label for="password">{"password:"}</label><br/>
                        <input type="password"
                            id="password"
                            class="text-violet-700 bg-pink-100"
                            value={(*password).clone()}
                            oninput={on_password_input}
                            required=true
                            />
                            <br/>
                        <input type="submit" 
                            value="Login" 
                            class="px-4 py-2 bg-indigo-800 rounded-xl font-medium hover:bg-violet-600 transition-colors"/>
                    </form>
                    
                    <div class="p-5 flex flex-col space-y-2">
                        <button onclick={on_google_click}
                            class="px-3 py-1 bg-indigo-800 rounded-xl font-medium hover:bg-indigo-600 transition-colors grid grid-cols-2">
                            <img src={format!("{}/uploads/google.png", crate::ADDR)} class="h-6 w-6"/>
                            {"Google"}
                        </button>
                        <button onclick={on_facebook_click}
                            class="px-3 py-1 bg-indigo-800 rounded-xl font-medium hover:bg-indigo-600 transition-colors grid grid-cols-2">
                            <img src={format!("{}/uploads/facebook.png", crate::ADDR)} class="h-6 w-6"/> 
                            {"Facebook"}
                        </button>
                        <button onclick={on_discord_click}
                            class="px-3 py-1 bg-indigo-800 rounded-xl font-medium hover:bg-indigo-600 transition-colors grid grid-cols-2">
                            <img src={format!("{}/uploads/discord.png", crate::ADDR)} class="h-6 w-6"/> 
                            {"Discord"}
                        </button>
                        <Link<crate::Route> to={crate::Route::Register}>{"New user? Register now!"}</Link<crate::Route>>
                    </div>
                </div>
            }
        }
    }
}

