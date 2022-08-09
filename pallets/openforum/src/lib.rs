#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

    use frame_support::inherent::Vec;
    use frame_support::traits::UnixTime;
    
    type IdType = Vec<u8>;
    type HashType = Vec<u8>;
    type ModelName = Vec<u8>;
    type ActionName = Vec<u8>;
    type Payload = Vec<u8>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type TimeProvider: UnixTime;
	}

	//#[pallet::storage]
	//#[pallet::getter(fn something)]
	//pub type Something<T> = StorageValue<_, u32>;

    #[pallet::storage]
    pub(super) type ModelIdHashDoubleMap<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, ModelName, Blake2_128Concat, IdType, HashType, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// [who, model, action, payload, time]
		Action(T::AccountId, ModelName, ActionName, Payload, u64),
		IndexUpdated(T::AccountId, ModelName, ActionName, Payload, u64),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn action(origin: OriginFor<T>, model: ModelName, action: ActionName, payload: Payload) -> DispatchResult {
			let who = ensure_signed(origin)?;

            let block_time: u64 = T::TimeProvider::now().as_secs();

            // In this call function, we do nothing now, excepting emitting the event back
            // This trick is to record the original requests from users to the blocks,
            // but not record it to the on-chain storage.
			Self::deposit_event(Event::Action(who, model, action, payload, block_time));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn index_update(origin: OriginFor<T>, model: ModelName, id: IdType, hash: HashType) -> DispatchResult {
			let who = ensure_signed(origin)?;

            let block_time: u64 = T::TimeProvider::now().as_secs();

            // Write the id-hash pair into each StorageMap, according to the model name
            ModelIdHashDoubleMap::<T>::set(model.clone(), id.clone(), hash.clone());

            let action = "index_update".as_bytes().to_vec();
            let mut payload: Payload = Vec::new();
            payload.append(&mut id.clone());
            payload.push(b',');
            payload.append(&mut hash.clone());

			Self::deposit_event(Event::IndexUpdated(who, model, action, payload, block_time));
			Ok(())
		}
	}

    // Work implementation
    impl<T: Config> Pallet<T> {
        pub fn check_pair_list(model: ModelName, pair_list: Vec<(IdType, HashType)>) -> bool {
            for (id, hash) in pair_list {
                let index_hash = ModelIdHashDoubleMap::<T>::get(&model, id);
                if index_hash != hash {
                    return false;
                }
            }
            return true;
        }
    }

}

