use crate::error::{Error, ErrorKind, Result};
use crate::parser::{IndentState, Parser, Reference};
use serde::de;
use std::str::FromStr;

#[must_use]
pub(crate) struct Deserializer<P> {
    parser: P,
    is_first: bool,
    scratch: Vec<u8>,
    should_parse_value: bool,
}

impl<'a, P> Deserializer<P>
where
    P: Parser<'a>,
{
    pub fn new(parser: P) -> Self {
        Self {
            parser,
            is_first: true,
            scratch: Vec::new(),
            should_parse_value: false,
        }
    }

    fn parse<'s>(&'s mut self) -> Result<(P::Bookmark, Reference<'a, 's, str>)> {
        if self.should_parse_value {
            self.parse_value()
        } else {
            self.parse_key()
        }
    }

    fn parse_from_str<T, E>(&mut self, error: E) -> Result<T>
    where
        T: FromStr,
        E: FnOnce() -> ErrorKind,
    {
        let (bookmark, value) = self.parse()?;

        match T::from_str(&value) {
            Ok(value) => Ok(value),
            Err(_) => {
                let position = self.parser.position_of_bookmark(bookmark);
                Err(Error::new(error(), position))
            }
        }
    }

    fn parse_bool(&mut self) -> Result<bool> {
        self.parse_from_str(|| ErrorKind::InvalidBool)
    }

    fn parse_int<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        self.parse_from_str(|| ErrorKind::InvalidInt)
    }

    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        self.parse_from_str(|| ErrorKind::InvalidFloat)
    }

    fn parse_char(&mut self) -> Result<char> {
        self.parse_from_str(|| ErrorKind::InvalidChar)
    }

    fn parse_key<'s>(&'s mut self) -> Result<(P::Bookmark, Reference<'a, 's, str>)> {
        self.parser.parse_key(&mut self.scratch)
    }

    fn parse_value<'s>(&'s mut self) -> Result<(P::Bookmark, Reference<'a, 's, str>)> {
        self.parser.parse_value(&mut self.scratch)
    }

    fn skip_whitespace(&mut self) -> Result<IndentState> {
        self.parser.skip_whitespace(&mut self.scratch)
    }
}

#[must_use]
pub struct KeyValueAccess<'a, P> {
    deserializer: &'a mut Deserializer<P>,
    key_indent: u32,
}

impl<'a, 'b, P> KeyValueAccess<'a, P>
where
    P: Parser<'b>,
{
    pub fn new(deserializer: &'a mut Deserializer<P>) -> Self {
        let key_indent = deserializer.parser.last_key_indent();

        let key_indent = if deserializer.is_first {
            deserializer.is_first = false;

            if key_indent == 0 {
                0
            } else {
                key_indent + 1
            }
        } else {
            key_indent + 1
        };

        Self {
            deserializer,
            key_indent,
        }
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
        let has_next = match self.deserializer.skip_whitespace()? {
            IndentState::Start(indent) => indent >= self.key_indent,
            IndentState::Middle => true,
            IndentState::Eof => false,
        };

        if !has_next {
            return Ok(None);
        }

        self.deserializer.should_parse_value = false;
        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.deserializer.should_parse_value = true;
        seed.deserialize(&mut *self.deserializer)
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
            let has_next = match self.deserializer.skip_whitespace()? {
                IndentState::Start(indent) => indent >= self.key_indent,
                IndentState::Middle => true,
                IndentState::Eof => false,
            };

            if !has_next {
                return Ok(None);
            }

            let (_, key) = self.deserializer.parse_key()?;

            if !key.is_empty() {
                self.deserializer.parse_value()?;
                continue;
            }

            self.deserializer.should_parse_value = true;
            break seed.deserialize(&mut *self.deserializer).map(Some);
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
        self.deserializer.should_parse_value = false;
        let key = seed.deserialize(&mut *self.deserializer)?;
        Ok((key, KeyValueAccess::new(&mut *self.deserializer)))
    }
}

impl<'de, 'a, P> de::VariantAccess<'de> for KeyValueAccess<'a, P>
where
    P: Parser<'de> + 'a,
{
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        self.deserializer.parse_value()?;
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.deserializer)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.deserializer, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.deserializer, visitor)
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
        let (_, value) = self.parse()?;
        visitor.visit_str(&value)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let (_, value) = self.parse()?;
        visitor.visit_str(&value)
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
        let is_some = match self.skip_whitespace()? {
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
        visitor.visit_seq(KeyValueAccess::new(self))
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
        visitor.visit_map(KeyValueAccess::new(self))
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
        visitor.visit_map(KeyValueAccess::new(self))
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
        visitor.visit_enum(KeyValueAccess::new(self))
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
