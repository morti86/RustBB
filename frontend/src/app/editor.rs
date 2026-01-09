use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement};
use yew::prelude::*;
use ammonia::clean;
use pulldown_cmark::{Parser, Options, html::push_html};

use crate::{UserContext, bind::{get_jwt_from_cookie, upload_file_with_fetch}, dto::Resp, forum::{add_post, edit_post}};

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub id: i64,
    pub post_id: Option<i64>,
    pub set_to_load: Callback<()>,
}

#[component]
pub fn Editor(props: &Props) -> Html {
    let raw = use_state(String::new);
    let ctx = use_context::<UserContext>().expect("no context");
    let logged_out = ctx.is_none();
    let thread_id = props.id;
    let error = use_state(String::new);
    //let image_data = use_state(Vec::<u8>::new);
    let is_image = use_state(|| false);
    let post_id = props.post_id.clone();
    let s_c = props.set_to_load.clone();
    let r_c = raw.clone();
    let e_c = error.clone();
    let on_submit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        e_c.set(String::new());
        let text = r_c.clone();
        let md_parse = Parser::new_ext(text.as_str(), Options::empty());
        let mut unsafe_html = String::new();
        push_html(&mut unsafe_html, md_parse);
        let safe_html = clean(&*unsafe_html);
        let s_c = s_c.clone();
        let e_c = e_c.clone();
        crate::c_log!("SUBMIT: {:?}", post_id);
        match post_id {
            None => { // New post
                wasm_bindgen_futures::spawn_local(async move {
                    match add_post(thread_id, &safe_html).await {
                        Ok(_) => {
                            s_c.emit(());
                            text.set(String::new());
                        }
                        Err(e) => {
                            let err = Resp::from(e);
                            e_c.set(err.message);
                        }
                    }
                });
            }
            Some(post_id) => { // Edit post
                wasm_bindgen_futures::spawn_local(async move {
                    match edit_post(post_id, &safe_html).await {
                        Ok(_) => {
                            s_c.emit(());
                            text.set(String::new());
                        }
                        Err(e) => {
                            let err = Resp::from(e);
                            e_c.set(err.message);
                        }
                    }
                });
            }
        }
    });

    let r_c = raw.clone();
    use_effect_with((), move |_| {
        if let Some(post_id) = post_id {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let element = document.get_element_by_id(&format!("post-{}",post_id))
                .expect("nothing to edit");
            let text = element.inner_html();
            r_c.set(text);
        }
    });

    let r_c = raw.clone();
    let on_text_input = {
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let v = input.value();
            r_c.set(v);
        })
    };

    let r_c = raw.clone();
    let insert_image = Callback::from(move |url: String| {
        let element = format!(" ![image]({})", url);
        let mut text = (*r_c).clone();
        text.push_str(&element);
        r_c.set(text);
    });

    let on_file_upload = {
        //let im_c = image_data.clone();
        let im_c = is_image.clone();
        let insert = insert_image.clone();
        Callback::from(move |_e: Event| {
            im_c.set(false);
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let element: HtmlInputElement = document
                .get_element_by_id("file_upload")
                .expect("element not found")
                .dyn_into()
                .expect("wrong thing");
            if let Some(files) = element.files()
                && let Some(file) = files.get(0) {

                let insert = insert.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let token = get_jwt_from_cookie("token")
                        .unwrap_or_default();
                    if let Ok(res) = upload_file_with_fetch(&token, "/forum/upload_image", &file).await {
                        crate::c_log!("{}", res.avatar_url);
                        crate::c_log!("{}", res.filename);
                        insert.emit(res.avatar_url.replace("0.0.0.0",crate::ADDR));
                    }
                });
            } else {
                crate::c_log!("No files");
            }
        })
    };

    html! {
        <form id={format!("post-form-{:?}",post_id)} onsubmit={on_submit}>
            <span class="text-red-500">{(*error).clone()}</span>
            <div class="grid grid-cols-6 space-y-5 space-x-2">
                <textarea 
                    rows="10"
                    required=true
                    maxlength="250"
                    class="bg-black/0 colspan=10 p-5 border rounded-2xl border-zinc-800 col-span-6"
                    disabled={logged_out}
                    oninput={on_text_input}
                    value={(*raw).clone()}
                    />
                <input 
                    type="submit" 
                    value="Submit" 
                    class="px-4 py-2 bg-indigo-800 rounded-xl font-medium hover:bg-violet-600 transition-colors col-span-4"
                    disabled={logged_out}
                    />
                <input
                    type="file"
                    id="file_upload"
                    accept="image/*"
                    class={classes!["px-4","py-2","rounded-xl","font-medium","hover:bg-violet-600","transition-colors", "col-span-2", "bg-neutral-secondary-medium", "block", "bg-rose-800"]}
                    onchange={on_file_upload}
                    />
            </div>
        </form>
    }
}

fn file_status(ush: UseStateHandle<Vec<u8>>) -> &'static str {
    if ush.is_empty() {
        "bg-indigo-800"
    } else {
        "bg-rose-800"
    }
}
