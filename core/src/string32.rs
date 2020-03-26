// Radicle Registry
// Copyright (C) 2019 Monadic GmbH <radicle@monadic.xyz>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

/// `String32` type, and its validation tests.
use alloc::prelude::v1::*;
use core::convert::Into;
use parity_scale_codec::{Decode, Encode, Error as CodecError, Input};

/// A [String] that is limited to 32 bytes in UTF-8 encoding.
///
/// ```rust
/// # use radicle_registry_core::String32;
/// assert!(String32::from_string("a string".to_string()).is_ok());
/// let long_string = "this string has more than 32 bytes".to_string();
/// assert!(String32::from_string(long_string).is_err());
/// ```
#[derive(Encode, Clone, Debug, Eq, PartialEq)]
pub struct String32(String);

impl String32 {
    const MAXIMUM_SUPPORTED_LENGTH: usize = 32;

    /// Returns an error if [String::len] of the provided is greater than 32.
    pub fn from_string(s: String) -> Result<Self, InordinateStringError> {
        if s.len() > Self::MAXIMUM_SUPPORTED_LENGTH {
            Err(InordinateStringError())
        } else {
            Ok(String32(s))
        }
    }
}

impl Into<String> for String32 {
    fn into(self) -> String {
        self.0
    }
}

impl core::str::FromStr for String32 {
    type Err = InordinateStringError;

    /// Returns an error if [String::len] of the provided is greater than 32.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        String32::from_string(s.to_string())
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for String32 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Decode for String32 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, CodecError> {
        let decoded: String = String::decode(input)?;
        if decoded.len() > 32 {
            Err(From::from("String32 length was more than 32 bytes."))
        } else {
            Ok(String32(decoded))
        }
    }
}

/// Error type for a failed attempt to build a String32 value from an inordinate String.
#[derive(Encode, Clone, Debug, Eq, PartialEq)]
pub struct InordinateStringError();

#[cfg(feature = "std")]
impl core::fmt::Display for InordinateStringError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "The provided string's length exceeds the String32 limit of {} bytes",
            String32::MAXIMUM_SUPPORTED_LENGTH,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn inordinate_string32() {
        let random_inordinate = random_string(String32::MAXIMUM_SUPPORTED_LENGTH + 1);
        let inordinate_string32 = String32::from_string(random_inordinate);
        assert_eq!(inordinate_string32, Err(InordinateStringError()));
    }

    #[test]
    fn valid_string32() {
        let string32 = String32::from_string(random_string(String32::MAXIMUM_SUPPORTED_LENGTH));
        assert!(
            string32.is_ok(),
            "Error: string with acceptable length failed conversion to String32."
        )
    }

    #[test]
    fn encode_then_decode() {
        let string = String32::from_string(String::from("ôítÏйгますいщαφδвы")).unwrap();

        let encoded = string.encode();

        let decoded = <String32>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(string, decoded)
    }

    fn random_string(size: usize) -> String {
        std::iter::repeat("X").take(size).collect::<String>()
    }
}
