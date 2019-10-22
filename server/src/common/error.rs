use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

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
        E: fmt::Debug + Send + Sync + 'static,
    {
        Error {
            kind,
            message: format!("{} => {:#?}", String::from(message), last_error),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.format_message(f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.format_message(f)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    fn format_message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if self.message.len() > 0 {
            write!(f, ": {}", self.message)?;
        }

        Ok(())
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
    RoomNotFound,
    AlreadyJoinedError,
    UnauthenticatedError,

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

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{:?}", self),
        }
    }
}

pub trait ContextExt<T> {
    fn context<F, M>(self, ctx_fn: F) -> Result<T>
    where
        F: FnOnce() -> (ErrorKind, M),
        String: From<M>;
}

impl<T, E> ContextExt<T> for std::result::Result<T, E>
where
    E: fmt::Debug + Send + Sync + 'static,
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

impl<T> ContextExt<T> for std::option::Option<T> {
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

pub trait ErrorKindExt<T> {
    fn kind<F>(self, ctx_fn: F) -> Result<T>
    where
        F: FnOnce() -> ErrorKind;
}

impl<T, E> ErrorKindExt<T> for std::result::Result<T, E> {
    fn kind<F>(self, ctx_fn: F) -> Result<T>
    where
        F: FnOnce() -> ErrorKind,
    {
        self.map_err(|_err| {
            let kind = ctx_fn();
            Error::from(kind)
        })
    }
}

impl<T> ErrorKindExt<T> for std::option::Option<T> {
    fn kind<F>(self, ctx_fn: F) -> Result<T>
    where
        F: FnOnce() -> ErrorKind,
    {
        self.ok_or_else(|| {
            let kind = ctx_fn();
            Error::from(kind)
        })
    }
}
