use std::{collections::HashMap, rc::Rc};

use yew::prelude::*;
use yew_router::prelude::*;

use crate::{dto::{PrivateMessage, UserData}, user::user_pms};

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub user_cache: Rc<std::cell::RefCell<HashMap<String, UserData>>>,
}

#[component]
pub fn Inbox(props: &Props) -> Html {
    let cache = props.user_cache.clone();
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
    let l_c = limit.clone();
    let on_next_page = Callback::<MouseEvent>::from(move |e: MouseEvent| {
        e.prevent_default();
        
    });

    let s_c = selected_message.clone();
    match s_c.as_ref() {
        Some(sm) => {
            html! {
                <div class="flex">
                    <div>{sm.author.clone().unwrap_or_default()}</div>
                </div>
            }
        }
        None => {
            let r_c = responses.clone();
            html! {
                <div>
                    <h2>{t!("msgs")}</h2>
                    {for r_c.iter().map(|r| {
                        html! {
                            <div class="">
                            </div>
                        }
                    })}
                </div>
            }
        }
    }
}
