use serde::{Deserializer, Serializer};

pub trait SerializeTuple {
  fn serialize_tuple<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer;
}

pub trait DeserializeTuple<'de>: Sized {
  fn deserialize_tuple<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>;
}
