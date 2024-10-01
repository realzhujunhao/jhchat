# jhchat
#### UPDATE
haven't taken security course when I made this. 
If I were to make such app now, a few changes would have been made:
- Instead of plain TCP connection, I would look for TLS libraries
- Dilemma below could be solved by either
  - trust a Certificate Authority
  - adopt DPKI, make use of block chain to exchange key pairs
- Use `thiserror` to define my error types
- Use `tonic` (grpc) or `axum` (http) instead of making my own protocol
- Replace the encryption library by openssl bindings
- Avoid some silly code in my current perspective

#### Goals

- Users do trust `Client` (They build it from source)
- `Client` can decide not to trust `Server` (By default public keys are exchanged through server)
  - User may manually set other's public key via third party communication
  - A pre-configured dummy message is sent if the key delivered by server is not identical
    - with warnings on client side
    - dummy message is encrypted by server's public key
      - `Client` behaves as if it hasn't discovered `Server`'s fraud

- `Client` can decide not to trust user's device
  - Private key is encrypted on local disk and decrypted by server during connection

In short, this project assumes that the server and the current computer user are malicious. Our goal is to have each potential hostile hold one piece of the puzzle, so that information leaks only if they refer to the same individual.

#### Dilemma

It is quite inconvenient to manually exchange public keys when users don't trust the server, but for now I am not aware of any alternative solution.

Ideally, users expect the following would happen when they send a message
<img width="534" alt="expected" src="https://github.com/realzhujunhao/jhchat/assets/63294481/3b72b5d7-6966-476a-89ed-40715cd9a63f">


However, no one can prevent the server from doing this

<img width="669" alt="unexpected" src="https://github.com/realzhujunhao/jhchat/assets/63294481/a5759071-cc2a-457e-81ef-1c872cfbf324">


#### MileStone

| Feature                        | Status      | Source Path                 | Lib            |
| :----------------------------- | ----------- | --------------------------- | -------------- |
| Application Protocol           | Done        | core/src/codec/msg_codec.rs | tokio-util     |
| Message Delivery               | Done        | server/src/process.rs       | N/A            |
| Log                            | Done        | server/src/init.rs          | tracing        |
| Config                         | Done        | core/src/config.rs          | serde + toml   |
| Client Encryption              | Done        | core/encryption/rsa_impl.rs | RustCrypto/rsa |
| Customizable Encryption        | Done        | core/traits/encrypt.rs      | N/A            |
| Exchange Public Key            | Done        | server/src/process.rs       | N/A            |
| Offline Pubkey Mode            | Coming Next |                             | N/A            |
| Update Key Strategy            | Coming Next |                             | N/A            |
| Authentication                 | To Do       |                             | TBD            |
| Unsafe Group Chat              | To Do       |                             | N/A            |
| Expensive Group Chat (e2ee)    | To Do       |                             | N/A            |
| File Server (RSA + AES)        | To Do       |                             | RustCrypto/aes |
| Encrypt Private Keys on Client | To Do       |                             | TBD            |
| Chat History Persistence       | To Do       |                             | diesel         |
| Horizontal Scaling             | To Do       |                             | TBD            |

#### Build From Source

##### Prerequisit: rustc 1.71
`git clone https://github.com/realzhujunhao/jhchat.git`
`cd jhchat`
`cargo build --release`
