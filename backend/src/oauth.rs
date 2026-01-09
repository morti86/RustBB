use oauth2::{self, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl, basic::*, reqwest::redirect::Policy
};
use reqwest::Client as RClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::ForumResult;
use std::env;

type Abomination = Client<oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>, oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, BasicTokenType>, oauth2::StandardTokenIntrospectionResponse<oauth2::EmptyExtraTokenFields, BasicTokenType>, oauth2::StandardRevocableToken, oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>, EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
}

impl OAuthConfig {
    pub fn google(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            user_info_url: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
        }
    }

    pub fn facebook(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            auth_url: "https://www.facebook.com/v19.0/dialog/oauth".to_string(),
            token_url: "https://graph.facebook.com/v19.0/oauth/access_token".to_string(),
            user_info_url: "https://graph.facebook.com/v19.0/me".to_string(),
        }
    }

    pub fn discord(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            auth_url: "https://discord.com/api/oauth2/authorize?response_type=code".to_string(),
            token_url: "https://discord.com/api/oauth2/token".to_string(),
            user_info_url: "https://graph.facebook.com/v19.0/me".to_string(),
        }
    }

    pub fn create_client(&self) -> ForumResult<Abomination> {
        let c = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(AuthUrl::new(self.auth_url.clone()).unwrap())
            .set_token_uri(TokenUrl::new(self.token_url.clone())?)
            .set_redirect_uri(RedirectUrl::new(self.redirect_uri.clone())?);

        Ok(c)
    }
}

#[derive(Debug, Clone)]
pub struct OAuthService {
    google_config: Option<OAuthConfig>,
    facebook_config: Option<OAuthConfig>,
    discord_config: Option<OAuthConfig>,
}

impl OAuthService {
    pub fn has_google(&self) -> bool {
        self.google_config.is_some()
    }

    pub fn has_facebook(&self) -> bool {
        self.facebook_config.is_some()
    }

    pub fn has_discord(&self) -> bool {
        self.discord_config.is_some()
    }

    pub fn from_env() -> Self {

        let google_config = if let (Some(client_id), Some(client_secret), Some(redirect_uri)) =
            (env::var("GOOGLE_CLIENT_ID").ok(), env::var("GOOGLE_CLIENT_SECRET").ok(), env::var("GOOGLE_REDIRECT_URI").ok()) {
                Some(OAuthConfig::facebook(client_id, client_secret, redirect_uri))
        } else {
            None
        };

        let facebook_config = if let (Some(client_id), Some(client_secret), Some(redirect_uri)) =
            (env::var("FACEBOOK_CLIENT_ID").ok(), env::var("FACEBOOK_CLIENT_SECRET").ok(), env::var("FACEBOOK_REDIRECT_URI").ok()) {
                Some(OAuthConfig::facebook(client_id, client_secret, redirect_uri))
        } else {
            None
        };

        let discord_config = if let (Some(client_id), Some(client_secret), Some(redirect_uri)) =
            (env::var("DISCORD_CLIENT_ID").ok(), env::var("DISCORD_CLIENT_SECRET").ok(), env::var("DISCORD_REDIRECT_URI").ok()) {
                Some(OAuthConfig::discord(client_id, client_secret, redirect_uri))
        } else {
            None
        };

        Self {
            google_config,
            facebook_config,
            discord_config
        }
    }

    pub fn get_google_auth_url(&self) -> Option<(String, CsrfToken, PkceCodeVerifier)> {
        if let Some(config) = self.google_config.as_ref()
            && let Ok(client) = config.create_client() {
            let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
            let (auth_url, csrf_token) = client
                .authorize_url(CsrfToken::new_random)
                .add_scope(Scope::new("email".to_string()))
                .add_scope(Scope::new("profile".to_string()))
                .set_pkce_challenge(pkce_challenge)
                .url();
            Some((auth_url.to_string(), csrf_token, pkce_verifier))
        } else {
            None
        }
    }

    pub fn get_facebook_auth_url(&self) -> Option<(String, CsrfToken)> {
       if let Some(config) = self.facebook_config.as_ref()
            && let Ok(client) = config.create_client() {
            let (auth_url, csrf_token) = client
                .authorize_url(CsrfToken::new_random)
                .add_scope(Scope::new("email".to_string()))
                .add_scope(Scope::new("public_profile".to_string()))
                .url();

            Some((auth_url.to_string(), csrf_token))
       } else {
           None
       }
    }

    pub fn get_discord_auth_url(&self) -> Option<(String, CsrfToken)> {
       if let Some(config) = self.discord_config.as_ref()
            && let Ok(client) = config.create_client() {
            let (auth_url, csrf_token) = client
                .authorize_url(CsrfToken::new_random)
                .add_scope(Scope::new("email".to_string()))
                .add_scope(Scope::new("public_profile".to_string()))
                .url();

            Some((auth_url.to_string(), csrf_token))
       } else {
           None
       }
    }

    pub async fn exchange_google_code(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
    ) -> ForumResult<GoogleUserInfo> {
        if let Some(google_config) = self.google_config.as_ref() {

            let client = google_config.create_client()?;

            let http_client = reqwest::ClientBuilder::new()
                // Following redirects opens the client up to SSRF vulnerabilities.
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("Client should build");

            let token_result = client
                .exchange_code(AuthorizationCode::new(code))
                .set_pkce_verifier(pkce_verifier)
                .request_async(&http_client)
                .await?;

            let access_token = token_result.access_token().secret();
            // Fetch user info from Google
            let http_client = RClient::builder()
                .redirect(Policy::none())
                .build()
                .expect("Failed to build HTTP client");

            let response = http_client
                .get(&google_config.user_info_url)
                .bearer_auth(access_token)
                .send()
                .await?;

            let user_info: GoogleUserInfo = response.json::<GoogleUserInfo>().await?;
            Ok(user_info)
        } else {
            Err(crate::error::ForumError::Http((200, "Google auth not configured".to_string())))
        }
    }

    pub async fn exchange_facebook_code(
        &self,
        code: String,
    ) -> ForumResult<FacebookUserInfo> {
        let config = self
            .facebook_config
            .as_ref()
            .ok_or("Facebook OAuth not configured")?;

        let http_client = reqwest::ClientBuilder::new()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");


        let client = config.create_client()?;
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(&http_client)
            .await?;

        let access_token = token_result.access_token().secret();

        // Fetch user info from Facebook
        let http_client = RClient::builder()
            .redirect(Policy::none())
            .build()
            .expect("Failed to build HTTP client");

        let mut params = HashMap::new();
        params.insert("fields", "id,name,email,picture");
        params.insert("access_token", access_token);

        let response = http_client
            .get(&config.user_info_url)
            .query(&params)
            .send()
            .await?;

        let user_info: FacebookUserInfo = response.json::<FacebookUserInfo>().await?;
        Ok(user_info)
    }

    pub async fn exchange_discord_code(&self, code: String) 
        -> ForumResult<DiscordUserInfo> {

        let config = self
            .discord_config
            .as_ref()
            .ok_or("Facebook OAuth not configured")?;

        let http_client = reqwest::ClientBuilder::new()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");
        let client = config.create_client()?;
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(&http_client)
            .await?;

        let access_token = token_result.access_token().secret();

        // Fetch user info from Facebook
        let http_client = RClient::builder()
            .redirect(Policy::none())
            .build()
            .expect("Failed to build HTTP client");

        let mut params = HashMap::new();
        params.insert("fields", "id,username,email,avatar,global_name");
        params.insert("access_token", access_token);

        let response = http_client
            .get(&config.user_info_url)
            .query(&params)
            .send()
            .await?;

        let user_info: DiscordUserInfo = response.json::<DiscordUserInfo>().await?;
        Ok(user_info)

    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,           // Google user ID
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookUserInfo {
    pub id: String,           // Facebook user ID
    pub name: String,
    pub email: Option<String>,
    pub picture: Option<FacebookPicture>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookPicture {
    pub data: FacebookPictureData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookPictureData {
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUserInfo {
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub id: String,
    pub global_name: String,
    pub username: String,
}


