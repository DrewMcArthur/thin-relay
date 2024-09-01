# thin-relay

an **extremely** basic implementation of an ATProto relay

given a list of PDS urls, it subscribes to each PDS (via `/xrpc/com.atproto.sync.subscribeRepos`)

for each message is receives from each PDS, it forwards the message to its own subscribers/clients.