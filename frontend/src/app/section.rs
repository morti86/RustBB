use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen::UnwrapThrowExt;
use crate::{UserContext, dto::ThreadListItemDto, forum::get_topics};

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub id: i64,
}

#[component]
pub fn Section(props: &Props) -> Html {
    let topic_list = use_state(|| Vec::<ThreadListItemDto>::new());
    let loaded = use_state(|| false);
    let ctx = use_context::<UserContext>().expect("no context");
    let page = use_state(|| None::<i32>);
    let limit = use_state(|| None::<usize>);
    let anon = ctx.is_none();
    let is_some = use_state(|| false);
    let section_id = props.id;
    let t_c = topic_list.clone();
    let l_c = loaded.clone();
    let some_c = is_some.clone();

    let pg_c = page.clone();
    let on_next_page = {
        let page = pg_c.clone();
        Callback::<MouseEvent>::from(move |_| {
            if page.is_none() {
                page.set(Some(2));
            } else {
                page.set((*page).map(|p| p+1));
            }
            l_c.set(false);
        })
    };

    let pg_c = page.clone();
    let l_c = loaded.clone();
    let on_start_page = {
        let page = pg_c.clone();
        Callback::<MouseEvent>::from(move |_| {
            page.set(None);
            l_c.set(false);
        })
    };

    let pg_c = page.clone();
    let lim_c = limit.clone();
    let l_c = loaded.clone();
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let topic_list = t_c.clone();
            let threads = get_topics(section_id, *pg_c, *lim_c).await
                .unwrap_throw();
            some_c.set(threads.len()>0);
            topic_list.set(threads);
            l_c.set(true);
        });
    });

    html! {
        <div>
            <Link<crate::Route> to={crate::Route::Topic { id: 0, s_id: section_id }}>{"New thread"}</Link<crate::Route>>
            {
            if (*topic_list).is_empty() {
        
                if *loaded {
                    if anon {
                        html! {}
                    } else {
                        html! {
                            <div >{"no threads"}</div>
                        }
                    }
                } else {
                    html! { "loading..." }
                }
            }
            else { html! {
                <div>
                {if let Some(page) = *page {
                    let page_num = format!("page {}", page);
                    html! { <span>{page_num}</span> }
                } else {
                    html! { "" }
                }}
                    <p class="text-zinc-400 text-sm mb-1"> {format!("Section {}", props.id)} </p>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
                            {for (*topic_list).iter().map(|topic| {
                                html! { 
                                    <Link<crate::Route> to={crate::Route::Topic { id: topic.id, s_id: section_id }}>
                                        <div class={classes!(vec!["flex","items-end","gap-2","border","rounded-2xl","p-5 hover:border-cyan-700", thread_border(topic.sticky), thread_background(topic.sticky)])}>
                                            <div class="grid grid-cols-2 gap-1">
                                                <span class="text-xs mb-1">{&topic.author_name}</span>
                                                <span class="text-l font-bold">{&topic.title}</span>
                                                <span class="text-xs text-red-500">{if topic.locked {"LOCKED"} else {""}}</span>
                                                <span class="text-xs text-emerald-500">{&topic.created_at.format(crate::DATEFORMAT).to_string()}</span>
                                            </div>
                                        </div> 
                                    </Link<crate::Route>>
                                }
                            })}
                    </div>
                                    </div>
            }}
        }
        <div class="space-x-2">
            <button id="next_page" onclick={on_next_page} disabled={!*is_some} >{"Next"}</button>
            <button id="first_page" onclick={on_start_page} disabled={(*page).is_none()} >{"First"}</button>
        </div>

        </div>
    }
}

fn thread_background(sticky: bool) -> &'static str {
    match sticky {
        false => "bg-zinc-900/50",
        true => "bg-zinc-800",
    }
}

fn thread_border(sticky: bool) -> &'static str {
    match sticky {
        true => "border-zinc-600",
        false => "border-zinc-800",
    }
}
