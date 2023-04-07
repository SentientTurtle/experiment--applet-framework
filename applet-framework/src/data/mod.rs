//! Various data types

/// Simple 3-channel color, supporting only RGB with no transparency
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Color3 {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

impl Color3 {
    /// Formats the current color into a css-compatible hex string.
    ///
    /// E.g. Color3 { red: 218, green: 153, blue: 41 } => "#da9929"
    pub fn as_css_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }

    /// Parses color from 7 character hex string
    ///
    /// Expects leading hash symbol.
    pub fn parse_from_hex(string: &str) -> Option<Self> {
        let bytes = string.strip_prefix('#')
            .map(str::as_bytes)
            .and_then(|bytes| <[u8; 6]>::try_from(bytes).ok());

        #[inline(always)]
        fn read_hex_u4(hex_digit: u8) -> Option<u8> {
            match hex_digit {
                b'0'..=b'9' => Some(hex_digit - b'0'),
                b'a'..=b'f' => Some(10 + hex_digit - b'a'),
                _ => None
            }
        }

        if let Some([red0, red1, green0, green1, blue0, blue1]) = bytes {
            Some(Color3 {
                red: (read_hex_u4(red0)? * 16) + read_hex_u4(red1)?,
                green: (read_hex_u4(green0)? * 16) + read_hex_u4(green1)?,
                blue: (read_hex_u4(blue0)? * 16) + read_hex_u4(blue1)?,
            })
        } else {
            None
        }
    }
}