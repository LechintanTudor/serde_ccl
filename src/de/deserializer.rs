use crate::error::{Error, Result};
use crate::parser::{IndentState, Parser, Reference};
use serde::de;

#[must_use]
pub(crate) struct Deserializer<P> {
    parser: P,
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
            scratch: Vec::new(),
            should_parse_value: false,
        }
    }

    pub fn parse<'s>(&'s mut self) -> Result<Reference<'a, 's, str>> {
        if self.should_parse_value {
            self.parse_value()
        } else {
            self.parse_key()
        }
    }

    pub fn parse_key<'s>(&'s mut self) -> Result<Reference<'a, 's, str>> {
        self.parser.parse_key(&mut self.scratch)
    }

    pub fn parse_value<'s>(&'s mut self) -> Result<Reference<'a, 's, str>> {
        self.parser.parse_value(&mut self.scratch)
    }

    pub fn skip_whitespace(&mut self) -> Result<IndentState> {
        self.parser.skip_whitespace(&mut self.scratch)
    }
}

#[must_use]
pub struct KeyValueAccess<'a, P> {
    deserializer: &'a mut Deserializer<P>,
    key_indent: u32,
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

            if !self.deserializer.parse_key()?.is_empty() {
                self.deserializer.parse_value()?;
                continue;
            }

            self.deserializer.should_parse_value = true;
            break seed.deserialize(&mut *self.deserializer).map(Some);
        }
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
        self.deserialize_map(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.parse()?.parse()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.parse()?.parse()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.parse()?.parse()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.parse()?.parse()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse()?.parse()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.parse()?.parse()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.parse()?.parse()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.parse()?.parse()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse()?.parse()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse()?.parse()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse()?.parse()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_char(self.parse()?.parse()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str::<Error>(&self.parse()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str::<Error>(&self.parse()?)
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

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
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
        visitor.visit_seq(KeyValueAccess {
            key_indent: self.parser.last_key_indent().map_or(0, |i| i + 1),
            deserializer: self,
        })
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
        visitor.visit_seq(KeyValueAccess {
            key_indent: self.parser.last_key_indent().map_or(0, |i| i + 1),
            deserializer: self,
        })
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
        visitor.visit_map(KeyValueAccess {
            key_indent: self.parser.last_key_indent().map_or(0, |i| i + 1),
            deserializer: self,
        })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
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
