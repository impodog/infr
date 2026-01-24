use bevy::prelude::*;

use infr_transfer as transfer;

pub type ErrorWriter<'w> = MessageWriter<'w, ErrorMessage>;

#[macro_export]
macro_rules! try_report {
    ($result: expr, $error_writer: expr, $returns: expr) => {
        match $result {
            Ok(result) => result,
            Err(err) => {
                $error_writer.write($crate::ErrorMessage(err));
                return $returns;
            }
        }
    };
    ($result: expr, $error_writer: expr) => {
        $crate::try_report!($result, $error_writer, Ok(()))
    };
}

#[derive(Debug, Clone, Message)]
pub struct ErrorMessage(pub transfer::ServerError);

pub(crate) fn handle_messages(mut reader: MessageReader<ErrorMessage>) {
    for message in reader.read() {
        match &message.0 {
            transfer::ServerError::ServerSide(msg) => {
                error!("Received server side error: {msg}");
            }
            transfer::ServerError::BadRequest(msg) => {
                error!("Client sent bad request: {msg}");
            }
            _ => {
                todo!("Implement game state error handler")
            }
        }
    }
}
