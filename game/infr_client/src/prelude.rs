pub use crate::config;
pub use crate::map::LevelError;
pub use infr_transfer::{self as transfer, Coord, ObjectId, ServerError, SessionId};

/// Marks http requests for counting.
#[derive(Default, bevy::prelude::Component)]
pub struct RequestMarker;

pub fn url_to(router: &str) -> String {
    if config::CONFIG.server.address.ends_with('/') {
        format!("http://{}{}", config::CONFIG.server.address, router)
    } else {
        format!("http://{}/{}", config::CONFIG.server.address, router)
    }
}

pub fn make_post_request<T: serde::Serialize>(
    router: &str,
    data: &T,
) -> bevy::prelude::Result<(RequestMarker, bevy_ehttp::HttpRequest)> {
    use bevy_ehttp::prelude::*;
    let body = serde_json::to_string(&data)?;
    let mut request = HttpRequest::post(&url_to(router), body.into_bytes());
    request.headers.insert("Content-Type", "application/json");
    Ok((RequestMarker, request))
}

pub fn make_get_request<T: serde::Serialize>(
    router: &str,
    data: &T,
) -> bevy::prelude::Result<(RequestMarker, bevy_ehttp::HttpRequest)> {
    use bevy_ehttp::prelude::*;
    let body = serde_json::to_string(&data)?;
    let mut request = HttpRequest::get(url_to(router));
    request.body = body.into();
    request.headers.insert("Content-Type", "application/json");
    Ok((RequestMarker, request))
}

#[derive(Debug, Clone)]
pub struct HttpError(pub String);
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Http request error: {}", self.0)
    }
}
impl std::error::Error for HttpError {}

pub fn parse_response<'de, T: serde::Deserialize<'de>>(
    response: &'de Result<bevy_ehttp::prelude::Response, String>,
) -> bevy::prelude::Result<Result<T, ServerError>> {
    let response = response.as_ref().map_err(|err| HttpError(err.into()))?;
    match response.status {
        // Ok
        200 => {
            let response: T = serde_json::from_slice(response.bytes.as_slice())?;
            Ok(Ok(response))
        }
        // Bad Request
        400 => {
            let response = response
                .bytes
                .clone()
                .try_into()
                .expect("Server should return valid UTF-8");
            bevy::prelude::warn!("Server responded Bad Request: {response}");
            Err(HttpError(response).into())
        }
        // Internal Server Error
        500 => {
            let response = response
                .bytes
                .clone()
                .try_into()
                .expect("Server should return valid UTF-8");
            bevy::prelude::warn!("Server responded Internal Server Error: {response}");
            Err(HttpError(response).into())
        }
        // Partial Content (used for reporting level errors)
        206 => {
            let response: transfer::ServerError =
                serde_json::from_slice(response.bytes.as_slice())?;
            Ok(Err(response))
        }
        status => {
            bevy::prelude::error!("Server responded with unexpected status code {status}");
            Err(HttpError(format!("Unexpected status code {status}")).into())
        }
    }
}

pub type LevelErrorWriter<'w> = bevy::prelude::MessageWriter<'w, LevelError>;

#[macro_export]
macro_rules! parse_response_and_report {
    ($type: ty, $writer: expr, $event: expr) => {
        $crate::parse_response_and_report!($type, $writer, $event, ())
    };
    ($type: ty, $writer: expr, $event: expr, $ok: expr) => {
        match parse_response::<$type>(&$event.response)? {
            Ok(value) => value,
            Err(server_error) => {
                ($writer).write($crate::prelude::LevelError(server_error));
                return Ok($ok);
            }
        }
    };
}

pub fn observe_discard_response(
    event: bevy::prelude::On<bevy_ehttp::ResponseString>,
    mut commands: bevy::prelude::Commands,
) {
    commands.entity(event.entity).despawn();
}
