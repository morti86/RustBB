#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use app::register::Register;
use app::user_list::UserList;
use chrono::Utc;
use dto::FilterUserDto;
use user::me;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::app::login::Login;
use crate::app::oauth_callback::OAuthCallback;
use crate::app::header::Header;
use crate::app::section::Section;
use crate::app::topic::Topic;
use crate::app::section_list::SectionList;
use crate::app::user_page::UserPage;
use crate::app::inbox::Inbox;
use std::collections::HashMap;

mod bind;
//mod error;
mod user;
mod dto;
mod forum;
mod app;
mod storage;
mod text;

static ADDR: &str = "http://localhost:8080";
static DATEFORMAT: &str = "%Y-%m-%d %H:%M";
static DATEFORMAT_SIMPLE: &str = "%Y-%m-%d";

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Ctx {
    user: Option<FilterUserDto>,
}

impl Reducible for Ctx {
    type Action = Option<FilterUserDto>;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        Ctx { user: action }.into()
    }
}

impl Ctx {
    pub fn is_some(&self) -> bool {
        self.user.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.user.is_none()
    }

    pub fn avatar(&self) -> String {
        if let Some(u) = self.user.as_ref()
            && let Some(a) = u.avatar.as_ref() {
                a.clone()
        } else {
            String::from("default.png")
        }
    }

    pub fn is_mod(&self) -> bool {
        if let Some(u) = self.user.as_ref() {
            u.is_mod()
        } else {
            false
        }
    }

    pub fn is_admin(&self) -> bool {
        if let Some(u) = self.user.as_ref() {
            u.is_admin()
        } else {
            false
        }
    }

    pub fn banned(&self) -> bool {
        if let Some(u) = self.user.as_ref()
            && let Some(b) = u.banned_until {
            b > Utc::now()
        } else {
            false
        }
    }

    pub fn id(&self) -> String {
        if let Some(u) = self.user.as_ref() {
            u.id.clone()
        } else {
            String::new()
        }
    }

    pub fn name(&self) -> String {
        if let Some(u) = self.user.as_ref() {
            u.name.clone()
        } else {
            String::new()
        }
    }

}

pub type UserContext = UseReducerHandle<Ctx>;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/register")]
    Register,
    #[at("/login")]
    Login,
    #[at("/oauth/callback")]
    OAuthCallback,
    #[at("/")]
    Content,
    #[at("/section/:id")]
    Section { id: i64 },
    #[at("/section/:s_id/:id")]
    Topic { s_id: i64, id: i64 },
    #[at("/user/:id")]
    User { id: String },
    #[at("/users")]
    UserList,
    #[at("/messages")]
    Messages,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[component]
fn App() -> Html {
    let ctx = use_reducer(Ctx::default);
    let c_c = ctx.clone();
    let user_cache = Rc::new( RefCell::new( HashMap::<String, dto::UserData>::new() ) );
    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let m = me().await;
            match m {
                Ok(user) => {
                    c_log!("User: {}", user.name);
                    c_c.dispatch(Some(user));

                }
                Err(e) => {
                    c_error!("{:?}",e);
                    c_c.dispatch(None);
                }
            }
        });
    });

    // Create callback for OAuth start
    let on_oauth_start = {
        Callback::from(move |_| {
            // This callback can be used to show loading state or handle OAuth start
            c_log!("OAuth login started");
            // You could add loading state management here if needed
        })
    };

    let u_c = user_cache.clone();
    html! {
        <ContextProvider<UserContext> context={ctx}>
        <BrowserRouter>
        <div class="min-h-screen  p-6 text-zinc-200">
            // Background
            <div class="fixed inset-0 -z-10">
                <div class="absolute top-0 left-1/4 w-96 h-96 bg-violet-500/10 rounded-full blur-3xl" />
                <div class="absolute bottom-1/4 right-1/4 w-96 h-96 bg-violet-500/10 rounded-full blur-3xl" />
            </div>

            <div class="max-w-4xl mx-auto">
                // Header
                <div class="flex items-center justify-between mb-8">
                    <Header/>
                </div>
                // Content
                <Switch<Route> render={move |routes: Route| {
                    let on_oauth_start = on_oauth_start.clone();
                    match routes {
                        Route::Content => html! { <SectionList /> },
                        Route::Register => html! { <Register/> },
                        Route::Login =>
                            html! {
                                <Login
                                on_oauth_start={on_oauth_start}
                                />
                            },
                        Route::Topic {s_id, id} => html! { <Topic id={id} section={s_id} user_cache={u_c.clone()}/> },
                        Route::User {id} => html! { <UserPage id={id} /> },
                        Route::UserList => html! { <UserList/> },
                        Route::Section {id} => html! { <Section id={id} /> },
                        Route::OAuthCallback => html! { <OAuthCallback/> },
                        Route::Messages => html! { <Inbox/> },
                        Route::NotFound => html! { <h1>{"404 not"}</h1> },
                    }
                }} />
            </div>
            <p class="text-center text-zinc-600 text-xs mt-8">
                {"Built with "}
                    <a href="https://yew.rs" class="hover:text-cyan-400">{"Yew"}</a>{" & "}
                    <a href="https://github.com/tokio-rs/axum" class="hover:text-cyan-400">{"Axum"}</a>{" & "}
                    <a href="https://tailwindcss.com/" class="hover:text-cyan-400">{"Tailwind CSS"}</a>
            </p>
        </div>
        </BrowserRouter>
        </ContextProvider<UserContext>>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
