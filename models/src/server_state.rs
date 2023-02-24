use std::{io, collections::HashMap};
use crate::{message::Message, user::User};
use tokio::io::AsyncWriteExt;


#[derive(Debug)]
pub struct OnlineUsers {
    pub list: HashMap<String, User>,
}

impl OnlineUsers {
    pub fn new() -> Self {
        let list = HashMap::new();
        Self { list }
    }

    pub async fn send_from(&mut self, from: &str, msg: Message) -> io::Result<()> {
        // let target_user = self
            // .list
            // .get_mut(name)
            // .ok_or(io::Error::new(io::ErrorKind::NotConnected, name.clone()))?;
        // target_user.send(content).await?;
        Ok(())
    }

    pub async fn kick(&mut self, name: &str) -> io::Result<()> {
        let user = self.list.remove(name);
        match user {
            Some(mut user) => {
                let stream = user.msg_frame.get_mut();
                let _ = stream.shutdown().await;
                Ok(())
            },
            None => Ok(())
        }
    }

    pub fn list(&self) -> Vec<String> {
        let name_vec: Vec<String> = self.list.keys().map(|s| s.clone()).collect();
        name_vec
    }

    pub async fn debug(&self) {
        for (key, val) in self.list.iter() {
            println!("{:?} -> {:?}", key, val);
        }
    }
}
