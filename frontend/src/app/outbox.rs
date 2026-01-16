use std::rc::Rc;
use yew::prelude::*;

use crate::user::send_pm;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub send_to: String,
    pub on_send: Callback<()>,
}

#[component]
pub fn Outbox(props: &Props) -> Html {
    let send_to = Rc::new(props.send_to.clone());
    let on_send = props.on_send.clone();
    let raw = use_state(String::new);
    let ctx = use_context::<crate::UserContext>()
        .expect("Expected context");

    let r_c = raw.clone();
    let on_text_input = {
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let v = input.value();
            r_c.set(v);
        })
    };
    
    let st_c = send_to.clone();
    let os_c = on_send.clone();
    let r_c = raw.clone();
    let on_submit = Callback::from(move |_e: SubmitEvent| {
        let st_c = st_c.clone();
        let os_c = os_c.clone();
        let r_c = r_c.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let st_c = st_c.clone();
            let os_c = os_c.clone();
            let r_c = r_c.clone();
            if let Ok(_) = send_pm(&*st_c, &*r_c).await {
                os_c.emit(());
            }
        });
    });


    let st_c = send_to.clone();
    html! {
        <form id="outbox" class="flex grid grid-cols-1" onsubmit={on_submit}>
            <textarea 
                rows="10"
                cols="50"
                required=true
                maxlength="250"
                class="bg-black/0 colspan=10 p-5 border rounded-2xl border-zinc-800 col-span-6"
                oninput={on_text_input}
                value={(*raw).clone()}
                />
            <input 
                type="submit" 
                disabled={ctx.id() == (*st_c).clone()}
                value={t!("send")} 
                class="px-4 py-2 bg-indigo-800 rounded-xl font-medium hover:bg-violet-600 transition-colors col-span-4 disabled:bg-opacity-0"
                />
        </form>
    }
}
