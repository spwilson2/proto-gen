use serde::{Deserialize, Serialize};

type MessageId = str;

pub trait ProtoSerial
where
    Self: Sized,
{
    fn serialize_into(&self, buf: &mut [u8]) -> Result<(), ()>;
    //fn serialized_size(&self);
    fn try_deserialize(buf: &[u8]) -> Option<Self>;
    fn deserialize(buf: &[u8]) -> Self {
        Self::try_deserialize(buf).unwrap()
    }
}
