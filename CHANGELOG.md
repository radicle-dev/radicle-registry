Change Log
==========

Upcoming
--------

### Breaking changes

* Eliminate `ClientWithExecutor`, use `Client::create_with_executor()` instead.
* Eliminate `MemoryClient`. The memory client is now called the emulator and can
  be created with `Client::new_emulator()`.

### Addition

* Offline transaction signing with the following new APIs
  * `ClientT::account_nonce()`
  * `ClientT::genesis_hash()`
  * `Transaction`
  * `Transaction::new_signed()`
