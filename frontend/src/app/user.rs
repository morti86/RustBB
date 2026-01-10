use std::{collections::HashMap, rc::Rc};

use yew::prelude::*;
use yew_router::prelude::Link;
use wasm_bindgen::UnwrapThrowExt;

use crate::{Route, dto::UserData};


#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub user_id: String,
    pub user_cache: Rc<std::cell::RefCell<HashMap<String, UserData>>>,
}

#[component]
pub fn User(props: &Props) -> Html {
    let user_id = Rc::new(props.user_id.clone());
    let cache = props.user_cache.clone();
    let user = use_state(|| None::<UserData>);
    let loaded = use_state(|| false);
    let banned = use_state(|| false);
    
    let u = user.clone();
    let l_c = loaded.clone();
    let b_c = banned.clone();
    let u_c = user_id.clone();
    use_effect(|| {
        wasm_bindgen_futures::spawn_local(async move {
            if !u_c.is_empty() && !*l_c {
                if let Some(user_data) = cache.borrow().get(&*u_c) {
                    let b = user_data.is_banned();
                    u.set(Some(user_data.clone()));
                    b_c.set(b);
                    l_c.set(true);
                } else {
                    let fu = crate::user::user(&u_c).await.unwrap_throw();
                    let b = fu.is_banned();
                    let mut ud = cache.borrow_mut();
                    ud.insert((*u_c).clone(), fu);
                    b_c.set(b);
                    l_c.set(false);
                }
            }
        });
    });
    
    
    html! {
            <div class="bg-zinc-900/50 rounded-2xl p-5 grid grid-cols-1">
                <div class="">
                    {match user.as_ref() {
                        Some(user) => html! {
                            <div class="flex grid grid-cols-1">
                                <p class="text-zinc-400 mb-1 text-xs">
                                <Link<Route> to={Route::User { id: user.id.clone() }}>
                                    {if *banned {
                                        html! {<span class="line-through">{ user.name.clone() }</span> }
                                    } else {
                                        html! { { user.name.clone() } }
                                    }}
                                </Link<Route>>
                                </p>
                                <p class="text-fuchsia-800 text-xs">{user.posts_n.unwrap_or(0)}</p>
                            </div>
                        },
                        None => html! { "unknown user" },
                    }}
                </div>
                <img src={
                    match user.as_ref() {
                        Some(user) => format!("{}/uploads/{}", crate::ADDR, user.avatar()),
                        None => format!("{}/uploads/default.png", crate::ADDR),
                    }}/>
                <span class={classes!(vec![ "bottom-0", "rounded-full", "border-2", "border-zinc-900", "flex", "justify-center", "text-xs",  {status_colors(user.as_ref())} ] ) }>{
                    match user.as_ref() {
                        Some(user) => user.role.clone(),
                        None => String::from("anonymous"),
                    }
                }</span>

            </div>
    }

}

fn status_colors(user: Option<&UserData>) -> &'static str {
    if let Some(u) = user {
        match u.role.to_lowercase().as_str() {
            "admin" => "text-red-500",
            "mod" => "text-amber-500",
            "user" => "text-emerald-500",
            _ => "text-zinc-400",
        }
    } else {
        "bg-zinc-600"
    }
}
