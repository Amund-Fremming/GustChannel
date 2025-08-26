# Tasklist

- [ ] Sending messages through functions
    - Start by just setting one default function
    - use this to send messages
- [ ] Change to payload sending
- [ ] Use registers to send messages
- [ ] Spawn senders for clients so the group is not locked

Needs
- Pass in vec of endpoints for connections
- Pass in what file to handle this connection
- Some kind of shared state of the connections/groups?
- Spawn a group if not exist
- join a group if exists by subscribing
- methods for disconnect if any errors occur on requests (Maybe implement a ping?)

- Docs for how to talk to this via ts or so