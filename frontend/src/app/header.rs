use yew::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use yew_router::prelude::*;

use crate::UserContext;
use crate::Route;

#[component]
pub fn Header() -> Html {
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

    let c_c = ctx.clone();
    let on_logout = Callback::from(move |_| {
        c_c.dispatch(None);
        navigator.push(&Route::Content);
        wasm_bindgen_futures::spawn_local(async move {
            crate::user::logout().await;
        });
        
    });
    html! {
        <div class="flex items-center justify-between mb-8 w-full">
            <div>
                <h1 class="text-2xl font-bold mb-1">
                    <Link<Route> to={Route::Content}>{"Forum"}</Link<Route>>
                </h1>
                <p class="text-zinc-400 text-sm">{"Description"}</p>
            </div>
            {if ctx.is_none() {
                html! {
                    <div class="space-x-2 flex">
                        <div class="rounded-2xl border-zinc-800 border px-2 py-1 bg-indigo-950/50">{"anonymous"}</div>
                        <button onclick={on_home}>{"Home"}</button> 
                        <button onclick={on_login}>{"Login"}</button> 
                        <button onclick={on_reg}>{"Register"}</button> 
                    </div>
                }
            } else {
                html! { 
                    <div class="space-x-2 flex">
                        <div class="rounded-2xl border-zinc-800 border px-2 py-1 bg-indigo-950/50">{ctx.name()}</div>
                        <button onclick={on_home}>{"Home"}</button> 
                        <button onclick={on_logout}>{"Logout"}</button> 
                        <button onclick={on_user_list}>{"Users"}</button> 
                    </div>
                }
            }}
        </div>
    }
}
