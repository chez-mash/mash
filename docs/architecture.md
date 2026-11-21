# How Mash works

Mash reads the history you choose from [supported agents](supported-agents.md)
and makes it searchable on your machine. The agents keep their original chats.

![Local Mash indexing reads agent histories and builds a SQLite chat copy and a Qdrant search index. The dashboard, CLI, and agents connected via MCP read and search that history. Optional local embeddings add search by meaning, and multiple machines can publish to an authenticated Mash server you run.](../assets/mash-architecture.svg)

## On each machine

Indexing keeps a copy of your chats in SQLite and builds a Qdrant search
index from that copy. The [dashboard](dashboard.md), [terminal commands](search.md), and
[agents you connect via MCP](mcp.md) use the same underlying history and search.
You can use the dashboard or terminal without connecting an agent.

Word and phrase search works without models. To [search by meaning](indexing.md#enable-semantic-search),
you can install a model that turns dialogue into embeddings for the Qdrant
index. Mash can build the embeddings on your own computer using a supported
GPU or other accelerator, or on the CPU. [How Mash chooses an embedding engine](configuration.md#embedding-inference)

## Share across machines if you choose

With [sharing enabled](shared-index.md), each computer keeps its own local
index and publishes its indexed history to an authenticated `mash-server`
you run. That server keeps chat copies in its own SQLite storage and has its
own Qdrant search index. Machines authorized for the same group can search
that shared history, and each machine can keep searching its own copy offline.
[Review what you share before publishing](shared-index.md#before-sharing)
