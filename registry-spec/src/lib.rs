//! This is a specification document meant to approximate the Registry described in
//! Oscoin whitepaper into concrete Rust code.
//! However, it is not meant to be an exact implementation.
//!
//! It is to serve as a form of documentation that will change over
//! time with the project.
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

    /// Accept a project that has been submitted for registration.
    fn accept_project(
        // Hash of the `register_project` transaction for the `Project` in
        // question.
        t_hash: types::TxHash,
    ) -> Result<types::TxHash, error::ValidationOfProjectRegistrationError>;

    /// Reject a project that has been submitted for registration.
    fn reject_project(
        // Hash of the `register_project` transaction for the `Project` in
        // question.
        t_hash: types::TxHash,
    ) -> Result<types::TxHash, error::ValidationOfProjectRegistrationError>;

    /// Registers a project on the Oscoin Registry and returns the new project’s ID.
    ///
    /// The transaction’s sender account becomes the initial maintainer of the project.
    ///
    /// The project ID is computed by hashing the sender’s nonce and the arguments. In the current
    /// implementation we use ethereum’s contract creation logic which generates the project ID.
    fn register_project(
        // Requested name of the project to be registered.
        project_name: types::ProjectName,
        // Domain under which the project is registered.
        project_domain: types::ProjectDomain,
        project_checkpoint: types::CheckpointId,
    ) -> Result<types::TxHash, error::RegisterProjectError>;

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
}
