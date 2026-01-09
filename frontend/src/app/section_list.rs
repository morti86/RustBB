use wasm_bindgen::UnwrapThrowExt;
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::{Route, dto::{CreateSectionDto, Section}, forum::{create_section, get_sections}};

#[component]
pub fn SectionList() -> Html {
    let section_list = use_state(|| Vec::<Section>::new());
    let new_section_data = use_state(|| None::<CreateSectionDto>);
    let loaded = use_state(|| false);
    let ctx = use_context::<crate::UserContext>()
        .expect("Expected context");


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
                    html! { <button class="bg-fuchsia-950/30 col-span-2 font-medium hover:bg-fuchsia-950/50" onclick={on_st_cr_sec}>{"Add section"}</button> }
                } else { html! { {""} } } }
                {if let Some(new_s) = new_section_data.as_ref() {
                    html! {
                        <form id="new_post" class="grid grid-cols-3 space-y-2 col-span-2" onsubmit={on_cr_sec.clone()}>
                            <input type="submit"
                                value="Add section sub"
                                class="px-4 py-1 bg-fuchsia-950/30 col-span-3  rounded-xl font-medium hover:bg-fuchsia-950/60 transition-colors"/>

                            <label for="s_name">{"section name"}</label>
                            <input type="text" maxlength="20"
                                id="s_name"
                                class="bg-fuchsia-950/30 col-span-2"
                                maxlength="50"
                                value={new_s.name.clone()}
                                oninput={on_name_change.clone()}/>
                            <label for="s_desc">{"description"}</label>
                            <input type="text" maxlength="50"
                                id="s_desc"
                                class="bg-fuchsia-950/30 col-span-2"
                                maxlength="50"
                                value={new_s.description.clone()}
                                oninput={on_desc_change.clone()}/>
                            <label for="s_name">{"allowed_for"}</label>
                            <div class="space-x-2 col-span-2">
                                <label for="bew_a">{"admins"}</label>
                                <input type="checkbox" id="bew_a" checked={true} disabled={true}/>
                                <label for="bew_m">{"mods"}</label>
                                <input type="checkbox" id="bew_m"
                                    checked={new_s.allowed_for.contains(&"Mod".to_string())}
                                    onchange={on_allowed_change.clone()}/>
                                <label for="bew_u">{"users"}</label>
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
                            "bg-zinc-900/50","border","hover:bg-zinc-700/30","transition-colors", new_posts(section.new_posts))}>
                            <p class="font-medium px-2 text-indigo-200">{&section.name}</p>
                            {if let Some(desc) = &section.description {
                                html! {<p class="py-3 px-2 text-sm text-zinc-400">{desc}</p>}
                            } else {
                                html! {<p class="py-3 px-2 text-sm text-zinc-400">{"-"}</p>}
                            }}
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
