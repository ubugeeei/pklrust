use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("pkl server error: {0}")]
    PklServer(String),

    #[error("pkl evaluation error: {0}")]
    Evaluation(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("msgpack decode error: {0}")]
    MsgpackDecode(#[from] rmpv::decode::Error),

    #[error("msgpack encode error: {0}")]
    MsgpackEncode(String),

    #[error("unexpected message type: 0x{0:02X}")]
    UnexpectedMessageType(u8),

    #[error("unknown pkl type code: 0x{0:02X}")]
    UnknownTypeCode(u8),

    #[error("unknown member code: 0x{0:02X}")]
    UnknownMemberCode(u8),

    #[error("decode error: {0}")]
    Decode(String),

    #[error("deserialize error: {0}")]
    Deserialize(String),

    #[error("process error: {0}")]
    Process(String),

    #[error("evaluator not found: {0}")]
    EvaluatorNotFound(i64),

    #[error("request timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, Error>;

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Deserialize(msg.to_string())
    }
}
