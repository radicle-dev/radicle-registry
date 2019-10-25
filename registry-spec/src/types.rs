use std::marker::PhantomData;

/// Type for human-readable registry addresses. Its specific data structure is
/// not important here.
pub struct Address;

/// Type for account identifier. Uniquely corresponds to an account in the
/// registry.
///
/// At present it is assumed to be the same as `Address`, but will eventually
/// diverge.
pub type AccountId = Address;

/// Identifier for a project.
///
/// At present it is assumed to be the same as `Address`, but will eventually
/// diverge.
pub type ProjectId = Address;

/// Type for Registry public keys. Its specific data structure is not
/// important here, just as it is with `Address`es.
pub struct PublicKey;

/// Type for the hash digest used in the Oscoin Registry. Useful to represent
/// commit hashes, hashed transactions or public keys.
pub struct Hash;

/// Representation of a URL. It can represent e.g. a project's page.
pub struct URL;

/// Numerical type representing Oscoin amounts. It is still to be decided, but
/// it may be `u64`, `u128` or even a rational type so fractional amounts can
/// be represented. Subject to discussion.
pub struct Oscoin;

/// Representation of a contribution's author.
pub struct Author;

/// Representation of a project in the Oscoin registry.
/// It is still unclear whether the project's keyset should be present in this
/// data structure, or if it will be in a different layer of the protocol.
pub struct Project {
    pub addr: AccountId,
    /// A project's latest commit hash.
    pub hash: Hash,
    pub url: URL,
    pub maintainers: Vec<AccountId>,
}

/// Datatype representing a hash-linked-list. Used in the whitepaper to
/// organize contributions when checkpointing.
///
/// The type it abstracts over - in the context of the whitepaper's Registry
/// section, contributions, here abbreviated as `C` - should be a tuple,
/// struct or equivalent with at least two fields e.g. `prev` and `commit`
/// such that for every two consecutive members of the hash-linked-list
/// `C {prev1 = hash1, commit1 = hash2 .. }, C {prev2 = hash3, commit2 = hash4 ..}`:
/// * it is true that `hash2 == hash3`;
/// * if the `hash1` is the first hash present in the list, it is either
///   * the same as the last hash present in the last contribution of the last
///     checkpoint, or
///   * an empty hash, in case this is a project's first checkpoint
///
/// In practice, it may not necessarily be a list, but conceptually the name
/// is explanatory.
pub struct HashLinkedList<T> {
    contributions: PhantomData<T>,
}

/// Datatype representing a contribution, one of the data required by a
/// checkpoint.
///
/// Questions that arose from the whitepaper:
/// * what is C_sig for? It is defined but not used anywhere
/// * what is the type of C_author? Is it the GPG public key used to sign the
///   commit? Is it a string with their name?
/// Answer:
/// TODO: author is PK of author, signoff is reviewer/approver (may be optional)
/// TODO: both PKs associated with accs on Oscoin.
pub struct Contribution {
    pub prev: Hash,
    pub commit: Hash,
    pub author: Author,
    pub signoff: PublicKey,
}

/// Representation of the indexes of a project's checkpoints.
///
/// They are 0-indexed, so they may be internally represented by a type like
/// `u64` or similar.
pub struct CheckpointIndex;

/// Datatype representing a dependency update, another segment of data required
/// in order to checkpoint a project in the Oscoin registry.
pub enum DependencyUpdate {
    /// Constructor to add a dependency.
    Depend {
        /// Address of the project being added to the dependency list.
        acc: AccountId,
        /// Zero-based index of the current checkpoint, in which the
        /// dependency is being added.
        cp_index: CheckpointIndex,
    },
    /// Constructor to remove a dependency.
    Undepend {
        /// Address of the project being removed from the dependency list.
        acc: AccountId,
        /// Zero-based index of the current checkpoint, in which the
        /// dependency is being removed.
        cp_index: CheckpointIndex,
    },
}

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
