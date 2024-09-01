# thin-relay

an **extremely** basic implementation of an ATProto relay

## what it does
given a list of PDS urls, it subscribes to each PDS (via `/xrpc/com.atproto.sync.subscribeRepos`)

for each message is receives from each PDS, it forwards the message to its own subscribers/clients.

## feature parity
- [x] subscribe to messages in realtime
- [ ] crawling repos
- [ ] backfill
- [ ] nice webview dashboard (see https://whtwnd.com/bnewbold.net/3kwzl7tye6u2y)
- [ ] filtering (e.g. only subscribe to `app.bsky.*` records)
