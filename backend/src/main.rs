#![allow(dead_code)]
use anyhow::Result;
use error::ForumResult;
use oauth::OAuthService;
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use tracing::{info, warn};
use db::DBClient;
use uuid::Uuid;
use std::sync::Arc;
use std::env;
use axum::{
    Extension, Router, 
    extract::FromRef,
    http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, Method},
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{
    trace::TraceLayer,
    cors::{CorsLayer, AllowOrigin},
};
use chrono::{DateTime, Utc};
use std::net::SocketAddr;
use axum_extra::extract::cookie::Key;
use dashmap::DashMap;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod models;
mod db;
mod dto;
mod utils;
mod error;
mod mail;
mod handler;
mod middleware;
mod oauth;

type TryResult<'a> = dashmap::try_result::TryResult<dashmap::mapref::one::RefMut<'a, Uuid, UserSession>>;

#[derive(Debug, Clone)]
pub struct AppState {
    pub oauth_service: OAuthService,
    pub env: config::Config,
    pub db_client: DBClient,
    pub key: Key,
    pub active_users: Arc<DashMap<Uuid, UserSession>>,
}

impl AppState {
    pub fn update_session(&self, user_id: &Uuid) -> ForumResult<()> {
        match self.active_users.try_get_mut(user_id) {
            TryResult::Present(r) => {
                r.map(|v| {
                    v.last_seen = Utc::now();
                    v
                });
                Ok(())
            }
            TryResult::Absent => {
                warn!("Lock absent for {}", user_id);
                Ok(())
            }
            TryResult::Locked => {
                warn!("Locked for {}", user_id);
                Ok(())
            }
        }
    }

    pub fn list_active(&self) -> Vec<UserSession> {
        self.active_users.iter()
            .filter_map(|x| {
                let now = Utc::now();
                let last_seen = x.last_seen;
                if now - last_seen < chrono::Duration::minutes(5) {
                    Some(x.clone())
                } else {
                    None
                }
            }).collect()
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct UserSession {
    pub user_id: Uuid,
    pub username: String,
    pub login_time: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl UserSession {
    fn new(user_id: Uuid, username: String) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            username,
            login_time: now,
            last_seen: now,
        }
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

use tower_http::services::ServeDir;

pub fn create_router(app_state: Arc<AppState>) -> Router<AppState> {
    Router::new()
        .nest("/auth", handler::oauth::auth_router())
        .nest("/auth", handler::auth::auth_handler())
        .nest("/users", handler::user::user_handler() )
        .nest("/forum", handler::forum::forum_handler() )
        .nest_service("/uploads", ServeDir::new(&app_state.env.upload_dir))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(app_state))
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();    
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();
    
    dotenv().ok();
    let config = config::Config::init();

    let pool = match PgPoolOptions::new()
            .max_connections(10)
            .connect(config.database_url.as_str())
            .await {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST,Method::PUT,Method::DELETE]);
    
    let db_client = DBClient::new(pool);

    let app_state = AppState {
        oauth_service: OAuthService::from_env(),
        env: config.clone(),
        db_client,
        key: Key::generate(),
        active_users: Arc::new(DashMap::new()),
    };

    let a = Arc::new(app_state.clone());
    let app = create_router(a).layer(cors).with_state(app_state);

    // Start server
    let app_url = env::var("HOST_URL").unwrap_or("127.0.0.1".to_string());
   
    if let Ok(tls_config) = RustlsConfig::from_pem_file("./cert.pem", "./key.pem").await {
        info!("Found cert files, starting HTTPS");
        let app_port = env::var("PORT_HTTPS")
            .unwrap_or("3000".to_string())
            .parse::<u16>()?;
        let url: SocketAddr = format!("{}:{}", app_url, app_port).parse()?;

        axum_server::bind_rustls(url, tls_config)
            .serve(app.into_make_service())
            .await?;
        info!("Started HTTPS service at {}:{}",app_url, app_port);
    } else {
        warn!("No cert files found, running unencrypted HTTP!");
        let app_port = env::var("PORT_HTTPS")
            .unwrap_or("5000".to_string())
            .parse::<u16>()?;

        let url: SocketAddr = format!("{}:{}", app_url, app_port).parse()?;
        axum_server::bind(url)
            .serve(app.into_make_service())
            .await?;
        info!("Started HTTP service at {}:{}",app_url, app_port);
    }
    
    Ok(())
}
