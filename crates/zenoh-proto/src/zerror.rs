//! This module defines all the errors that should exists in a `zenoh` context.

crate::declare_zerror! {
    // --- Protocol related errors ---

    #[doc = "Errors related to zenoh bytes."]
    enum BytesError {
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
    enum CodecError: BytesError {
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
        #[doc = "Could not read the extension."]
        #[err = "could not read extension"]
        CouldNotReadExtension = 4,
    }

    #[doc = "Errors related to zenoh key expression parsing."]
    pub enum KeyexprError {
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

    // --- IO related errors (not used in this crate) ---

    #[doc = "Errors related to zenoh endpoints."]
    enum EndpointError {
        #[doc = "Missing protocol separator in endpoint."]
        #[err = "missing protocol separator in endpoint"]
        NoProtocolSeparator = 10,
        #[doc = "Metadata is not supported in endpoint."]
        #[err = "metadata not supported in endpoint"]
        MetadataNotSupported = 11,
        #[doc = "Configuration is not supported in endpoint."]
        #[err = "configuration not supported in endpoint"]
        ConfigNotSupported = 12,
        #[doc = "Could not parse the endpoint address."]
        #[err = "could not parse endpoint address"]
        CouldNotParseAddress = 13,
        #[doc = "Could not parse the endpoint protocol."]
        #[err = "could not parse endpoint protocol"]
        CouldNotParseProtocol = 14,
    }

    #[doc = "Errors related to zenoh connections."]
    enum ConnectionError: CodecError {
        #[doc = "Could not get address info."]
        #[err = "could not get address info"]
        CouldNotGetAddrInfo = 30,
        #[doc = "Could not connect to the remote."]
        #[err = "could not connect to remote"]
        CouldNotConnect = 31,
    }

    #[doc = "Errors related to zenoh links."]
    enum LinkError: ConnectionError + CodecError + EndpointError {
        #[doc = "Link transmission failed."]
        #[err = "link transmission failed"]
        LinkTxFailed = 33,
        #[doc = "Link reception failed."]
        #[err = "link reception failed"]
        LinkRxFailed = 34,
    }

    #[doc = "Errors related to zenoh transports."]
    enum TransportError: LinkError + CodecError {
        #[doc = "Received invalid data."]
        #[err = "received invalid data"]
        InvalidRx = 40,
        #[doc = "Transport open timed out."]
        #[err = "transport open timed out"]
        OpenTimeout = 41,
        #[doc = "Transport lease timed out."]
        #[err = "transport lease timed out"]
        LeaseTimeout = 42,
        #[doc = "Transport has been closed."]
        #[err = "transport has been closed"]
        TransportClosed = 53,
    }

    // --- Collections related errors ---

    #[doc = "Errors related to zenoh collections."]
    enum CollectionError {
        #[doc = "Key not found in collection."]
        #[err = "key not found in collection"]
        KeyNotFound = 60,
        #[doc = "Key already exists in collection."]
        #[err = "key already exists in collection"]
        KeyAlreadyExists = 61,
        #[doc = "Collection is full."]
        #[err = "collection is full"]
        CollectionIsFull = 62,
        #[doc = "Collection is empty."]
        #[err = "collection is empty"]
        CollectionIsEmpty = 63,
    }

    // --- Session related errors ---

    #[doc = "Errors related to zenoh embassy integration."]
    enum SessionError {
        #[doc = "Could not spawn embassy task."]
        #[err = "could not spawn embassy task"]
        CouldNotSpawnEmbassyTask = 70,
        #[doc = "Channel is closed."]
        #[err = "channel is closed"]
        ChannelClosed = 80,
        #[doc = "Request timed out."]
        #[err = "request timed out"]
        RequestTimedout = 81,
    }
}

#[macro_export]
macro_rules! zbail {
    ($err:expr) => {
        return Err($err.into())
    };

    ($err:expr, $($arg:tt)+) => {{
        $crate::error!("{}: {}", $err, $crate::zctx!());
        $crate::error!($($arg)+);
        $crate::zbail!($err);
    }};
}

#[macro_export]
macro_rules! zctx {
    () => {
        concat!(core::file!(), ":", core::line!(), ":", core::column!(),)
    };
}
