use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::delete_object::DeleteObjectError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::put_object::PutObjectError;
use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use image::ImageError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Error while deleting object: {0}")]
    DeleteObjectError(#[from] SdkError<DeleteObjectError>),
    #[error("Error while getting image: {0}")]
    GetObjectError(#[from] SdkError<GetObjectError>),
    #[error("Error while inserting image: {0}")]
    PutObjectError(#[from] SdkError<PutObjectError>),
    #[error("Error while manipulating image bytes: {0}")]
    ImageError(#[from] ImageError),
    #[error("Error while getting data from multipart: {0}")]
    Multipart(#[from] MultipartError),
    #[error("Error while converting AWS ByteStream to Vec<u8>: {0}")]
    ByteStream(#[from] aws_sdk_s3::primitives::ByteStreamError),
    #[error("IO error: {0}")]
    IO(#[from] IoError),
    #[error("Body is empty")]
    EmptyBody,
    #[error("HTTP error: {0}")]
    HTTPError(#[from] axum::http::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let response = match self {
            Self::DeleteObjectError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::GetObjectError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::PutObjectError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::ImageError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::Multipart(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::IO(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::EmptyBody => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::HTTPError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::ByteStream(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        response.into_response()
    }
}
