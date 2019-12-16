//! Provides [Transaction] and [TransactionExtra].
use parity_scale_codec::Encode;
use radicle_registry_runtime::UncheckedExtrinsic;
use sr_primitives::generic::{Era, SignedPayload};
use sr_primitives::traits::SignedExtension;
use std::marker::PhantomData;

pub use radicle_registry_runtime::{
    registry::{Project, ProjectId},
    AccountId, Balance,
};
pub use substrate_primitives::crypto::{Pair as CryptoPair, Public as CryptoPublic};
pub use substrate_primitives::ed25519;

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
    let check_version = paint_system::CheckVersion::new();
    let check_genesis = paint_system::CheckGenesis::new();
    let check_era = paint_system::CheckEra::from(Era::Immortal);
    let check_nonce = paint_system::CheckNonce::from(extra.nonce);
    let check_weight = paint_system::CheckWeight::new();
    let charge_transaction_payment = paint_transaction_payment::ChargeTransactionPayment::from(0);

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
    use sr_primitives::traits::{Checkable, IdentityLookup};
    use sr_primitives::BuildStorage as _;

    #[test]
    /// Assert that extrinsics created with [create_and_sign] are validated by the runtime.
    fn check_extrinsic() {
        let genesis_config = GenesisConfig {
            paint_aura: None,
            paint_balances: None,
            paint_sudo: None,
            system: None,
            paint_grandpa: None,
        };
        let mut test_ext = sr_io::TestExternalities::new(genesis_config.build_storage().unwrap());
        let (key_pair, _) = ed25519::Pair::generate();

        type System = paint_system::Module<Runtime>;
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
            paint_system::Call::fill_block().into(),
            TransactionExtra {
                nonce: 0,
                genesis_hash,
            },
        );

        test_ext
            .execute_with(move || xt.check(&IdentityLookup::default()))
            .unwrap();
    }
}
