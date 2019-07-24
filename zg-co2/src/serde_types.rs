use super::SingleReading;
use serde::de::{EnumAccess, Error, SeqAccess, Unexpected, VariantAccess, Visitor};
use serde::export;
use serde::ser::SerializeTupleVariant;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Formatter};
use std::marker::PhantomData;

impl Serialize for SingleReading {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SingleReading::Humidity(val) => {
                serializer.serialize_newtype_variant("SingleReading", 0, "Humidity", val)
            }
            SingleReading::Temperature(val) => {
                serializer.serialize_newtype_variant("SingleReading", 1, "Temperature", val)
            }
            SingleReading::CO2(val) => {
                serializer.serialize_newtype_variant("SingleReading", 2, "CO2", val)
            }
            SingleReading::Unknown(kind, val) => {
                let mut tv =
                    serializer.serialize_tuple_variant("SingleReading", 4, "Unknown", 2)?;
                tv.serialize_field(kind)?;
                tv.serialize_field(val)?;
                tv.end()
            }
            _ => unreachable!(),
        }
    }
}

impl<'de> Deserialize<'de> for SingleReading {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Humidity,
            Temperature,
            CO2,
            Unknown,
        }

        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("variant identifier")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    0 => Ok(Field::Humidity),
                    1 => Ok(Field::Temperature),
                    2 => Ok(Field::CO2),
                    3 => Ok(Field::Unknown),
                    _ => Err(Error::invalid_value(
                        Unexpected::Unsigned(v),
                        &"variant index 0 <= i < 4",
                    )),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    "Humidity" => Ok(Field::Humidity),
                    "Temperature" => Ok(Field::Temperature),
                    "CO2" => Ok(Field::CO2),
                    "Unknown" => Ok(Field::Unknown),
                    _ => Err(Error::unknown_variant(v, VARIANTS)),
                }
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    b"Humidity" => Ok(Field::Humidity),
                    b"Temperature" => Ok(Field::Temperature),
                    b"CO2" => Ok(Field::CO2),
                    b"Unknown" => Ok(Field::Unknown),
                    _ => {
                        let __value = &export::from_utf8_lossy(v);
                        Err(Error::unknown_variant(__value, VARIANTS))
                    }
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct SingleReadingVisitor<'de> {
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de> Visitor<'de> for SingleReadingVisitor<'de> {
            type Value = SingleReading;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("enum SingleReading")
            }

            fn visit_enum<E>(self, data: E) -> Result<Self::Value, E::Error>
            where
                E: EnumAccess<'de>,
            {
                match data.variant()? {
                    (Field::Humidity, variant) => {
                        variant.newtype_variant().map(SingleReading::Humidity)
                    }
                    (Field::Temperature, variant) => {
                        variant.newtype_variant().map(SingleReading::Temperature)
                    }
                    (Field::CO2, variant) => variant.newtype_variant().map(SingleReading::CO2),
                    (Field::Unknown, variant) => {
                        struct UnknownVisitor<'de> {
                            lifetime: PhantomData<&'de ()>,
                        }

                        impl<'de> Visitor<'de> for UnknownVisitor<'de> {
                            type Value = SingleReading;

                            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                                formatter.write_str("tuple variant SingleReading::Unknown")
                            }

                            #[inline]
                            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                            where
                                S: SeqAccess<'de>,
                            {
                                let kind = seq.next_element()?.ok_or_else(|| {
                                    Error::invalid_length(
                                        0,
                                        &"tuple variant SingleReading::Unknown with 2 elements",
                                    )
                                })?;
                                let value = seq.next_element()?.ok_or_else(|| {
                                    Error::invalid_length(
                                        1,
                                        &"tuple variant SingleReading::Unknown with 2 elements",
                                    )
                                })?;
                                Ok(SingleReading::Unknown(kind, value))
                            }
                        }
                        variant.tuple_variant(
                            2,
                            UnknownVisitor {
                                lifetime: PhantomData,
                            },
                        )
                    }
                }
            }
        }

        const VARIANTS: &'static [&'static str] = &["Humidity", "Temperature", "CO2", "Unknown"];
        deserializer.deserialize_enum(
            "SingleReading",
            VARIANTS,
            SingleReadingVisitor {
                lifetime: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SingleReading;
    use serde_test::Token;

    #[test]
    fn test_serde() {
        let measurement = SingleReading::Humidity(52.3);
        serde_test::assert_tokens(
            &measurement,
            &[
                Token::NewtypeVariant {
                    name: "SingleReading",
                    variant: "Humidity",
                },
                Token::F32(52.3),
            ],
        );

        let measurement = SingleReading::Temperature(20.0);
        serde_test::assert_tokens(
            &measurement,
            &[
                Token::NewtypeVariant {
                    name: "SingleReading",
                    variant: "Temperature",
                },
                Token::F32(20.0),
            ],
        );

        let measurement = SingleReading::CO2(462);
        serde_test::assert_tokens(
            &measurement,
            &[
                Token::NewtypeVariant {
                    name: "SingleReading",
                    variant: "CO2",
                },
                Token::U16(462),
            ],
        );

        let measurement = SingleReading::Unknown(b'R', 10438);
        serde_test::assert_tokens(
            &measurement,
            &[
                Token::TupleVariant {
                    name: "SingleReading",
                    variant: "Unknown",
                    len: 2,
                },
                Token::U8(b'R'),
                Token::U16(10438),
                Token::TupleVariantEnd,
            ],
        );
    }
}
