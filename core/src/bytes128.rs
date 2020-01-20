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

/// `Bytes128` type, and its validation tests.
use alloc::format;
use alloc::prelude::v1::*;
use parity_scale_codec::{Decode, Encode, Error as CodecError, Input};

use sp_std::fmt;

/// This type is used to represent project metadata fields.
///
/// Radicle limits the size of the project metadata field to 128 bytes.
/// To guarantee that at the type-level, a smart constructor is provided to check validity.
#[derive(Encode, Clone, Debug, Eq, PartialEq)]
pub struct Bytes128(Vec<u8>);

impl Bytes128 {
    const MAXIMUM_SUPPORTED_LENGTH: usize = 128;

    pub fn from_vec(vector: Vec<u8>) -> Result<Self, String> {
        if vector.len() > Self::MAXIMUM_SUPPORTED_LENGTH {
            Err(format!(
                "The provided vectors's length exceeded {:?} bytes: {:?}",
                Self::MAXIMUM_SUPPORTED_LENGTH,
                vector
            ))
        } else {
            Ok(Bytes128(vector))
        }
    }

    //TODO(nuno): remove lint once used elsewhere besides the tests.
    #[warn(dead_code)]
    pub fn random() -> Self {
        Bytes128(
            (0..Self::MAXIMUM_SUPPORTED_LENGTH)
                .map(|_| rand::random::<u8>())
                .collect(),
        )
    }
}

#[cfg(feature = "std")]
impl fmt::Display for Bytes128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Decode for Bytes128 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, CodecError> {
        let decoded: Vec<u8> = Vec::decode(input)?;
        Bytes128::from_vec(decoded)
            .or_else(|_| Err(From::from("Bytes128 length was more than 128 characters.")))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_valid_sized_vectors() {
        (0..Bytes128::MAXIMUM_SUPPORTED_LENGTH + 1)
            .map(|size| random_vector(size))
            .map(|random_vector| {
                assert_eq!(
                    Bytes128::from_vec(random_vector.clone()).unwrap(),
                    Bytes128(random_vector.clone())
                )
            })
            .collect()
    }

    #[test]
    fn test_from_inordinate_vectors() {
        (Bytes128::MAXIMUM_SUPPORTED_LENGTH + 1..Bytes128::MAXIMUM_SUPPORTED_LENGTH + 10)
            .map(|size| random_vector(size))
            .map(|random_vector| assert!(Bytes128::from_vec(random_vector.clone()).is_err()))
            .collect()
    }

    #[test]
    fn decode_after_encode_is_identity() {
        let bytes128 = Bytes128::random();
        let encoded = bytes128.encode();
        let decoded = <Bytes128>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(bytes128, decoded)
    }

    fn random_vector(size: usize) -> Vec<u8> {
        (0..size).map(|_| rand::random::<u8>()).collect()
    }
}
