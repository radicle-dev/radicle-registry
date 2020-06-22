# radicle-registry-core

This package provides the types that constitute the registry ledger and provides
exhaustive documentation how these types behave in the ledger.

These types are the entities that are stored in the state, the different
transaction message types, and their constituent types.

## Transaction Messages

Transaction messages effect a change in the ledger state. They are submitted to
the ledger as part of a transaction. All possible transaction messages are
defined in the `message` module.

For each message we document how it changes the state and what preconditions
must be satisfied. The documentation must be comprehensive and exhaustive and
cover all edge cases. The documentation for a message has the following sections

<dl>
  <dt>State changes</dt>
  <dd>Describes which entities are added, removed, or updated in the ledger
  state.</dd>
  <dt>State-dependent validations</dt>
  <dd>Describes the validations that are required for the message to be applied
  successfully and that depend on the current ledger state state.</dd>
</dd>

## State

All entities that are stored in the ledger state are defined in the `state`
module.

For each entity version the documentation has the following sections

<dl>
  <dt>Storage</dt>
  <dd>Describes how the entity is stored in the state and how the state storage
  key is calculated.</dd>
  <dt>Invariants</dt>
  <dd>Describes the invariants of the data in the state entity that always hold.</dd>
  <dt>Relevant messages</dt>
  <dd>Links to message types that effect or use the entity.</dd>
</dd>

### Versioning

To make the runtime state backwards compatible, every state entity that is added
must be versioned using the following schema.
Please follow the naming convention introduced in the examples as closely as possible.

The storage defines the structure of the data.
If it's altered, e.g. a key type is changed or it's converting from a map to a list,
a new version must be added.
The storage must include its version in its name:

```rust
// registry.rs

pub mod store {
    ...
      decl_storage! {
          ...
              pub Users1: map hasher(blake2_128_concat) Id => Option<state::Users1Data>;
              pub Users2: map hasher(blake2_128_concat) NewId => Option<state::Users2Data>;
```

The stored data must be an enum capable of containing different data versions.
It must be versioned consistently and independently of the storage version:

```rust
// state.rs

pub enum Users1Data {
    V1(UserV1)
    V2(UserV2)
}

pub enum Users2Data {
    V3(UserId)
    V4(UserV4)
}
```

If the stored data is a specialized data structure,
it must be versioned same as the stored data, which contains it:

```rust
// state.rs

pub struct UserV1 {
    ...
}

pub enum UserV2 {
    ...
}

// UserId is general purpose

pub struct UserV4 {
    ...
}
```

Existing version variants may never be altered. Only new variants may be added.

## Errors

To maintain compatibility between different runtime versions we follow a simple
policy for changes to `RegistryError`.

Error variants are not removed immediately if they become unused in the runtime.
Instead we annotate them with the `#[deprecated]` attribute and remove them once
we intend to not support them any more. Whenever a variant is removed its
discriminant value may not be used anymore. New variants are assigned a
discriminant that has never been taken by an old variant. For this reason all
discriminants are explicitly set
