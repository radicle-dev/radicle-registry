Change Log
==========

Upcoming
--------

### Breaking changes

* `Client::create_with_executor` now returns a future instead of a `Result`
* `Client::submit` and  `Client::submit_transaction` now return a wrapped
  future. This allows consumers to distinguish the accepted and applied states
  of a transaction.
* Remove convenience methods for submitting transactions from the client
  - `Client::transfer`
  - `Client::register_project`
  - `Client::create_checkpoint`
  - `Client::set_checkpoint`
  Calls to these functions can be replaced by calls to `Client::submit`.
* Eliminate `ClientWithExecutor`, use `Client::create_with_executor()` instead.
* Eliminate `MemoryClient`. The memory client is now called the emulator and can
  be created with `Client::new_emulator()`.
* All library and binary names have been changed to use dashes instead of
  underscores e.g. `radicle_registry_node` becomes `radicle-registry-node`.

### Addition

* Offline transaction signing with the following new APIs
  * `ClientT::account_nonce()`
  * `ClientT::genesis_hash()`
  * `Transaction`
  * `Transaction::new_signed()`
