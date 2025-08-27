# Tasklist

- [ ] Sending messages through functions
- [ ] Use registers to send messages

- [ ] Add possibility for multiple ws hubs, with their own brokers

- [ ] Spawn senders for clients so the group is not locked
- [ ] tracing

Needs
- Pass in vec of endpoints for connections
- Pass in what file to handle this connection
- Some kind of shared state of the connections/groups?
- Spawn a group if not exist
- join a group if exists by subscribing
- methods for disconnect if any errors occur on requests (Maybe implement a ping?)

- Docs for how to talk to this via ts or so