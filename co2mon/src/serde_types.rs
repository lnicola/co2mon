use super::Reading;
use serde::de::{Error, IgnoredAny, MapAccess, SeqAccess, Unexpected, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Formatter};
use std::marker::PhantomData;

impl Serialize for Reading {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Reading", 2)?;
        s.serialize_field("temperature", &self.temperature)?;
        s.serialize_field("co2", &self.co2)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for Reading {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Temperature,
            CO2,
            Ignore,
        }

        struct FieldVisitor;
        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("field identifier")
            }
            fn visit_u64<E>(self, __value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match __value {
                    0 => Ok(Field::Temperature),
                    1 => Ok(Field::CO2),
                    _ => Err(Error::invalid_value(
                        Unexpected::Unsigned(__value),
                        &"field index 0 <= i < 2",
                    )),
                }
            }
            fn visit_str<E>(self, __value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match __value {
                    "temperature" => Ok(Field::Temperature),
                    "co2" => Ok(Field::CO2),
                    _ => Ok(Field::Ignore),
                }
            }
            fn visit_bytes<E>(self, __value: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match __value {
                    b"temperature" => Ok(Field::Temperature),
                    b"co2" => Ok(Field::CO2),
                    _ => Ok(Field::Ignore),
                }
            }
        }
        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        struct __Visitor<'de> {
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de> Visitor<'de> for __Visitor<'de> {
            type Value = Reading;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("struct Reading")
            }

            #[inline]
            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let temperature = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &"struct Reading with 2 elements"))?;
                let co2 = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(1, &"struct Reading with 2 elements"))?;
                Ok(Reading { temperature, co2 })
            }

            #[inline]
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut temperature = None;
                let mut co2 = None;
                while let Some(__key) = map.next_key()? {
                    match __key {
                        Field::Temperature => {
                            if temperature.is_some() {
                                return Err(Error::duplicate_field("temperature"));
                            }
                            temperature = Some(map.next_value()?);
                        }
                        Field::CO2 => {
                            if co2.is_some() {
                                return Err(Error::duplicate_field("co2"));
                            }
                            co2 = Some(map.next_value()?);
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }
                let temperature = temperature.ok_or_else(|| Error::missing_field("temperature"))?;
                let co2 = co2.ok_or_else(|| Error::missing_field("co2"))?;
                Ok(Reading { temperature, co2 })
            }
        }

        const FIELDS: &'static [&'static str] = &["temperature", "co2"];
        deserializer.deserialize_struct(
            "Reading",
            FIELDS,
            __Visitor {
                lifetime: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Reading;
    use serde_test::Token;

    #[test]
    fn test_serde() {
        let measurement = Reading {
            temperature: 20.5,
            co2: 645,
        };
        serde_test::assert_tokens(
            &measurement,
            &[
                Token::Struct {
                    name: "Reading",
                    len: 2,
                },
                Token::Str("temperature"),
                Token::F32(20.5),
                Token::Str("co2"),
                Token::U16(645),
                Token::StructEnd,
            ],
        );
    }
}
