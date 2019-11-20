// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-subxt.
//
// subxt is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// subxt is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with substrate-subxt.  If not, see <http://www.gnu.org/licenses/>.

//! A library to **sub**mit e**xt**rinsics to a
//! [substrate](https://github.com/paritytech/substrate) node via RPC.

#![deny(missing_docs)]

use futures::future::{
    self,
    Either,
    Future,
    IntoFuture,
};
use jsonrpc_core_client::transports::ws;
use metadata::Metadata;
use parity_scale_codec::{
    Codec,
    Decode,
    Encode,
};
use runtime_primitives::{
    generic::UncheckedExtrinsic,
    traits::{
        IdentifyAccount,
        Verify,
    },
    MultiSignature,
};
use sr_version::RuntimeVersion;
use std::marker::PhantomData;
use substrate_primitives::{
    storage::{
        StorageChangeSet,
        StorageKey,
    },
    Pair,
};
use url::Url;

use crate::{
    codec::Encoded,
    extrinsic::{
        DefaultExtra,
        SignedExtra,
    },
    metadata::MetadataError,
    paint::{
        balances::Balances,
        system::{
            System,
            SystemStore,
        },
        ModuleCalls,
    },
    rpc::{
        BlockNumber,
        ChainBlock,
        MapStream,
        Rpc,
    },
};

mod codec;
mod error;
mod extrinsic;
mod metadata;
mod paint;
mod rpc;
mod runtimes;

pub use error::Error;
pub use paint::*;
pub use rpc::ExtrinsicSuccess;
pub use runtimes::*;

fn connect<T: System>(url: &Url) -> impl Future<Item = Rpc<T>, Error = Error> {
    ws::connect(url).map_err(Into::into)
}

/// ClientBuilder for constructing a Client.
pub struct ClientBuilder<T: System, S = MultiSignature> {
    _marker: std::marker::PhantomData<(T, S)>,
    url: Option<Url>,
}

impl<T: System, S> ClientBuilder<T, S> {
    /// Creates a new ClientBuilder.
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            url: None,
        }
    }

    /// Set the substrate rpc address.
    pub fn set_url(mut self, url: Url) -> Self {
        self.url = Some(url);
        self
    }

    /// Creates a new Client.
    pub fn build(self) -> impl Future<Item = Client<T, S>, Error = Error> {
        let url = self.url.unwrap_or_else(|| {
            Url::parse("ws://127.0.0.1:9944").expect("Is valid url; qed")
        });
        connect::<T>(&url).and_then(|rpc| {
            rpc.metadata()
                .join3(rpc.genesis_hash(), rpc.runtime_version(None))
                .map(|(metadata, genesis_hash, runtime_version)| {
                    Client {
                        url,
                        genesis_hash,
                        metadata,
                        runtime_version,
                        _marker: PhantomData,
                    }
                })
        })
    }
}

/// Client to interface with a substrate node.
pub struct Client<T: System, S = MultiSignature> {
    url: Url,
    genesis_hash: T::Hash,
    metadata: Metadata,
    runtime_version: RuntimeVersion,
    _marker: PhantomData<fn() -> S>,
}

impl<T: System, S> Clone for Client<T, S> {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            genesis_hash: self.genesis_hash.clone(),
            metadata: self.metadata.clone(),
            runtime_version: self.runtime_version.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: System + Balances + 'static, S: 'static> Client<T, S> {
    /// Connect a new RPC client and return it.
    pub fn connect(&self) -> impl Future<Item = Rpc<T>, Error = Error> {
        connect(&self.url)
    }

    /// Returns the chain metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Fetch a StorageKey.
    pub fn fetch<V: Decode>(
        &self,
        key: StorageKey,
    ) -> impl Future<Item = Option<V>, Error = Error> {
        self.connect().and_then(|rpc| rpc.storage::<V>(key))
    }

    /// Fetch a StorageKey or return the default.
    pub fn fetch_or<V: Decode>(
        &self,
        key: StorageKey,
        default: V,
    ) -> impl Future<Item = V, Error = Error> {
        self.fetch(key).map(|value| value.unwrap_or(default))
    }

    /// Fetch a StorageKey or return the default.
    pub fn fetch_or_default<V: Decode + Default>(
        &self,
        key: StorageKey,
    ) -> impl Future<Item = V, Error = Error> {
        self.fetch(key).map(|value| value.unwrap_or_default())
    }

    /// Get a block hash. By default returns the latest block hash
    pub fn block_hash(
        &self,
        hash: Option<BlockNumber<T>>,
    ) -> impl Future<Item = Option<T::Hash>, Error = Error> {
        self.connect()
            .and_then(|rpc| rpc.block_hash(hash.map(|h| h)))
    }

    /// Get a block
    pub fn block<H>(
        &self,
        hash: Option<H>,
    ) -> impl Future<Item = Option<ChainBlock<T>>, Error = Error>
    where
        H: Into<T::Hash> + 'static,
    {
        self.connect()
            .and_then(|rpc| rpc.block(hash.map(|h| h.into())))
    }

    /// Subscribe to events.
    pub fn subscribe_events(
        &self,
    ) -> impl Future<Item = MapStream<StorageChangeSet<T::Hash>>, Error = Error> {
        self.connect().and_then(|rpc| rpc.subscribe_events())
    }

    /// Subscribe to new blocks.
    pub fn subscribe_blocks(
        &self,
    ) -> impl Future<Item = MapStream<T::Header>, Error = Error> {
        self.connect().and_then(|rpc| rpc.subscribe_blocks())
    }

    /// Subscribe to finalized blocks.
    pub fn subscribe_finalized_blocks(
        &self,
    ) -> impl Future<Item = MapStream<T::Header>, Error = Error> {
        self.connect()
            .and_then(|rpc| rpc.subscribe_finalized_blocks())
    }

    /// Create a transaction builder for a private key.
    pub fn xt<P>(
        &self,
        signer: P,
        nonce: Option<T::Index>,
    ) -> impl Future<Item = XtBuilder<T, P, S>, Error = Error>
    where
        P: Pair,
        P::Signature: Codec,
        S: Verify,
        S::Signer: From<P::Public> + IdentifyAccount<AccountId = T::AccountId>,
    {
        let client = self.clone();
        let account_id = S::Signer::from(signer.public()).into_account();
        match nonce {
            Some(nonce) => Either::A(future::ok(nonce)),
            None => Either::B(self.account_nonce(account_id)),
        }
        .map(|nonce| {
            let genesis_hash = client.genesis_hash.clone();
            let runtime_version = client.runtime_version.clone();
            XtBuilder {
                client,
                nonce,
                runtime_version,
                genesis_hash,
                signer,
                call: None,
                marker: PhantomData,
            }
        })
    }
}

/// The extrinsic builder is ready to finalize construction.
pub enum Valid {}
/// The extrinsic builder is not ready to finalize construction.
pub enum Invalid {}

/// Transaction builder.
pub struct XtBuilder<T: System, P, S, V = Invalid> {
    client: Client<T, S>,
    nonce: T::Index,
    runtime_version: RuntimeVersion,
    genesis_hash: T::Hash,
    signer: P,
    call: Option<Result<Encoded, MetadataError>>,
    marker: PhantomData<fn() -> V>,
}

impl<T: System + Balances + 'static, P, S: 'static, V> XtBuilder<T, P, S, V>
where
    P: Pair,
{
    /// Returns the chain metadata.
    pub fn metadata(&self) -> &Metadata {
        self.client.metadata()
    }

    /// Returns the nonce.
    pub fn nonce(&self) -> T::Index {
        self.nonce.clone()
    }

    /// Sets the nonce to a new value.
    pub fn set_nonce(&mut self, nonce: T::Index) -> &mut XtBuilder<T, P, S, V> {
        self.nonce = nonce;
        self
    }

    /// Increment the nonce
    pub fn increment_nonce(&mut self) -> &mut XtBuilder<T, P, S, V> {
        self.set_nonce(self.nonce() + 1.into());
        self
    }

    /// Sets the module call to a new value
    pub fn set_call<F>(&self, module: &'static str, f: F) -> XtBuilder<T, P, S, Valid>
    where
        F: FnOnce(ModuleCalls<T, P>) -> Result<Encoded, MetadataError>,
    {
        let call = self
            .metadata()
            .module(module)
            .and_then(|module| f(ModuleCalls::new(module)))
            .map_err(Into::into);

        XtBuilder {
            client: self.client.clone(),
            nonce: self.nonce.clone(),
            runtime_version: self.runtime_version.clone(),
            genesis_hash: self.genesis_hash.clone(),
            signer: self.signer.clone(),
            call: Some(call),
            marker: PhantomData,
        }
    }
}

impl<T: paint_system::Trait + 'static + System, P, S: 'static, V> XtBuilder<T, P, S, V>
where
    P: Pair,
    <T as paint_system::Trait>::Call: Encode,
{
    /// Sets the module call to a new value of the [paint_system::Trait::Call] type associated with
    /// the runtime.
    pub fn set_system_call(
        &self,
        call: impl Into<<T as paint_system::Trait>::Call>,
    ) -> XtBuilder<T, P, S, Valid> {
        let call = Ok(Encoded(call.into().encode()));
        XtBuilder {
            client: self.client.clone(),
            nonce: self.nonce.clone(),
            runtime_version: self.runtime_version.clone(),
            genesis_hash: self.genesis_hash.clone(),
            signer: self.signer.clone(),
            call: Some(call),
            marker: PhantomData,
        }
    }
}

impl<T: System + Balances + Send + Sync + 'static, P, S: 'static>
    XtBuilder<T, P, S, Valid>
where
    P: Pair,
    S: Verify + Codec + From<P::Signature>,
    S::Signer: From<P::Public> + IdentifyAccount<AccountId = T::AccountId>,
    T::Address: From<T::AccountId>,
{
    /// Creates and signs an Extrinsic for the supplied `Call`
    pub fn create_and_sign(
        &self,
    ) -> Result<
        UncheckedExtrinsic<
            T::Address,
            Encoded,
            S,
            <DefaultExtra<T> as SignedExtra<T>>::Extra,
        >,
        Error,
    > {
        let signer = self.signer.clone();
        let account_nonce = self.nonce.clone();
        let version = self.runtime_version.spec_version;
        let genesis_hash = self.genesis_hash.clone();
        let call = self
            .call
            .clone()
            .expect("A Valid extrinisic builder has a call; qed")?;

        log::info!(
            "Creating Extrinsic with genesis hash {:?} and account nonce {:?}",
            genesis_hash,
            account_nonce
        );

        let extra = extrinsic::DefaultExtra::new(version, account_nonce, genesis_hash);
        let xt = extrinsic::create_and_sign::<_, _, _, S, _>(signer, call, extra)?;
        Ok(xt)
    }

    /// Submits a transaction to the chain.
    pub fn submit(&self) -> impl Future<Item = T::Hash, Error = Error> {
        let cli = self.client.connect();
        self.create_and_sign()
            .into_future()
            .map_err(Into::into)
            .and_then(move |extrinsic| {
                cli.and_then(move |rpc| rpc.submit_extrinsic(extrinsic))
            })
    }

    /// Submits transaction to the chain and watch for events.
    pub fn submit_and_watch(
        &self,
    ) -> impl Future<Item = ExtrinsicSuccess<T>, Error = Error> {
        let cli = self.client.connect();
        self.create_and_sign()
            .into_future()
            .map_err(Into::into)
            .join(cli)
            .and_then(move |(extrinsic, rpc)| {
                rpc.submit_and_watch_extrinsic(extrinsic)
            })
    }
}
