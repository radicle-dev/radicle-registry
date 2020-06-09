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

//! Provides [Transaction] and [TransactionExtra].
use core::marker::PhantomData;
use parity_scale_codec::Encode;
use sp_runtime::generic::{Era, SignedPayload};
use sp_runtime::traits::{Hash as _, SignedExtension};

use crate::{ed25519, message::Message, CryptoPair as _, TxHash};
use radicle_registry_core::state::AccountTransactionIndex;
use radicle_registry_runtime::{
    fees::PayTxFee, Balance, Call as RuntimeCall, Hash, Hashing, SignedExtra, UncheckedExtrinsic,
};

#[derive(Clone, Debug)]
/// Transaction the can be submitted to the blockchain.
///
/// A transaction includes
/// * the author
/// * the runtime message
/// * extra data to like the gensis hash and account nonce
/// * a valid signature
///
/// The transaction type is generic over the runtime message parameter which must implement [Message].
///
/// A transaction can be created with [Transaction::new_signed]. The necessary transaction data
/// must be obtained from the client with [crate::ClientT::account_nonce] and [crate::ClientT::genesis_hash].
pub struct Transaction<Message_: Message> {
    _phantom_data: PhantomData<Message_>,
    pub(crate) extrinsic: UncheckedExtrinsic,
}

impl<Message_: Message> Transaction<Message_> {
    /// Create and sign a transaction for the given message.
    pub fn new_signed(
        signer: &ed25519::Pair,
        message: Message_,
        transaction_extra: TransactionExtra,
    ) -> Self {
        let extrinsic = signed_extrinsic(signer, message.into_runtime_call(), transaction_extra);
        Transaction {
            _phantom_data: PhantomData,
            extrinsic,
        }
    }

    pub fn hash(self) -> TxHash {
        Hashing::hash_of(&self.extrinsic)
    }
}

#[derive(Copy, Clone, Debug)]
/// The data that is required from the blockchain state to create a valid transaction.
pub struct TransactionExtra {
    /// The nonce of the account that is the transaction author.
    pub nonce: AccountTransactionIndex,
    pub genesis_hash: Hash,
    /// The fee to cover the transaction fees and gain priority.
    pub fee: Balance,
}

/// Return a properly signed [UncheckedExtrinsic] for the given parameters that passes all
/// validation checks. See the `Checkable` implementation of [UncheckedExtrinsic] for how
/// validation is performed.
///
/// `genesis_hash` is the genesis hash of the block chain this intrinsic is valid for.
fn signed_extrinsic(
    signer: &ed25519::Pair,
    call: RuntimeCall,
    extra: TransactionExtra,
) -> UncheckedExtrinsic {
    let (runtime_extra, additional_signed) = transaction_extra_to_runtime_extra(extra);
    let raw_payload = SignedPayload::from_raw(call, runtime_extra, additional_signed);
    let signature = raw_payload.using_encoded(|payload| signer.sign(payload));
    let (call, extra, _) = raw_payload.deconstruct();

    UncheckedExtrinsic::new_signed(call, signer.public(), signature, extra)
}

/// Return the [SignedExtra] data that is part of [UncheckedExtrinsic] and the associated
/// `AdditionalSigned` data included in the signature.
fn transaction_extra_to_runtime_extra(
    extra: TransactionExtra,
) -> (
    SignedExtra,
    <SignedExtra as SignedExtension>::AdditionalSigned,
) {
    let check_genesis = frame_system::CheckGenesis::new();
    let check_era = frame_system::CheckEra::from(Era::Immortal);
    let check_nonce = frame_system::CheckNonce::from(extra.nonce);
    let check_weight = frame_system::CheckWeight::new();
    let pay_tx_fee = PayTxFee { fee: extra.fee };

    let additional_signed = (
        // Genesis hash
        extra.genesis_hash,
        // Era
        extra.genesis_hash,
        check_nonce
            .additional_signed()
            .expect("statically returns Ok"),
        check_weight
            .additional_signed()
            .expect("statically returns Ok"),
        pay_tx_fee
            .additional_signed()
            .expect("statically returns Ok"),
    );

    let extra = (
        check_genesis,
        check_era,
        check_nonce,
        check_weight,
        pay_tx_fee,
    );

    (extra, additional_signed)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::message;
    use radicle_registry_runtime::{GenesisConfig, Runtime};
    use sp_core::H256;
    use sp_runtime::traits::{Checkable, IdentityLookup};
    use sp_runtime::{BuildStorage as _, Perbill};

    #[test]
    /// Assert that extrinsics created with [create_and_sign] are validated by the runtime.
    fn check_extrinsic() {
        let genesis_config = GenesisConfig {
            pallet_balances: None,
            pallet_sudo: None,
            system: None,
        };
        let mut test_ext = sp_io::TestExternalities::new(genesis_config.build_storage().unwrap());
        let (key_pair, _) = ed25519::Pair::generate();

        type System = frame_system::Module<Runtime>;
        let genesis_hash = test_ext.execute_with(|| {
            System::initialize(
                &1,
                &[0u8; 32].into(),
                &[0u8; 32].into(),
                &Default::default(),
                frame_system::InitKind::Full,
            );
            System::block_hash(0)
        });

        let xt = signed_extrinsic(
            &key_pair,
            frame_system::Call::fill_block(Perbill::from_parts(0)).into(),
            TransactionExtra {
                nonce: 0,
                genesis_hash,
                fee: 3,
            },
        );

        test_ext
            .execute_with(move || xt.check(&IdentityLookup::default()))
            .unwrap();
    }

    #[test]
    /// Check that a signed transaction's hash equals its extrinsic's hash.
    fn check_transaction_hash() {
        let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
        let signed_tx = Transaction::new_signed(
            &alice,
            message::Transfer {
                recipient: alice.public(),
                balance: 1000,
            },
            TransactionExtra {
                nonce: 0,
                genesis_hash: H256::random(),
                fee: 9,
            },
        );
        let extrinsic_hash = Hashing::hash_of(&signed_tx.extrinsic);

        assert_eq!(signed_tx.hash(), extrinsic_hash);
    }
}
