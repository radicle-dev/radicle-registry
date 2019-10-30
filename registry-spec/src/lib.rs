//! This is a specification document meant to approximate the Registry described in
//! Oscoin whitepaper into concrete Rust code.
//! However, it is not meant to be an exact implementation.
//!
//! It is to serve as a form of documentation that will change over
//! time with the project, as well as the in-depth specification present in
//! https://github.com/oscoin/registry-spec.
pub mod error;
pub mod types;

/// A trait exposing the Oscoin registry transactions described in the
/// whitepaper.
///
/// The methods here return `Result<types::TxHash, E>` for some error type `E` as they
/// will be applying a modification on the Registry global state, and return
/// the hash of the applied transaction if they succeed.
pub trait RegistryTransactions {
    /// Transfer Oscoin from one account to another.
    ///
    /// Method preconditions:
    /// * Amount to be transferred is greater than or equal to 1 (one) unit of
    /// currency.
    fn transfer_oscoin(
        // Account from which to send Oscoin.
        from_acc: types::AccountId,
        // Account to which Oscoin will be sent.
        to_acc: types::AccountId,
        // Amount of Oscoin to be sent.
        amount: types::Oscoin,
    ) -> Result<types::TxHash, error::TransferError>;

    /// Registers a project on the Oscoin Registry and returns the new project’s ID.
    ///
    /// The transaction’s sender account becomes the initial maintainer of the
    /// project, once it has been accepted into the registry.
    ///
    /// After submitting this transaction, the project will enter the
    /// `PendingRegistrations` set, after which it can either be accepted or
    /// rejected by an account in the `ROOT_ACCOUNT`s set with the appropriate
    /// `accept_project/reject_project` transaction.
    ///
    /// Further, after submitting this transaction, regardless of whether the
    /// project is later accepted or rejected, the registration fee is deducted
    /// from the transaction sender's account - if no such available balance is
    /// found, it will result in an error.
    ///
    /// Preconditions:
    /// * The ownership proof can only be up to 4096 bytes long.
    fn register_project(
        // Requested name of the project to be registered.
        project_id: types::ProjectId,
        project_checkpoint: types::CheckpointId,
        project_contract: types::Contract,
        project_ownership_proof: types::Proof,
        project_meta: types::Meta,
    ) -> Result<types::TxHash, error::RegisterProjectError>;

    /// Accept a project that has been submitted for registration.
    ///
    /// It is then removed from the pending registrations set, and can then be
    /// retrieved using the `get_project()` method from `RegistryView`.
    fn accept_project(
        // Hash of the `register_project` transaction for the `Project` in
        // question.
        t_hash: types::TxHash,
    ) -> Result<types::ProjectId, error::ProjectRegistrationVoteError>;

    /// Reject a project that has been submitted for registration.
    /// It is then removed from the pending registrations set.
    fn reject_project(
        // Hash of the `register_project` transaction for the `Project` in
        // question.
        t_hash: types::TxHash,
    ) -> Result<types::TxHash, error::ProjectRegistrationVoteError>;

    /// Transaction used to annul a previous `register_project` transaction,
    /// if it has not been approved/rejected yet.
    ///
    /// If successful, the project referred to will be removed from the
    /// pending registrations set
    /// (see RegistryView::get_pending_project_registrations).
    fn withdraw_project(
        // Hash of the `register_project` whose candidacy is being withdrawn.
        t_hash: types::TxHash,
    ) -> Result<types::TxHash, error::WithdrawProjectError>;

    /// Unregistering a project from the Oscoin Registry.
    ///
    /// As is the case above, this transaction may also be handled outside the
    /// registry.
    fn unregister_project(
        id: types::ProjectId,
    ) -> Result<types::TxHash, error::UnregisterProjectError>;

    /// Checkpointing a project in Oscoin's registry.
    fn checkpoint(
        // Hash of the previous `checkpoint` associated with this project.
        parent: Option<types::CheckpointId>,
        // New project hash - if `set-checkpoint` if used with this checkpoint,
        // this will become the project's current state hash.
        new_project_hash: types::Hash,
        new_project_version: types::Version,
        // Hash-linked list of the checkpoint's contributions. To see more
        // about this type, go to types::Contribution.
        contribution_list: types::HashLinkedList<types::Contribution>,
        // A vector of dependency updates. See types::DependencyUpdate
        // for more information.
        //
        // It is to be treated as a list i.e. processed from left to right.
        dependency_updates: Vec<types::DependencyUpdate>,
    ) -> Result<types::TxHash, error::CheckpointError>;
}

/// Functions to access information from the registry state.
pub trait RegistryView {
    /// Returns the project registered at the given address.
    ///
    /// Returns `None` if no project was registered or the project was unregistered.
    fn get_project(project_address: types::AccountId) -> Option<types::Project>;

    /// Returns the [Account] at the given address.
    ///
    /// An account exists for every address. If it has not receveived any money the empty account
    /// with zero nonce and balance is returned.
    fn get_account(address: types::AccountId) -> types::Account;

    /// The set of all registered projects in the Oscoin registry.
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
