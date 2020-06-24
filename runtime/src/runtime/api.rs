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

//! Implements Substrate runtime APIs and provide a function based interface for the runtime APIs.
use alloc::vec::Vec;
use frame_support::{ensure, fail, traits::Randomness};
use sp_core::OpaqueMetadata;
use sp_runtime::traits::Block as BlockT;
use sp_runtime::{
    transaction_validity::{InvalidTransaction, TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
use sp_version::RuntimeVersion;

use super::{
    registry, AllModules, Block, Call, Header, InherentDataExt, RandomnessCollectiveFlip, Runtime,
    UncheckedExtrinsic, VERSION,
};

type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllModules,
>;

pub const VERSIONS: sp_version::ApisVec = RUNTIME_API_VERSIONS;

/// See [sp_api::Core::initialize_block]
pub fn initialize_block(header: &Header) {
    Executive::initialize_block(header)
}

/// See [sp_block_builder::BlockBuilder::inherent_extrinsics].
pub fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<UncheckedExtrinsic> {
    data.create_extrinsics()
}

/// See [sp_block_builder::BlockBuilder::apply_extrinsic].
pub fn apply_extrinsic(extrinsic: UncheckedExtrinsic) -> ApplyExtrinsicResult {
    validate_extrinsic_call(&extrinsic)?;
    Executive::apply_extrinsic(extrinsic)
}

/// See [sp_block_builder::BlockBuilder::finalize_block].
pub fn finalize_block() -> Header {
    Executive::finalize_block()
}

const SIGNED_INHERENT_CALL_ERROR: InvalidTransaction = InvalidTransaction::Custom(1);
const FOBIDDEN_CALL_ERROR: InvalidTransaction = InvalidTransaction::Custom(2);
const UNSGINED_CALL_ERROR: InvalidTransaction = InvalidTransaction::Custom(3);

/// Validate that the call of the extrinsic is allowed.
///
/// * We forbid calls reserved for inherents when the extrinsic is not signed.
/// * We forbid any calls to the [super::Balances] or [super::System] module.
/// * We ensure that the extrinsic is signed for non-inherent calls.
///
fn validate_extrinsic_call(xt: &UncheckedExtrinsic) -> Result<(), InvalidTransaction> {
    match xt.function {
        // Inherents are only allowed if they are unsigned.
        Call::Timestamp(_) | Call::Registry(registry::Call::set_block_author(_)) => {
            ensure!(xt.signature.is_none(), SIGNED_INHERENT_CALL_ERROR)
        }

        // Forbidden internals.
        Call::Balances(_) | Call::System(_) => fail!(FOBIDDEN_CALL_ERROR),

        // Impossible cases that cannot be constructed.
        Call::RandomnessCollectiveFlip(_) => fail!(FOBIDDEN_CALL_ERROR),

        // Allowed calls for signed extrinsics.
        Call::Registry(_) | Call::Sudo(_) => ensure!(xt.signature.is_some(), UNSGINED_CALL_ERROR),
    }

    Ok(())
}

sp_api::impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            initialize_block(header);
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            inherent_extrinsics(data)
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(source: TransactionSource, tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
            validate_extrinsic_call(&tx)?;
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    // An implementation for the `SessionKeys` runtime API is required by the types
    // of [sc_service::ServiceBuilder]. However, the implementation is otherwise unused
    // and has no effect on the behavior of the runtime. Hence we implement a dummy
    // version.
    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(_seed: Option<Vec<u8>>) -> Vec<u8> {
            Default::default()
        }

        fn decode_session_keys(
            _encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            None
        }
    }

    impl sp_consensus_pow::TimestampApi<Block, u64> for Runtime {
        fn timestamp() -> u64 {
            pallet_timestamp::Module::<Runtime>::get()
        }
    }
}
