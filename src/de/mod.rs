use std::fmt::{self, Display};
use serde::de::{self, Deserialize, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use api::{ConfigItem, ConfigValue};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum DeError {
    #[fail(display = "No more values left, this should never happen")] NoMoreValuesLeft,
    #[fail(display = "Error from deserialization: {}", _0)] SerdeError(String),
    #[fail(display = "Expecting values to contain a single entry")] ExpectSingleValue,
    #[fail(display = "Expecting string")] ExpectString,
    #[fail(display = "Expecting string of length one, received `{}`", _0)] ExpectChar(String),
    #[fail(display = "Expecting boolean")] ExpectBoolean,
    #[fail(display = "Expecting number")] ExpectNumber,
    #[fail(display = "Expecting struct")] ExpectStruct,
    #[fail(display = "Could not deserialize as datatype not supported")] DataTypeNotSupported,
}

#[derive(Debug)]
pub struct Error(DeError);

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(DeError::SerdeError(msg.to_string()))
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        "an with deserialization error"
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy)]
enum DeType<'a> {
    Struct(&'a ConfigItem<'a>),
    Seq(&'a ConfigValue<'a>),
}

pub struct Deserializer<'a> {
    input: &'a [ConfigItem<'a>],
    depth: Vec<DeType<'a>>,
    root: bool,
}

impl<'a> Deserializer<'a> {
    pub fn from_collectd(input: &'a [ConfigItem]) -> Self {
        Deserializer {
            input: input,
            depth: vec![],
            root: true,
        }
    }

    fn current(&self) -> Result<DeType<'a>> {
        if self.depth.is_empty() {
            return Err(Error(DeError::NoMoreValuesLeft));
        }

        Ok(self.depth[self.depth.len() - 1])
    }

    fn grab_val(&self) -> Result<&ConfigValue<'a>> {
        match self.current()? {
            DeType::Struct(item) => {
                if item.values.len() != 1 {
                    return Err(Error(DeError::ExpectSingleValue));
                }

                Ok(&item.values[0])
            }
            DeType::Seq(item) => Ok(item),
        }
    }

    fn grab_string(&self) -> Result<&'a str> {
        if let ConfigValue::String(x) = *self.grab_val()? {
            Ok(x)
        } else {
            Err(Error(DeError::ExpectString))
        }
    }

    fn grab_bool(&self) -> Result<bool> {
        if let ConfigValue::Boolean(x) = *self.grab_val()? {
            Ok(x)
        } else {
            Err(Error(DeError::ExpectBoolean))
        }
    }

    fn grab_number(&self) -> Result<f64> {
        if let ConfigValue::Number(x) = *self.grab_val()? {
            Ok(x)
        } else {
            Err(Error(DeError::ExpectNumber))
        }
    }
}

pub fn from_collectd<'a, T>(s: &'a [ConfigItem<'a>]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_collectd(s);
    T::deserialize(&mut deserializer)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_bool().and_then(|x| visitor.visit_bool(x))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_string()
            .and_then(|x| visitor.visit_string(String::from(x)))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_string()
            .and_then(|x| visitor.visit_borrowed_str(x))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_i8(x as i8))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_i16(x as i16))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_i32(x as i32))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_i64(x as i64))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_u8(x as u8))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_u16(x as u16))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_u32(x as u32))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_u64(x as u64))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_f32(x as f32))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_number().and_then(|x| visitor.visit_f64(x))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.grab_string().and_then(|x| {
            if x.len() != 1 {
                Err(Error(DeError::ExpectChar(String::from(x))))
            } else {
                visitor.visit_char(x.chars().next().unwrap())
            }
        })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = &self.depth[self.depth.len() - 1];
        if let DeType::Struct(item) = *v {
            if item.children.is_empty() || item.values.is_empty() {
                visitor.visit_borrowed_str(item.key)
            } else if let ConfigValue::String(x) = item.values[0] {
                visitor.visit_borrowed_str(x)
            } else {
                Err(Error(DeError::ExpectString))
            }
        } else {
            Err(Error(DeError::ExpectStruct))
        }
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.current()? {
            DeType::Struct(item) => visitor.visit_seq(SeqSeparated::new(&mut self, &item.values)),
            DeType::Seq(_item) => Err(Error(DeError::ExpectStruct)),
        }
    }

    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.root {
            self.root = false;
            visitor.visit_map(FieldSeparated::new(&mut self, self.input))
        } else if let DeType::Struct(item) = self.current()? {
            visitor.visit_map(FieldSeparated::new(&mut self, &item.children[..]))
        } else {
            Err(Error(DeError::ExpectStruct))
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_none()
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error(DeError::DataTypeNotSupported))
    }

    forward_to_deserialize_any! {
        bytes
        byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map enum
    }
}

struct FieldSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    items: &'de [ConfigItem<'de>],
    first: bool,
}

impl<'a, 'de> FieldSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, items: &'de [ConfigItem<'de>]) -> Self {
        FieldSeparated {
            de: de,
            first: true,
            items: items,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for FieldSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.items.is_empty() {
            if !self.first {
                self.de.depth.pop().unwrap();
            }
            return Ok(None);
        }

        if self.first || self.de.depth.is_empty() {
            self.de.depth.push(DeType::Struct(&self.items[0]));
            self.first = false;
        } else {
            let ind = self.de.depth.len() - 1;
            self.de.depth[ind] = DeType::Struct(&self.items[0]);
        }

        self.items = &self.items[1..];
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct SeqSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    values: &'de [ConfigValue<'de>],
    first: bool,
}

impl<'a, 'de> SeqSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, v: &'de [ConfigValue<'de>]) -> Self {
        SeqSeparated {
            de: de,
            values: v,
            first: true,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.values.is_empty() {
            if !self.first {
                self.de.depth.pop().unwrap();
            }
            return Ok(None);
        }

        if self.first {
            self.de.depth.push(DeType::Seq(&self.values[0]));
            self.first = false;
        } else {
            let ind = self.de.depth.len() - 1;
            self.de.depth[ind] = DeType::Seq(&self.values[0]);
        }

        self.values = &self.values[1..];
        seed.deserialize(&mut *self.de).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_simple_bool() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            my_bool: bool,
        };

        let items = vec![
            ConfigItem {
                key: "my_bool",
                values: vec![ConfigValue::Boolean(true)],
                children: vec![],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(MyStruct { my_bool: true }, actual);
    }

    #[test]
    fn test_serde_empty_bool() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            my_bool: Option<bool>,
        };

        let actual = from_collectd(Default::default()).unwrap();
        assert_eq!(MyStruct { my_bool: None }, actual);
    }

    #[test]
    fn test_serde_simple_number() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            my_int: i8,
        };

        let items = vec![
            ConfigItem {
                key: "my_int",
                values: vec![ConfigValue::Number(1.0)],
                children: vec![],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(MyStruct { my_int: 1 }, actual);
    }

    #[test]
    fn test_serde_simple_string() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            my_string: String,
        };

        let items = vec![
            ConfigItem {
                key: "my_string",
                values: vec![ConfigValue::String("HEY")],
                children: vec![],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(
            MyStruct {
                my_string: String::from("HEY"),
            },
            actual
        );
    }

    #[test]
    fn test_serde_simple_str() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct<'a> {
            my_string: &'a str,
        };

        let items = vec![
            ConfigItem {
                key: "my_string",
                values: vec![ConfigValue::String("HEY")],
                children: vec![],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(MyStruct { my_string: "HEY" }, actual);
    }

    #[test]
    fn test_serde_bool_vec() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            my_bool: Vec<bool>,
        };

        let items = vec![
            ConfigItem {
                key: "my_bool",
                values: vec![ConfigValue::Boolean(true), ConfigValue::Boolean(false)],
                children: vec![],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(
            MyStruct {
                my_bool: vec![true, false],
            },
            actual
        );
    }

    #[test]
    fn test_serde_options() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            my_bool: Option<bool>,
            my_string: Option<String>,
        };

        let items = vec![
            ConfigItem {
                key: "my_bool",
                values: vec![ConfigValue::Boolean(true)],
                children: vec![],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(
            MyStruct {
                my_bool: Some(true),
                my_string: None,
            },
            actual
        );
    }

    #[test]
    fn test_serde_char() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            my_char: char,
        };

        let items = vec![
            ConfigItem {
                key: "my_char",
                values: vec![ConfigValue::String("/")],
                children: vec![],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(MyStruct { my_char: '/' }, actual);
    }

    #[test]
    fn test_serde_ignore() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            my_char: char,
        };

        let items = vec![
            ConfigItem {
                key: "my_char",
                values: vec![ConfigValue::String("/")],
                children: vec![],
            },
            ConfigItem {
                key: "my_boat",
                values: vec![ConfigValue::String("/")],
                children: vec![],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(MyStruct { my_char: '/' }, actual);
    }

    #[test]
    fn test_serde_graphite() {
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Node {
            port: i32,
        };

        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct MyStruct {
            example: Node,
        };

        let items = vec![
            ConfigItem {
                key: "node",
                values: vec![ConfigValue::String("example")],
                children: vec![
                    ConfigItem {
                        key: "port",
                        values: vec![ConfigValue::Number(2003.0)],
                        children: vec![],
                    },
                ],
            },
        ];

        let actual = from_collectd(&items).unwrap();
        assert_eq!(
            MyStruct {
                example: Node { port: 2003 },
            },
            actual
        );
    }
}
