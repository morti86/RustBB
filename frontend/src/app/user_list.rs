use yew::prelude::*;
use yew_router::prelude::*;

use crate::{dto::FilterUserDto, user::user_list};

#[component]
pub fn UserList() -> Html {
    //let ctx = use_context::<crate::UserContext>();
    let users = use_state(Vec::<FilterUserDto>::new);
    let page = use_state(|| Some(1));
    let limit = use_state(|| Some(20));

    let p_c = page.clone();
    let l_c = limit.clone();
    let u_c = users.clone();
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(ul) = user_list(*p_c, *l_c).await {
                u_c.set(ul.users);
            }
        });
    });

    let pg_c = page.clone();
    let on_next_page = {
        let page = pg_c.clone();
        Callback::<MouseEvent>::from(move |_| {
            if page.is_none() {
                page.set(Some(2));
            } else {
                page.set((*page).map(|p| p+1));
            }
        })
    };

    let pg_c = page.clone();
    let on_start_page = {
        let page = pg_c.clone();
        Callback::<MouseEvent>::from(move |_| {
            page.set(None);
        })
    };


    html! {
        <div class="grid p-2 grid-cols-1 space-y-2">
        <div class="space-x-2">
            <button id="first_page" onclick={on_start_page} disabled={(*page).clone().unwrap_or(1)==1} >{"First"}</button>
            <button id="next_page" onclick={on_next_page} disabled={users.is_empty()} >{"Next"}</button>
        </div>

        {for users.iter().map(|u| {
            let banned = u.is_banned();
            html! {
                <Link<crate::Route> to={crate::Route::User { id: u.id.clone() }}>
                    <div class="border rounded-xl border-zinc-800 grid grid-cols-6 text-xs">
                        <div class={classes!{"py-2", "px-5", banned_indicator(banned)}}>{u.name.clone()}</div>
                        <div class="py-2 px-5">{u.email.clone()}</div>
                        <div class="py-2 px-5">{u.role.clone()}</div>
                        <div class="py-2 px-5 col-span-2">{u.description()}</div>
                        <div class="py-2 px-5">{u.created_at.format(crate::DATEFORMAT).to_string()}</div>
                    </div>
                </Link<crate::Route>>
            }
        })}
        </div>
    }
}

fn banned_indicator(b: bool) -> &'static str {
    if b { "line-through" } else { "" }
}
