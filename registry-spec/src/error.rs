/// Description of errors that a transfer of Oscoin may raise.
pub enum TransferError {
    /// Amount to be transferred is not greater than or equal to 1 (one)
    /// unit of currency.
    InvalidTransferAmount,

    /// The transaction origin account's balance minus any transaction fee
    /// was not greater than or equal to the amount being sent.
    InsufficientBalance,

    /// As mentioned in the whitepaper, the contracts associated with the
    /// sending and receiving addresses must authorize the transfer for it
    /// to be valid, otherwise it will result in this error.
    ContractDenied,
}

pub enum ValidationOfProjectRegistrationError {
    /// The origin of the `accept/reject_project` transaction is not
    /// in the set of root accounts.
    OriginNotRoot,

    /// The hash of the transaction is invalid e.g. it does not correspond to
    /// a `register_project` transaction, or has improper structure.
    InvalidTransactionHash,

    /// The project in question has already been validated i.e. it has already
    /// been previously accepted/rejected.
    ProjectVoteClosed,
}

/// Description of errors that may occur when registering a project in the
/// Oscoin registry (`register` transaction). Not exhaustive, but should cover
/// most common cases.
pub enum RegisterProjectError {
    /// The name with which the project was to be registered is invalid e.g.
    /// it is not valid UTF-8, it is longer than the protocol allows, or
    /// it has already been used.
    NameAlreadyExists,

    /// The project's supplied domain was invalid e.g nonexistent or just
    /// invalid UTF-8.
    InvalidDomain,

    /// The project's supplied checkpoint did not exist.
    InvalidCheckpointId,
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
    /// The supplied parent contribution hash was not valid
    /// e.g. it was not empty in case of a project's first contribution, or was
    /// empty when it was not the first contribution.
    ParentCheckpointDoesNotExist,

    /// The project state hash supplied with the checkpoint was not valid
    /// e.g. it is improperly formed.
    InvalidCheckpointHash,

    /// The new project version supplied with this checkpoint was not valid
    /// e.g. it has already been used before, or it does not have acceptable
    /// length.
    InvalidNewVersion,

    /// The contribution list supplied with the checkpoint was not well-formed.
    /// See `ContributionListError`.
    InvalidContributionList(ContributionListError),

    /// Problem with the dependency list. See `DependencyListError`.
    InvalidDependencyList(DependencyListError),
}

/// Errors that may occur when processing a checkpoint's contribution list.
pub enum ContributionListError {
    InvalidParentHash,

    InvalidCommitHash,

    /// The suplied public signing key of the commit the contribution refers to
    /// did not match the author's actual key.
    InvalidContributionAuthor,

    /// The supplied GPG signature of the contribution's commit (which is
    /// referenced by its hash) was not valid.
    InvalidContributionSignature,
}

/// Errors that may happen when processing the dependency update list of a
/// checkpoint.
pub enum DependencyListError {
    /// A dependency update is invalid if it adds a dependency the project
    /// already uses.
    UsedDependencyAdded,

    /// A dependency update is invalid if it removes a dependency the project
    /// does not use.
    UnusedDependencyRemoved,

    /// As the whitepaper says, a checkpoint is invalid if the dependency
    /// update list containts duplicate dependencies.
    DuplicateDependencies,

    /// The dependency update's project id was invalid e.g. it does not have
    /// the right structure.
    ///
    /// Note that per the specification, it does not have to refer to an
    /// existing project.
    InvalidProjectId,

    /// The dependency update's project version was invalid e.g. improper
    /// structure.
    ///
    /// Note that per the specification, it does not have to refer to a
    /// project's existing version.
    InvalidProjectVersion,
}
