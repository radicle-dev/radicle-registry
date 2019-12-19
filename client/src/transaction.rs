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
use parity_scale_codec::Encode;
use radicle_registry_runtime::UncheckedExtrinsic;
use sp_runtime::{
    generic::{Era, SignedPayload},
    traits::{IdentifyAccount, SignedExtension},
    MultiSigner,
};
use std::marker::PhantomData;

pub use radicle_registry_runtime::{
    registry::{Project, ProjectId},
    AccountId, Balance,
};
pub use sp_core::crypto::{Pair as CryptoPair, Public as CryptoPublic};
pub use sp_core::ed25519;

pub use crate::call::Call;
pub use radicle_registry_runtime::{Call as RuntimeCall, Hash, Index, SignedExtra};

#[derive(Clone, Debug)]
/// Transaction the can be submitted to the blockchain.
///
/// A transaction includes
/// * the author
/// * the runtime call
/// * extra data to like the gensis hash and account nonce
/// * a valid signature
///
/// The transaction type is generic over the runtime call parameter which must implement [Call].
///
/// A transaction can be created with [Transaction::new_signed]. The necessary transaction data
/// must be obtained from the client with [crate::ClientT::account_nonce] and [crate::ClientT::genesis_hash].
pub struct Transaction<Call_: Call> {
    _phantom_data: PhantomData<Call_>,
    pub(crate) extrinsic: UncheckedExtrinsic,
}

impl<Call_: Call> Transaction<Call_> {
    /// Create and sign a transaction for the given call.
    pub fn new_signed(
        signer: &ed25519::Pair,
        call: Call_,
        transaction_extra: TransactionExtra,
    ) -> Self {
        let extrinsic = signed_extrinsic(signer, call.into_runtime_call(), transaction_extra);
        Transaction {
            _phantom_data: PhantomData,
            extrinsic,
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// The data that is required from the blockchain state to create a valid transaction.
pub struct TransactionExtra {
    /// The nonce of the account that is the transaction author.
    pub nonce: Index,
    pub genesis_hash: Hash,
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

    UncheckedExtrinsic::new_signed(
        call,
        MultiSigner::from(signer.public()).into_account().into(),
        signature.into(),
        extra,
    )
}

/// Return the [SignedExtra] data that is part of [UncheckedExtrinsic] and the associated
/// `AdditionalSigned` data included in the signature.
fn transaction_extra_to_runtime_extra(
    extra: TransactionExtra,
) -> (
    SignedExtra,
    <SignedExtra as SignedExtension>::AdditionalSigned,
) {
    let check_version = frame_system::CheckVersion::new();
    let check_genesis = frame_system::CheckGenesis::new();
    let check_era = frame_system::CheckEra::from(Era::Immortal);
    let check_nonce = frame_system::CheckNonce::from(extra.nonce);
    let check_weight = frame_system::CheckWeight::new();
    let charge_transaction_payment = pallet_transaction_payment::ChargeTransactionPayment::from(0);

    let additional_signed = (
        check_version
            .additional_signed()
            .expect("statically returns ok"),
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
        charge_transaction_payment
            .additional_signed()
            .expect("statically returns Ok"),
    );

    let extra = (
        check_version,
        check_genesis,
        check_era,
        check_nonce,
        check_weight,
        charge_transaction_payment,
    );

    (extra, additional_signed)
}

#[cfg(test)]
mod test {
    use super::*;
    use radicle_registry_runtime::{GenesisConfig, Runtime};
    use sp_runtime::traits::Checkable;
    use sp_runtime::BuildStorage as _;

    #[test]
    /// Assert that extrinsics created with [create_and_sign] are validated by the runtime.
    fn check_extrinsic() {
        let genesis_config = GenesisConfig {
            aura: None,
            balances: None,
            sudo: None,
            indices: None,
            system: None,
            grandpa: None,
        };
        let mut test_ext = sp_io::TestExternalities::new(genesis_config.build_storage().unwrap());
        let (key_pair, _) = ed25519::Pair::generate();

        type System = frame_system::Module<Runtime>;
        let context = frame_system::ChainContext::<Runtime>::default();
        let genesis_hash = test_ext.execute_with(|| {
            System::initialize(
                &1,
                &[0u8; 32].into(),
                &[0u8; 32].into(),
                &Default::default(),
            );
            System::block_hash(0)
        });

        let xt = signed_extrinsic(
            &key_pair,
            frame_system::Call::fill_block().into(),
            TransactionExtra {
                nonce: 0,
                genesis_hash,
            },
        );

        test_ext.execute_with(move || xt.check(&context)).unwrap();
    }
}
