pub mod codec;
pub mod command;
pub mod error;
pub mod inverter;

#[cfg(test)]
mod test {
    use crate::codec::Codec;
    use crate::command::{Command, Response};
    use crate::error::Result;
    use bytes::BytesMut;
    use std::str::FromStr;
    use tokio_util::codec::{Decoder, Encoder};

    struct QPI;

    impl Command for QPI {
        const PROTOCOL_ID: &'static [u8] = b"QPI";
        const COMMAND_NAME: &'static str = "QueryProtocolId";

        type Request = ();
        type Response = QPIResponse;
    }

    #[derive(Debug)]
    struct QPIResponse(pub usize);

    impl Response for QPIResponse {
        fn decode(src: &mut BytesMut) -> Result<Self> {
            assert!(src.starts_with(b"PI"));

            Ok(Self(usize::from_str(std::str::from_utf8(
                src[2..].as_ref(),
            )?)?))
        }
    }

    #[test]
    fn test() {
        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::init();

        let mut codec = Codec::<QPI>::new();

        let mut buf = BytesMut::new();
        codec.encode((), &mut buf).unwrap();

        let mut buf = BytesMut::from(b"(PI1234\xe3\x52\r".as_ref());
        codec.decode(&mut buf).unwrap().unwrap();
    }
}
