use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

#[derive(Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    #[inline]
    pub fn new<M>(kind: ErrorKind, message: M) -> Self
    where
        String: From<M>,
    {
        Error {
            kind,
            message: String::from(message),
        }
    }

    // last_error is formatted as part of string instead of being an object
    // because automock requires return to implement Clone trait, and
    // Box<dyn Error> cannot be cloned
    #[inline]
    pub fn new_with_last_error<M, E>(kind: ErrorKind, message: M, last_error: E) -> Self
    where
        String: From<M>,
        E: Debug + Send + Sync + 'static,
    {
        Error {
            kind,
            message: format!("{} => {:#?}", String::from(message), last_error),
        }
    }
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl StdError for Error {}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    ServerListenError,
    InternalServerError,
    RemoteServerError,
    BadRequest,

    MissingDatabaseUrl,
    ActixRuntimeError,
    ConnectionPoolError,
    InsertionError,
    QueryError,

    InvalidOAuthConfig,
    OAuthClientNotBuild,
    UnauthorizedError,
    UnsupportedProviderError,

    MissingClientError,

    SendMessageError,
    DeserializationError,

    InvalidParams,
    UnknownError,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::new(kind, "")
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            _ => write!(f, "{:?}", self),
        }
    }
}

pub trait ResultExt<T> {
    fn context<F, M>(self, ctx_fn: F) -> Result<T>
    where
        F: FnOnce() -> (ErrorKind, M),
        String: From<M>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Debug + Send + Sync + 'static,
{
    fn context<F, M>(self, ctx_fn: F) -> Result<T>
    where
        F: FnOnce() -> (ErrorKind, M),
        String: From<M>,
    {
        self.map_err(|err| {
            let (kind, message) = ctx_fn();
            Error::new_with_last_error(kind, message, Box::new(err))
        })
    }
}

impl<T> ResultExt<T> for std::option::Option<T> {
    fn context<F, M>(self, ctx_fn: F) -> Result<T>
    where
        F: FnOnce() -> (ErrorKind, M),
        String: From<M>,
    {
        self.ok_or_else(|| {
            let (kind, message) = ctx_fn();
            Error::new(kind, message)
        })
    }
}
