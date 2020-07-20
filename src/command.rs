use crate::error::Result;
use bytes::BytesMut;
use std::fmt::Debug;

pub trait Command {
    const PROTOCOL_ID: &'static [u8];
    const COMMAND_NAME: &'static str;

    type Request: Debug + Request;
    type Response: Debug + Response;
}

pub trait Request {
    fn encode(&self) -> Result<Option<BytesMut>> {
        Ok(None)
    }
}

impl Request for () {}

pub trait Response
where
    Self: Sized,
{
    fn decode(src: &mut BytesMut) -> Result<Self>;
}
