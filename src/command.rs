use crate::error::Result;
use bytes::BytesMut;

pub trait Command {}

pub trait Request {
    const PROTOCOL_ID: &'static [u8];
    const COMMAND_NAME: &'static str;

    fn encode(&self) -> Result<Option<BytesMut>> {
        Ok(None)
    }
}

pub trait Response
where
    Self: Sized,
{
    fn decode(src: &mut BytesMut) -> Result<Self>;
}
