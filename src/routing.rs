use crate::AppState;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use image::io::Reader;
use image::ImageFormat;
use std::io::Cursor;

use crate::errors::ApiError;

#[tracing::instrument]
pub async fn retrieve_image(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Image, ApiError> {
    let res = state
        .s3
        .get_object()
        .bucket("my-bucket")
        .key(&filename)
        .send()
        .await?;

    let body: Vec<u8> = res.body.collect().await?.to_vec();

    Ok((filename, body).into())
}

#[tracing::instrument]
pub async fn delete_image(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Image, ApiError> {
    state
        .s3
        .delete_object()
        .bucket("my-bucket")
        .key(&filename)
        .send()
        .await?;

    tracing::info!("Image deleted with filename: {filename}");

    Ok(filename.into())
}

#[tracing::instrument]
pub async fn upload_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Image, ApiError> {
    let mut field: Option<Vec<u8>> = None;
    while let Some(formitem) = multipart.next_field().await.unwrap() {
        field = Some(formitem.bytes().await?.to_vec());
    }

    let Some(data) = field else {
        tracing::error!("User tried to upload an empty body");
        return Err(ApiError::EmptyBody);
    };

    let img2 = Reader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    let mut new_vec: Vec<u8> = Vec::new();
    img2.write_to(&mut Cursor::new(&mut new_vec), ImageFormat::Jpeg)?;

    let filename = "my_file.jpeg";

    let _ = state
        .s3
        .put_object()
        .bucket("my-bucket")
        .key(filename)
        .body(new_vec.into())
        .send()
        .await?;

    tracing::info!("Image uploaded with filename: {filename}");

    Ok(filename.into())
}

pub enum Image {
    Filename(String),
    File(String, Vec<u8>),
}

impl Into<Image> for (String, Vec<u8>) {
    fn into(self) -> Image {
        Image::File(self.0, self.1)
    }
}

impl Into<Image> for String {
    fn into(self) -> Image {
        Image::Filename(self)
    }
}

impl Into<Image> for &str {
    fn into(self) -> Image {
        Image::Filename(self.to_owned())
    }
}

impl IntoResponse for Image {
    fn into_response(self) -> Response {
        match self {
            Self::Filename(name) => (StatusCode::OK, name).into_response(),
            Self::File(filename, data) => {
                let filename_header_value = format!("attachment; filename=\"{filename}\"");

                Response::builder()
                    .header("Content-Disposition", filename_header_value)
                    .header("Content-Type", "image/jpeg")
                    .body(Body::from(data))
                    .unwrap()
            }
        }
    }
}
