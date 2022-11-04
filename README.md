# Simple Peer-to-Peer Messaging

## Assumptions To Simplify Implementation
- No need to punch through a NAT
  - No STUN/TURN
- All processes bind to their own localhost
- Only 2 clients and 1 server
- All components blindly trust each other
  - No encryption/authentication

## Step-by-step
1. Client 1 connects to server
2. Server responds asking for its P2P socket info
3. Client 1 sends it, server stores it
4. Client 1 then listens on its socket
5. Client 2 connects to server
6. Server responds by sending the information about Client 1's P2P socket
8. Client 2 sends message to Client 1
9. Client 1 responds to Client 2

## Sockets
- Use TCP for client-server connection

## Possible Future Extensions
- Have the server keep track of multiple clients
- Allow clients to ask the server for a specific client
- Allow clients to connect to multiple other clients
