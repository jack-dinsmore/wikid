use std::str::{FromStr};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor, MapAccess};

pub type MyResult<T> = Result<T, &'static str>;

pub const WIKID_VERSION_MAJOR: u32 = 0;
pub const WIKID_VERSION_MINOR: u32 = 0;

#[derive(Debug, Deserialize)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{:x}{:x}{:x};", self.r, self.g, self.b)
    }
}

impl FromStr for Color {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if match s.bytes().nth(0) {
            None => return Err("Color must be 8 characters long"),
            Some(c) => c
        } != 35 {// Pound
            return Err("Color must start with #");
        }
        if s.len() != 8 {
            return Err("Color must be 8 characters long");
        }

        let r = match u8::from_str_radix(&s[1..3], 16) {
            Ok(u) => u,
            Err(_) => return Err("Could not parse the red value")
        };
        let g = match u8::from_str_radix(&s[3..5], 16) {
            Ok(u) => u,
            Err(_) => return Err("Could not parse the green value")
        };
        let b = match u8::from_str_radix(&s[5..7], 16) {
            Ok(u) => u,
            Err(_) => return Err("Could not parse the blue value")
        };

        Ok(Color { r, g, b })
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
