use web_sys::Element;
use yew::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use yew_router::hooks::use_navigator;
use crate::{dto::{Post, Thread}, forum::{delete_post, edit_thread, get_thread, new_thread}};
use super::user::User;
use super::editor::Editor;
use wasm_bindgen::JsCast;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub section: i64,
    pub id: i64,
}

#[component]
pub fn Topic(props: &Props) -> Html {
    let id = props.id;
    let s_id = props.section;
    let meta = use_state(Thread::default);
    let posts = use_state(Vec::<Post>::new);
    let page = use_state(|| 1);
    let limit = use_state(|| 10);
    let loaded = use_state(|| false);
    let editing = use_state(|| None::<i64>);
    let thread_edit = use_state(|| false);
    let navigator = use_navigator().unwrap_throw();
    let ctx = use_context::<crate::UserContext>()
        .expect("Expected context");
    let anon = ctx.is_none();
    let p_c = posts.clone();
    let l_c = loaded.clone();
    use_effect_with(l_c, move |_| {
        p_c.iter().for_each(|p| {
            let id = p.id;
            let element_id = format!("post-{}", id);
            let window = web_sys::window().expect("global window does not exists");
            let document = window.document().expect("expecting a document on window");
            let val = document.get_element_by_id(&element_id)
                .expect("No element id!");
            val.set_inner_html(&p.content);
        });
    });

    if id == 0 || *thread_edit { // NEW THREAD

        let m_c = meta.clone();
        let on_title_change = Callback::from(move |e: InputEvent| {
            let mut th = (*m_c).clone();
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let v = input.value();
            th.title = v;
            m_c.set(th);
        });

        let m_c = meta.clone();
        let on_content_change = Callback::from(move |e: InputEvent| {
            let mut th = (*m_c).clone();
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let v = input.value();
            th.content = v;
            m_c.set(th);
        });


        let m_c = meta.clone();
        let n_c = navigator.clone();
        let n_th_submit = Callback::from(move |e: SubmitEvent| {
            let meta = m_c.clone();
            e.prevent_default();
            if id > 0 {
                wasm_bindgen_futures::spawn_local(async move {
                    let meta = (*meta).clone();
                    if let Err(e) = edit_thread(id, &meta.title, &meta.content).await {
                        crate::c_error!("{:?}", e);
                    }
                });
            } else {
                let n_c = n_c.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let meta = (*meta).clone();
                    if let Err(e) = new_thread(&meta.title, &meta.content, s_id, vec![]).await {
                        crate::c_error!("{:?}", e);
                    }
                    n_c.push(&crate::Route::Section { id: s_id });
                });
            }
        });
        html! {
            <form id="new_thread" onsubmit={n_th_submit} class="space-y-5 p-2 space-x-2 grid grid-cols-1">
                <input type="text" maxlength="20" value={(*meta).title.clone()} class="bg-fuchsia-950/40" oninput={on_title_change}/>
                <textarea rows="10" maxlength="250" value={(*meta).content.clone()} class="bg-fuchsia-950/40" oninput={on_content_change}/>
                <input type="submit" value="New thread" class="disabled:bg-zinc-900 disabled:hover:bg-zinc-900 hover:bg-fuchsia-600 bg-fuchsia-800 rounded-xl" disabled={anon}/>
            </form>
        }
    } else { // VIEW EXISTING

        let l_c = loaded.clone();
        let e_c = editing.clone();
        let set_to_load = Callback::<()>::from(move |_| {
            l_c.set(false);
            e_c.set(None);
        });

       
        let l_c = loaded.clone();
        let on_click_d = Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(target) = e.target() {
                let elem: web_sys::Element = target.dyn_into().unwrap();
                let id = elem.id();
                    if id.starts_with("delete-") {
                    let post_id: i64 = (&id[7..]).parse()
                        .expect(&format!("failed to parse: {}", id));

                    let l_c = l_c.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Err(e) = delete_post(post_id).await {
                            crate::c_error!("Failed to delete post: {:?}", e);
                        }
                        l_c.set(false);
                    });

                }
            }
        });

        let e_c = editing.clone();
        let l_c = loaded.clone();
        let on_click_ed = Callback::from(move |e: MouseEvent| {
            if let Some(target) = e.target()
                && let Ok(element) = target.dyn_into::<Element>() {
                let element_id = element.id();
                if element_id.starts_with("edit-") {
                    let id: i64 = (&element_id[5..])
                        .parse()
                        .expect(&format!("Failed to parse ID: {}", element_id));
                    e_c.set(Some(id));
                    l_c.set(false);
                }
            }
        });

        let m_c = meta.clone();
        let p_c = posts.clone();
        let l_c = loaded.clone();
        let pg_c = page.clone();
        if id>=0 && !*loaded {
            let p_c2 = p_c.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let th = get_thread(id, Some(*pg_c), Some(*limit)).await
                    .unwrap_throw();
                m_c.set(th.info);
                p_c2.set(th.posts);
                l_c.set(true);
            });
        }

        let pg_c = page.clone();
        let l_c = loaded.clone();
        let pg_first = Callback::from(move |_| {
            pg_c.clone().set(1);
            l_c.set(false);
        });
        let pg_c = page.clone();
        let l_c = loaded.clone();
        let pg_next = Callback::from(move |_| {
            pg_c.clone().set(*pg_c+1);
            l_c.set(false);
        });

        html! {
            <div class="space-y-5">
                <div class="space-x-2">
                    <button onclick={pg_first} disabled={*page == 1}>{"First page"}</button>
                    <button onclick={pg_next} disabled={posts.is_empty()}>{"Next page"}</button>
                </div>
                <div class="bg-zinc-900/50 border border-zinc-800 rounded-2xl p-5 grid grid-cols-6 space-x-2">
                    <div>
                        <User user_id={meta.author.clone()}/>
                    </div>
                    <div class="col-span-5 grid grid-cols-1">
                        <span class="text-xl text-cyan-200">{&meta.title}</span>
                        <span class="text-zinc-400">{&meta.content}</span>
                    </div>
                    {if ctx.is_mod() || ctx.is_admin() || ctx.id() == meta.author {
                        html! { {""} }
                    } else {
                        html! { {""} }
                    }}
                </div>
                {for posts.iter().map(|p| {
                    let post_id = format!("post-{}", p.id);
                    let author = p.author.clone().unwrap_or_default();
                    html! {
                        <div class="grid grid-cols-6 space-x-2">
                            <div>
                                <User user_id={p.author.clone().unwrap_or_default()}/>
                            </div>
                            <div class="col-span-5 grid grid-cols-1 bg-zinc-900/50 p-5 rounded-2xl">
                                    <span class="text-zinc-400 row-span-6" id={post_id}>
                                        // Here goes Html
                                    </span>
                                    <div class="flex justify-end flex-col">
                                        <span>{if ctx.is_mod() || ctx.is_admin() || ctx.id()==author {
                                            if let Some(p_id) = (*editing) && p_id ==  p.id {
                                                html! { <Editor id={id} post_id={Some(p.id)} set_to_load={set_to_load.clone()} /> }
                                            } else {
                                                html! {
                                                    <div class="space-x-2">
                                                        <button 
                                                            class="px-4 py-1 bg-indigo-800 rounded-xl font-medium hover:bg-violet-600 transition-colors col-span-4" 
                                                            onclick={on_click_ed.clone()}
                                                            id={format!("edit-{}", p.id)}>
                                                            {"Edit"}
                                                        </button>
                                                        <button 
                                                            class="px-4 py-1 bg-indigo-800 rounded-xl font-medium hover:bg-violet-600 transition-colors col-span-4" 
                                                            onclick={on_click_d.clone()}
                                                            id={format!("delete-{}", p.id)}>
                                                            {"Delete"}
                                                        </button>
                                                    </div>}
                                            }
                                        } else {
                                            html! {""}
                                        }}</span>
                                        <span class="text-zinc-700 text-xs">{p.created_at.format(crate::DATEFORMAT).to_string()}</span>
                                    </div>
                            </div>
                        </div>
                    }
                })}
                {
                    if ctx.banned() {
                        html! { <span class="text-weight-bold text-red-500">{"You have been banned"}</span> }
                    } else if !ctx.is_some() {
                        html! { {""} }
                    } else {
                        html! { <Editor id={id} post_id={None} set_to_load={set_to_load.clone()} /> }
                    }
                }
            </div>
        }
    }
}

