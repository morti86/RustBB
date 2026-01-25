use emojis;

use yew::prelude::*;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub pick_emoji: Callback<String>,
    pub class: String,
}

pub enum Msg {
    PickGroup(emojis::Group),
    ClearGroup,
    PickEmoji(String),
}


pub struct EmojiPicker {
    selection: Option<emojis::Group>,
}

impl Component for EmojiPicker {
    type Message = Msg;
    type Properties = Props;

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_grp_ch = ctx.link().callback(|e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let v = target.value();
            if let Ok(g) = serde_json::from_str::<emojis::Group>(&v) {
                Msg::PickGroup(g)
            } else {
                Msg::ClearGroup
            }
        });

        let on_click_emoji = ctx.link().callback(|e: MouseEvent| {
            let target = e.target_dyn_into::<web_sys::HtmlElement>().unwrap();
            let emoji = target.id();
            Msg::PickEmoji(emoji)
        });

        html! {
            <div class={ctx.props().class.clone()}>
                <select onchange={on_grp_ch} class="bg-black">
                {for emojis::Group::iter().map(|g| {
                    html! { <option value={serde_json::to_string(&g).unwrap_or_default()} selected={self.selection == Some(g)}>{serde_json::to_string(&g).unwrap_or_default()}</option> }
                })}
                </select>
                {if let Some(grp) = self.selection {
                    html! {
                        <div class="grid grid-cols-8 md:grid-cols-12 space-2">
                        {for grp.emojis().map(|em| {
                            html! { <button class="p-2 bg-opacity-0 hover:bg-zinc-900" id={em.as_str()} onclick={on_click_emoji.clone()}>{em.as_str()}</button> }
                        })}
                        </div>
                    }
                } else { html! {} }}
            </div>
        }
    }

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            selection: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PickGroup(group) => {
                self.selection = Some(group);
                true
            }
            Msg::PickEmoji(em) => {
                ctx.props().pick_emoji.emit(em);
                self.selection = None;
                true
            }
            Msg::ClearGroup => {
                self.selection = None;
                true
            }
        }
    }

}
