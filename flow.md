## 🔹 Full Flow

### 1. Client connects
- TS client opens **WebSocket** → Rust upgrades to WS.
- Server checks if the target **group** exists in the pool (`HashMap<GroupId, Group>`).
- If not, create it.

---

### 2. Client is registered
- Create a new **Client** struct for the connection:
  - `ws_sender` → used to write back to TS client.
  - `rx = group.tx.subscribe()` → listens for broadcast messages in this group.
- Insert the client into the pool (`group.clients.insert(client_id, client)`).

---

### 3. Incoming messages from TS
- The WS **receiver loop** reads messages from the TS client.
- Forward these to a **top-level handler** (often via the pool).
- Handler decides what to do:
  - Map to a command/function
  - Query/update state
  - Broadcast if necessary

---

### 4. Broadcasting
- The handler gets the group, calls `group.tx.send(data)`.
- All subscribed `rx` (including every client’s receiver task) gets the data.

---

### 5. Fanout to TS
- Each client’s receiver task listens on its own `rx`.
- On each message → forwards it over `ws_sender.send(Message::Text(...))`.
- All connected TS clients in that group see the message.
