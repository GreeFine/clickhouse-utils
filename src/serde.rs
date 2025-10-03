use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "fastnum")]
pub mod fastnum_d256 {
    use super::*;

    use fastnum::bint::UInt;
    use fastnum::decimal::D256;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<D256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: [u64; 4] = Deserialize::deserialize(deserializer)?;
        dbg!(&bytes);

        Ok(D256::from_parts(
            UInt::from_digits(bytes),
            0,
            fastnum::decimal::Sign::Plus,
            fastnum::decimal::Context::default(),
        ))
    }

    pub fn serialize<S: Serializer>(value: &D256, serializer: S) -> Result<S::Ok, S::Error> {
        let mut digits: Vec<u8> = value.digits().to_radix_be(10);
        digits.resize(32, 0);
        // Safety, we resize to 32 bytes, above
        let bytes: [u8; 32] = digits.try_into().expect("should be 32 bytes");

        bytes.serialize(serializer)
    }
}

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
