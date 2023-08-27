use crate::{
    codec::message::Message,
    error::{GlobalResult, ClientError},
};

use std::collections::HashMap;
use tokio::sync::{mpsc, Mutex, RwLock};

type Tx = mpsc::UnboundedSender<Message>;
type Uid = String;

/// `OnlineUsers` holds a read write locked map
/// each entry is a pair of (unique_id, sender)
#[derive(Debug)]
pub struct OnlineUsers {
    pub list: RwLock<HashMap<Uid, Mutex<Tx>>>,
}

impl OnlineUsers {
    pub fn new() -> Self {
        let list = RwLock::new(HashMap::new());
        Self { list }
    }

    /// generate a `Message` that contains current list of online unique_id
    /// the `Message` will have `Text` content tyle
    pub async fn to_msg(&self) -> Message {
        let list_map = self.list.read().await;
        let list_keys: Vec<String> = list_map.keys().map(|key| key.into()).collect();
        Message::online_list(&list_keys.join("\n"))
    }

    /// insert an entry (unique_id, sender) to the map
    pub async fn add_user(&self, uid: &str, tx: Tx) {
        let mut list = self.list.write().await;
        list.insert(uid.into(), Mutex::new(tx));
    }

    /// remove an entry (unique_id, sender) from the map
    pub async fn remove_user(&self, uid: &str) {
        let mut list = self.list.write().await;
        list.remove(uid);
    }

    /// send a `Message` to `receiver`
    /// `Offline` error if `receiver` is not a key in the map
    /// `Channel` error if the sender fails
    pub async fn send(&self, receiver: &str, msg: Message) -> GlobalResult<()> {
        let list = self.list.read().await;
        let tx = list.get(receiver).ok_or(ClientError::ReceiverNotExist.info(&msg.receiver))?;
        let tx = tx.lock().await;
        tx.send(msg.set_receiver(receiver))?;
        Ok(())
    }
}

impl Default for OnlineUsers {
    fn default() -> Self {
        Self::new()
    }
}
