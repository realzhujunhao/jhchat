use crate::handler;
use bytes::Bytes;
use futures::SinkExt;
use models::{command::Command, msg_codec::MsgCodec, user::OnlineUsers};
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

pub async fn process(
    stream: TcpStream,
    addr: SocketAddr,
    online_users: Arc<Mutex<OnlineUsers>>,
) -> Result<(), Box<dyn Error>> {
    let (rd, wt) = stream.into_split();
    let mut rd_frame = FramedRead::new(rd, MsgCodec::new());
    let mut wt_frame = FramedWrite::new(wt, BytesCodec::new());
    wt_frame.send(Bytes::from("username: ")).await?;
    let username = handler::login(Arc::clone(&online_users), &mut rd_frame, wt_frame, addr).await?;
    loop {
        match rd_frame.next().await {
            Some(Ok(msg)) => {
                match msg.command {
                    Command::Help => handler::help(Arc::clone(&online_users), &username).await?,
                    Command::OnlineList => handler::online_list(Arc::clone(&online_users), &username).await?,
                    Command::SendMsgToUser => handler::send_msg(Arc::clone(&online_users), &msg, &username).await?,
                    _ => (),
                }
            }
            _ => {
                println!("disconnect");
                online_users.lock().await.kick(&username).await?;
                break;
            }
        }
    }

    Ok(())
}
