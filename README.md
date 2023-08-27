# jhchat
#### MileStone

| Feature                    | Status      | Source Path                 | Lib            |
| :------------------------- | ----------- | --------------------------- | -------------- |
| Application Protocol       | Done        | core/src/codec/msg_codec.rs | tokio-util     |
| Message Delivery           | Done        | server/src/process.rs       | N/A            |
| Log                        | Done        | server/src/init.rs          | tracing        |
| Client Encryption          | Done        | core/encryption/rsa_impl.rs | RustCrypto/RSA |
| Customizable Encryption    | Done        | core/traits/encrypt.rs      | N/A            |
| Exchange Public Key        | Done        | server/src/process.rs       | N/A            |
| "do not trust server" Mode | Coming Next |                             | N/A            |
| Authentication             | To Do       |                             | TBD            |
| Encrypt Keys on Client     | To Do       |                             | TBD            |
| Persistence                | To Do       |                             | diesel         |
| Horizontal Scaling         | To Do       |                             | TBD            |

#### Build From Source

##### Prerequisit: rustc 1.71
`git clone https://github.com/realzhujunhao/jhchat.git`
`cd jhchat`
`cargo build --release`
