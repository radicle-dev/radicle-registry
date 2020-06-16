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

//! `Id` is the unique identifier for Orgs and Users.
//!
//! [orgs spec](https://github.com/radicle-dev/registry-spec/blob/0b7699ac597bd935b13facc9152789d111e138ca/body.tex#L110-L119)
//! [user spec](https://github.com/radicle-dev/registry-spec/blob/1b7699ac597bd935b13facc9152789d111e138ca/body.tex#L452-L459)

use alloc::prelude::v1::*;
use core::convert::{From, Into, TryFrom};
use parity_scale_codec as codec;

#[derive(codec::Encode, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", serde(try_from = "String"))]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct Id(String);

impl Id {
    fn from_string(input: String) -> Result<Self, InvalidIdError> {
        // Must be at least 1 character.
        if input.is_empty() {
            return Err(InvalidIdError("must be at least 1 character"));
        }
        // Must be no longer than 32.
        if input.len() > 32 {
            return Err(InvalidIdError("must not exceed 32 characters"));
        }
        // Must only contain a-z, 0-9 and '-' characters.
        if !input
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-')
        {
            return Err(InvalidIdError("must only include a-z, 0-9 and '-'"));
        }

        // Must not start with a '-'.
        if input.starts_with('-') {
            return Err(InvalidIdError("must not start with a '-'"));
        }
        // Must not end with a '-'.
        if input.ends_with('-') {
            return Err(InvalidIdError("must not end with a '-'"));
        }
        // Must not contain sequences of more than one '-'.
        if input.contains("--") {
            return Err(InvalidIdError(
                "must not have more than one consecutive '-'",
            ));
        }

        let id = Self(input);

        Ok(id)
    }
}

impl codec::Decode for Id {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let decoded: String = String::decode(input)?;

        match Id::try_from(decoded) {
            Ok(id) => Ok(id),
            Err(err) => Err(codec::Error::from(err.what())),
        }
    }
}

impl Into<String> for Id {
    fn into(self) -> String {
        self.0
    }
}

impl TryFrom<String> for Id {
    type Error = InvalidIdError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        Self::from_string(input)
    }
}

impl TryFrom<&str> for Id {
    type Error = InvalidIdError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Self::from_string(input.into())
    }
}

impl core::str::FromStr for Id {
    type Err = InvalidIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s.to_string())
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Id {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error type when conversion from an input failed.
#[derive(codec::Encode, Clone, Debug, Eq, PartialEq)]
pub struct InvalidIdError(&'static str);

impl InvalidIdError {
    /// Error description
    ///
    /// This function returns an actual error str.
    pub fn what(&self) -> &'static str {
        self.0
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for InvalidIdError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(f, "InvalidIdError({})", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidIdError {
    fn description(&self) -> &str {
        self.0
    }
}

impl From<&'static str> for InvalidIdError {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod test {
    use super::Id;
    use parity_scale_codec::{Decode, Encode};

    #[test]
    fn id_too_short() {
        assert!(Id::from_string("".into()).is_err());
    }

    #[test]
    fn id_too_long() {
        let input = std::iter::repeat("X").take(33).collect::<String>();
        let too_long = Id::from_string(input);
        assert!(too_long.is_err());
    }

    #[test]
    fn id_invalid_characters() {
        let invalid_characters = Id::from_string("AZ+*".into());
        assert!(invalid_characters.is_err());
    }

    #[test]
    fn id_invalid_prefix() {
        let invalid_prefix = Id::from_string("-radicle".into());
        assert!(invalid_prefix.is_err());
    }

    #[test]
    fn id_invalid_suffix() {
        let invalid_suffix = Id::from_string("radicle-".into());
        assert!(invalid_suffix.is_err());
    }

    #[test]
    fn id_double_dash() {
        let double_dash = Id::from_string("radicle--registry".into());
        assert!(double_dash.is_err());
    }

    #[test]
    fn id_valid() {
        let valid = Id::from_string("radicle-registry001".into());
        assert!(valid.is_ok());
    }

    #[test]
    fn encode_then_decode() {
        let id = Id::from_string("monadic".into()).unwrap();
        let encoded = id.encode();
        let decoded = Id::decode(&mut &encoded[..]).unwrap();

        assert_eq!(id, decoded)
    }

    #[test]
    fn encoded_then_decode_invalid() {
        let invalid = Encode::encode("-Invalid-");
        let decoded = Id::decode(&mut &invalid[..]);

        assert!(decoded.is_err());
    }
}
