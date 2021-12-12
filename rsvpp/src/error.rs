macro_rules! lazy_error_impl {
    ($fn:ident, $kind:ident) => {
        pub fn $fn<S: Into<String>>(msg: S) -> Self {
            Self::$kind(msg.into())
        }
    };
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Unknown error: `{0}`")]
    Unknown(String),
    #[error("Internal error: `{0}`")]
    Internal(String),
    #[error("Argument error: `{0}`")]
    Argument(String),
    #[error("Message id mismatch: `{0}`")]
    MsgIdMismatch(String),
    #[error("Crc id mismatch: `{0}`")]
    CrcMismatch(String),
    #[error("App api error: `{0}`")]
    VppApi(String),
    #[error("Timeout error: `{0}`")]
    Timeout(String),
}

impl Error {
    lazy_error_impl! {internal, Internal}
    lazy_error_impl! {argument, Argument}
    lazy_error_impl! {msg_id_mismatch, MsgIdMismatch}
    lazy_error_impl! {crc_mismatch, CrcMismatch}
    lazy_error_impl! {vpp_api, VppApi}
    lazy_error_impl! {timeout, Timeout}
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Self::internal(format!("{}", e))
    }
}
