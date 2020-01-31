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

//! This is a specification document meant to approximate the Registry described in [1]
//! into concrete Rust code.
//! However, it is not meant to be an exact implementation.
//!
//! It is to serve as a form of documentation that will change over
//! time with the project, as well as the in-depth specification present in
//! https://github.com/oscoin/registry-spec.
pub mod error;
pub mod types;

/// Functions to access information from the registry state.
pub trait RegistryView {
    /// Returns the project registered at the given address.
    ///
    /// Returns `None` if no project was registered or the project was unregistered.
    fn get_project(project_address: types::ProjectId) -> Option<types::Project>;

    /// Returns the [Account] at the given address.
    ///
    /// An account exists for every address. If it has not receveived any money the empty account
    /// with zero nonce and balance is returned.
    fn get_account(address: types::AccountId) -> types::Account;

    /// The set of all registered projects in the Radicle registry.
    fn get_registered_projects() -> std::collections::HashSet<types::ProjectId>;

    /// The set of projects that are pending acceptance into the registry,
    /// having been submitted with the `register_project` transaction.
    fn get_pending_project_registrations() -> std::collections::HashSet<types::ProjectId>;

    /// Returns the set of accounts that are authorized to accept or reject
    /// projects.
    ///
    /// This set of root accounts is specified at genesis and cannot be
    /// changed.
    fn get_root_accounts() -> std::collections::HashSet<types::AccountId>;
}
