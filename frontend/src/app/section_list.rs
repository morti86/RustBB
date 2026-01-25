use wasm_bindgen::UnwrapThrowExt;
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::{Route, dto::{CreateSectionDto, Section}, forum::{create_section, delete_section, edit_section, get_sections}};

#[component]
pub fn SectionList() -> Html {
    let section_list = use_state(|| Vec::<Section>::new());
    let new_section_data = use_state(|| None::<CreateSectionDto>);
    let loaded = use_state(|| false);
    let ctx = use_context::<crate::UserContext>()
        .expect("Expected context");
    let navigator = use_navigator()
        .expect("Where is navigator?");

    // Clone section_list before moving into async closure
    let sl_c = section_list.clone();
    let l_c = loaded.clone();
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let sections = get_sections().await.unwrap_throw();
            sl_c.set(sections);
            l_c.set(true);
        })
    });

    let n_c = new_section_data.clone();
    let on_st_cr_sec = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        if n_c.is_none() {
            n_c.set(Some(CreateSectionDto::default()));
        }
    });

    let n_c = new_section_data.clone();
    let on_cr_sec = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        let dto = (*n_c).clone();
        if let Some(n) = dto {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = create_section(&n).await {
                    crate::c_error!("Error creating section: {:?}", e);
                }
                if let Some(window) = window() {
                        let _ = window.location().reload();
                }
            });
            n_c.set(None);
            
        } else {
            crate::c_log!("NO Data!");
        }
    });

    // Callback for updating section name
    let n_c_name = new_section_data.clone();
    let on_name_change = Callback::from(move |e: InputEvent| {
        let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
        let value = input.value();
        if let Some(mut dto) = (*n_c_name).clone() {
            dto.name = value;
            n_c_name.set(Some(dto));
        }
    });

    // Callback for updating section description
    let n_c_desc = new_section_data.clone();
    let on_desc_change = Callback::from(move |e: InputEvent| {
        let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
        let value = input.value();
        if let Some(mut dto) = (*n_c_desc).clone() {
            dto.description = value;
            n_c_desc.set(Some(dto));
        }
    });

    let on_sc_del = {
        let n_c = navigator.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            let n_c = n_c.clone();
            let id = input.id();
            let id = &id[4..];

            let window = web_sys::window().expect("no window exists");
            let user_input = window.prompt_with_message(&t!("del_c"));

            if let Ok(Some(v)) = user_input
                && v == String::from("delete") 
                && let Ok(id) = id.parse::<i32>() {
                wasm_bindgen_futures::spawn_local(async move {
                    let n_c = n_c.clone();
                    if let Ok(_) = delete_section(id).await {
                        n_c.push(&crate::Route::Content);
                    }
                });
            }
        })
    };

    let on_sc_edit = {
        let sl_c = section_list.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            let sl_c = sl_c.clone();
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            let id = input.id();
            let id = &id[4..];
            let window = web_sys::window().expect("no window exists");
            let user_input = window.prompt_with_message(&t!("edit_c"));

            if let Ok(Some(v)) = user_input 
                && let Ok(id) = id.parse::<i32>() {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(_) = edit_section(id, &v).await {
                        let mut sl = (*sl_c).clone();
                        sl.iter_mut().for_each(|s| {
                            if s.id == (id as i64) {
                                s.description = Some(v.clone());
                            }
                        });
                        sl_c.set(sl);
                    }
                });
            }

        })
    };

    // Callback for updating allowed_for checkboxes
    let n_c_allowed = new_section_data.clone();
    let on_allowed_change = Callback::from(move |e: Event| {
        let checkbox = e.target_unchecked_into::<web_sys::HtmlInputElement>();
        let id = checkbox.id();
        let checked = checkbox.checked();

        if let Some(mut dto) = (*n_c_allowed).clone() {
            let mut allowed_for = dto.allowed_for.clone();

            match id.as_str() {
                "bew_m" => {
                    if checked && !allowed_for.contains(&"Mod".to_string()) {
                        allowed_for.push("Mod".to_string());
                    } else if !checked {
                        allowed_for.retain(|role| role != "Mod");
                    }
                }
                "bew_u" => {
                    if checked && !allowed_for.contains(&"User".to_string()) {
                        allowed_for.push("User".to_string());
                    } else if !checked {
                        allowed_for.retain(|role| role != "User");
                    }
                }
                _ => {}
            }

            // Always include "admin" since it's disabled and checked
            if !allowed_for.contains(&"Admin".to_string()) {
                allowed_for.push("Admin".to_string());
            }

            dto.allowed_for = allowed_for;
            n_c_allowed.set(Some(dto));
        }
    });

    html! {
        <div class="section-list">
            <div class="rounded-2xl grid grid-cols-2 gap-4">
                {if ctx.is_admin() && new_section_data.is_none() {
                    html! { <button class="bg-fuchsia-950/30 col-span-2 font-medium hover:bg-fuchsia-950/50" onclick={on_st_cr_sec}>{t!("adds")}</button> }
                } else { html! { {""} } } }
                {if let Some(new_s) = new_section_data.as_ref() {
                    html! {
                        <form id="new_post" class="grid grid-cols-3 space-y-2 col-span-2" onsubmit={on_cr_sec.clone()}>
                            <input type="submit"
                                value={t!("addsc")}
                                class="px-4 py-1 bg-fuchsia-950/30 col-span-3  rounded-xl font-medium hover:bg-fuchsia-950/60 transition-colors"/>

                            <label for="s_name">{t!("sname")}</label>
                            <input type="text" maxlength="20"
                                id="s_name"
                                class="bg-fuchsia-950/30 col-span-2"
                                maxlength="50"
                                value={new_s.name.clone()}
                                oninput={on_name_change.clone()}/>
                            <label for="s_desc">{t!("desc")}</label>
                            <input type="text" maxlength="50"
                                id="s_desc"
                                class="bg-fuchsia-950/30 col-span-2"
                                maxlength="50"
                                value={new_s.description.clone()}
                                oninput={on_desc_change.clone()}/>
                            <label for="s_name">{t!("allf")}</label>
                            <div class="space-x-2 col-span-2">
                                <label for="bew_a">{t!("adms")}</label>
                                <input type="checkbox" id="bew_a" checked={true} disabled={true}/>
                                <label for="bew_m">{t!("mods")}</label>
                                <input type="checkbox" id="bew_m"
                                    checked={new_s.allowed_for.contains(&"Mod".to_string())}
                                    onchange={on_allowed_change.clone()}/>
                                <label for="bew_u">{t!("users")}</label>
                                <input type="checkbox" id="bew_u"
                                    checked={new_s.allowed_for.contains(&"User".to_string())}
                                    onchange={on_allowed_change.clone()}/>
                            </div>
                        </form>
                    }
                } else {
                    html! { {""} }
                }}
                {for (*section_list).iter().map(|section| {
                    html! {
                        <Link<Route> to={Route::Section { id: section.id }}>
                        <div class={classes!("rounded-2xl","items-center","justify-between","p-4",
                            "bg-zinc-900/50","border","hover:bg-zinc-700/30","transition-colors", "relative", new_posts(section.new_posts))}>
                            <p class="font-medium px-2 text-indigo-200">{&section.name}</p>
                            {if let Some(desc) = &section.description {
                                html! {<p class="py-3 px-2 text-sm text-zinc-400">{desc}</p>}
                            } else {
                                html! {<p class="py-3 px-2 text-sm text-zinc-400">{"-"}</p>}
                            }}
                            <p class="text-xs pl-5">{t!("thrs")}{section.threads.unwrap_or_default()}</p>
                            {if ctx.is_admin() {html! {
                                <>
                                    <button class="absolute right-0 top-0 bg-opacity-0 hover:bg-opacity-20" id={format!("del-{}", section.id)} onclick={on_sc_del.clone()}>{"❌"}</button>
                                    <button class="absolute right-0 top-20 bg-opacity-0 hover:bg-opacity-20" id={format!("edt-{}", section.id)} onclick={on_sc_edit.clone()}>{"❔"}</button>
                                </>
                            }}
                                else { html! {""} }
                            }
                        </div>
                        </Link<Route>>
                    }
                })}
            </div>
        </div>
    }
}

fn new_posts(n: bool) -> &'static str {
    if n { "border-cyan-500" } else { "border-zinc-900" }
}
