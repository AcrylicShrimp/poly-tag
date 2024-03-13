use crate::{
    db::models::File,
    dto::StaticError,
    guards::AuthUserSession,
    services::{FileService, FileServiceError},
};
use rocket::{
    delete, get, http::Status, post, response::stream::ReaderStream, routes, serde::json::Json,
    Build, Responder, Rocket, State,
};
use std::{pin::Pin, sync::Arc};
use tokio::io::AsyncRead;
use uuid::Uuid;

pub fn register_routes(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/files",
        routes![create_file, remove_file, get_file, get_file_data],
    )
}

#[derive(Responder)]
#[response(content_type = "json")]
enum Error {
    #[response(status = 404)]
    NotFoundError(StaticError),
    #[response(status = 500)]
    InternalServerError(StaticError),
    Error((Status, Json<crate::dto::Error>)),
}

impl Error {
    pub fn not_found_error() -> Self {
        Error::NotFoundError(StaticError::not_found())
    }

    pub fn internal_server_error() -> Self {
        Error::InternalServerError(StaticError::internal_server_error())
    }

    pub fn error(status: Status, error: String) -> Self {
        Error::Error((status, Json(crate::dto::Error { error })))
    }
}

impl From<FileServiceError> for Error {
    fn from(err: FileServiceError) -> Self {
        match err {
            FileServiceError::FileNotYetFilled => {
                Error::error(Status::UnprocessableEntity, format!("file not yet filled"))
            }
            _ => Self::internal_server_error(),
        }
    }
}

#[post("/<staging_file_id>")]
async fn create_file(
    #[allow(unused_variables)] user_session: AuthUserSession<'_>,
    file_service: &State<Arc<FileService>>,
    staging_file_id: Uuid,
) -> Result<(Status, Json<File>), Error> {
    let file = file_service
        .create_file_from_staging_file_id(staging_file_id)
        .await?;
    let file = match file {
        Some(file) => file,
        None => {
            return Err(Error::not_found_error());
        }
    };

    Ok((Status::Created, Json(file)))
}

#[delete("/<file_id>")]
async fn remove_file(
    #[allow(unused_variables)] user_session: AuthUserSession<'_>,
    file_service: &State<Arc<FileService>>,
    file_id: Uuid,
) -> Result<(Status, Json<File>), Error> {
    let file = file_service.remove_file_by_id(file_id).await?;
    let file = match file {
        Some(file) => file,
        None => {
            return Err(Error::not_found_error());
        }
    };

    Ok((Status::Ok, Json(file)))
}

#[get("/<file_id>")]
async fn get_file(
    #[allow(unused_variables)] user_session: AuthUserSession<'_>,
    file_service: &State<Arc<FileService>>,
    file_id: Uuid,
) -> Result<(Status, Json<File>), Error> {
    let file = file_service.get_file_by_id(file_id).await?;
    let file = match file {
        Some(file) => file,
        None => {
            return Err(Error::not_found_error());
        }
    };

    Ok((Status::Ok, Json(file)))
}

#[get("/<file_id>/data")]
async fn get_file_data(
    #[allow(unused_variables)] user_session: AuthUserSession<'_>,
    file_service: &State<Arc<FileService>>,
    file_id: Uuid,
) -> Result<(Status, ReaderStream![Pin<Box<dyn AsyncRead + Send>>]), Error> {
    let data = file_service.get_file_data_by_id(file_id).await?;
    let data = match data {
        Some(data) => data,
        None => {
            return Err(Error::not_found_error());
        }
    };

    let stream = ReaderStream::one(data);
    Ok((Status::Ok, stream))
}
