use std::str::FromStr;
use serde::{Serialize, Deserialize, Serializer};

pub type MyResult<T> = Result<T, String>;

const LIGHT_SHRINK: f32 = 0.2;

#[derive(Debug, Deserialize)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn bw(&self) -> Color {
        let sum = self.r  as u16 + self.g  as u16 + self.b as u16;
        if sum > 387 {
            Color { r:0, g:0, b:0 }
        } else {
            Color { r:255, g:255, b:255 }
        }
    }

    pub fn light(&self) -> Color {
        Color {
            r: (255.0 * (1.0 - LIGHT_SHRINK * (255 - self.r) as f32 / 255.0)) as u8,
            g: (255.0 * (1.0 - LIGHT_SHRINK * (255 - self.g) as f32 / 255.0)) as u8,
            b: (255.0 * (1.0 - LIGHT_SHRINK * (255 - self.b) as f32 / 255.0)) as u8,
        }
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{}{}{}", if self.r > 0 {format!("{:02x}", self.r)} else {"00".to_owned()},
                            if self.g > 0 {format!("{:02x}", self.g)} else {"00".to_owned()},
                            if self.b > 0 {format!("{:02x}", self.b)} else {"00".to_owned()})
    }
}

impl FromStr for Color {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if match s.bytes().nth(0) {
            None => return Err("Color must be 7 characters long"),
            Some(c) => c
        } != 35 {// Pound
            return Err("Color must start with #");
        }
        if s.len() != 7 {
            return Err("Color must be 7 characters long");
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
