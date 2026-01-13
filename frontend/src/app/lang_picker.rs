use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub uc: Callback<String>,
}

#[component]
pub fn LangPicker(props: &Props) -> Html {
    let locales = rust_i18n::available_locales!();
    let selected_value = use_state(|| "en".to_string());
    let uc = props.uc.clone();

    let on_pick = {
        let sel = selected_value.clone();
        let uc = uc.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let v = target.value();
            crate::c_log!("lang={}",v);
            uc.emit(v);
            sel.set(target.value());
        })
    };


    html! {
        <select id="langselect" onchange={on_pick} class="bg-black">
        {for locales.iter().map(|&loc| {
            html! {<option value={loc}>{loc}</option>}
        })}
        </select>
    }
}
