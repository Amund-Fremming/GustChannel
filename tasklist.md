# Tasklist

- [ ] Change hub registration
    - when creating hub, return a channel sender
    - hub spawns a new task for listening to the channel and broadcasting
    - when registring functions, call this channel to broadcast data
    - create envelope for channel to send structured data (group_id, payload)
- [ ] Registry might not need to be RwLock since only one thread adds, could be succicient with arc when just reading happends after
- [ ] Look for optimalizations
- [ ] Look for deadlocks 
- [ ] Benchmark test