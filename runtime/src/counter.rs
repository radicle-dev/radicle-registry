use codec::{Decode, Encode};
use sr_primitives::weights::SimpleDispatchInfo;
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result};
use srml_system::ensure_signed;

#[derive(Decode, Encode, Copy, Clone, Debug, Eq, PartialEq)]
pub struct CounterValue(pub u32);

pub trait Trait: srml_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as srml_system::Trait>::Event>;
}

decl_storage! {
    pub trait Store for Module<T: Trait> as Counter {
        pub Value: CounterValue = CounterValue(0);
    }
}

use srml_system as system;
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn inc(origin) -> Result {
            let author = ensure_signed(origin)?;

            let CounterValue(value) = Value::get();
            let next = value + 1;
            Value::put(CounterValue(next));

            Self::deposit_event(RawEvent::Incremented(next, author));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as srml_system::Trait>::AccountId,
    {
        Incremented(u32, AccountId),
    }
);
