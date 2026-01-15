use web_sys::Element;
use yew::prelude::*;
use wasm_bindgen::JsCast;

use crate::{dto::PrivateMessage, user::user_pms};

#[component]
pub fn Inbox() -> Html {
    let responses = use_state(|| Vec::<PrivateMessage>::new());

    let selected_message = use_state(|| None::<PrivateMessage>);
    let page = use_state(|| 1);
    let limit = use_state(|| 20);

    let r_c = responses.clone();
    let p_c = page.clone();
    let l_c = limit.clone();
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            match user_pms(Some(*p_c), Some(*l_c)).await {
                Ok(p) => {
                    r_c.set(p.pms);
                }
                Err(e) => {
                    crate::c_error!("{:?}", e);
                }
            }
        });
    });

    let p_c = page.clone();
    let on_next_page = Callback::<MouseEvent>::from(move |e: MouseEvent| {
        e.prevent_default();
        p_c.set(*p_c + 1);
    });

    let p_c = page.clone();
    let on_first_page = Callback::<MouseEvent>::from(move |e: MouseEvent| {
        e.prevent_default();
        p_c.set(1);
    });

    let on_select_message = {
        let s_c = selected_message.clone();
        let r_c = responses.clone();
        Callback::from(move |e: MouseEvent| {
            if let Some(target) = e.target() 
                && let Some(element) = target.dyn_ref::<Element>() {
                    let id = element.id();
                    let id: i64 = id[4..].parse()
                        .expect("failed to parse message id");
                    let msg = r_c.iter().find(|x| x.id == id);
                    if msg.is_some() {
                        s_c.set(msg.cloned());
                    } else {
                        s_c.set(None);
                    }
            }
        })
    };

    let s_c = selected_message.clone();
    match s_c.as_ref() {
        Some(sm) => {
            html! {
                <div class="flex">
                    <div>{sm.author.clone().unwrap_or_default()}</div>
                    <div>{sm.content.clone()}</div>
                </div>
            }
        }
        None => {
            let r_c = responses.clone();
            html! {
                <div>
                    <h2>{t!("msgs")}</h2>
                    <div class="space-x-2">
                        <button id="next_page" onclick={on_next_page} disabled={responses.is_empty() || responses.len() < (*limit)} >{t!("next")}</button>
                        <button id="first_page" onclick={on_first_page} disabled={(*page) <= 1} >{t!("firstp")}</button>
                    </div>

                    {for r_c.iter().map(|r| {
                        html! {
                            <a href="#" onclick={on_select_message.clone()} id={format!("msg-{}", r.id)}>
                            <div class="border rounded-xl border-zinc-800 grid grid-cols-6 text-xs">
                                <div class="col span-1">{r.id}</div>
                                <div class="col-span-1">{r.author.clone().unwrap_or_default()}</div>
                                <div class="col-span-4">{r.content.clone().truncate(30)}</div>
                            </div>
                            </a>
                        }
                    })}
                </div>
            }
        }
    }
}
