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

use crate::String32;
use alloc::prelude::v1::*;
use core::convert::TryFrom;
use core::str::FromStr;

use parity_scale_codec::{Decode, Encode, Error as CodecError, Input};

/// A Project Domain, limited to 32 bytes and to the supported d
#[derive(Encode, Clone, Debug, Eq, PartialEq)]
pub struct ProjectDomain(String32);

impl ProjectDomain {
    /// Build a ProjectDomain from a given string.
    /// Fails when the given domain is longer than 32 bytes in length or
    /// if its not yet supported by Radicle.
    ///
    /// Currently only supporting the "rad" domain.
    pub fn from_string(domain: String) -> Result<Self, ProjectDomainError> {
        if domain == "rad" {
            Ok(ProjectDomain::rad_domain())
        } else {
            Err(ProjectDomainError::NotYetSupported)
        }
    }

    pub fn from_string32(domain: String32) -> Result<Self, ProjectDomainError> {
        Self::from_string(domain.into())
    }

    pub fn rad_domain() -> ProjectDomain {
        ProjectDomain(String32::from_str("rad").expect("statically valid"))
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for ProjectDomain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Possible errors when attempting to build a ProjectDomain.
#[derive(Encode, Clone, Debug, Eq, PartialEq)]
pub enum ProjectDomainError {
    /// The provided domain exceeds the 32 bytes length limit.
    Inordinate,
    /// The provided domain is not yet supported by Radicle.
    NotYetSupported,
}

#[cfg(feature = "std")]
impl std::error::Error for ProjectDomainError {}

#[cfg(feature = "std")]
impl core::fmt::Display for ProjectDomainError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Conversion implementations

impl Decode for ProjectDomain {
    fn decode<I: Input>(input: &mut I) -> Result<Self, CodecError> {
        let decoded = String32::decode(input)?;
        ProjectDomain::from_string32(decoded).or_else(|_| {
            Err(CodecError::from(
                "Failed to decode an unsupported project domain.",
            ))
        })
    }
}

impl core::str::FromStr for ProjectDomain {
    type Err = ProjectDomainError;

    /// Returns an error if the domain is longer than 32 bytes
    /// or is not supported by radicle.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ProjectDomain::from_string(s.into())
    }
}

impl TryFrom<String> for ProjectDomain {
    type Error = ProjectDomainError;

    /// Returns an error if the domain is longer than 32 bytes
    /// or is not supported by radicle.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        ProjectDomain::from_string(value)
    }
}

impl From<ProjectDomain> for String {
    fn from(value: ProjectDomain) -> Self {
        (value.0).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn radicle_domain() {
        assert_eq!(
            ProjectDomain::rad_domain(),
            ProjectDomain::from_str("rad").unwrap()
        )
    }

    #[test]
    fn from_inordinate_domain() {
        assert_eq!(
            ProjectDomain::from_string("rad".repeat(11)),
            Err(ProjectDomainError::NotYetSupported)
        )
    }

    #[test]
    fn from_unsupported_domain() {
        assert_eq!(
            ProjectDomain::from_str("github"),
            Err(ProjectDomainError::NotYetSupported)
        )
    }

    #[test]
    fn decode_after_encode_is_identity() {
        let domain = ProjectDomain::rad_domain();
        let encoded = domain.encode();
        let decoded = <ProjectDomain>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(domain, decoded)
    }
}
