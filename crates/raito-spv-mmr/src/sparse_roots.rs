//! Sparse roots representation for MMR peaks compatible with Cairo implementation.

use accumulators::mmr::elements_count_to_leaf_count;
use num_bigint::{BigInt, BigUint};
use num_traits::Num;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use std::str::FromStr;

/// Sparse roots is MMR peaks for all heights, where missing ones are filled with zeros
/// This representation is different from the "compact" one, which contains only non-zero peaks
/// but with total number of elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseRoots {
    /// Block height
    #[serde(skip)]
    pub block_height: u32,
    /// MMR peaks for all heights, where missing ones are filled with zeros
    #[serde(
        serialize_with = "serialize_u256_array",
        deserialize_with = "deserialize_u256_array"
    )]
    pub roots: Vec<String>,
}

impl SparseRoots {
    pub fn try_from_peaks(
        peaks: Vec<String>,
        mut elements_count: usize,
    ) -> Result<Self, anyhow::Error> {
        let leaf_count = elements_count_to_leaf_count(elements_count)?;
        let null_root = format!("0x{:064x}", 0);

        let mut max_height = elements_count.ilog2() + 1;
        let mut root_idx = 0;
        let mut result = vec![];

        while elements_count != 0 || max_height != 0 {
            // Number of elements of the perfect binary tree of the current max height
            let elements_per_height = (1 << max_height) - 1;
            if elements_count >= elements_per_height {
                result.insert(0, peaks[root_idx].clone());
                root_idx += 1;
                elements_count -= elements_per_height;
            } else {
                result.insert(0, null_root.clone());
            }
            if max_height != 0 {
                max_height -= 1;
            }
        }

        if result.last().unwrap() != &null_root {
            result.push(null_root);
        }

        Ok(Self {
            roots: result,
            // Last block height is the number of leaves - 1
            block_height: leaf_count as u32 - 1,
        })
    }
}

/// Custom serialization for Vec<String> to serialize as array of u256 (in Cairo)
pub fn serialize_u256_array<S>(items: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(items.len()))?;
    for item in items {
        let num_str = item.strip_prefix("0x").unwrap_or(&item);
        // TODO: figure out how to forward `truncated` flag here from hasher
        if false {
            // Cast to BigInt and back to string to handle leading zeros
            let json_number = num_str_to_json_number::<S>(num_str)?;
            seq.serialize_element(&json_number)?;
        } else {
            assert_eq!(num_str.len(), 64);
            let (hi, lo) = num_str.split_at(32);
            let hi_json_number = num_str_to_json_number::<S>(hi)?;
            let lo_json_number = num_str_to_json_number::<S>(lo)?;
            // Serialize as a dict with `hi` and `lo` keys (u256 in Cairo)
            let mut dict = serde_json::Map::new();
            dict.insert("hi".to_string(), hi_json_number.into());
            dict.insert("lo".to_string(), lo_json_number.into());
            seq.serialize_element(&dict)?;
        }
    }
    seq.end()
}

/// Custom deserialization for Vec<String> where items are Cairo u256 objects {hi, lo}
pub fn deserialize_u256_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::Deserialize as _;

    let values = Vec::<serde_json::Value>::deserialize(deserializer)?;
    let mut result: Vec<String> = Vec::with_capacity(values.len());

    for value in values {
        match value {
            serde_json::Value::Object(mut map) => {
                let hi_val = map
                    .remove("hi")
                    .ok_or_else(|| D::Error::custom("missing 'hi' in u256 object"))?;
                let lo_val = map
                    .remove("lo")
                    .ok_or_else(|| D::Error::custom("missing 'lo' in u256 object"))?;

                let hi_hex = json_value_to_padded_hex::<D::Error>(hi_val)?;
                let lo_hex = json_value_to_padded_hex::<D::Error>(lo_val)?;
                result.push(format!("0x{}{}", hi_hex, lo_hex));
            }
            other => {
                return Err(D::Error::custom(format!(
                    "unsupported u256 representation: {}",
                    other
                )))
            }
        }
    }

    Ok(result)
}

fn json_value_to_padded_hex<E>(value: serde_json::Value) -> Result<String, E>
where
    E: DeError,
{
    match value {
        serde_json::Value::Number(n) => {
            // Convert decimal string to 32-byte (64 hex chars) lowercased hex
            let dec_str = n.to_string();
            let big = BigUint::from_str(&dec_str)
                .map_err(|e| E::custom(format!("invalid decimal number: {}", e)))?;
            Ok(format!("{:032x}", big))
        }
        other => Err(E::custom(format!(
            "unexpected type for u256 limb: {}",
            other
        ))),
    }
}

/// Convert a hex string to a JSON number
/// What we are doing here is making sure we get `{"key": 123123}` instead of `{"key": "123123"}`
fn num_str_to_json_number<S>(num_str: &str) -> Result<serde_json::Number, S::Error>
where
    S: Serializer,
{
    let bigint = BigInt::from_str_radix(num_str, 16)
        .map_err(|e| serde::ser::Error::custom(format!("Failed to parse BigInt: {}", e)))?;
    let json_number = serde_json::Number::from_str(&bigint.to_string())
        .map_err(|e| serde::ser::Error::custom(format!("Failed to serialize BigInt: {}", e)))?;
    Ok(json_number)
}
