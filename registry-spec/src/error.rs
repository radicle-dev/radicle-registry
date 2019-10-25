/// Description of errors that a transfer of Oscoin may raise.
pub enum TransferError {
    /// Amount to be transferred is not greater than or equal to 1 (one)
    /// unit of currency.
    InvalidTransferAmountError,

    /// This type of error is only here tentatively since the validation of a
    /// transfer's data may not necessarily occur in the Registry layer, meaning
    /// it may not have to deal with this.
    InsufficientBalanceError,

    /// As mentioned in the whitepaper, the contracts associated with the
    /// sending and receiving addresses must authorize the transfer for it
    /// to be valid, otherwise it will result in this error.
    ContractDeniedError,
}

pub enum ProjectValidationError {
    /// The origin of the `accept/reject_project` transaction is not
    /// in the set of root accounts.
    OriginNotRootError,

    /// The hash of the transaction is invalid e.g. it does not correspond to
    /// a `register_project` transaction, or has improper structure.
    InvalidTransactionHashError,

    /// The project in question has already been validated i.e. it has already
    /// been previously accepted/rejected.
    ProjectAlreadyValidatedError,
}

/// Description of errors that may occur when registering a project in the
/// Oscoin registry (`register` transaction). Not exhaustive, but should cover
/// most common cases.
pub enum RegisterProjectError {
    /// The canonical source URL used to register the project is invalid.
    ///
    /// The 1.0 version of the whitepaper establishes only one condition for
    /// the validity of the URL - the source code retrieved from it must
    /// always hash to the `hash` field of the `Project` structure -
    /// but this definition of validity can be tentatively extended to include:
    ///
    /// 1. it is a proper URL as defined in [RFC 3986](https:///tools.ietf.org/html/rfc3986#section-1.1.3), and
    /// 2. it hosts the repository's page in a distributed version control
    ///    system's website e.g. GitLab, BitBucket, SourceForge, GitHub, and
    /// 3. it can be accessed without restrictions (^)
    ///
    /// (^) This part can be harder to define - if the URL permanently returns
    /// `404`s *after* it has been inducted into the registry, but not before,
    /// is it still valid?
    InvalidURLError,
}

/// Representation of errors that may occur in `addkey` or `removekey`
/// transactions.
pub enum KeysetError {
    /// Version 1.0 of the whitepaper does not mention what happens when
    /// `addkey`/`removekey` are called with projects that have not yet been
    /// added to the registry, so here that is tentatively treated as an error.
    AccountIfNotInUseError,
}

/// Errors that may happen when unregistering a project.
///
/// Empty for now.
pub enum UnregisterProjectError {}

/// Errors that may occur when checkpointing a project.
///
/// Question:
/// * Does an invalid dependency update list in a checkpoint invalidate it
/// entirely?
pub enum CheckpointError {
    /// A dependency update is invalid if it adds a dependency the project
    /// already uses.
    UsedDependencyAddedError,

    /// A dependency update is invalid if it removes a dependency the project
    /// does not use.
    UnusedDependencyRemovedError,

    /// As the whitepaper says, a checkpoint is invalid if the dependency
    /// update list containts duplicate dependencies.
    DuplicateDependenciesError,
}
