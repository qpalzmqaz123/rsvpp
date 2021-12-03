use std::fmt::{Display, Formatter};

macro_rules! lazy_error_impl {
    ($fn:ident, $kind:ident) => {
        pub fn $fn<S: Into<String>>(msg: S) -> Self {
            Self {
                kind: ErrorKind::$kind,
                message: msg.into(),
            }
        }
    };
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Argument,
    Internal,
    MsgIdMismatch,
    VppApi,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl Error {
    lazy_error_impl! {argument, Argument}
    lazy_error_impl! {internal, Internal}
    lazy_error_impl! {msg_id_mismatch, Internal}
    lazy_error_impl! {vpp_api, VppApi}
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Self::internal(format!("{}", e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::internal(format!("{}", e))
    }
}
