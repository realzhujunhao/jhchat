use crate::{
    command::Command,
    message::{Content, Message},
    error::{Result, Error},
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

    pub async fn send_from(&mut self, from: &str, mut msg: Message) -> Result<()> {
        let target = msg
            .args
            .get(0)
            .ok_or(Error::InvalidMessage)?;
        let tx = self
            .list
            .get_mut(target)
            .ok_or(Error::Offline)?;
        if msg.args.len() == 0 {
            return Err(Error::InvalidMessage);
        }
        msg.args[0] = String::from(from);
        tx.send(msg)
            .await
            .map_err(|_| Error::Channel)?;
        Ok(())
    }

    pub fn list(&self) -> Vec<String> {
        let name_vec: Vec<String> = self.list.keys().map(|s| s.clone()).collect();
        name_vec
    }

    pub fn msg_list(&self) -> Message {
        let list_vec: Vec<String> = self.list.keys().map(|s| s.clone()).collect();
        let list_string = list_vec.join(",");
        Message::plain_text(Command::OnlineList, &list_string)
    }

    pub fn kick(&mut self, name: &str) -> Result<()> {
        self.list.remove(name).ok_or(Error::Offline)?;
        Ok(())
    }

    pub fn debug(&self) {
        for (key, val) in self.list.iter() {
            println!("{:?} -> {:?}", key, val);
        }
    }
}
