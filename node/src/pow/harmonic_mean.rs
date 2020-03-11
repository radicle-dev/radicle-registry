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

//! Calculates harmonic mean of a series of values
//!
//! The algorithm is designed to use only integers. It also makes no rounding except for the final
//! step, when the final result is rounded down to to an integer.
//!
//! # How it works
//! The harmonic mean of `count` items `a`, `b`, `c` ... is defined as:
//! ```raw
//! count / (1/a + 1/b + 1/c + ... )
//! ```
//! This is equivalent to:
//! ```raw
//! count * a * b * c * ... / (b * c... + a * c... + a * b... + ...)
//! ```
//!
//! ## Adding values
//! Let's take a look at an example of harmonic mean of `a`, `b` and `c`:
//! ```raw
//! 3 * a * b * c / (b * c + a * c + a * b)
//! ```
//! After adding value `d` it becomes:
//! ```raw
//! 4 * a * b * c * d / (b * c * d + a * c * d + a * b * d + a * b * c)
//! ```
//! Which is equivalent to:
//! ```raw
//! (3 + 1) * (a * b * c) * d / ((b * c + a * c + a * b) * d + (a * b * c))
//! ```
//! Which can be described as:
//! ```raw
//! (count + 1) * nominator * value / (denominator * value + nominator)
//! ```
//! Which can be broken up into updates of individual variables:
//! ```raw
//! count = count + 1
//! denominator = denominator * value + nominator
//! nominator = nominator * value
//! mean = count * nominator / denominator
//! ```
//!
//! ## The initial state
//! The only exception is addition of the first item, which should go from the initial state:
//! ```raw
//! 0 * 1 / 1 = 0
//! ```
//! Which can be broken up into individual variables:
//! ```raw
//! count = 0
//! denominator = 1
//! nominator = 1
//! mean = count * nominator / denominator = 0
//! ```
//! To:
//! ```raw
//! 1 * a / 1 = a
//! ```
//! Which can be described as:
//! ```raw
//! (count + 1) * nominator * value / denominator
//! ```
//! Which can be broken up into updates of individual variables:
//! ```raw
//! count = count + 1
//! denominator = denominator
//! nominator = nominator * value
//! mean = count * nominator / denominator = value
//! ```
//! The only difference from a normal variables update is skipping of the update of `denominator`.
//! The updates of `count` `nominator` and `mean` are all unchanged.

use num_bigint::BigUint;
use num_traits::One;
use sp_core::U256;

#[derive(Debug)]
pub struct HarmonicMean {
    nominator: BigUint,
    denominator: BigUint,
    count: u32,
}

impl HarmonicMean {
    pub fn new() -> Self {
        HarmonicMean {
            nominator: BigUint::one(),
            denominator: BigUint::one(),
            count: 0,
        }
    }

    pub fn push(&mut self, value: U256) {
        let value = u256_to_biguint(value);
        if self.count > 0 {
            self.denominator *= &value;
            self.denominator += &self.nominator;
        }
        self.nominator *= value;
        self.count += 1;
    }

    pub fn calculate(self) -> U256 {
        let mean = self.count * self.nominator / self.denominator;
        biguint_to_u256(mean)
    }
}

fn u256_to_biguint(mut value: U256) -> BigUint {
    let mut u32_digits = Vec::with_capacity(8);
    for _ in 0..8 {
        u32_digits.push(value.low_u32());
        value >>= 32;
    }
    BigUint::new(u32_digits)
}

fn biguint_to_u256(value: BigUint) -> U256 {
    value
        .to_u32_digits()
        .into_iter()
        .rev()
        .fold(U256::zero(), |result, digit| {
            (result << 32) | U256::from(digit)
        })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn u256_to_biguint_to_u256() {
        assert_u256_to_biguint_to_u256_with_u128(0);
        assert_u256_to_biguint_to_u256_with_u128(1);
        assert_u256_to_biguint_to_u256_with_u128(1_000_000_000);
        assert_u256_to_biguint_to_u256_with_u128(10_000_000_000_000_000_000);
        assert_u256_to_biguint_to_u256_with_u128(u128::max_value());
        assert_u256_to_biguint_to_u256(U256::from(u128::max_value()) + 1);
        assert_u256_to_biguint_to_u256(U256::MAX - 1_000_000_000);
        assert_u256_to_biguint_to_u256(U256::MAX - 1);
        assert_u256_to_biguint_to_u256(U256::MAX);
    }

    fn assert_u256_to_biguint_to_u256_with_u128(value: u128) {
        let expected_u256 = U256::from(value);
        let expected_biguint = BigUint::from(value);
        let actual_biguint = u256_to_biguint(expected_u256);
        assert_eq!(
            expected_biguint, actual_biguint,
            "Conversion of U256 to BigUint failed"
        );
        let actual_u256 = biguint_to_u256(actual_biguint);
        assert_eq!(
            expected_u256, actual_u256,
            "Conversion of BigUint to U256 failed"
        );
    }

    fn assert_u256_to_biguint_to_u256(expected: U256) {
        let biguint = u256_to_biguint(expected);
        let actual = biguint_to_u256(biguint);
        assert_eq!(
            expected, actual,
            "Conversion of U256 to BigUint to U256 failed"
        );
    }

    #[test]
    fn harmonic_mean() {
        let max = U256::MAX;
        assert_mean(0, &[]);
        assert_mean(0, &[0]);
        assert_mean(1, &[1]);
        assert_mean(1, &[1, 1]);
        assert_mean(2, &[1, 4, 4]);
        assert_mean(max, &[max]);
        assert_mean(max, &[max, max]);
    }

    fn assert_mean<I>(expected: I, inputs: &[I])
    where
        I: Clone + std::fmt::Debug + Into<U256>,
    {
        let mut mean = HarmonicMean::new();
        for input in inputs {
            mean.push(input.clone().into());
        }
        assert_eq!(
            expected.into(),
            mean.calculate(),
            "Invalid result for inputs {:?}",
            inputs
        );
    }
}
