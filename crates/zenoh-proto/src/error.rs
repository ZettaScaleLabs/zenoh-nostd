crate::declare_zerror! {
    #[doc = "Errors related to zenoh bytes."]
    enum ZBytesError {
        #[doc = "Source buffer is empty."]
        #[err = "source buffer is empty"]
        SrcIsEmpty = 100,
        #[doc = "Destination buffer is full."]
        #[err = "destination buffer is full"]
        DstIsFull = 101,
        #[doc = "Destination buffer is too small."]
        #[err = "destination buffer is too small"]
        DstIsTooSmall = 102,
        #[doc = "Source buffer is too small."]
        #[err = "source buffer is too small"]
        SrcIsTooSmall = 103,
    }

    #[doc = "Errors related to zenoh codec."]
    enum ZCodecError: ZBytesError {
        #[doc = "Could not complete a read operation."]
        #[err = "could not read"]
        CouldNotRead = 0,
        #[doc = "Could not complete a write operation."]
        #[err = "could not write"]
        CouldNotWrite = 1,
        #[doc = "Could not parse the header."]
        #[err = "could not parse header"]
        CouldNotParseHeader = 2,
        #[doc = "Could not parse the field."]
        #[err = "could not parse field"]
        CouldNotParseField = 3,
        #[doc = "Missing mandatory extension in protocol."]
        #[err = "missing mandatory extension"]
        MissingMandatoryExtension = 4,
    }

    #[doc = "Errors related to zenoh endpoints."]
    enum ZEndpointError {
        #[doc = "Missing protocol separator in endpoint."]
        #[err = "missing protocol separator in endpoint"]
        NoProtocolSeparator = 10,
        #[doc = "Metadata is not supported in endpoint."]
        #[err = "metadata not supported in endpoint"]
        MetadataNotSupported = 11,
        #[doc = "Configuration is not supported in endpoint."]
        #[err = "configuration not supported in endpoint"]
        ConfigNotSupported = 12,
        #[doc = "Could not parse the endpoint."]
        #[err = "could not parse endpoint"]
        CouldNotParseEndpoint = 13,
    }

    #[doc = "Errors related to zenoh key expression parsing."]
    pub enum ZKeyexprError {
        #[doc = "A lone `$*` was found in an expression."]
        #[err = "lone '$*' in expression"]
        LoneDollarStar = 20,
        #[doc = "A single `*` was found after a `**` in an expression."]
        #[err = "single '*' after '**' in expression"]
        SingleStarAfterDoubleStar = 21,
        #[doc = "A double `**` was found after a `**` in an expression."]
        #[err = "double '**' after '**' in expression"]
        DoubleStarAfterDoubleStar = 22,
        #[doc = "An empty chunk was found in an expression."]
        #[err = "empty chunk in expression"]
        EmptyChunk = 23,
        #[doc = "A `*` was found in the middle of a chunk in an expression."]
        #[err = "'*' in middle of chunk in expression"]
        StarInChunk = 24,
        #[doc = "A `$` was found after another `$` in an expression."]
        #[err = "'$' after '$' in expression"]
        DollarAfterDollar = 25,
        #[doc = "A `#` or `?` was found in an expression."]
        #[err = "'#' or '?' in expression"]
        SharpOrQMark = 26,
        #[doc = "An unbound `$n` was found in an expression."]
        #[err = "unbound '$n' in expression"]
        UnboundDollar = 27,
        #[doc = "A wildcard chunk was found where it is not allowed."]
        #[err = "wildcard chunk not allowed"]
        WildChunk = 28,
    }

    #[doc = "Errors related to zenoh connections."]
    enum ZConnectionError: ZCodecError {
        #[doc = "Could not get address info."]
        #[err = "could not get address info"]
        CouldNotGetAddrInfo = 30,
        #[doc = "Could not connect to the remote."]
        #[err = "could not connect to remote"]
        CouldNotConnect = 31,
    }

    #[doc = "Errors related to zenoh links."]
    enum ZLinkError: ZConnectionError + ZCodecError + ZEndpointError {
        #[doc = "Write operation failed."]
        #[err = "write operation failed"]
        WriteOperationFailed = 33,
        #[doc = "Read operation failed."]
        #[err = "read operation failed"]
        ReadOperationFailed = 34,
    }

    #[doc = "Errors related to zenoh transports."]
    enum ZTransportError: ZLinkError + ZCodecError {
        #[doc = "Invalid received data."]
        #[err = "invalid received data"]
        InvalidRx = 40,

        #[doc = "Operation timed out."]
        #[err = "operation timed out"]
        Timeout = 41,
    }

    #[doc = "Other errors."]
    enum ZGeneralError {
        #[doc = "Could not receive from channel."]
        #[err = "could not receive from channel"]
        CouldNotRecvFromChannel = 50,

        #[doc = "Capacity exceeded."]
        #[err = "capacity exceeded"]
        CapacityExceeded = 51,

        #[doc = "Callback already set for given id."]
        #[err = "callback already set for given id"]
        CallbackAlreadySet = 52,

        #[doc = "Connection closed by peer."]
        #[err = "connection closed by peer"]
        ConnectionClosed = 53,

        #[doc = "Could not spawn task."]
        #[err = "could not spawn task"]
        CouldNotSpawnTask = 54,
    }
}

pub type ZResult<T, E = ZError> = ::core::result::Result<T, E>;

#[macro_export]
macro_rules! zbail {
    ($err:expr) => {
        return Err($err)
    };
}
