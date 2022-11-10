# Simple Peer-to-Peer Messaging

## Usage
(with cargo)
- signaling server: `cargo run --bin server`
- client: `cargo run --bin client -- [arguments]`
  - to see all client arguments, put `--help` as an argument

## Assumptions
- No need to punch through a NAT
  - No STUN/TURN
- All processes bind to their own localhost
- Only 1 server and clients just get paired off first-come-first-serve
- All components blindly trust each other
  - No encryption/authentication

## Step-by-step
1. Client opens P2P socket
2. Client connects to server, tells it its P2P port number
3. If server already has one stored
   1. Server sends the P2P socket address to the client
   2. Client then connects to peer
   3. Client can talk directly to peer
4. Else
   1. Server calculates the P2P socket address and stores it
   2. Server tells client to wait
   3. Client waits until peer connects to it
   4. Client can talk directly to peer

## Possible Future Extensions
- Have the server keep track of multiple clients
- Allow clients to ask the server for a specific client
- Allow clients to connect to multiple other clients
- Allow for custom server address through command line argument
  - Currently hardcoded to `127.0.0.1:8888`
- Make everything asynchronous
- Switch away from byte-by-byte socket reading in client
- Write some integration tests
