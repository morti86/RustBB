use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::section_list::SectionList;
use crate::app::section::Section;
use crate::Route;

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub token: Option<String>,
}


#[derive(Clone, Routable, PartialEq)]
pub enum ContentRoute {
    #[at("/")]
    Main,
    #[at("/section/:id")]
    Section { id: i64 },
    #[at("/topic/:id")]
    Topic { id: i64 },
    #[at("/me")]
    Profile,
    #[at("/user/:id")]
    User { id: String },
    #[at("/settings")]
    Settings,
}

#[component]
pub fn Content(props: &Props) -> Html {
    let token = props.token.clone();
    
    html! {
        <div>
            <Link<Route> to={Route::Content}>{"back"}</Link<Route>>
            <Switch<ContentRoute> render={move |routes: ContentRoute| {
                match routes {
                    ContentRoute::Main => html! { <SectionList token={token.clone()}/> },
                    ContentRoute::Section { id } => html! { <Section id={id} token={token.clone()}/> },
                    ContentRoute::Topic { id } => html! { <>{"topic: "}{id}</> },
                    ContentRoute::Profile => html! { <h2>{"profile"}</h2> },
                    ContentRoute::User { id } => html! { <>{"user"}{id}</> },
                    ContentRoute::Settings => html!{ <>{"settings"}</> }
                }
            }}/>
        </div>
    }
}

