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

/// The name associated to a [`Project`].
///
/// https://github.com/radicle-dev/registry-spec/blob/master/body.tex#L306
use alloc::prelude::v1::*;
use core::convert::{From, Into, TryFrom};
use parity_scale_codec as codec;

#[derive(codec::Encode, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", serde(try_from = "String"))]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct ProjectName(String);

impl ProjectName {
    fn from_string(input: String) -> Result<Self, InvalidProjectNameError> {
        // Must be at least 1 character.
        if input.is_empty() {
            return Err(InvalidProjectNameError("must be at least 1 character"));
        }
        // Must be no longer than 32.
        if input.len() > 32 {
            return Err(InvalidProjectNameError("must not exceed 32 characters"));
        }

        // Must only contain a-z, 0-9, '-', '_' and '.' characters.
        {
            let check_charset = |c: char| {
                c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-' || c == '_' || c == '.'
            };

            if !input.chars().all(check_charset) {
                return Err(InvalidProjectNameError(
                    "must only include a-z, 0-9, '-', '_' and '.'",
                ));
            }
        }

        // Must not equal '.' or '..'.
        if input == "." || input == ".." {
            return Err(InvalidProjectNameError("must not be equal to '.' or '..'"));
        }

        let id = Self(input);

        Ok(id)
    }
}

impl codec::Decode for ProjectName {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let decoded: String = String::decode(input)?;

        match Self::try_from(decoded) {
            Ok(id) => Ok(id),
            Err(err) => Err(codec::Error::from(err.what())),
        }
    }
}

impl Into<String> for ProjectName {
    fn into(self) -> String {
        self.0
    }
}

impl TryFrom<String> for ProjectName {
    type Error = InvalidProjectNameError;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        Self::from_string(input)
    }
}

impl TryFrom<&str> for ProjectName {
    type Error = InvalidProjectNameError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Self::from_string(input.to_string())
    }
}

impl core::str::FromStr for ProjectName {
    type Err = InvalidProjectNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s.to_string())
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for ProjectName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error type when conversion from an inordinate input failed.
#[derive(codec::Encode, Clone, Debug, Eq, PartialEq)]
pub struct InvalidProjectNameError(&'static str);

impl InvalidProjectNameError {
    /// Error description
    ///
    /// This function returns an actual error str when running in `std`
    /// environment, but `""` on `no_std`.
    #[cfg(feature = "std")]
    pub fn what(&self) -> &'static str {
        self.0
    }

    /// Error description
    ///
    /// This function returns an actual error str when running in `std`
    /// environment, but `""` on `no_std`.
    #[cfg(not(feature = "std"))]
    pub fn what(&self) -> &'static str {
        ""
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for InvalidProjectNameError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(f, "InvalidProjectNameError({})", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidProjectNameError {
    fn description(&self) -> &str {
        self.0
    }
}

impl From<&'static str> for InvalidProjectNameError {
    #[cfg(feature = "std")]
    fn from(s: &'static str) -> Self {
        Self(s)
    }

    #[cfg(not(feature = "std"))]
    fn from(s: &'static str) -> Self {
        InvalidProjectNameError(s)
    }
}

#[cfg(test)]
mod test {
    use super::ProjectName;
    use parity_scale_codec::{Decode, Encode};

    #[test]
    fn name_too_short() {
        assert!(ProjectName::from_string("".into()).is_err());
    }

    #[test]
    fn name_too_long() {
        let input = std::iter::repeat("X").take(33).collect::<String>();
        let too_long = ProjectName::from_string(input);
        assert!(too_long.is_err());
    }

    #[test]
    fn name_invalid_characters() {
        let invalid_characters = ProjectName::from_string("AZ+*".into());
        assert!(invalid_characters.is_err());
    }

    #[test]
    fn name_is_dot() {
        let dot = ProjectName::from_string(".".into());
        assert!(dot.is_err());
    }

    #[test]
    fn name_is_double_dot() {
        let dot = ProjectName::from_string("..".into());
        assert!(dot.is_err());
    }

    #[test]
    fn name_valid() {
        let valid = ProjectName::from_string("--radicle_registry001".into());
        assert!(valid.is_ok());
    }

    #[test]
    fn encode_then_decode() {
        let id = ProjectName::from_string("monadic".into()).unwrap();
        let encoded = id.encode();
        let decoded = <ProjectName>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(id, decoded)
    }
}
