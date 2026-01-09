use web_sys::{HtmlInputElement, Response};
use yew::prelude::*;
use wasm_bindgen::{UnwrapThrowExt, JsCast};

use crate::{bind::upload_file_with_fetch, c_log, dto::UserData, user::{unban_user, update_user, user, warn_user}};

macro_rules! display_thing {
    ($name:ident, $value:expr) => {
        html!{
            <div class="flex flex-auto p-2 space-x-4">
                <div class="flex-none w-20">{stringify!($name)}</div>
                <div class="flex-1 w-80">{$value}</div>
            </div>
        }
    };
    ($name:ident, $value:expr, $cb:ident) => {
        html! {
            <div class="flex flex-auto p-2 space-x-4 rounded-l">
                <div class="flex-none w-20">{stringify!($name)}</div>
                <div class="flex-1 w-80">
                    <input 
                        type="text"
                        id="$name"
                        class="bg-violet-950/20"
                        oninput={$cb}
                        value={$value}
                        />
                </div>
            </div>
        }
    };
    ($name:ident, $value:expr, $cb:ident, $typ:literal) => {
        html! {
            <div class="flex flex-auto p-2 space-x-4 rounded-l">
                <div class="flex-none w-20">{stringify!($name)}</div>
                <div class="flex-1 w-80">
                    <input 
                        type={$typ}
                        id="$name"
                        class="bg-violet-950/20"
                        oninput={$cb}
                        value={$value}
                        />
                </div>
            </div>
        }
    };

}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

fn can_edit(ctx: &crate::Ctx, role: &str) -> bool {
    if ctx.is_none() {
        return false;
    }

    match role {
        "Admin" => false,
        "Mod" => ctx.is_admin(),
        _ => ctx.is_admin() || ctx.is_mod(),
    }
}

macro_rules! on_input {
    ($name:ident, $on_name:ident, $ctx:expr) => {
        let $on_name = {
            Callback::from(move |e: InputEvent| {
                let mut c_c = (*$ctx).clone();
                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                c_c.$name = input.value();
                $ctx.set(c_c);
            })
        };
    };
    (OPT: $name:ident, $on_name:ident, $ctx:expr) => {
        let $on_name = {
            Callback::from(move |e: InputEvent| {
                let mut c_c = (*$ctx).clone();
                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                c_c.$name = Some(input.value());
                $ctx.set(c_c);
            })
        };
    };

}

#[component]
pub fn UserPage(props: &Props) -> Html {
    let user_data = use_state_eq(UserData::default);
    let edit_mode = use_state_eq(|| false);
    let user_id = props.id.clone();
    let self_edit = use_state(|| false);
    let ban_length = use_state_eq(|| 0);
    let ban_comment = use_state(String::new);
    let error = use_state(String::new);
    let ctx = use_context::<crate::UserContext>()
        .expect("Expected context");

    let u_c = user_data.clone();
    on_input!(name, on_name_input, u_c);
    let u_c = user_data.clone();
    on_input!(email, on_email_input, u_c);
    let u_c = user_data.clone();
    on_input!(OPT: description, on_description_input, u_c);
    let u_c = user_data.clone();
    on_input!(OPT: facebook, on_facebook_input, u_c);
    let u_c = user_data.clone();
    on_input!(OPT: x_id, on_x_id_input, u_c);
    let u_c = user_data.clone();
    on_input!(OPT: discord, on_discord_input, u_c);
    let b_c = ban_length.clone();
    let on_ban_change = Callback::from(move |e: InputEvent| {
        let b_c = b_c.clone();
        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
        let value: i32 = input.value().parse()
            .expect("Expected integer");
        b_c.set(value);
    });

    let b_c = ban_comment.clone();
    let on_comment_change = Callback::from(move |e: InputEvent| {
        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
        b_c.set(input.value());
    });
    let u_c = user_data.clone();
    let c_c = ctx.clone();
    let b_l = ban_length.clone();

    let e_c = error.clone();
    let bc_c = ban_comment.clone();
    let on_ban_submit = {
        let u_c = user_data.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let user_id = u_c.id.clone();
            let warned_by = c_c.id();
            let banned = if *b_l > 0 { Some(*b_l) } else { None };
            let c = (*bc_c).clone();
            let comment = if c.is_empty() { None } else { Some(c) };
            e_c.set(String::new());
            let e_c = e_c.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = warn_user(&user_id, comment.as_deref(), &warned_by, banned).await {
                    let es: Response = e.dyn_into()
                        .expect("unexpected error response");
                    let es_str = es.as_string().unwrap_or_default();
                    let status = es.status();
                    let err = format!("Error saving data [{}]: {}", status, es_str);
                    e_c.set(err);
                }
            });
        })
    };

    let e_c = edit_mode.clone();
    let c_c = ctx.clone();
    let s_c = self_edit.clone();
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let user = user(&user_id).await
                .unwrap_throw();
            let edit = can_edit(&c_c, &user.role);
            let s = if let Some(u) = c_c.user.as_ref() {
                u.id == user.id
            } else {
                c_log!("ctx=None");
                false
            };
            u_c.set(user);
            c_log!("s={}",s);
            s_c.set(s);
            e_c.set(edit);
            
        });
    });

    let e_c = error.clone();
    let on_submit = {
        let u_c = user_data.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let current_user = (*u_c).clone();
            e_c.set(String::new());
            let e_c = e_c.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = update_user(&current_user).await {
                    let es: Response = e.dyn_into()
                        .expect("unexpected error response");
                    let es_str = es.as_string().unwrap_or_default();
                    let status = es.status();
                    let err = format!("Error saving data [{}]: {}", status, es_str);
                    e_c.set(err);
                }
            });
        })
    };


    let e_c = error.clone();
    let u_c = user_data.clone();
    let on_unban = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        let user_id = u_c.id.clone();
        e_c.set(String::new());
        let e_c = e_c.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = unban_user(&user_id).await {
                let es: Response = e.dyn_into()
                    .expect("unexpected error response");
                let es_str = es.as_string().unwrap_or_default();
                let status = es.status();
                let err = format!("Error saving data [{}]: {}", status, es_str);
                e_c.set(err);
            }
        });

    });

    let u_c = user_data.clone();
    let on_file_upload = {
        //let im_c = image_data.clone();
        Callback::from(move |_e: Event| {
            let u_c = u_c.clone();
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let element: HtmlInputElement = document
                .get_element_by_id("file_upload")
                .expect("element not found")
                .dyn_into()
                .expect("wrong thing");
            if let Some(files) = element.files()
                && let Some(file) = files.get(0) {

                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(res) = upload_file_with_fetch("/user/avatar", &file).await {
                        let mut u = (*u_c).clone();
                        u.avatar = Some(res.filename);
                        u_c.set(u);
                    } else {
                        let mut u = (*u_c).clone();
                        u.avatar = None;
                        u_c.set(u);
                    }
                });
            } else {
                crate::c_log!("No files");
            }
        })
    };

    let u_c = user_data.clone();
    let on_av_delete = Callback::from(move |_e: MouseEvent| {
        let u_c = u_c.clone();
        let mut u = (*u_c).clone();
        u.avatar = None;
        u_c.set(u);
    });

    let c_c = ctx.clone();
    let user = user_data.clone();
    if !*edit_mode && !*self_edit {
        html! {
            <div>
                {display_thing!(id, user.id.clone())}
                {display_thing!(name, user.name.clone())}
                {display_thing!(email, user.email.clone())}
                {display_thing!(role, user.role.clone())}
                {display_thing!(description, user.description.clone().unwrap_or_default())}
                {display_thing!(facebook, user.facebook.clone().unwrap_or_default())}
                {display_thing!(discord, user.discord.clone().unwrap_or_default())}
                {display_thing!(x, user.x_id.clone().unwrap_or_default())}
                <div class="flex flex-auto p-2 space-x-4">
                    <div class="flex-none w-20">{"avatar"}</div>
                    <img src={format!("{}/uploads/{}", crate::ADDR, user.avatar())} class="w-32"/>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="space-y-5">
                <span class="text-red-500 text-xl font-bold">{(*error).clone()}</span>
                <form id="edit_user" onsubmit={on_submit}>
                    <div>
                        {display_thing!(id, user.id.clone())}
                        {display_thing!(name, user.name.clone(), on_name_input)}
                        {display_thing!(email, user.email.clone(), on_email_input, "email")}
                        {display_thing!(role, user.role.clone())}
                        {display_thing!(description, user.description.clone().unwrap_or_default(), on_description_input)}
                        {display_thing!(facebook, user.facebook.clone().unwrap_or_default(), on_facebook_input)}
                        {display_thing!(discord, user.discord.clone().unwrap_or_default(), on_discord_input)}
                        {display_thing!(x, user.x_id.clone().unwrap_or_default(), on_x_id_input)}
                        <div class="flex flex-auto p-2 space-x-4">
                            <div class="flex-none w-20">{"avatar"}</div>
                            <div id="avatar_update" class="grid grid-flow-col grid-rows-3 gap-4">
                                <img src={format!("{}/uploads/{}", crate::ADDR, user.avatar())} class="w-32 row-span-3"/>
                                <div></div>
                                <input
                                    type="file"
                                    id="file_upload"
                                    accept="image/*"
                                    disabled={user.id != c_c.id()}
                                    class="px-4 py-2 rounded-xl font-medium hover:bg-violet-600 transition-colors col-span-2 bg-neutral-secondary-medium block bg-rose-800"
                                    onchange={on_file_upload}
                                    />
                                <button id="delete_av" onclick={on_av_delete}>
                                    {"delete"}
                                </button>
                            </div>
                        </div>

                    </div>
                    <input type="submit" 
                        class="px-3 py-1 bg-indigo-700 rounded-xl font-medium hover:bg-violet-600 transition-colors"
                        value="Update"/>
                </form>
                {if !*self_edit {
                    html! { 
                        <div>
                            { if user.is_banned() {
                                html! { 
                                    <button id="unban_user"
                                        class="px-3 py-1 bg-indigo-700 rounded-xl font-medium hover:bg-violet-600 transition-colors"
                                        onclick={on_unban}>
                                        {"Unban"}
                                    </button> 
                                }
                            } else {
                                html! { 
                                    <form id="ban_user" onsubmit={on_ban_submit} class="space-x-2">
                                        <input type="number" 
                                            name="length" 
                                            value={format!("{}",*ban_length.clone())}
                                            class="bg-violet-950/20"
                                            oninput={on_ban_change}/>
                                        <input type="text"
                                            name="comment"
                                            class="bg-violet-950/20"
                                            oninput={on_comment_change}/>
                                        <input type="submit" 
                                            class="px-3 py-1 bg-indigo-700 rounded-xl font-medium hover:bg-violet-600 transition-colors"
                                            value="Ban User"/>
                                    </form>
                                }
                            }}
                        </div>
                    }
                } else {
                    html! { {""} }
                }}
            </div>
        }
    }
}


