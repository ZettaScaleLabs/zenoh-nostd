zenoh_derive::make_error! {
    #[doc = "Errors related to zenoh bytes."]
    enum ZBytesError {
        #[doc = "Destination buffer is full."]
        #[err = "destination buffer is full"]
        DstIsFull = 0,

        #[doc = "Source buffer is empty."]
        #[err = "source buffer is empty"]
        SrcIsEmpty = 1,

        #[doc = "Destination buffer is too small."]
        #[err = "destination buffer is too small"]
        DstIsTooSmall = 2,
    }

    #[doc = "Errors related to zenoh codec."]
    enum ZCodecError: ZBytesError {
        #[doc = "Could not parse header."]
        #[err = "could not parse header"]
        CouldNotParseHeader = 3,

        #[doc = "Could not parse field."]
        #[err = "could not parse field"]
        CouldNotParseField = 4,
    }


    #[doc = "Errors related to zenoh link."]
    enum ZLinkError: ZCodecError + ZBytesError {
        #[doc = "Link transmission error."]
        #[err = "link transmission error"]
        LinkTxError = 5,

        #[doc = "Link destination buffer is too small."]
        #[err = "link destination buffer is too small"]
        LinkRxDstIsTooSmall = 6,
    }
}

fn main() {
    let berror = ZBytesError::DstIsFull;
    println!("{}", berror);

    let cerror = ZCodecError::CouldNotParseField;
    println!("{}", cerror);

    let lerror = ZLinkError::LinkTxError;
    println!("{}", lerror);

    let zerror = ZError::from(lerror);

    println!("{}", zerror);
}
