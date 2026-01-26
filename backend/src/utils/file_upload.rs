use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use uuid::Uuid;
use axum::{
    extract::Multipart,
};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use crate::AppState;
use crate::error::{ForumError, ForumResult};
use crate::config::Config;

/// Validates and saves an uploaded image file
pub async fn save_uploaded_image(
    mut multipart: Multipart,
    config: &Config,
    user_id: Uuid,
) -> Result<String, ForumError> {
    // Create upload directory if it doesn't exist
    fs::create_dir_all(&config.upload_dir).await
        .map_err(|e| ForumError::ServerError(format!("Failed to create upload directory: {}", e)))?;

    let mut avatar_filename = None;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| ForumError::ServerError(format!("Failed to read multipart field: {}", e)))? {
        tracing::debug!("uploading SOME");
        let name = field.name().unwrap_or("").to_string();
        
        tracing::debug!("name={}", name);
        if name == "file" {
            let content_type = field.content_type().unwrap_or("").to_string();

            // Validate content type
            if !config.allowed_image_types.contains(&content_type) {
                return Err(ForumError::Forum(
                    format!("Invalid file type. Allowed types: {}",
                        config.allowed_image_types.join(", "))
                ));
            }

            // Generate unique filename
            let extension = match content_type.as_str() {
                "image/jpeg" => "jpg",
                "image/jpg" => "jpg",
                "image/png" => "png",
                "image/gif" => "gif",
                "image/webp" => "webp",
                _ => return Err(ForumError::Forum(
                    "Unsupported image format".to_string()
                )),
            };

            let filename = format!("avatar_{}_{}.{}", user_id, Uuid::new_v4(), extension);
            let filepath = std::path::Path::new(&config.upload_dir).join(&filename);

            // Read file data with size limit
            let data = field.bytes().await
                .map_err(|e| ForumError::ServerError(format!("Failed to read file data: {}", e)))?;

            // Validate file size
            if data.len() > config.max_file_size {
                return Err(ForumError::Forum(
                    format!("File too large. Maximum size is {} bytes", config.max_file_size)
                ));
            }

            // Save file
            let mut file = fs::File::create(&filepath).await
                .map_err(|e| ForumError::ServerError(format!("Failed to create file: {}", e)))?;

            file.write_all(&data).await
                .map_err(|e| ForumError::ServerError(format!("Failed to write file: {}", e)))?;
            tracing::debug!("filename: {}", filename);
            avatar_filename = Some(filename);
            break; // We only process the first avatar field
        }
    }

    match avatar_filename {
        Some(filename) => Ok(filename),
        None => Err(ForumError::Forum("No avatar file provided".to_string())),
    }
}

/// Deletes an old avatar file if it exists and is not the default
pub async fn delete_old_avatar(
    old_avatar: Option<&str>,
    config: &Config,
) -> Result<(), ForumError> {
    if let Some(old_filename) = old_avatar {
        // Don't delete the default avatar
        if old_filename != "default.png" {
            let old_path = std::path::Path::new(&config.upload_dir).join(old_filename);
            if old_path.exists() {
                fs::remove_file(&old_path).await
                    .map_err(|e| ForumError::ServerError(format!("Failed to delete old avatar: {}", e)))?;
            }
        }
    }
    Ok(())
}

/// Gets the full URL for an avatar file
pub fn get_avatar_url(config: &Config, filename: &str) -> String {
    // In production, you might want to use a CDN or different URL structure
    format!("{}/uploads/{}", config.host_url, filename)
}

pub async fn serve_avatar(
    Path(image_id): Path<String>,
    Extension(app_state): Extension<Arc<AppState>>) -> ForumResult<impl IntoResponse> {
    let upload_path = env!("UPLOAD_DIR");

    let file_path: PathBuf = [upload_path, &image_id].iter().collect();

    if !file_path.exists() {
        let default_path = format!("{}/default.png", upload_path);
        let default = match app_state.image_cache.get(default_path.as_str()).await {
            Some(c) => c,
            None => fs::read(default_path.as_str()).await?,
        };
        let body = axum::body::Body::from(default);
        let response = axum::http::Response::builder()
            .header("Content-Type", "image/png") // adjust based on format
            .body(body)?;
        Ok(response.into_response())
    } else {
        let fp = file_path.display().to_string();
        let contents = app_state.image_cache.get(fp.as_str()).await;
        let contents = match contents {
            Some(c) => c,
            None => {
                let data = fs::read(file_path).await?;
                app_state.image_cache.insert(fp, data.clone()).await;
                data
            }
        };

        let mime_type = if let Some(kind) = infer::get(&contents) {
            kind.mime_type()
        } else {
            // Fallback if detection fails (optional: log warning)
            "application/octet-stream"
        };
        let body = axum::body::Body::from(contents);
        let response = axum::http::Response::builder()
            .header("Content-Type", mime_type) // adjust based on format
            .body(body)?;
        Ok(response.into_response())


    }
}

pub async fn serve_localization(Path(loc_id): Path<String>) -> ForumResult<impl IntoResponse> {
    let upload_path = env!("UPLOAD_DIR");
    let file_path: PathBuf = [upload_path, &format!("locale/{}.json", loc_id)].iter().collect();

    if !file_path.exists() {
        let default_path = format!("{}/en.json", upload_path);
        let default = fs::read(default_path.as_str())
            .await?;

      let body = axum::body::Body::from(default);
        let response = axum::http::Response::builder()
            .header("Content-Type", "application/json") // adjust based on format
            .body(body)?;
        Ok(response.into_response())
    } else {
        let contents = fs::read(&file_path)
            .await?;

        let body = axum::body::Body::from(contents);
        let response = axum::http::Response::builder()
            .header("Content-Type", "application/json") // adjust based on format
            .body(body)?;
        Ok(response.into_response())
    }
}

pub async fn list_locales() -> ForumResult<impl IntoResponse> {
    let upload_path = env!("UPLOAD_DIR");
    let locales_dir = std::path::Path::new(upload_path).join("locale");

    // Check if the locales directory exists
    if !locales_dir.exists() {
        return Ok(Json(Vec::<String>::new()));
    }

    let mut locales = Vec::new();

    // Read the directory
    match std::fs::read_dir(&locales_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    // Check if it's a file and has .json extension
                    if path.is_file() {
                        if let Some(extension) = path.extension() {
                            if extension == "json" {
                                if let Some(filename) = path.file_stem() {
                                    if let Some(locale_name) = filename.to_str() {
                                        locales.push(locale_name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(ForumError::ServerError(format!(
                "Failed to read locales directory: {}", e
            )));
        }
    }

    // Sort locales alphabetically
    locales.sort();

    Ok(Json(locales))
}
