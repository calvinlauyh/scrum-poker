use std::collections::HashMap;

use super::ClientStore;
use crate::client::channel::ClientChannel;
use crate::client::{Client, ClientId};

pub struct DefaultClientStore<T>
where
    T: ClientChannel,
{
    store: HashMap<ClientId, Client<T>>,
}

impl<T> Default for DefaultClientStore<T>
where
    T: ClientChannel,
{
    fn default() -> Self {
        DefaultClientStore {
            store: HashMap::new(),
        }
    }
}

impl<T> ClientStore<T> for DefaultClientStore<T>
where
    T: ClientChannel,
{
    fn insert(&mut self, id: ClientId, client: Client<T>) -> () {
        self.store.insert(id, client);
    }

    fn get(&self, id: &ClientId) -> Option<&Client<T>> {
        self.store.get(id)
    }

    fn get_mut(&mut self, id: &ClientId) -> Option<&mut Client<T>> {
        self.store.get_mut(id)
    }

    fn contains_key(&self, id: &ClientId) -> bool {
        return self.store.contains_key(id);
    }
}
