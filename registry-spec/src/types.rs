pub struct Account {
    pub id: AccountId,

    /// Transaction counter that is increased whenever a transaction is sent from this account.
    ///
    /// A transaction is only valid if its nonce matches the nonce of its sender account when the
    /// transaction is applied.
    pub nonce: u64,
    pub balance: Balance,
}

/// Identifier for an account.
///
/// An `AccountId` is the same as the public key of that is used to verify transactions originating
/// from an account.
pub struct AccountId;

/// A public key that is understood by GPG.
///
/// This is used for verifying project contributions that are signed with GPG keys.
pub struct GpgPublicKey;

/// Type representing a GPG signature of data.
pub struct GpgSignature;

/// Type for the hash digest used in the Oscoin Registry. Useful to represent
/// commit hashes or public keys.
pub struct Hash;

/// The hash of a transaction. Uniquely identifies a transaction.
pub struct TxHash;

/// Token balance associated with an account.
///
/// The balance is always positive. It is still to be decided, but it may be `u64`, `u128` or even
/// a rational type so fractional amounts can be represented. Subject to discussion.
pub struct Balance;

/// ID of a project's current checkpoint.
pub struct CheckpointId;

/// The name a `Project` has been registered with.
///
/// It is a UTF-8 `String` between 1 and 32 characters long.
pub type ProjectName = String;

/// The domain under which the `Project`'s `ProjectName` is registered.
pub type ProjectDomain = String;

/// The `ProjectId` tuple serves to uniquely identify a project.
pub type ProjectId = (ProjectName, ProjectDomain);

/// A structure that contains a proof that the entity that submitted the
/// `register_project` transaction actually has ownership of the submitted
/// `Project`.
pub struct ProjectOwnershipProof;

/// A structure that contains a dictionary of metadata to associate with a
/// project. for example the Radicle id that project uses on that service.
/// It is immutable once defined.
pub struct Meta;

/// Representation of a project in the Oscoin registry.
pub struct Project {
    pub id: ProjectId,
    /// The project's latest checkpoint's ID.
    pub checkpoint: CheckpointId,
    pub contract: Contract,
    pub proof: ProjectOwnershipProof,
    pub meta: Meta,
}

/// A project's "smart" contract.
pub struct Contract;

/// A project's version at a given point in time.
pub type Version = String;

/// Hash linked list of [Contribution]s.
///
/// A [ContributionList] `cs` is valid if for every item `c` excluding the first item `c.parent`
/// equals `Some(d.hash)` where `d` is the previous item in the list.
pub struct ContributionList(Vec<Contribution>);

/// Datatype representing a contribution, one of the data required by a
/// checkpoint.
///
/// A contribution is valid if [sig] is a valid signature of [hash] for the public key [author].
pub struct Contribution {
    pub hash: Hash,
    /// Hash of the previous contribution’s [hash] or `None` if thi is the firt contribution.
    pub parent: Option<Hash>,
    pub author: GpgPublicKey,
    pub sig: GpgSignature,
}

/// Datatype representing a dependency update, another segment of data required
/// in order to checkpoint a project in the Oscoin registry.
pub enum DependencyUpdate {
    /// Constructor to add a dependency.
    Depend {
        /// Address of the project being added to the dependency list.
        acc: AccountId,
        /// Version of the project that is going to be depended on.
        version: Version,
    },
    /// Constructor to remove a dependency.
    Undepend {
        /// Address of the project being removed from the dependency list.
        acc: AccountId,
        /// Version of the project that is going to be removed as a dependency.
        version: Version,
    },
}
