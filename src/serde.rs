use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serde serializer/deserializer for fastnum::decimal::D256 with a clickhouse Decimal256(25)
#[cfg(feature = "fastnum")]
pub mod fastnum_decimal_25 {
    use super::*;

    use fastnum::decimal::D256;
    use fastnum::{bint::UInt, dec256};

    const FIXED_EXPONENT: i16 = 25;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<D256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;

        // Convert the [u8; 32] into [u64; 4] by chunking and using from_le_bytes
        let mut uint = UInt::from_radix_le(&bytes, 256)
            .ok_or(D::Error::custom("failed to deserialize uint from bytes"))?;

        // We receive the number scaled to the fixed exponent defined in the schema
        uint /= UInt::from_u16(10u16).pow((FIXED_EXPONENT as u16).into());

        let value = D256::from_parts(
            UInt::from_le(uint),
            -FIXED_EXPONENT as i32,
            fastnum::decimal::Sign::Plus,
            fastnum::decimal::Context::default(),
        );
        Ok(value)
    }

    pub fn serialize<S: Serializer>(value: &D256, serializer: S) -> Result<S::Ok, S::Error> {
        // Scale the value to match the fixed exponent defined in the schema
        let mut value = value.rescale(FIXED_EXPONENT);
        // Transform the fractional part to have significant zeros, so we have them in the value.digits()
        value *= dec256!(10).pow(FIXED_EXPONENT.into());

        let mut digits_le: Vec<u8> = value.digits().to_radix_le(256);

        let padding_bytes = vec![0; 32 - digits_le.len()];
        digits_le.extend(padding_bytes);

        // We want to serialize a bytes not the vector struct
        let exact_bytes: [u8; 32] = digits_le.try_into().unwrap();
        exact_bytes.serialize(serializer)
    }
}

/// Serde serializer/deserializer for HashMap<String, String> for a clickhouse Map(String, String)
pub mod map {
    use std::collections::HashMap;

    use super::*;

    pub fn serialize<S: Serializer>(
        value: &HashMap<String, String>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // Clickhouse expects a Vec<(String, String)> for maps
        let value: Vec<(String, String)> = value.clone().into_iter().collect();
        value.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Vec<(String, String)> = Deserialize::deserialize(deserializer)?;
        Ok(value.into_iter().collect())
    }
}
