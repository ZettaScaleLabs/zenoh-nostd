#![no_std]

pub use zenoh_log;

#[cfg(feature = "ctx")]
use heapless::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ZErrorKind {
    // ==== Generic & Internal ====
    Failed,
    UnsupportedPlatform,
    Timeout,
    CapacityExceeded,
    FmtError,
    UnImplemented,

    // ==== IO / Serialization ====
    WriteFailure,
    ReadFailure,
    ConversionFailure,
    SiphonFailure,
    MalformedVLE,

    // ==== Addressing / Connection ====
    InvalidAddress,
    InvalidConfiguration,
    InvalidProtocol,
    ConnectionRefused,

    // ==== Validation / Arguments ====
    InvalidArgument,
    InvalidId,
    InvalidBits,
    InvalidPriorityValue,
    InvalidPriorityRange,
    InvalidReliabilityValue,
    InvalidEndPoint,
    EndPointTooBig,
    InvalidWhatAmI,
    MandatoryFieldMissing,

    // ==== Expression / KeyExpr ====
    InvalidKeyExpr,
    KeyExprValidationFailed,
    ScopedKeyExprUnsupported,
    SuffixedKeyExprUnsupported,
    WildExprContainsInvalidChunks,

    // ==== Message ====
    InvalidMessage,
    CannotAccessField,
}

pub type ZE = ZErrorKind;

impl core::fmt::Display for ZErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <Self as core::fmt::Debug>::fmt(self, f)
    }
}

impl core::error::Error for ZErrorKind {}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ZError {
    kind: ZErrorKind,
    file: &'static str,
    line: u32,
    column: u32,

    #[cfg(feature = "ctx")]
    contexts: Vec<&'static str, 16>,
}

impl core::fmt::Debug for ZError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} at {}:{}:{}",
            self.kind(),
            self.file(),
            self.line(),
            self.column()
        )?;

        #[cfg(feature = "ctx")]
        if self.contexts.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "ctx")]
        if self.contexts.len() == 1 {
            write!(f, "\nContext:")?;
        } else {
            write!(f, "\nContexts:")?;
        }

        #[cfg(feature = "ctx")]
        let mut i = 0;
        #[cfg(feature = "ctx")]
        for context in &self.contexts {
            write!(f, "\n  {}: {}", i, context)?;
            i += 1;
        }

        Ok(())
    }
}

impl core::fmt::Display for ZError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <Self as core::fmt::Debug>::fmt(self, f)
    }
}

impl ZError {
    pub fn new(kind: ZErrorKind, file: &'static str, line: u32, column: u32) -> Self {
        Self {
            kind,
            file,
            line,
            column,
            #[cfg(feature = "ctx")]
            contexts: Vec::new(),
        }
    }

    pub fn kind(&self) -> &ZErrorKind {
        &self.kind
    }

    pub fn file(&self) -> &'static str {
        self.file
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn column(&self) -> u32 {
        self.column
    }
}

pub trait WithContext {
    fn ctx(self, context: &'static str) -> Self;
}

impl WithContext for ZError {
    #[cfg(feature = "ctx")]
    fn ctx(mut self, context: &'static str) -> Self {
        let _ = self.contexts.push(context);

        self
    }
    #[cfg(not(feature = "ctx"))]
    fn ctx(self, _context: &'static str) -> Self {
        self
    }
}

impl<T> WithContext for Result<T, ZError> {
    #[cfg(feature = "ctx")]
    fn ctx(self, context: &'static str) -> Self {
        self.map_err(|e| e.ctx(context))
    }
    #[cfg(not(feature = "ctx"))]
    fn ctx(self, _context: &'static str) -> Self {
        self
    }
}

pub type ZResult<T> = core::result::Result<T, ZError>;

#[macro_export]
macro_rules! zerr {
    ($kind:expr) => {
        $crate::ZError::new($kind, file!(), line!(), column!())
    };
}

#[cfg(feature = "ctx")]
#[macro_export]
macro_rules! zctx {
    () => {
        format_args!("at {}:{}:{}", file!(), line!(), column!())
            .as_str()
            .unwrap() // Safe to unwrap
    };

    ($ctx:expr) => {{
        const CHECK: fn(&'static str) = |_| {};
        CHECK($ctx);

        format_args!("{} at {}:{}:{}", $ctx, file!(), line!(), column!())
            .as_str()
            .unwrap() // Safe to unwrap because $ctx is &'static str
    }};

    ($ctx:expr, $($arg:tt)*) => {{
        const CHECK: fn(&'static str) = |_| {};
        CHECK($ctx);

        match format_args!($ctx, $($arg)*).as_str() {
            Some(_) => format_args!("{} at {}:{}:{}", format_args!($ctx, $($arg)*), file!(), line!(), column!())
                .as_str()
                .unwrap(),
            None => {
                $crate::zenoh_log::warn!("zctx! macro at {}:{}:{}: context cannot be known at compile time!", file!(), line!(), column!());

                format_args!("{} at {}:{}:{}", $ctx, file!(), line!(), column!())
                .as_str()
                .unwrap()
            },
        }
    }};
}

#[cfg(not(feature = "ctx"))]
#[macro_export]
macro_rules! zctx {
    () => {
        ""
    };

    ($ctx:expr) => {{
        ""
    }};

    ($ctx:expr, $($arg:tt)*) => {{
        ""
    }};
}

#[macro_export]
macro_rules! zbail {
    ($kind:expr) => {
        return Err($crate::zerr!($kind));
    };
}

impl From<core::fmt::Error> for ZError {
    fn from(_: core::fmt::Error) -> Self {
        zerr!(ZE::FmtError)
    }
}
