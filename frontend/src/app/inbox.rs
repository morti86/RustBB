use web_sys::Element;
use yew::prelude::*;
use wasm_bindgen::JsCast;

use crate::{app::outbox::Outbox, dto::PrivateMessage, user::user_pms};


#[component]
pub fn Inbox() -> Html {
    let responses = use_state(|| Vec::<PrivateMessage>::new());
    let ctx = use_context::<crate::UserContext>()
        .expect("Expected context");

    let selected_message = use_state(|| None);
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

    let on_send = {
        let sm_c = selected_message.clone();
        Callback::from(move |_| {
            sm_c.set(None);
        })
    };

    let on_select_message = {
        let s_c = selected_message.clone();
        let r_c = responses.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if let Some(target) = e.target()
                && let Some(element) = target.dyn_ref::<Element>() 
                && let Some(id) = element.get_attribute("data-link-id") {
                    if id.len() > 4 {
                        let id: i64 = id[4..].parse()
                            .expect("failed to parse message id");
                        let msg = r_c.iter().find(|x| x.id == id);
                        if let Some(msg) = msg {
                            s_c.set(Some(msg.clone()));
                        } else {
                            s_c.set(None);
                        }
                    }
            }
        })
    };

    let s_c = selected_message.clone();
    
    html! {
        match &(*s_c) {
            Some(sm) => {
                html! {
                    <Outbox on_send={on_send} send_to={sm.author.clone().unwrap()}/>
                }
            }
            None => {
                let r_c = responses.clone();
                html! {
                    <div class="py-2 space-y-2">
                        <h2>{t!("msgs")}</h2>
                        <div class="space-x-2">
                            <button id="next_page" onclick={on_next_page} disabled={responses.is_empty() || responses.len() < (*limit)} >{t!("next")}</button>
                            <button id="first_page" onclick={on_first_page} disabled={(*page) <= 1} >{t!("firstp")}</button>
                        </div>
                        <button class="border rounded-xl border-zinc-800 grid grid-cols-6 text-xs px-5 py-2 disabled:bg-opacity-0 disabled:hover:bg-opacity-0 w-full" disabled={true}>
                            <div>{t!("sender")}</div>
                            <div>{t!("recver")}</div>
                            <div class="col-span-4">{t!("contt")}</div>
                        </button>
                        {for r_c.iter().map(|r| {
                            let id = format!("msg-{}", r.id);
                            html! {
                                <button 
                                    onclick={on_select_message.clone()} 
                                data-link-id={id.clone()} 
                                class="bg-opacity-0 hover:bg-pink-950/50 w-full"
                                disabled={ctx.id() == r.author.clone().unwrap_or_default()}>
                                    <div class="border rounded-xl border-zinc-800 grid grid-cols-6 text-xs" data-link-id={id.clone()}>
                                        <div class="col span-1" data-link-id={id.clone()}>{r.author_name.clone().unwrap_or_default()}</div>
                                        <div class="col-span-1" data-link-id={id.clone()}>{r.receiver_name.clone().unwrap_or_default()}</div>
                                        <div class="col-span-4" data-link-id={id.clone()}>{r.content.clone()}</div>
                                    </div>
                                </button>
                            }
                        })}
                    </div>
                }
            }
        }
    }
}

#[derive(Clone, Properties, PartialEq)]
struct ItemProps {
    id: String,
    on_click: Callback<String>,
}

#[component]
fn MessageListItem(props: &ItemProps) -> Html {
    html! {
    }
}
