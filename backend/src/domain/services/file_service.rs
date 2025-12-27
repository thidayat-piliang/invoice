#![allow(dead_code)]

use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use mime_guess::MimeGuess;
use serde::{Serialize, Deserialize};

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Invalid file name")]
    InvalidFileName,
    #[error("File too large: {0} bytes")]
    FileTooLarge(u64),
    #[error("Invalid file type")]
    InvalidFileType,
    #[error("File not found: {0}")]
    NotFound(String),
}

impl From<std::io::Error> for FileError {
    fn from(err: std::io::Error) -> Self {
        FileError::Io(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: String,
    pub file_hash: String,
}

#[derive(Clone)]
pub struct FileService {
    upload_dir: PathBuf,
    max_file_size: u64,
    allowed_types: Vec<String>,
}

impl FileService {
    pub fn new(upload_dir: &str, max_file_size: u64) -> Result<Self, FileError> {
        let upload_path = PathBuf::from(upload_dir);

        // Create upload directory if it doesn't exist
        std::fs::create_dir_all(&upload_path)
            .map_err(|e| FileError::Io(e.to_string()))?;

        Ok(Self {
            upload_dir: upload_path,
            max_file_size,
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "application/pdf".to_string(),
                "image/heic".to_string(),
                "image/heif".to_string(),
            ],
        })
    }

    /// Upload a file with byte data
    pub async fn upload_file(
        &self,
        file_data: &[u8],
        original_name: &str,
        mime_type: &str,
    ) -> Result<UploadedFile, FileError> {
        // Validate file size
        if file_data.len() as u64 > self.max_file_size {
            return Err(FileError::FileTooLarge(file_data.len() as u64));
        }

        // Validate mime type
        if !self.allowed_types.contains(&mime_type.to_string()) {
            return Err(FileError::InvalidFileType);
        }

        // Generate unique file name
        let extension = Path::new(original_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin");

        let unique_name = format!("{}_{}.{}", Uuid::new_v4().to_string(), chrono::Utc::now().timestamp(), extension);
        let file_path = self.upload_dir.join(&unique_name);

        // Write file
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(file_data).await?;
        file.flush().await?;

        // Calculate file hash (simple implementation)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(file_data);
        let file_hash = format!("{:x}", hasher.finalize());

        Ok(UploadedFile {
            file_name: unique_name,
            file_path: file_path.to_string_lossy().to_string(),
            file_size: file_data.len() as u64,
            mime_type: mime_type.to_string(),
            file_hash,
        })
    }

    /// Upload a file from bytes with automatic mime type detection
    pub async fn upload_file_auto(
        &self,
        file_data: &[u8],
        original_name: &str,
    ) -> Result<UploadedFile, FileError> {
        // Detect mime type
        let mime_type = MimeGuess::from_path(original_name)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        self.upload_file(file_data, original_name, &mime_type).await
    }

    /// Get file content
    pub async fn get_file(&self, file_name: &str) -> Result<Vec<u8>, FileError> {
        let file_path = self.upload_dir.join(file_name);

        if !file_path.exists() {
            return Err(FileError::NotFound(file_name.to_string()));
        }

        let content = fs::read(&file_path).await?;
        Ok(content)
    }

    /// Delete a file
    pub async fn delete_file(&self, file_name: &str) -> Result<(), FileError> {
        let file_path = self.upload_dir.join(file_name);

        if !file_path.exists() {
            return Err(FileError::NotFound(file_name.to_string()));
        }

        fs::remove_file(&file_path).await?;
        Ok(())
    }

    /// Check if file exists
    pub async fn file_exists(&self, file_name: &str) -> bool {
        self.upload_dir.join(file_name).exists()
    }

    /// Get file info
    pub async fn get_file_info(&self, file_name: &str) -> Result<UploadedFile, FileError> {
        let file_path = self.upload_dir.join(file_name);

        if !file_path.exists() {
            return Err(FileError::NotFound(file_name.to_string()));
        }

        let metadata = fs::metadata(&file_path).await?;
        let mime_type = MimeGuess::from_path(&file_path)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Ok(UploadedFile {
            file_name: file_name.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_size: metadata.len(),
            mime_type,
            file_hash: "".to_string(), // Would need to re-read file to calculate
        })
    }

    /// List all files
    pub async fn list_files(&self) -> Result<Vec<UploadedFile>, FileError> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(&self.upload_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if !file_name.is_empty() {
                    let metadata = entry.metadata().await?;
                    let mime_type = MimeGuess::from_path(&path)
                        .first()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "application/octet-stream".to_string());

                    files.push(UploadedFile {
                        file_name: file_name.clone(),
                        file_path: path.to_string_lossy().to_string(),
                        file_size: metadata.len(),
                        mime_type,
                        file_hash: "".to_string(),
                    });
                }
            }
        }

        Ok(files)
    }

    /// Get the public URL for a file (for API responses)
    pub fn get_file_url(&self, file_name: &str) -> String {
        format!("/api/v1/files/{}", file_name)
    }
}
