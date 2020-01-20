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
use std::convert::TryFrom;

/// This type is used to represent project metadata fields.
///
/// Radicle limits the size of the project metadata field to 128 bytes.
/// To guarantee that at the type-level, a smart constructor is provided to check validity.
#[derive(Encode, Clone, Debug, Eq, PartialEq)]
pub struct Bytes128(Vec<u8>);

/// TODO review
/// 1. From and To vec traits

impl Bytes128 {
    const MAXIMUM_SUPPORTED_LENGTH: usize = 128;

    pub fn from_vec(vector: Vec<u8>) -> Result<Self, String> {
        if vector.len() > Self::MAXIMUM_SUPPORTED_LENGTH {
            Err(format!(
                "The provided vectors's length exceeded is {} bytes while Bytes128 is limited to {} bytes",
                vector.len(),
                Self::MAXIMUM_SUPPORTED_LENGTH
            ))
        } else {
            Ok(Bytes128(vector))
        }
    }
}

impl TryFrom<Vec<u8>> for Bytes128 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Bytes128::from_vec(value)
    }
}

/// Bytes128 random functions useful for unit testing.
/// Note that since these fuctions make use of rand,
/// we need to guard with the std feature to
/// be able to compile it for wasm.
#[cfg(feature = "std")]
impl Bytes128 {
    pub fn random() -> Self {
        Self::from_vec(Self::random_vector(Self::MAXIMUM_SUPPORTED_LENGTH)).unwrap()
    }

    pub fn random_with_size(size: usize) -> Result<Self, String> {
        Bytes128::from_vec(Self::random_vector(size))
    }

    fn random_vector(size: usize) -> Vec<u8> {
        (0..size).map(|_| rand::random::<u8>()).collect()
    }
}

impl Decode for Bytes128 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, CodecError> {
        let decoded: Vec<u8> = Vec::decode(input)?;
        Bytes128::from_vec(decoded).or_else(|_| {
            Err(CodecError::from(
                "Bytes128 length was more than 128 characters.",
            ))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_valid_sized_vectors() {
        for size in 0..=Bytes128::MAXIMUM_SUPPORTED_LENGTH {
            let random_vector = random_vector(size);
            assert_eq!(
                Bytes128::from_vec(random_vector.clone()).unwrap(),
                Bytes128(random_vector)
            );
        }
    }

    #[test]
    fn test_from_inordinate_vectors() {
        for size in Bytes128::MAXIMUM_SUPPORTED_LENGTH + 1..Bytes128::MAXIMUM_SUPPORTED_LENGTH + 10
        {
            let random_vector = random_vector(size);
            assert!(Bytes128::from_vec(random_vector).is_err())
        }
    }

    #[test]
    fn decode_after_encode_is_identity() {
        let bytes128 = Bytes128::random();
        let encoded = bytes128.encode();
        let decoded = <Bytes128>::decode(&mut &encoded[..]).unwrap();

        assert_eq!(bytes128, decoded)
    }

    #[test]
    fn decode_inordinate_vector_fails() {
        // Encode a malformed bytes128 and verify that it fails to decode.
        // Note that we use Bytes128(vec) instead of Bytes128::from_vec().
        let inordinate_bytes128 = Bytes128(random_vector(129));
        let encoded = inordinate_bytes128.encode();
        let decoding_result = <Bytes128>::decode(&mut &encoded[..]);

        assert!(decoding_result.is_err())
    }

    fn random_vector(size: usize) -> Vec<u8> {
        (0..size).map(|_| rand::random::<u8>()).collect()
    }
}
