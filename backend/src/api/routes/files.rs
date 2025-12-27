use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::Multipart;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::error::ApiError;
use crate::api::middleware::AuthUser;
use crate::domain::services::{FileService, UploadedFile, FileError};

#[derive(Clone)]
struct FileState {
    file_service: Arc<FileService>,
}

#[derive(Serialize)]
struct FileListResponse {
    files: Vec<UploadedFile>,
}

#[derive(Deserialize)]
struct FileDeleteRequest {
    file_name: String,
}

pub fn create_router(file_service: Arc<FileService>) -> Router {
    let state = FileState { file_service };

    Router::new()
        .route("/upload", post(upload_file))
        .route("/list", get(list_files))
        .route("/download/{file_name}", get(download_file))
        .route("/delete", post(delete_file))
        .with_state(state)
}

/// Upload a file using multipart form data
async fn upload_file(
    _auth_user: AuthUser,
    State(state): State<FileState>,
    mut multipart: Multipart,
) -> Result<Json<UploadedFile>, ApiError> {
    let mut uploaded_file = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let file_name = field
            .file_name()
            .ok_or_else(|| ApiError::BadRequest("No file name provided".to_string()))?
            .to_string();

        let content_type = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let file_data = field.bytes().await.map_err(|e| {
            ApiError::BadRequest(format!("Failed to read file data: {}", e))
        })?;

        // Upload the file
        let result = state
            .file_service
            .upload_file(&file_data, &file_name, &content_type)
            .await
            .map_err(|_e| ApiError::Internal)?;

        uploaded_file = Some(result);
    }

    match uploaded_file {
        Some(file) => Ok(Json(file)),
        None => Err(ApiError::BadRequest("No file uploaded".to_string())),
    }
}

/// List all uploaded files
async fn list_files(
    _auth_user: AuthUser,
    State(state): State<FileState>,
) -> Result<Json<FileListResponse>, ApiError> {
    let files = state
        .file_service
        .list_files()
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(FileListResponse { files }))
}

/// Download a file
async fn download_file(
    _auth_user: AuthUser,
    State(state): State<FileState>,
    axum::extract::Path(file_name): axum::extract::Path<String>,
) -> Result<Response, ApiError> {
    let file_data = state
        .file_service
        .get_file(&file_name)
        .await
        .map_err(|e| match e {
            FileError::NotFound(_) => {
                ApiError::BadRequest(format!("File not found: {}", file_name))
            }
            _ => ApiError::Internal,
        })?;

    let file_info = state
        .file_service
        .get_file_info(&file_name)
        .await
        .map_err(|_| ApiError::Internal)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        file_info.mime_type.parse().unwrap(),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_name)
            .parse()
            .unwrap(),
    );

    Ok((headers, file_data).into_response())
}

/// Delete a file
async fn delete_file(
    _auth_user: AuthUser,
    State(state): State<FileState>,
    Json(payload): Json<FileDeleteRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .file_service
        .delete_file(&payload.file_name)
        .await
        .map_err(|e| match e {
            FileError::NotFound(_) => {
                ApiError::BadRequest(format!("File not found: {}", payload.file_name))
            }
            _ => ApiError::Internal,
        })?;

    Ok(StatusCode::NO_CONTENT)
}
