Change Log
==========

Upcoming
--------

### Breaking changes
* Add `metadata` field to Project, a vector of at most 128 bytes.
* Drop project fields `description` and `img_url`
* Rename `Client::submit` to `Client::sign_and_submit_call`.
* `Client::create` and `Client::create_with_executor` now require a `host`
  argument. Use `host = url::Host::parse("127.0.0.1").unwrap()` to have the old
  behavior.
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
* The error type in `get_dispatch_result` has changed from
  `Option<&'static str>` to `sp_runtime::DispatchError`.
* The `Client` and `ClientT` methods are now `async`.

### Addition

* Offline transaction signing with the following new APIs
  * `ClientT::account_nonce()`
  * `ClientT::genesis_hash()`
  * `Transaction`
  * `Transaction::new_signed()`
