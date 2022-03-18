use crate::{
    scrt::{
        PrefixedStorage, ReadonlyPrefixedStorage, ReadonlyStorage, StdError, StdResult, Storage,
    },
    secret_toolkit::serialization::{Bincode2, Json, Serde},
    storage::{bincode_state::*, json_state::*},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::any::type_name;

pub mod json_state {

    use super::*;

    /// Returns StdResult<()> resulting from saving an item to storage using Json (de)serialization
    /// because bincode2 annoyingly uses a float op when deserializing an enum
    ///
    /// # Arguments
    ///
    /// * `storage` - a mutable reference to the storage this item should go to
    /// * `key` - a byte slice representing the key to access the stored item
    /// * `value` - a reference to the item to store
    pub fn json_save<T: Serialize, S: Storage>(
        storage: &mut S,
        key: &[u8],
        value: &T,
    ) -> StdResult<()> {
        storage.set(key, &Json::serialize(value)?);
        Ok(())
    }

    /// Returns StdResult<T> from retrieving the item with the specified key using Json
    /// (de)serialization because bincode2 annoyingly uses a float op when deserializing an enum.
    /// Returns a StdError::NotFound if there is no item with that key
    ///
    /// # Arguments
    ///
    /// * `storage` - a reference to the storage this item is in
    /// * `key` - a byte slice representing the key that accesses the stored item
    pub fn json_load<T: DeserializeOwned, S: ReadonlyStorage>(
        storage: &S,
        key: &[u8],
    ) -> StdResult<T> {
        Json::deserialize(
            &storage
                .get(key)
                .ok_or_else(|| StdError::not_found(type_name::<T>()))?,
        )
    }

    /// Returns StdResult<Option<T>> from retrieving the item with the specified key using Json
    /// (de)serialization because bincode2 annoyingly uses a float op when deserializing an enum.
    /// Returns Ok(None) if there is no item with that key
    ///
    /// # Arguments
    ///
    /// * `storage` - a reference to the storage this item is in
    /// * `key` - a byte slice representing the key that accesses the stored item
    pub fn json_may_load<T: DeserializeOwned, S: ReadonlyStorage>(
        storage: &S,
        key: &[u8],
    ) -> StdResult<Option<T>> {
        match storage.get(key) {
            Some(value) => Json::deserialize(&value).map(Some),
            None => Ok(None),
        }
    }
}

pub mod bincode_state {
    use super::*;
    /// Returns StdResult<()> resulting from saving an item to storage
    ///
    /// # Arguments
    ///
    /// * `storage` - a mutable reference to the storage this item should go to
    /// * `key` - a byte slice representing the key to access the stored item
    /// * `value` - a reference to the item to store
    pub fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
        storage.set(key, &Bincode2::serialize(value)?);
        Ok(())
    }

    /// Removes an item from storage
    ///
    /// # Arguments
    ///
    /// * `storage` - a mutable reference to the storage this item is in
    /// * `key` - a byte slice representing the key that accesses the stored item
    pub fn remove<S: Storage>(storage: &mut S, key: &[u8]) {
        storage.remove(key);
    }

    /// Returns StdResult<T> from retrieving the item with the specified key.  Returns a
    /// StdError::NotFound if there is no item with that key
    ///
    /// # Arguments
    ///
    /// * `storage` - a reference to the storage this item is in
    /// * `key` - a byte slice representing the key that accesses the stored item
    pub fn load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
        Bincode2::deserialize(
            &storage
                .get(key)
                .ok_or_else(|| StdError::not_found(type_name::<T>()))?,
        )
    }

    /// Returns StdResult<Option<T>> from retrieving the item with the specified key.
    /// Returns Ok(None) if there is no item with that key
    ///
    /// # Arguments
    ///
    /// * `storage` - a reference to the storage this item is in
    /// * `key` - a byte slice representing the key that accesses the stored item
    pub fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(
        storage: &S,
        key: &[u8],
    ) -> StdResult<Option<T>> {
        match storage.get(key) {
            Some(value) => Bincode2::deserialize(&value).map(Some),
            None => Ok(None),
        }
    }
}

pub mod traits {

    use super::*;
    /// Example - b"config" -> Config
    pub trait SingletonStorable: Serialize + DeserializeOwned {
        /// Example - b"config".to_vec()
        fn namespace() -> Vec<u8> {
            Vec::new()
        }

        fn new<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
            load(storage, Self::namespace().as_slice())
        }

        fn get<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
            load(storage, Self::namespace().as_slice())
        }

        fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
            save(storage, Self::namespace().as_slice(), &self)?;
            Ok(())
        }

        fn remove<S: Storage>(self, storage: &mut S) -> StdResult<()> {
            remove(storage, Self::namespace().as_slice());
            Ok(())
        }

        fn new_json<S: ReadonlyStorage>(storage: &S) -> StdResult<Self> {
            json_load(storage, Self::namespace().as_slice())
        }

        fn save_json<S: Storage>(self, storage: &mut S) -> StdResult<()> {
            json_save(storage, Self::namespace().as_slice(), &self)?;
            Ok(())
        }
    }

    /// Example - position_map-(address) -> (position)
    pub trait KeyedStorable: Serialize + DeserializeOwned {
        fn namespace() -> Vec<u8> {
            Vec::new()
        }

        fn storage<S: ReadonlyStorage>(storage: &S) -> ReadonlyPrefixedStorage<S> {
            ReadonlyPrefixedStorage::new(Self::namespace().as_slice(), storage)
        }

        fn mut_storage<S: Storage>(storage: &mut S) -> PrefixedStorage<S> {
            PrefixedStorage::new(Self::namespace().as_slice(), storage)
        }

        fn load<S: ReadonlyStorage, T: DeserializeOwned>(storage: &S, key: &[u8]) -> StdResult<T> {
            let object = may_load(&Self::storage(storage), key)?;
            match object {
                Some(object) => Ok(object),
                None => Err(StdError::generic_err("Could not find item.")),
            }
        }

        fn save<S: Storage, T: Serialize>(storage: &mut S, key: &[u8], val: T) -> StdResult<()> {
            save(&mut Self::mut_storage(storage), key, &val)?;
            Ok(())
        }

        fn remove<S: Storage>(storage: &mut S, key: &[u8]) -> StdResult<()> {
            remove(&mut Self::mut_storage(storage), key);
            Ok(())
        }

        fn load_json<S: ReadonlyStorage, T: DeserializeOwned>(
            storage: &S,
            key: &[u8],
        ) -> StdResult<T> {
            let object = json_may_load(&Self::storage(storage), key)?;
            match object {
                Some(object) => Ok(object),
                None => Err(StdError::generic_err("Could not find item.")),
            }
        }

        fn save_json<S: Storage, T: Serialize>(
            storage: &mut S,
            key: &[u8],
            val: T,
        ) -> StdResult<()> {
            json_save(&mut Self::mut_storage(storage), key, &val)?;
            Ok(())
        }
    }

}
