use yew::prelude::*;
use yew_router::hooks::use_navigator;

use crate::user::register;

#[component]
pub fn Register() -> Html {
    let navigator = use_navigator().expect("navigator missing!");
    let username = use_state(|| String::new());
    let email = use_state(|| String::new());
    let password = use_state(|| String::new());
    let password_confirm = use_state(|| String::new());
    let error = use_state(|| None::<String>);

    let on_submit = {
        let username = username.clone();
        let email = email.clone();
        let password = password.clone();
        let password_confirm = password_confirm.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let current_username = (*username).clone();
            let current_email = (*email).clone();
            let current_password = (*password).clone();
            let current_password_confirm = (*password_confirm).clone();

            if current_username.is_empty() 
                || current_password.is_empty() 
                || current_email.is_empty() 
                || current_password_confirm.is_empty() {
                error.set(Some("all fields are required".to_string()));
                return;
            }

            // Clear previous errors
            error.set(None);

            // Create a callback for the async login operation
            let username_clone = current_username.clone();
            let email_clone = current_email.clone();
            let password_clone = current_password.clone();
            let password_confirm_clone = current_password_confirm.clone();
            let error_clone = error.clone();
            let navigator_clone = navigator.clone();

            

            wasm_bindgen_futures::spawn_local(async move {
                match register(&username_clone, &email_clone, &password_clone, &password_confirm_clone).await {
                    Ok(_) => {
                        navigator_clone.push(&crate::Route::Content);
                    }
                    Err(e) => {
                        // Show error
                        let error_msg = e.as_string().unwrap_or_else(|| "Register failed".to_string());
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

    let on_password_confirm_input = {
        let password = password_confirm.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let on_email_input = {
        let email = email.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            email.set(input.value());
        })
    };


    html! {
        <div id="register-form" class="flex items-center justify-center min-h-screen">
            <form id="login" onsubmit={on_submit}>
                if let Some(err) = (*error).as_ref() {
                    <div class="error">{err}</div>
                }
                <label for="username">{"username:"}</label><br/>
                <input type="text"
                    id="username"
                    class="text-violet-700 bg-zinc-300"
                    required=true
                    oninput={on_username_input}
                    value={(*username).clone()}
                    pattern="[A-Za-z0-9]+"
                    /><br/>
                <label for="password">{"password:"}</label><br/>
                <input type="password"
                    id="password"
                    class="text-violet-700 bg-zinc-300"
                    value={(*password).clone()}
                    oninput={on_password_input}
                    required=true
                    /><br/>
                <label for="password_confirm">{"confirm password:"}</label><br/>
                <input type="password"
                    id="password_confirm"
                    class="text-violet-700 bg-zinc-300"
                    value={(*password_confirm).clone()}
                    oninput={on_password_confirm_input}
                    required=true
                    /><br/>
                <label for="email">{"email:"}</label><br/>
                <input type="email"
                    id="email"
                    class="text-violet-700 bg-zinc-300"
                    value={(*email).clone()}
                    oninput={on_email_input}
                    required=true
                    /><br/>
                <div class="p-4 flex justify-center">
                    <input type="submit" 
                        class="px-3 py-1 bg-indigo-700 rounded-xl font-medium hover:bg-violet-600 transition-colors"
                        value="Register"/>
                </div>
            </form>
        </div>
    }
}


