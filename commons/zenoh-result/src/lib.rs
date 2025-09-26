#![no_std]

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ZErrorKind {
    DidntConvert,
    DidntWrite,
    DidntRead,
    DidntSiphon,
    KeyExprValidation,
    InvalidKeyExpr,
    NonWildExprContainsWildChunks,
    CapacityExceeded,
    FmtError,
    CannotAccessField,
    EndPointTooBig,
    InvalidEndPoint,
    InvalidID,
    InvalidPriorityValue,
    InvalidPriorityRangeValue,
    InvalidReliabilityValue,
    InvalidBits,
    InvalidWhatAmI,
    InvalidArgument,
    ScopedKeyExpr,
    SuffixedKeyExpr,
    MandatoryFieldMissing,
    ConnectionRefused,
    InvalidAddress,
    InvalidConfiguration,
    InvalidProtocol,
    Timeout,
    InvalidMessage,
    Failed,
    UnsupportedPlatform,
}

pub type ZE = ZErrorKind;

impl core::fmt::Debug for ZErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DidntConvert => write!(f, "ZError::DidntConvert"),
            Self::DidntWrite => write!(f, "ZError::DidntWrite"),
            Self::DidntRead => write!(f, "ZError::DidntRead"),
            Self::DidntSiphon => {
                write!(f, "ZError::zerror!(zenoh_result::ZErrorKind::DidntSiphon)")
            }
            Self::KeyExprValidation => write!(f, "ZError::InvalidKeyExpr"),
            Self::InvalidKeyExpr => write!(f, "ZError::InvalidKeyExpr"),
            Self::NonWildExprContainsWildChunks => {
                write!(f, "ZError::NonWildExprContainsWildChunks")
            }
            Self::CapacityExceeded => write!(f, "ZError::CapacityExceeded"),
            Self::FmtError => write!(f, "ZError::FmtError"),
            Self::CannotAccessField => write!(f, "ZError::CannotAccessField"),
            Self::EndPointTooBig => write!(f, "ZError::EndPointTooBig"),
            Self::InvalidEndPoint => write!(f, "ZError::InvalidEndPoint: Endpoints must be of the form <protocol>/<address>[?<metadata>][#<config>]"),
            Self::InvalidID => write!(f, "ZError::InvalidID"),
            Self::InvalidPriorityValue => write!(f, "ZError::InvalidPriorityValue"),
            Self::InvalidPriorityRangeValue => write!(f, "ZError::InvalidPriorityRangeValue"),
            Self::InvalidReliabilityValue => write!(f, "ZError::InvalidReliabilityValue"),
            Self::InvalidBits => write!(f, "ZError::InvalidBits"),
            Self::InvalidWhatAmI => write!(f, "ZError::InvalidWhatAmI"),
            Self::InvalidArgument => write!(f, "ZError::InvalidArgument"),
            Self::ScopedKeyExpr => write!(f, "ZError::ScopedKeyExpr"),
            Self::SuffixedKeyExpr => write!(f, "ZError::SuffixedKeyExpr"),
            Self::MandatoryFieldMissing => write!(f, "ZError::MandatoryFieldMissing"),
            Self::ConnectionRefused => write!(f, "ZError::ConnectionRefused"),
            Self::InvalidAddress => write!(f, "ZError::InvalidAddress"),
            Self::InvalidConfiguration => write!(f, "ZError::InvalidConfiguration"),
            Self::InvalidProtocol => write!(f, "ZError::InvalidProtocol"),
            Self::Timeout => write!(f, "ZError::Timeout"),
            Self::InvalidMessage => write!(f, "ZError::InvalidMessage"),
            Self::Failed => write!(f, "ZError::Failed"),
            Self::UnsupportedPlatform => write!(f, "ZError::UnsupportedPlatform"),
        }
    }
}

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

    contexts: [Option<&'static str>; 16],
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

        write!(f, "\nContext:")?;

        for context in &self.contexts {
            if let Some(ctx) = context {
                write!(f, "\n  - {}", ctx)?;
            }
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
            contexts: [None; 16],
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
    fn context(self, context: &'static str) -> Self;
}

impl WithContext for ZError {
    fn context(mut self, context: &'static str) -> Self {
        for slot in &mut self.contexts {
            if slot.is_none() {
                *slot = Some(context);
                break;
            }
        }

        self
    }
}

impl<T> WithContext for Result<T, ZError> {
    fn context(self, context: &'static str) -> Self {
        self.map_err(|e| e.context(context))
    }
}

pub type ZResult<T> = core::result::Result<T, ZError>;

#[macro_export]
macro_rules! zerr {
    ($kind:expr) => {
        $crate::ZError::new($kind, file!(), line!(), column!())
    };
}

#[macro_export]
macro_rules! zctx {
    ($ctx:expr) => {
        format_args!("{} at {}:{}:{}", $ctx, file!(), line!(), column!())
            .as_str()
            .unwrap()
    };
}

#[macro_export]
macro_rules! bail {
    ($kind:expr) => {
        return Err($crate::zerr!($kind))
    };
}
