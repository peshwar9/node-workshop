#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // The struct on which we build all of our Pallet logic.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /* Placeholder for defining custom types. */
use frame_support::{traits::UnixTime};

    /* Placeholder for defining custom storage items. */
/// Oracle Event Feed for storing event details posted by root account
#[pallet::storage]
#[pallet::getter(fn oracle_event_feed)]
pub type OracleEventFeed<T: Config> = StorageValue<
    _,
    BoundedVec<OracleEvent<T::OracleEventLength>, T::OracleEventLength>,
    ValueQuery,
>;

    // Your Pallet's configuration trait, representing custom external types and interfaces.
#[pallet::config]
pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    ///Time provider for getting timestamp
    type TimeProvider: UnixTime;

    /// Maximum length for Oracle Event.
    #[pallet::constant]
    type OracleEventLength: Get<u32>;

    /// Maximum time for storing an Oracle Event.
    #[pallet::constant]
    type MaxTimeForEvents: Get<u64>;
}


    // Your Pallet's events.
/// Data related to Oracle Event
#[derive(Encode, Decode, Default, TypeInfo, Clone, MaxEncodedLen)]
#[scale_info(skip_type_params(OracleEventLength))]
pub struct OracleEvent<OracleEventLength> {
    pub event_name: BoundedVec<u8, OracleEventLength>,
    pub event_details: BoundedVec<u8, OracleEventLength>,
    pub timestamp: u64,
}

impl<OracleEventLength> OracleEvent<OracleEventLength> {
    pub fn new(
        event_name: BoundedVec<u8, OracleEventLength>,
        event_details: BoundedVec<u8, OracleEventLength>,
        timestamp: u64,
    ) -> OracleEvent<OracleEventLength> {
        Self { event_name, event_details, timestamp }
    }
}

    // Your Pallet's events.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    /// New feed data is submitted.
    NewEventFeedData {
        event_name: BoundedVec<u8, T::OracleEventLength>,
        event_details: BoundedVec<u8, T::OracleEventLength>,
    },
}

// Your Pallet's error messages.
#[pallet::error]
pub enum Error<T> {
    /// If event feed length this error is dispatched.
    EventFeedLengthExceeded,
}




// Your Pallet's callable functions.
#[pallet::call]
impl<T: Config> Pallet<T> {
        /// A Sudo extrinsic used for feeding events which can be only called once for each block.
        /// It takes two main parameters : event_name & event_details.
        /// This extrinsic fails when event length gets exceeded.
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn feed_event(
            origin: OriginFor<T>,
            event_name: BoundedVec<u8, T::OracleEventLength>,
            event_details: BoundedVec<u8, T::OracleEventLength>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let now: u64 = T::TimeProvider::now().as_secs();

            let new_oracle_event = OracleEvent::new(event_name.clone(), event_details.clone(), now);

            <OracleEventFeed<T>>::mutate(|event_list| event_list.try_push(new_oracle_event))
                .map_err(|_| <Error<T>>::EventFeedLengthExceeded)?;

            // Emit an event.
            Self::deposit_event(Event::NewEventFeedData { event_name, event_details });
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }
}



    // Your Pallet's internal functions.
    impl<T: Config> Pallet<T> {}


    // Your Pallet's hooks functions.
#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_: T::BlockNumber) -> Weight {
            let time: u64 =
                T::TimeProvider::now().as_secs().saturating_sub(T::MaxTimeForEvents::get());

            <OracleEventFeed<T>>::mutate(|event_feed| {
                let high = event_feed.partition_point(|x| x.timestamp <= time);
                event_feed.drain(..high);
            });

            T::DbWeight::get().reads_writes(1, 1)
        }
    }

}
