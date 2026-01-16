use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

use crate::{dto::PostReactionsList, forum::{add_reaction, get_reactions}};

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub thread: i64,
    pub post: Option<i64>,
}

#[component]
pub fn Reaction(props: &Props) -> Html {
    let number = use_state(PostReactionsList::default);
    let post_id = Rc::new(props.post.clone());
    let post_id_n = props.post.unwrap_or(0);
    let loaded = use_state(|| false);
    let thread = props.thread;

    let num = number.clone();
    let p_id = post_id.clone();
    let l_c = loaded.clone();
    use_effect(move || {
        if !*l_c {
            let n_c = num.clone();
            let l_c = l_c.clone();
            crate::c_log!("reactions");
            wasm_bindgen_futures::spawn_local(async move {
                let num = n_c.clone();
                let l_c = l_c.clone();
                if let Ok(rcs) = get_reactions(*p_id, thread).await {
                    num.set(rcs);
                    l_c.set(true);
                }
            });
        }
    });
    
    let num = number.clone();
    let on_click = {
        let num = number.clone();
        let p_id = post_id.clone();
        let l_c = loaded.clone();
        Callback::from(move |e: MouseEvent| {
            if let Some(target) = e.target() 
                && let Some(element) = target.dyn_ref::<Element>() {
                let num = num.clone();
                let p_id = p_id.clone();
                let rk = element.id();
                let l_c = l_c.clone();
                let r_kind = format!("{}", rk.chars().nth(0).unwrap_or('?'));
                wasm_bindgen_futures::spawn_local(async move {
                    let num = num.clone();
                    let p_id = p_id.clone();
                    let l_c = l_c.clone();
                    if let Ok(true) = add_reaction(*p_id, thread, &r_kind).await {
                        let mut reactions = (*num).clone();
                        reactions.reactions.iter_mut()
                            .for_each(|rc| {
                                if rc.r_type == r_kind {
                                    rc.count = rc.count + 1;
                                }
                            });
                        num.set(reactions);
                        l_c.set(false);
                    }
                });
            }
        })
    };

    html! {
        <div class="flex">
            {for crate::REACTIONS.iter().map(|rc| {
                let reaction_data = num.reactions.iter().find(|x| x.r_type.eq(rc));
                match reaction_data {
                    Some(reaction_data) => {
                        html! { 
                            <button id={format!("{}-rc-{}", rc, post_id_n)} onclick={on_click.clone()} 
                                class="bg-zinc-950/20 disabled:bg-opacity-0 hover:bg-pink-800/40 disabled:opacity-40 disabled:hover:bg-opacity-0" 
                                disabled={reaction_data.user_clicked}>
                                {format!("{} {}", rc, reaction_data.count)}
                            </button>
                        }
                    }
                    None => {
                        html! {
                            <button id={format!("{}-rc-{}", rc, post_id_n)} onclick={on_click.clone()} class="bg-zinc-950/20 hover:bg-pink-800/40">
                                {format!("{} 0", rc)}
                            </button>

                        }
                    }
                }
            })}
        </div>
    }
}
