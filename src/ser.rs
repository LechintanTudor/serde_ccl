use serde::ser::{self, Impossible};
use serde::Serialize;

use crate::Error;

#[derive(Default)]
pub struct Serializer {
    pub(crate) output: String,
    indents: Vec<(usize, i8)>,
    line: usize,
}

/// Serializes the value to a string.
pub fn to_string<T>(value: &T) -> crate::Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    let mut indent = 0;
    serializer.indents.reverse();

    let mut next_indent = serializer.indents.pop();
    let lines: Vec<_> = serializer.output.split("\n").collect();
    let output: String = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if let Some((indent_line, delta)) = next_indent {
                if indent_line == i {
                    indent += delta;
                    next_indent = serializer.indents.pop();
                }
            }
            let mut padded = String::new();
            for _ in 0..2 * indent {
                padded.push(' ');
            }
            padded.push_str(line);
            if i < lines.len() - 1 {
                padded.push('\n');
            }
            padded
        })
        .collect();

    Ok(output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeSeq = Self;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output += v;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.indents.push((self.line, 1));
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.output += "=\n";
        self.line += 1;
        self.indents.push((self.line, 1));
        let res = value.serialize(&mut **self);
        self.indents.push((self.line, -1));
        res
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.indents.push((self.line, -1));
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + ser::Serialize,
        V: ?Sized + ser::Serialize,
    {
        key.serialize(&mut **self)?;
        self.output += " = ";
        value.serialize(&mut **self)?;
        self.output += "\n";
        self.line += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(&mut **self)?;
        self.output += " = ";
        value.serialize(&mut **self)?;
        self.output += "\n";
        self.line += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{from_str, to_string};

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Test {
        a: u8,
        b: bool,
        c: String,
    }

    #[test]
    fn serialize_struct() {
        let t = Test {
            a: 1,
            b: true,
            c: "test".to_string(),
        };
        let s = to_string(&t);
        assert!(s.is_ok());
        assert_eq!(s.unwrap(), "a = 1\nb = true\nc = test\n")
    }

    #[test]
    fn serialize_map() {
        let mut map = std::collections::BTreeMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.insert("c", 3);

        let s = to_string(&map);
        assert!(s.is_ok());
        assert_eq!(s.unwrap(), "a = 1\nb = 2\nc = 3\n")
    }

    #[test]
    fn serialize_hard() {
        let seq = vec![
            Test {
                a: 1,
                b: true,
                c: "test".to_string(),
            },
            Test {
                a: 2,
                b: false,
                c: "test2".to_string(),
            },
        ];
        let s = to_string(&seq);
        assert!(s.is_ok());
        let s = s.unwrap();
        let d: Vec<Test> = from_str(&s).unwrap();
        assert_eq!(d, seq);
    }
}
