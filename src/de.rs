use crate::error::{Error, ErrorCode, Result};
use crate::parser::{IndentState, Parser};
use core::str::FromStr;
use serde_core::de;

#[must_use]
pub(crate) struct Deserializer<P> {
    parser: P,
    is_first: bool,
    next_to_parse: ElemType,
}

#[derive(Clone, Copy)]
enum ElemType {
    Key,
    Value,
}

impl<'a, P> Deserializer<P>
where
    P: Parser<'a>,
{
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            is_first: true,
            next_to_parse: ElemType::Key,
        }
    }

    fn parse(&mut self) -> Result<&'a str> {
        match self.next_to_parse {
            ElemType::Key => self.parser.parse_key(),
            ElemType::Value => self.parser.parse_value(),
        }
    }

    fn parse_from_str<T, E>(&mut self, error: E) -> Result<T>
    where
        T: FromStr,
        E: FnOnce() -> ErrorCode,
    {
        let value = self.parse()?;

        T::from_str(value).map_err(|_| {
            Error::new(error(), unsafe {
                self.parser.position_of_ptr(value.as_ptr())
            })
        })
    }

    fn parse_bool(&mut self) -> Result<bool> {
        self.parse_from_str(|| ErrorCode::InvalidBool)
    }

    fn parse_int<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        self.parse_from_str(|| ErrorCode::InvalidInt)
    }

    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        self.parse_from_str(|| ErrorCode::InvalidFloat)
    }

    fn parse_char(&mut self) -> Result<char> {
        self.parse_from_str(|| ErrorCode::InvalidChar)
    }

    #[must_use]
    fn locate_error(&self, error: Error, last_key_index: usize) -> Error {
        if !error.position().is_default() {
            return error;
        }

        error.with_position(self.parser.position_of_index(last_key_index))
    }
}

#[must_use]
struct KeyValueAccess<'a, P> {
    de: &'a mut Deserializer<P>,
    key_indent: u32,
}

impl<'a, 'b, P> KeyValueAccess<'a, P>
where
    P: Parser<'b>,
{
    fn new(de: &'a mut Deserializer<P>) -> Self {
        let key_indent = de.parser.last_key_indent();

        let key_indent = if de.is_first {
            de.is_first = false;

            if key_indent == 0 {
                0
            } else {
                key_indent + 1
            }
        } else {
            key_indent + 1
        };

        Self { de, key_indent }
    }
}

impl<'de, P> de::MapAccess<'de> for KeyValueAccess<'_, P>
where
    P: Parser<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        let has_next = match self.de.parser.skip_whitespace()? {
            IndentState::Start(indent) => indent >= self.key_indent,
            IndentState::Middle => true,
            IndentState::Eof => false,
        };

        if !has_next {
            return Ok(None);
        }

        self.de.next_to_parse = ElemType::Key;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.next_to_parse = ElemType::Value;
        seed.deserialize(&mut *self.de)
    }
}

impl<'de, 'a, P> de::SeqAccess<'de> for KeyValueAccess<'a, P>
where
    P: Parser<'de> + 'a,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        loop {
            let has_next = match self.de.parser.skip_whitespace()? {
                IndentState::Start(indent) => indent >= self.key_indent,
                IndentState::Middle => true,
                IndentState::Eof => false,
            };

            if !has_next {
                return Ok(None);
            }

            let key = self.de.parser.parse_key()?;

            if !key.is_empty() {
                self.de.parser.parse_value()?;
                continue;
            }

            self.de.next_to_parse = ElemType::Value;
            break seed.deserialize(&mut *self.de).map(Some);
        }
    }
}

impl<'de, 'a, P> de::EnumAccess<'de> for KeyValueAccess<'a, P>
where
    P: Parser<'de> + 'a,
{
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.next_to_parse = ElemType::Key;
        let key = seed.deserialize(&mut *self.de)?;
        Ok((key, KeyValueAccess::new(&mut *self.de)))
    }
}

impl<'de, 'a, P> de::VariantAccess<'de> for KeyValueAccess<'a, P>
where
    P: Parser<'de> + 'a,
{
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        self.de.parser.parse_value()?;
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.de.next_to_parse = ElemType::Value;
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

impl<'de, P> de::Deserializer<'de> for &mut Deserializer<P>
where
    P: Parser<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.parse_int()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_char(self.parse_char()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.parse()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.parse()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let is_some = match self.parser.skip_whitespace()? {
            IndentState::Start(indent) => indent > self.parser.last_key_indent(),
            IndentState::Middle => true,
            IndentState::Eof => false,
        };

        if is_some {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let last_key_index = self.parser.last_key_index();

        visitor
            .visit_seq(KeyValueAccess::new(self))
            .map_err(|e| self.locate_error(e, last_key_index))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let last_key_index = self.parser.last_key_index();

        visitor
            .visit_map(KeyValueAccess::new(self))
            .map_err(|e| self.locate_error(e, last_key_index))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let last_key_index = self.parser.last_key_index();

        visitor
            .visit_enum(KeyValueAccess::new(self))
            .map_err(|e| self.locate_error(e, last_key_index))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
