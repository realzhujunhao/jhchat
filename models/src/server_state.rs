use crate::{
    codec::message::Message,
    error::{Error, Result},
};

use std::collections::HashMap;
use tokio::sync::{mpsc, Mutex, RwLock};

type Tx = mpsc::Sender<Message>;

#[derive(Debug)]
pub struct OnlineUsers {
    pub list: RwLock<HashMap<String, Mutex<Tx>>>,
}

impl OnlineUsers {
    pub fn new() -> Self {
        let list = RwLock::new(HashMap::new());
        Self { list }
    }

    pub async fn to_msg(&self) -> Message {
        let list = self.list.read().await;
        let mut list_vec: Vec<String> = Vec::new();
        list.keys().for_each(|key| {
            list_vec.push(key.into());
        });
        Message::online_list(&list_vec.join(","))
    }

    pub async fn add_user(&self, name: &str, tx: Tx) {
        let mut list = self.list.write().await;
        list.insert(name.into(), Mutex::new(tx));
    }

    pub async fn remove_user(&self, name: &str) {
        let mut list = self.list.write().await;
        list.remove(name);
    }

    pub async fn send(&self, receiver: &str, msg: Message) -> Result<()> {
        let list = self.list.read().await;
        let tx = list.get(receiver).ok_or(Error::Offline(receiver.into()))?;
        let tx = tx.lock().await;
        tx.send(msg.set_receiver(receiver)).await.map_err(|_| Error::Channel)?;
        Ok(())
    }

    // async fn is_online(&self, name: &str) -> bool {
    //     let list = self.list.read().await;
    //     list.get(name).is_some()
    // }
}
