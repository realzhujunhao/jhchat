# jhchat
#### MileStone

| Feature                    | Status      | Source Path                   | Lib            |
| :------------------------- | ----------- | ----------------------------- | -------------- |
| Application Protocol       | Done        | models/src/codec/msg_codec.rs | tokio-util     |
| Message Delivery           | Done        | server/src/process.rs         | N/A            |
| Log                        | Done        | server/src/init.rs            | tracing        |
| Client Encryption          | Coming Next |                               | RustCrypto/RSA |
| Exchange Public Key        | Coming Next |                               | N/A            |
| "do not trust server" Mode | Coming Next |                               | N/A            |
| Persistence                | To Do       |                               | diesel         |
| Horizontal Scaling         | To Do       |                               | TBD            |

#### Build From Source

##### Prerequisit: rustc 1.71
`git clone https://github.com/realzhujunhao/jhchat.git`
`cd jhchat`
`cargo build --release`
