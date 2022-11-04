# Simple Peer-to-Peer Messaging

## Assumptions To Simplify Implementation
- No need to punch through a NAT
  - No STUN/TURN
- All processes bind to their own localhost
- Only 2 clients and 1 server
- All components blindly trust each other
  - No encryption/authentication

## Step-by-step
1. Client opens P2P socket
2. Client connects to server, tells it its P2P port numbers
3. If server already has one stored
   1. Server sends the P2P socket address to the client
   2. Client then connects to it
   3. Client can talk directly to peer
4. Else
   1. Server calculates the P2P socket address and stores it
   2. Server tells client to wait
   3. Client waits until another client connects to it
   4. Client can talk directly to peer

## Possible Future Extensions
- Have the server keep track of multiple clients
- Allow clients to ask the server for a specific client
- Allow clients to connect to multiple other clients
- Make everything asynchronous
