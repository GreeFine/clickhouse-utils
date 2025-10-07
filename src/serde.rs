use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "fastnum")]
pub mod fastnum {
    use super::*;

    #[cfg(feature = "fastnum")]
    use ::fastnum::{
        bint::UInt,
        decimal::{Context, Decimal as FastnumDecimal, Sign},
    };
    use serde::de::Error as _;

    pub struct Decimal<const E: i16, const S: usize>;

    /// Serde serializer/deserializer for fastnum::decimal with a clickhouse Decimal<FIXED_EXPONENT, N>
    ///
    /// N is the number of internal 64 bits words, for example a D256 is a Decimal<X, 4>  
    /// DECIMAL_PLACE is the number of digits after the decimal point
    #[cfg(feature = "fastnum")]
    impl<const DECIMAL_PLACE: i16, const N: usize> Decimal<DECIMAL_PLACE, N> {
        pub fn deserialize<'de, D>(deserializer: D) -> Result<FastnumDecimal<N>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let bytes: [u8; 32] = Deserialize::deserialize(deserializer)?;

            // Convert the [u8; 32] into [u64; 4] by chunking and using from_le_bytes
            let mut uint = UInt::from_radix_le(&bytes, 256)
                .ok_or(D::Error::custom("failed to deserialize uint from bytes"))?;

            // We receive the number scaled to the fixed exponent defined in the schema
            uint /= UInt::from_u16(10u16).pow((DECIMAL_PLACE as u16).into());

            let value = FastnumDecimal::from_parts(
                UInt::from_le(uint),
                -DECIMAL_PLACE as i32,
                Sign::Plus,
                Context::default(),
            );
            Ok(value)
        }

        pub fn serialize<S: Serializer>(
            value: &FastnumDecimal<N>,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            // Scale the value to match the fixed exponent defined in the schema
            let mut value = value.rescale(DECIMAL_PLACE);
            // Transform the fractional part to have significant zeros, so we have them in the value.digits()
            value *= FastnumDecimal::from_u16(10u16).pow((DECIMAL_PLACE as u16).into());

            let mut digits_le: Vec<u8> = value.digits().to_radix_le(256);

            let padding_bytes = vec![0; 32 - digits_le.len()];
            digits_le.extend(padding_bytes);

            // We want to serialize a bytes not the vector struct
            let exact_bytes: [u8; 32] = digits_le.try_into().unwrap();
            exact_bytes.serialize(serializer)
        }
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
