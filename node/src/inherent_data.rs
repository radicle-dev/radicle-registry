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

//! Provides [InherentDataProviders] for the registry [InherentData].
use radicle_registry_runtime::registry::InherentData;
use radicle_registry_runtime::AccountId;
use sp_inherents::InherentDataProviders;

/// Return [InherentDataProviders] that provides [InherentData] for registry blocks required by
/// full nodes.
pub fn new_full_providers() -> InherentDataProviders {
    let providers = InherentDataProviders::new();
    let data = InherentData {
        block_author: AccountId::from_raw([0u8; 32]),
    };
    // Can only fail if a provider with the same name is already registered.
    providers.register_provider(data).unwrap();
    providers
}
