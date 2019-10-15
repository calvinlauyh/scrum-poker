use std::clone::Clone;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(test)]
use mockall::automock;

use crate::client::channel::ClientChannel;
use crate::client::{Client, ClientId};

mod default;

pub use default::DefaultClientStore;

pub struct SharedClientStore<S, T>
where
    S: ClientStore<T>,
    T: ClientChannel,
{
    client_store: Arc<RwLock<S>>,
    client_channel_type: PhantomData<T>,
}

impl<S, T> Clone for SharedClientStore<S, T>
where
    S: ClientStore<T>,
    T: ClientChannel,
{
    fn clone(&self) -> Self {
        SharedClientStore {
            client_store: self.client_store.clone(),
            client_channel_type: self.client_channel_type.clone(),
        }
    }
}

impl<S, T> SharedClientStore<S, T>
where
    S: ClientStore<T>,
    T: ClientChannel,
{
    pub fn new(client_store: S) -> Self {
        SharedClientStore {
            client_store: Arc::new(RwLock::new(client_store)),
            client_channel_type: PhantomData,
        }
    }

    pub fn get_readable(&self) -> RwLockReadGuard<S> {
        self.client_store
            .read()
            .expect("Poison error when acquiring client_store read lock")
    }

    pub fn get_writable(&self) -> RwLockWriteGuard<S> {
        self.client_store
            .write()
            .expect("Poison error when acquiring client_store write lock")
    }
}

// #[cfg_attr(test, automock)]
// pub trait ClientStore<T>
// where
//     T: ClientChannel,
// {
//     // Insert client Id to Client pair into store
//     fn insert(&mut self, id: ClientId, client: Client<T>) -> ();

//     /// Find client by given client Id
//     fn get(&self, id: &ClientId) -> Option<&Client<T>>;

//     /// Returns true if the store contains client for the specified client Id.
//     fn contains_key(&self, id: &ClientId) -> bool;
// }
