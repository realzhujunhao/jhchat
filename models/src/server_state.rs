use crate::{
    codec::message::Message,
    error::{Error, Result},
};

use std::collections::HashMap;
use tokio::sync::mpsc;

type Tx = mpsc::Sender<Message>;

#[derive(Debug)]
pub struct OnlineUsers {
    pub list: HashMap<String, Tx>,
}

impl OnlineUsers {
    pub fn new() -> Self {
        let list = HashMap::new();
        Self { list }
    }

    pub async fn send(&mut self, msg: Message) -> Result<()> {
        let tx = self
            .list
            .get_mut(&msg.receiver)
            .ok_or(Error::Offline(msg.receiver.clone()))?;
        tx.send(msg).await.map_err(|_| Error::Channel)?;
        Ok(())
    }

    pub fn list(&self) -> Vec<String> {
        let name_vec: Vec<String> = self.list.keys().map(|s| s.clone()).collect();
        name_vec
    }

    pub fn msg_list(&self) -> Message {
        let list_vec: Vec<String> = self.list.keys().map(|s| s.clone()).collect();
        let list_string = list_vec.join(",");
        Message::send_text("Server", "", &list_string)
    }

    pub fn kick(&mut self, name: &str) -> Result<()> {
        self.list.remove(name).ok_or(Error::Offline(name.into()))?;
        Ok(())
    }

    pub fn debug(&self) {
        for (key, val) in self.list.iter() {
            println!("{:?} -> {:?}", key, val); } }
}
