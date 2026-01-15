use std::{collections::HashMap, rc::Rc};

use yew::prelude::*;
use wasm_bindgen::UnwrapThrowExt;

use crate::dto::UserData;


#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub data: Rc<crate::dto::Post>,
    pub user_cache: Rc<std::cell::RefCell<HashMap<String, UserData>>>,
    pub r_cb: Callback<bool>,
}

#[component]
pub fn Post(props: &Props) -> Html {
    let data = props.data.clone();
    let cache = props.user_cache.clone();
    let loaded = use_state(|| false);

    let post_id = format!("post-{}", data.id);
    let author = data.author.clone().unwrap_or_default();


    use_effect_with((), |_| {
    });

    html! {
        <div class="grid grid-cols-6 space-x-2">
        </div>
    }
}
