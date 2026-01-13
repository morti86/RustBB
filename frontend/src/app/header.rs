use yew::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use yew_router::prelude::*;

use crate::UserContext;
use crate::Route;
use super::lang_picker::LangPicker;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub uc: Callback<String>,
}

#[component]
pub fn Header(props: &Props) -> Html {
    let navigator = use_navigator().unwrap_throw();
    let ctx = use_context::<UserContext>()
        .expect_throw("No user context");
    let n_c = navigator.clone();
    let on_login = Callback::from(move |_| n_c.push(&Route::Login));
    let n_c = navigator.clone();
    let on_reg = Callback::from(move |_| n_c.push(&Route::Register));
    let n_c = navigator.clone();
    let on_home = Callback::from(move |_| n_c.push(&Route::Content));
    let n_c = navigator.clone();
    let on_user_list = Callback::from(move |_| n_c.push(&Route::UserList));
    let n_c = navigator.clone();
    let on_inbox = Callback::from(move |_| n_c.push(&Route::Messages));
    let c_c = ctx.clone();
    let on_logout = Callback::from(move |_| {
        c_c.dispatch(None);
        navigator.push(&Route::Content);
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = crate::user::logout().await {
                crate::c_error!("Logout error: {:?}", e);
            }
        });
        
    });

    html! {
        <div class="flex items-center justify-between mb-8 w-full">
            <div>
                <h1 class="text-2xl font-bold mb-1">
                    <Link<Route> to={Route::Content}>{t!("title")}</Link<Route>>
                </h1>
                <p class="text-zinc-400 text-sm">{t!("fdesc")}</p>
            </div>
            {if ctx.is_none() {
                html! {
                    <div class="space-x-2 flex">
                        <div class="rounded-2xl border-zinc-800 border px-2 py-1 bg-indigo-950/50">{"anonymous"}</div>
                        <button onclick={on_home}>{t!("home")}</button> 
                        <button onclick={on_login}>{t!("login")}</button> 
                        <button onclick={on_reg}>{t!("register")}</button>
                        <LangPicker uc={props.uc.clone()}/>
                    </div>
                }
            } else {
                html! { 
                    <div class="space-x-2 flex">
                        <div class="rounded-2xl border-zinc-800 border px-2 py-1 bg-indigo-950/50">{ctx.name()}</div>
                        <button onclick={on_home}>{t!("home")}</button> 
                        <button onclick={on_logout}>{t!("logout")}</button> 
                        <button onclick={on_user_list}>{t!("users")}</button> 
                        <button onclick={on_inbox}>{t!("inbox")}</button> 
                        <LangPicker uc={props.uc.clone()}/>
                    </div>
                }
            }}
        </div>
    }
}
