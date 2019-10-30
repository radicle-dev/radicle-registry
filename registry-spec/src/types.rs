use std::marker::PhantomData;

pub struct Account {
    /// Hash of the account owner's public key.
    pub id: AccountId,

    /// Transaction counter that is increased whenever a transaction is sent from this account.
    ///
    /// A transaction is only valid if its nonce matches the nonce of its sender account when the
    /// transaction is applied.
    pub nonce: u64,
    pub balance: Oscoin,
}

/// Type for human-readable registry addresses. Its specific data structure is
/// not important here.
pub struct Address;

/// Type for account identifier. Uniquely corresponds to an account in the
/// registry.
///
/// At present it is assumed to be the same as `Address`, but will eventually
/// diverge.
pub type AccountId = Address;

/// Type for Registry public keys. Its specific data structure is not
/// important here, just as it is with `Address`es.
pub struct PublicKey;

/// Type representing a GPG signature of data.
pub struct Signature;

/// Type for the hash digest used in the Oscoin Registry. Useful to represent
/// commit hashes or public keys.
pub struct Hash;

/// The hash of a transaction. Uniquely identifies a transaction.
pub struct TxHash;

/// Numerical type representing Oscoin amounts. It is still to be decided, but
/// it may be `u64`, `u128` or even a rational type so fractional amounts can
/// be represented. Subject to discussion.
pub struct Oscoin;

/// Representation of a contribution's author.
pub type Author = PublicKey;

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
pub type Proof = Vec<u8>;

/// A structure that contains a dictionary of metadata to associate with a
/// project. for example the Radicle id that project uses on that service.
/// It is immutable once defined.
pub struct Meta;

/// Representation of a project in the Oscoin registry.
pub struct Project {
    pub addr: ProjectId,
    /// The project's latest checkpoint's ID.
    pub hash: CheckpointId,
    pub id: ProjectId,
    pub contract: Contract,
    pub proof: Proof,
    pub meta: Meta,
}

/// A project's "smart" contract.
pub struct Contract;

/// A project's version at a given point in time.
pub type Version = String;

/// Datatype representing a hash-linked-list. Used to organize contributions
/// when checkpointing.
///
/// The type it abstracts over - with contributions here abbreviated as `C` -
/// should be a tuple, struct or equivalent with at least two fields e.g.
/// `prev` and `commit` such that for every two consecutive members of the
/// hash-linked-list `C {prev1 = hash1, commit1 = hash2 .. }, C {prev2 = hash3, commit2 = hash4 ..}`:
/// * it is true that `hash2 == hash3`;
/// * if `hash1` is the first hash present in the list, it is either
///   * the same as the last hash present in the last contribution of the last
///     checkpoint, or
///   * an empty hash, in case this is a project's first checkpoint.
///
/// In practice, it may not necessarily be a list, but conceptually the name
/// is explanatory.
pub struct HashLinkedList<T> {
    contributions: PhantomData<T>,
}

/// Datatype representing a contribution, one of the data required by a
/// checkpoint.
pub struct Contribution {
    pub parent: Hash,
    pub hash: Hash,
    // Note that `author` must be the public key that signed the contribution
    // a data structure of this type must refer to.
    pub author: Author,
    pub sig: Signature,
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
