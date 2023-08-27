# jhchat
#### Goals

- Users should trust `Client` (They build it from source)

- `Client` can decide not to trust `Server`  (In progress)
  - User may manually set other's public key via third party communication
  - A pre-configured dummy message is sent if the key delivered by server is not identical
    - with warnings on client side
    - dummy message is encrypted by server's public key
      - `Client` behaves as if it hasn't discovered `Server`'s fraud

- `Client` can decide not to trust user's device (In case stolen)
  - Private key is encrypted on local disk and decrypted by server during connection

In short, this project assumes that the server and the current computer user are malicious. Our goal is to have each potential hostile hold one piece of the puzzle, so that information leaks only if they refer to the same individual.

#### Dilemma

It is quite inconvenient to manually exchange public keys when users don't trust the server, but for now  I am not aware of any alternative solution.

Ideally, users expect the following would happen when they send a message
![image-20230828014621450](/Users/junhaozhu/Library/Application Support/typora-user-images/image-20230828014621450.png)

However, no one can prevent the server from doing this

![image-20230828014714082](/Users/junhaozhu/Library/Application Support/typora-user-images/image-20230828014714082.png)

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
