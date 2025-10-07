use crate::error::CairoSerializeError;
use anyhow::anyhow;
use num_bigint::BigUint;
use num_traits::Num;
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

/// Convert any serializable value to a list of felts (BigUint)
pub fn to_felts<T: Serialize>(value: &T) -> std::result::Result<Vec<BigUint>, CairoSerializeError> {
    let mut serializer = CairoArgsSerializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.felts)
}

/// Convert any serializable value to hex strings for Scarb runner
pub fn to_runner_args_hex<T: Serialize>(
    value: &T,
) -> std::result::Result<Vec<String>, CairoSerializeError> {
    let felts = to_felts(value)?;
    Ok(felts
        .into_iter()
        .map(|felt| format!("0x{}", felt.to_str_radix(16)))
        .collect())
}

/// Custom Serde serializer for Cairo types
pub struct CairoArgsSerializer {
    felts: Vec<BigUint>,
}

impl CairoArgsSerializer {
    pub fn new() -> Self {
        Self { felts: Vec::new() }
    }

    fn push_felt(&mut self, felt: BigUint) {
        self.felts.push(felt);
    }

    fn serialize_digest_64hex(
        &mut self,
        hex_str: &str,
    ) -> std::result::Result<(), CairoSerializeError> {
        // Convert 64-char hex to 8 u32 words (reversed for little-endian)
        let bytes = hex::decode(hex_str)
            .map_err(|e| CairoSerializeError::from(anyhow!("Invalid hex: {}", e)))?;
        for chunk in bytes.chunks(4).rev() {
            let mut word_bytes = [0u8; 4];
            word_bytes[..chunk.len()].copy_from_slice(chunk);
            self.push_felt(BigUint::from(u32::from_be_bytes(word_bytes)));
        }
        Ok(())
    }

    fn serialize_bytearray_0xhex(
        &mut self,
        hex_str: &str,
    ) -> std::result::Result<(), CairoSerializeError> {
        // Remove 0x prefix
        let hex_data = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(hex_data)
            .map_err(|e| CairoSerializeError::from(anyhow!("Invalid hex: {}", e)))?;

        // Calculate chunks and remainder
        let chunk_size = 31; // 31 bytes per chunk (248 bits, fits in felt252)
        let num_chunks = bytes.len() / chunk_size;
        let remainder_len = bytes.len() % chunk_size;

        // Serialize: num_chunks, chunks..., remainder, rem_len
        self.push_felt(BigUint::from(num_chunks));

        // Serialize chunks
        for chunk in bytes.chunks(chunk_size) {
            if chunk.len() == chunk_size {
                let mut chunk_bytes = [0u8; 32];
                chunk_bytes[1..=chunk_size].copy_from_slice(chunk);
                self.push_felt(BigUint::from_bytes_be(&chunk_bytes));
            }
        }

        // Serialize remainder
        if remainder_len > 0 {
            let remainder = &bytes[bytes.len() - remainder_len..];
            let mut rem_bytes = [0u8; 32];
            rem_bytes[1..=remainder_len].copy_from_slice(remainder);
            self.push_felt(BigUint::from_bytes_be(&rem_bytes));
        } else {
            self.push_felt(BigUint::from(0u8));
        }

        self.push_felt(BigUint::from(remainder_len));
        Ok(())
    }

    fn serialize_bytearray_string(
        &mut self,
        s: &str,
    ) -> std::result::Result<(), CairoSerializeError> {
        let hex_str = format!("0x{}", hex::encode(s.as_bytes()));
        self.serialize_bytearray_0xhex(&hex_str)
    }

    fn serialize_u256_decimal(
        &mut self,
        decimal_str: &str,
    ) -> std::result::Result<(), CairoSerializeError> {
        let bigint = BigUint::from_str_radix(decimal_str, 10)
            .map_err(|e| CairoSerializeError::from(anyhow!("Invalid decimal: {}", e)))?;
        let (lo, hi) = split_u256(bigint);
        self.push_felt(lo);
        self.push_felt(hi);
        Ok(())
    }
}

// Helper to split u256 into lo/hi parts
fn split_u256(value: BigUint) -> (BigUint, BigUint) {
    let mask = (BigUint::from(1u128) << 128) - BigUint::from(1u8);
    let lo = &value & &mask;
    let hi = &value >> 128;
    (lo, hi)
}

impl<'a> ser::Serializer for &'a mut CairoArgsSerializer {
    type Ok = ();
    type Error = CairoSerializeError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(if v { 1u8 } else { 0u8 }));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v as u8));
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v as u16));
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v as u32));
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v as u64));
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v as u128));
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v));
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v));
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v));
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v));
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v));
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> std::result::Result<(), CairoSerializeError> {
        Err(CairoSerializeError::from(anyhow!(
            "f32 not supported in Cairo"
        )))
    }

    fn serialize_f64(self, _v: f64) -> std::result::Result<(), CairoSerializeError> {
        Err(CairoSerializeError::from(anyhow!(
            "f64 not supported in Cairo"
        )))
    }

    fn serialize_char(self, v: char) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(v as u32));
        Ok(())
    }

    fn serialize_str(self, v: &str) -> std::result::Result<(), CairoSerializeError> {
        // Determine string type based on format
        if v.len() == 64 && v.chars().all(|c| c.is_ascii_hexdigit()) {
            // 64-char hex string -> Digest (8 u32 words, reversed)
            self.serialize_digest_64hex(v)?;
        } else if v.starts_with("0x") {
            // 0x-prefixed hex -> ByteArray
            self.serialize_bytearray_0xhex(v)?;
        } else if v.chars().all(|c| c.is_ascii_digit()) {
            // Decimal string -> u256 (lo, hi)
            self.serialize_u256_decimal(v)?;
        } else {
            // Regular string -> ByteArray
            self.serialize_bytearray_string(v)?;
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> std::result::Result<(), CairoSerializeError> {
        // Convert bytes to ByteArray format
        let hex_str = format!("0x{}", hex::encode(v));
        self.serialize_bytearray_0xhex(&hex_str)
    }

    fn serialize_none(self) -> std::result::Result<(), CairoSerializeError> {
        self.push_felt(BigUint::from(0u8));
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        self.push_felt(BigUint::from(1u8));
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_unit(self) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }

    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<(), CairoSerializeError> {
        // Serialize variant name as string
        variant.serialize(self)?;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        variant.serialize(&mut *self)?;
        value.serialize(&mut *self)?;
        Ok(())
    }

    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> std::result::Result<&'a mut CairoArgsSerializer, CairoSerializeError> {
        if let Some(len) = len {
            self.push_felt(BigUint::from(len));
        }
        Ok(self)
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<&'a mut CairoArgsSerializer, CairoSerializeError> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<&'a mut CairoArgsSerializer, CairoSerializeError> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<&'a mut CairoArgsSerializer, CairoSerializeError> {
        Ok(self)
    }

    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> std::result::Result<&'a mut CairoArgsSerializer, CairoSerializeError> {
        if let Some(len) = len {
            self.push_felt(BigUint::from(len));
        }
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<&'a mut CairoArgsSerializer, CairoSerializeError> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<&'a mut CairoArgsSerializer, CairoSerializeError> {
        Ok(self)
    }
}

// Serializer implementations for collections
impl<'a> SerializeSeq for &'a mut CairoArgsSerializer {
    type Ok = ();
    type Error = CairoSerializeError;

    fn serialize_element<T: ?Sized>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }
}

impl<'a> SerializeTuple for &'a mut CairoArgsSerializer {
    type Ok = ();
    type Error = CairoSerializeError;

    fn serialize_element<T: ?Sized>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for &'a mut CairoArgsSerializer {
    type Ok = ();
    type Error = CairoSerializeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }
}

impl<'a> SerializeTupleVariant for &'a mut CairoArgsSerializer {
    type Ok = ();
    type Error = CairoSerializeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }
}

impl<'a> SerializeMap for &'a mut CairoArgsSerializer {
    type Ok = ();
    type Error = CairoSerializeError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)?;
        Ok(())
    }

    fn serialize_value<T: ?Sized>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }
}

impl<'a> SerializeStruct for &'a mut CairoArgsSerializer {
    type Ok = ();
    type Error = CairoSerializeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }
}

impl<'a> SerializeStructVariant for &'a mut CairoArgsSerializer {
    type Ok = ();
    type Error = CairoSerializeError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> std::result::Result<(), CairoSerializeError>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> std::result::Result<(), CairoSerializeError> {
        Ok(())
    }
}
