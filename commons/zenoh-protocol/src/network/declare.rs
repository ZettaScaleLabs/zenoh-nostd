use crate::network::declare::{
    common::DeclareFinal,
    keyexpr::{DeclareKeyExpr, UndeclareKeyExpr},
    queryable::{DeclareQueryable, UndeclareQueryable},
    subscriber::{DeclareSubscriber, UndeclareSubscriber},
    token::{DeclareToken, UndeclareToken},
};

pub mod flag {
    pub const I: u8 = 1 << 5; // 0x20 Interest      if I==1 then the declare is in a response to an Interest with future==false
                              // pub const X: u8 = 1 << 6; // 0x40 Reserved
    pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
}

/// ```text
/// Flags:
/// - I: Interest       If I==1 then interest_id is present
/// - X: Reserved
/// - Z: Extension      If Z==1 then at least one extension is present
///
/// 7 6 5 4 3 2 1 0
/// +-+-+-+-+-+-+-+-+
/// |Z|X|I| DECLARE |
/// +-+-+-+---------+
/// ~interest_id:z32~  if I==1
/// +---------------+
/// ~  [decl_exts]  ~  if Z==1
/// +---------------+
/// ~  declaration  ~
/// +---------------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declare<'a> {
    pub interest_id: Option<super::interest::InterestId>,
    pub ext_qos: ext::QoSType,
    pub ext_tstamp: Option<ext::TimestampType>,
    pub ext_nodeid: ext::NodeIdType,
    pub body: DeclareBody<'a>,
}

pub mod ext {
    use crate::{zextz64, zextzbuf};

    pub type QoS = zextz64!(0x1, false);
    pub type QoSType = crate::network::ext::QoSType<{ QoS::ID }>;

    pub type Timestamp<'a> = zextzbuf!('a, 0x2, false);
    pub type TimestampType = crate::network::ext::TimestampType<{ Timestamp::ID }>;

    pub type NodeId = zextz64!(0x3, true);
    pub type NodeIdType = crate::network::ext::NodeIdType<{ NodeId::ID }>;
}

pub mod id {
    pub const D_KEYEXPR: u8 = 0x00;
    pub const U_KEYEXPR: u8 = 0x01;

    pub const D_SUBSCRIBER: u8 = 0x02;
    pub const U_SUBSCRIBER: u8 = 0x03;

    pub const D_QUERYABLE: u8 = 0x04;
    pub const U_QUERYABLE: u8 = 0x05;

    pub const D_TOKEN: u8 = 0x06;
    pub const U_TOKEN: u8 = 0x07;

    pub const D_FINAL: u8 = 0x1A;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclareBody<'a> {
    DeclareKeyExpr(DeclareKeyExpr<'a>),
    UndeclareKeyExpr(UndeclareKeyExpr),
    DeclareSubscriber(DeclareSubscriber<'a>),
    UndeclareSubscriber(UndeclareSubscriber<'a>),
    DeclareQueryable(DeclareQueryable<'a>),
    UndeclareQueryable(UndeclareQueryable<'a>),
    DeclareToken(DeclareToken<'a>),
    UndeclareToken(UndeclareToken<'a>),
    DeclareFinal(DeclareFinal),
}

pub mod common {
    /// ```text
    /// Flags:
    /// - X: Reserved
    /// - X: Reserved
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|X|X| D_FINAL |
    /// +---------------+
    /// ~ [final_exts]  ~  if Z==1
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DeclareFinal;

    pub mod ext {
        use crate::{
            core::wire_expr::{ExprId, WireExpr},
            network::Mapping,
            zextzbuf,
        };

        /// ```text
        /// Flags:
        /// - N: Named          If N==1 then the key expr has name/suffix
        /// - M: Mapping        if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
        ///
        ///  7 6 5 4 3 2 1 0
        /// +-+-+-+-+-+-+-+-+
        /// |X|X|X|X|X|X|M|N|
        /// +-+-+-+---------+
        /// ~ key_scope:z16 ~
        /// +---------------+
        /// ~  key_suffix   ~  if N==1 -- <u8;z16>
        /// +---------------+
        /// ```
        pub type WireExprExt<'a> = zextzbuf!('a, 0x0f, true);
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct WireExprType<'a> {
            pub wire_expr: WireExpr<'a>,
        }

        impl WireExprType<'_> {
            pub fn null() -> Self {
                Self {
                    wire_expr: WireExpr {
                        scope: ExprId::MIN,
                        suffix: "",
                        mapping: Mapping::Receiver,
                    },
                }
            }

            pub fn is_null(&self) -> bool {
                self.wire_expr.is_empty()
            }
        }
    }
}

pub mod keyexpr {
    use crate::core::wire_expr::{ExprId, WireExpr};

    pub mod flag {
        pub const N: u8 = 1 << 5; // 0x20 Named         if N==1 then the key expr has name/suffix
                                  // pub const X: u8 = 1 << 6; // 0x40 Reserved
        pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
    }

    /// ```text
    /// Flags:
    /// - N: Named          If N==1 then the key expr has name/suffix
    /// - X: Reserved
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    ///  7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|X|N| D_KEXPR |
    /// +---------------+
    /// ~  expr_id:z16  ~
    /// +---------------+
    /// ~ key_scope:z16 ~
    /// +---------------+
    /// ~  key_suffix   ~  if N==1 -- <u8;z16>
    /// +---------------+
    /// ~  [decl_exts]  ~  if Z==1
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DeclareKeyExpr<'a> {
        pub id: ExprId,
        pub wire_expr: WireExpr<'a>,
    }

    /// ```text
    /// Flags:
    /// - X: Reserved
    /// - X: Reserved
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|X|X| U_KEXPR |
    /// +---------------+
    /// ~  expr_id:z16  ~
    /// +---------------+
    /// ~  [decl_exts]  ~  if Z==1
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UndeclareKeyExpr {
        pub id: ExprId,
    }
}

pub mod subscriber {
    use super::*;
    use crate::core::{wire_expr::WireExpr, EntityId};

    pub type SubscriberId = EntityId;

    pub mod flag {
        pub const N: u8 = 1 << 5; // 0x20 Named         if N==1 then the key expr has name/suffix
        pub const M: u8 = 1 << 6; // 0x40 Mapping       if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
        pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
    }

    /// ```text
    /// Flags:
    /// - N: Named          If N==1 then the key expr has name/suffix
    /// - M: Mapping        if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|M|N|  D_SUB  |
    /// +---------------+
    /// ~  subs_id:z32  ~
    /// +---------------+
    /// ~ key_scope:z16 ~
    /// +---------------+
    /// ~  key_suffix   ~  if N==1 -- <u8;z16>
    /// +---------------+
    /// ~  [decl_exts]  ~  if Z==1
    /// +---------------+
    ///
    /// - if R==1 then the subscription is reliable, else it is best effort    ///
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DeclareSubscriber<'a> {
        pub id: SubscriberId,
        pub wire_expr: WireExpr<'a>,
    }

    /// ```text
    /// Flags:
    /// - X: Reserved
    /// - X: Reserved
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|X|X|  U_SUB  |
    /// +---------------+
    /// ~  subs_id:z32  ~
    /// +---------------+
    /// ~  [decl_exts]  ~  if Z==1
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UndeclareSubscriber<'a> {
        pub id: SubscriberId,
        pub ext_wire_expr: common::ext::WireExprType<'a>,
    }
}

pub mod queryable {
    use super::*;
    use crate::core::{wire_expr::WireExpr, EntityId};

    pub type QueryableId = EntityId;

    pub mod flag {
        pub const N: u8 = 1 << 5; // 0x20 Named         if N==1 then the key expr has name/suffix
        pub const M: u8 = 1 << 6; // 0x40 Mapping       if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
        pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
    }

    /// ```text
    /// Flags:
    /// - N: Named          If N==1 then the key expr has name/suffix
    /// - M: Mapping        if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|M|N|  D_QBL  |
    /// +---------------+
    /// ~  qbls_id:z32  ~
    /// +---------------+
    /// ~ key_scope:z16 ~
    /// +---------------+
    /// ~  key_suffix   ~  if N==1 -- <u8;z16>
    /// +---------------+
    /// ~  [decl_exts]  ~  if Z==1
    /// +---------------+
    ///
    /// - if R==1 then the queryable is reliable, else it is best effort
    /// - if P==1 then the queryable is pull, else it is push
    /// - if C==1 then the queryable is complete and the N parameter is present
    /// - if D==1 then the queryable distance is present
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DeclareQueryable<'a> {
        pub id: QueryableId,
        pub wire_expr: WireExpr<'a>,
        pub ext_info: ext::QueryableInfoType,
    }

    pub mod ext {
        use crate::zextz64;

        pub type QueryableInfo = zextz64!(0x01, false);

        pub mod flag {
            pub const C: u8 = 1; // 0x01 Complete      if C==1 then the queryable is complete
        }
        ///
        /// ```text
        ///  7 6 5 4 3 2 1 0
        /// +-+-+-+-+-+-+-+-+
        /// |Z|0_1|    ID   |
        /// +-+-+-+---------+
        /// |x|x|x|x|x|x|x|C|
        /// +---------------+
        /// ~ distance <z16>~
        /// +---------------+
        /// ```
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct QueryableInfoType {
            pub complete: bool, // Default false: incomplete
            pub distance: u16,  // Default 0: distance is null (e.g. intra-process communication)
        }

        impl QueryableInfoType {
            pub const DEFAULT: Self = Self {
                complete: false,
                distance: 0,
            };
        }

        impl Default for QueryableInfoType {
            fn default() -> Self {
                Self::DEFAULT
            }
        }
    }

    /// ```text
    /// Flags:
    /// - X: Reserved
    /// - X: Reserved
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|X|X|  U_QBL  |
    /// +---------------+
    /// ~  qbls_id:z32  ~
    /// +---------------+
    /// ~  [decl_exts]  ~  if Z==1
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UndeclareQueryable<'a> {
        pub id: QueryableId,
        pub ext_wire_expr: common::ext::WireExprType<'a>,
    }
}

pub mod token {
    use crate::core::wire_expr::WireExpr;

    use super::*;

    pub type TokenId = u32;

    pub mod flag {
        pub const N: u8 = 1 << 5; // 0x20 Named         if N==1 then the key expr has name/suffix
        pub const M: u8 = 1 << 6; // 0x40 Mapping       if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
        pub const Z: u8 = 1 << 7; // 0x80 Extensions    if Z==1 then an extension will follow
    }

    /// ```text
    /// Flags:
    /// - N: Named          If N==1 then the key expr has name/suffix
    /// - M: Mapping        if M==1 then key expr mapping is the one declared by the sender, else it is the one declared by the receiver
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|M|N|  D_TKN  |
    /// +---------------+
    /// ~ token_id:z32  ~
    /// +---------------+
    /// ~ key_scope:z16 ~
    /// +---------------+
    /// ~  key_suffix   ~  if N==1 -- <u8;z16>
    /// +---------------+
    /// ~  [decl_exts]  ~  if Z==1
    /// +---------------+
    ///
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DeclareToken<'a> {
        pub id: TokenId,
        pub wire_expr: WireExpr<'a>,
    }

    /// ```text
    /// Flags:
    /// - X: Reserved
    /// - X: Reserved
    /// - Z: Extension      If Z==1 then at least one extension is present
    ///
    /// 7 6 5 4 3 2 1 0
    /// +-+-+-+-+-+-+-+-+
    /// |Z|X|X|  U_TKN  |
    /// +---------------+
    /// ~ token_id:z32  ~
    /// +---------------+
    /// ~  [decl_exts]  ~  if Z==1
    /// +---------------+
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UndeclareToken<'a> {
        pub id: TokenId,
        pub ext_wire_expr: common::ext::WireExprType<'a>,
    }
}
