radicle-registry-core
=====================

This package provides the types that constitute the registry ledger and provides
exhaustive documentation how these types behave in the ledger.

These types are the entities that are stored in the state, the different
transaction message types, and their constituent types.

Transaction Messages
--------------------

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

State
-----

All entities that are stored in the ledger state are defined in the `state`
module.

For each entity the documentation has the following sections

<dl>
  <dt>Storage</dt>
  <dd>Describes how the entity is stored in the state and how the state storage
  key is calculated.</dd>
  <dt>Invariants</dt>
  <dd>Describes the invariants of the data in the state entity that always hold.</dd>
  <dt>Relevant messages</dt>
  <dd>Links to message types that effect or use the entity.</dd>
</dd>
