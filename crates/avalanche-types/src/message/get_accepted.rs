use std::io::{self, Error, ErrorKind};

use crate::{ids, message, proto::pb::p2p};
use prost::Message as ProstMessage;

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub msg: p2p::GetAccepted,
    pub gzip_compress: bool,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            msg: p2p::GetAccepted {
                chain_id: prost::bytes::Bytes::new(),
                request_id: 0,
                deadline: 0,
                container_ids: Vec::new(),
            },
            gzip_compress: false,
        }
    }
}

impl Message {
    #[must_use]
    pub fn chain_id(mut self, chain_id: ids::Id) -> Self {
        self.msg.chain_id = prost::bytes::Bytes::from(chain_id.to_vec());
        self
    }

    #[must_use]
    pub const fn request_id(mut self, request_id: u32) -> Self {
        self.msg.request_id = request_id;
        self
    }

    #[must_use]
    pub const fn deadline(mut self, deadline: u64) -> Self {
        self.msg.deadline = deadline;
        self
    }

    #[must_use]
    pub fn container_ids(mut self, container_ids: &[ids::Id]) -> Self {
        let mut container_ids_bytes: Vec<prost::bytes::Bytes> =
            Vec::with_capacity(container_ids.len());
        for id in container_ids {
            container_ids_bytes.push(prost::bytes::Bytes::from(id.to_vec()));
        }
        self.msg.container_ids = container_ids_bytes;
        self
    }

    #[must_use]
    pub const fn gzip_compress(mut self, gzip_compress: bool) -> Self {
        self.gzip_compress = gzip_compress;
        self
    }

    /// Serializes the message into bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the serialization fails.
    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let msg = p2p::Message {
            message: Some(p2p::message::Message::GetAccepted(self.msg.clone())),
        };
        let encoded = ProstMessage::encode_to_vec(&msg);
        if !self.gzip_compress {
            return Ok(encoded);
        }

        let uncompressed_len = encoded.len();
        let compressed = message::compress::pack_gzip(&encoded)?;
        let msg = p2p::Message {
            message: Some(p2p::message::Message::CompressedZstd(
                prost::bytes::Bytes::from(compressed),
            )),
        };

        let compressed_len = msg.encoded_len();
        if uncompressed_len > compressed_len {
            log::debug!(
                "get_accepted compression saved {} bytes",
                uncompressed_len - compressed_len
            );
        } else {
            log::debug!(
                "get_accepted compression added {} byte(s)",
                compressed_len - uncompressed_len
            );
        }

        Ok(ProstMessage::encode_to_vec(&msg))
    }

    /// Deserializes the message from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the deserialization fails.
    ///
    /// # Panics
    ///
    /// Panics if the message field is None.
    pub fn deserialize(d: impl AsRef<[u8]>) -> io::Result<Self> {
        let buf = bytes::Bytes::from(d.as_ref().to_vec());
        let p2p_msg: p2p::Message = ProstMessage::decode(buf).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("failed prost::Message::decode '{e}'"),
            )
        })?;

        match p2p_msg
            .message
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "message field is None"))?
        {
            // was not compressed
            p2p::message::Message::GetAccepted(msg) => Ok(Self {
                msg,
                gzip_compress: false,
            }),

            // was compressed, so need decompress first
            p2p::message::Message::CompressedZstd(msg) => {
                let decompressed = message::compress::unpack_gzip(msg.as_ref())?;
                let decompressed_msg: p2p::Message =
                    ProstMessage::decode(prost::bytes::Bytes::from(decompressed)).map_err(|e| {
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("failed prost::Message::decode '{e}'"),
                        )
                    })?;
                match decompressed_msg.message.ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        "message field is None after decompression",
                    )
                })? {
                    p2p::message::Message::GetAccepted(msg) => Ok(Self {
                        msg,
                        gzip_compress: false,
                    }),
                    _ => Err(Error::new(
                        ErrorKind::InvalidInput,
                        "unknown message type after decompress",
                    )),
                }
            }

            // unknown message enum
            _ => Err(Error::new(ErrorKind::InvalidInput, "unknown message type")),
        }
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `message::get_accepted::test_message` --exact --show-output
#[test]
fn test_message() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let msg1_with_no_compression = Message::default()
        .chain_id(ids::Id::from_slice(
            &random_manager::secure_bytes(32).unwrap(),
        ))
        .request_id(random_manager::u32())
        .deadline(random_manager::u64())
        .container_ids(&[
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::empty(),
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
            ids::Id::from_slice(&random_manager::secure_bytes(32).unwrap()),
        ]);

    let data1 = msg1_with_no_compression.serialize().unwrap();
    let msg1_with_no_compression_deserialized = Message::deserialize(data1).unwrap();
    assert_eq!(
        msg1_with_no_compression,
        msg1_with_no_compression_deserialized
    );

    let msg2_with_compression = msg1_with_no_compression.clone().gzip_compress(true);
    assert_ne!(msg1_with_no_compression, msg2_with_compression);

    let data2 = msg2_with_compression.serialize().unwrap();
    let msg2_with_compression_deserialized = Message::deserialize(data2).unwrap();
    assert_eq!(msg1_with_no_compression, msg2_with_compression_deserialized);
}
