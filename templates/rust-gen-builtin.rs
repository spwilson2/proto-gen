use serde::{Deserialize, Serialize};

type MessageId = &'static str;

pub trait ProtoMessage
where
    Self: Sized,
{
    /// Serialize the message and a header into a buffer
    /// TODO: Need to serialize RPCs, not Messages...
    fn serialize_body_into(&self, buf: &mut [u8]) -> Result<(), ()>;
    fn serialized_size(&self) -> usize;
    fn try_deserialize_body(buf: &[u8]) -> Option<Self>;
    fn deserialize_body(buf: &[u8]) -> Self {
        Self::try_deserialize_body(buf).unwrap()
    }
}
pub trait ProtoRpcArg
where
    Self: Sized,
{
    type Arg: ProtoMessage;
    fn serialize_rpc_msg_into(&self, buf: &mut [u8]) -> Result<(), ()>;
    fn serialized_size(&self) -> usize;
    fn try_deserialize_body(buf: &[u8]) -> Option<<Self as ProtoRpcArg>::Arg> {
        Self::Arg::try_deserialize_body(buf)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RpcHeader {
    msg_id: String,
}

impl ProtoMessage for RpcHeader {
    fn serialize_body_into(&self, buf: &mut [u8]) -> Result<(), ()> {
        // Not supported for headers. Nothing to serialize.
        Err(())
    }
    fn serialized_size(&self) -> usize {
        serde_json::to_string(self).unwrap().len()
    }
    fn try_deserialize_body(buf: &[u8]) -> Option<Self> {
        let bounds = find_struct_bounds(buf)?;
        serde_json::from_slice(&buf[..bounds]).ok()
    }
}

// Required for Serde JSON serialization
pub fn find_struct_bounds(r: &[u8]) -> Option<usize> {
    let mut iter = r.iter();
    if iter.next() == Some(&('{'.to_ascii_lowercase() as u8)) {
        let mut count = 1;
        for b in iter {
            count += 1;
            if &('}'.to_ascii_lowercase() as u8) == b {
                return Some(count);
            }
        }
    }
    None
}
