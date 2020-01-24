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
use alloc::prelude::v1::*;
use core::convert::TryFrom;
use parity_scale_codec::{Decode, Encode, Error as CodecError, Input};

/// Byte vector that is limited to 128 bytes.
#[derive(Encode, Clone, Debug, Eq, PartialEq)]
pub struct Bytes128(Vec<u8>);

impl Bytes128 {
    const MAXIMUM_SUPPORTED_LENGTH: usize = 128;

    /// Smart constructor that attempts to build a Bytes128
    /// from a Vec<u8> with an arbitrary size. It fails if the
    /// input vector is larger than Bytes128::MAXIMUM_SUPPORTED_LENGTH.
    pub fn from_vec(vector: Vec<u8>) -> Result<Self, InordinateVectorError> {
        if vector.len() > Self::MAXIMUM_SUPPORTED_LENGTH {
            Err(InordinateVectorError())
        } else {
            Ok(Bytes128(vector))
        }
    }
}

impl TryFrom<Vec<u8>> for Bytes128 {
    type Error = InordinateVectorError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Bytes128::from_vec(value)
    }
}

impl From<Bytes128> for Vec<u8> {
    fn from(value: Bytes128) -> Self {
        value.0
    }
}

/// Bytes128 random functions useful for unit testing.
///
/// Note that since these fuctions make use of rand, we need to guard
/// with the std feature to be able to compile it for wasm.
#[cfg(feature = "std")]
impl Bytes128 {
    /// Generate a random Bytes128 vector with as many bytes as its limit.
    pub fn random() -> Self {
        Self::from_vec(Self::random_vector(Self::MAXIMUM_SUPPORTED_LENGTH)).unwrap()
    }

    /// Generate a random Bytes128 vector with as many bytes as specified with 'size'.
    pub fn random_with_size(size: usize) -> Result<Self, InordinateVectorError> {
        Bytes128::from_vec(Self::random_vector(size))
    }

    /// Generate a random Vec<u8> with as many bytes as specified with 'size'.
    fn random_vector(size: usize) -> Vec<u8> {
        (0..size).map(|_| rand::random::<u8>()).collect()
    }
}

impl Decode for Bytes128 {
    fn decode<I: Input>(input: &mut I) -> Result<Self, CodecError> {
        let decoded: Vec<u8> = Vec::decode(input)?;
        Bytes128::from_vec(decoded)
            .or_else(|_| Err(CodecError::from("Failed to decode an inordinate Bytes128.")))
    }
}

/// Error type for a failed attempt to build a Bytes128 value from an inordinate Vec<u8>.
#[derive(Encode, Clone, Debug, Eq, PartialEq)]
pub struct InordinateVectorError();

#[cfg(feature = "std")]
impl core::fmt::Display for InordinateVectorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "The provided vectors's length exceeds the Bytes128 limit of {} bytes",
            Bytes128::MAXIMUM_SUPPORTED_LENGTH,
        )
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
            assert_eq!(
                Bytes128::from_vec(random_vector),
                Err(InordinateVectorError())
            );
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
