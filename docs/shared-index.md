# Set up a shared index

If your laptop and desktop both save agent chats, a shared index lets you find
them from either computer. Each computer keeps an index of its own chats and
publishes it to one Mash server you run.

| What you want to do | Where to start |
| --- | --- |
| Use history from computers you already connected | [Search across your machines](search-across-machines.md) |
| Connect a computer with a join string you received | [Join the existing server](#join-and-review-each-computer) |
| Run your own server | [Set up with Tailscale](#the-quick-way) or [set up by hand](#1-set-up-the-server) |
| Read one computer’s chats from your phone or another browser | [Open that computer’s dashboard remotely](dashboard-access.md) |

## Before sharing

**Publishing sends your indexed chats and retained attachments to the server.**
Any computer with a token for the same group can search and read that history.
Use a server and network you trust. Review the chats you already have indexed,
including older ones that Mash has kept, before you start publishing.
[When history can leave your machine](privacy.md#when-history-can-leave-your-machine)

## What runs where

![Three machines, a laptop, a desktop and a build box, each run several agents. On each one, Mash reads every agent's history into its own local index, which keeps working offline. Each machine publishes its own chats to one Mash server you run and searches everything the group has published. The server checks each machine's token, keeps the chats from each machine in SQLite, and builds one shared Qdrant search index.](../assets/mash-shared-index.svg)

Each computer indexes the agents installed on it, as it would without
sharing, and then publishes that index to the server. Search on any computer
in the group combines its own index with what the other computers published.
If the server is unreachable, each computer still searches its own chats.

| Where | What runs | You set it up |
| --- | --- | --- |
| One server machine | `mash-server` and a Qdrant 1.20+ instance for it, both with persistent storage | once |
| Each computer that indexes chats | `mash` with its local index and local Qdrant, as in [installation](installation.md) | once per computer |

The server machine can be one of your computers or a separate always-on host.
`mash-server` ships with `mash`; run the same release on the server and on
every computer.

If you already have a server, ask its operator for a join string for each
computer. You can also connect using the server URL, group ID, a name for the
computer (its node ID), and its own token. [Connect each computer](#2-connect-each-computer)

<span id="the-quick-way"></span>

## Set up with Tailscale

Use this path when the server runs Linux or macOS, has Qdrant running, and
can use Tailscale Serve. Connect the server and the computers to your tailnet.
Enable MagicDNS and HTTPS certificates for that network.
[Start Qdrant](installation.md#start-the-local-search-service) · [Enable HTTPS in Tailscale](https://tailscale.com/docs/how-to/set-up-https-certificates)

On the server, add your laptop to a group called `personal`.

```sh
mash-server --setup --add-node personal/laptop
```

This writes the server settings under `~/.mash/server`, starts a user service,
and prints its private HTTPS address and a `join` value. To add another
computer, run the command on the server again with a different name, such as
`personal/desktop`. Use a different join string for each computer. The join
string contains its credential, so pass it privately. Each join string works
once, within 24 hours; if it expires or is used, make a new one. Rerunning
setup with a name already added repairs the service and reuses that
registration.

The first setup also makes an admin key and saves it for the dashboard on the
server machine, so you can add, remove and replace machines from Machines
there. To manage machines from another computer, see
[Manage machines from any computer](#manage-machines-from-any-computer).

### Join and review each computer

On a computer you want to connect, [install Mash](installation.md) and pause
any background indexing while you review. In Mash, open Machines or Settings,
paste the join value into **Join a shared index**, and choose **Check**. Mash
shows the server, group and name the string joins as before saving anything;
**Save and continue** then joins. Or run this in a terminal with the full join
value for that computer.

```sh
mash init --join 'JOIN_STRING_FROM_SERVER' --no-index --no-agents
```

Joining saves the server, group, and node settings, and stores the credential
in `~/.mash/node.env`, readable only by you. These options skip indexing and
leave agent settings alone. A plain `mash init --join` also connects detected
supported agents and starts indexing and publishing without a wizard.

Run `mash status --installation` to check the configured history folders and use
the local dashboard to review any chats already indexed. When you are ready
to send that history to the server, [publish and check each computer](#3-publish-and-check).

This setup uses word search on the server and needs no model download. For a
host without Tailscale or a server that needs semantic search, use the manual
setup below. To take a server created with `--setup` offline, run
`mash-server --setup --remove`. It keeps the settings and history for the next
time you start the server.

## 1. Set up the server

On the server machine:

1. Start Qdrant for the server.
   [`mash qdrant install`](installation.md#start-the-local-search-service)
   runs one on `127.0.0.1`, ports 6333 (HTTP) and 6334 (gRPC). The
   [Docker image](local-search.md#optional-qdrant-docker-image) or an existing
   Qdrant 1.20+ service also works. Keep Qdrant's ports private to this host.

2. Create one token per computer. Each must be 32 to 4096 printable ASCII
   characters with no spaces, for example:

   ```sh
   openssl rand -hex 32
   ```

3. Save this as `server.toml`. It lists every computer allowed to connect and
   uses word search only, so the server needs no model download:

   ```toml
   version = 1
   listen = "127.0.0.1:8787"
   qdrant_url_env = "MASH_QDRANT_URL"
   qdrant_grpc_url_env = "MASH_QDRANT_GRPC_URL"

   [views.build]
   representations = "lexical-only"

   [[nodes]]
   group_id = "personal"
   node_id = "laptop"
   token_env = "MASH_LAPTOP_TOKEN"

   [[nodes]]
   group_id = "personal"
   node_id = "desktop"
   token_env = "MASH_DESKTOP_TOKEN"
   ```

   The file names environment variables; it never holds the secrets. Keep
   tokens out of Git.

4. Set those variables and start the server. Both Qdrant variables must point
   at the **same** Qdrant instance:

   ```sh
   export MASH_QDRANT_URL=http://127.0.0.1:6333
   export MASH_QDRANT_GRPC_URL=http://127.0.0.1:6334
   export MASH_LAPTOP_TOKEN='the token you made for the laptop'
   export MASH_DESKTOP_TOKEN='the token you made for the desktop'
   mash-server --server-config /absolute/path/to/server.toml
   ```

   Keep it running under your service manager, with the same variables.

5. Make it reachable. `listen = "127.0.0.1:8787"` accepts connections from the
   server machine only. Put an HTTPS reverse proxy in front of it, or set
   `listen` to the machine's private-network IP and use that address as the
   server URL.

This configuration is for a new server. Back up an existing one before
changing its settings.

## 2. Connect each computer

Do this on every computer that should publish or search shared history. Mash
must already be [installed](installation.md) there, with its local Qdrant
running.

### With a join string

On the server, register each computer with a unique node ID in the group.
Supply the address that computer will use to reach the server.

```sh
mash-server --server-config /absolute/path/to/server.toml \
  --add-node personal/laptop --api-url https://mash.example.net
```

It prints a `join` value starting with `mash-join.`. Restart the server so it
accepts the new computer, then [join and review that computer](#join-and-review-each-computer).
The server saves a hash of the credential in a separate file beside its
configuration, such as `server.toml.nodes`.
[Replace or revoke a join credential](#revoke-or-replace-a-computers-credential)

### By hand

Or set the token and settings yourself.

First put this computer's token in an environment variable, for example
`MASH_NODE_TOKEN`. Mash stores only the variable's name, so the variable must
be set wherever Mash runs: your shell, the
[watcher](synchronization.md), and the agent clients that start Mash's MCP
server.

Then either use the wizard or edit the configuration.

**Wizard.** Run `mash init --no-index --no-agents` in a terminal. If you start on
a review screen, choose **Customize setup** (called **Review and edit settings**
in v0.1.1-alpha and earlier). When asked where to search, choose **Connect to an existing shared server**.
Enter the server URL, group ID,
this computer's node ID, and the name of the token variable. Review the
history folders and choose **Set up without indexing**. The next step sends
the history to the server.

**By hand.** Edit the node configuration that `mash init` created. Keep its
providers and local Qdrant settings. In the existing `[index]` table set
`destination`, and add a `[shared]` table:

```toml
[index]
destination = "both"
since_days = 90

[shared]
group_id = "personal"
node_id = "laptop"
api_url = "https://mash.example.net"
token_env = "MASH_NODE_TOKEN"
```

`group_id` and `node_id` must match one `[[nodes]]` entry on the server, and
the variable must hold that entry's token. On the desktop, use
`node_id = "desktop"` and the desktop's token.

Search settings must suit the server. For the word-search server above, each
computer uses `[views.build] representations = "lexical-only"` and
`[views.query] intent = "exact-mention"`, which is what `mash init` writes
when no models are installed. For semantic search, ask the server
administrator for matching model settings.

## 3. Publish and check

On **each** computer, [start the local search service](installation.md#start-the-local-search-service)
and run these commands.

```sh
mash index
mash status --shared
```

`mash index` updates the local index, then publishes the history it retains.
The configured date range controls which history files Mash reads on this
pass. Older chats already in the local index can still be published.
`mash status --shared` reports what the server received under `shared_index`.
If you see `shared_error`, check the server address, connection, and token.

A computer's chats appear on the others only after that computer has
published. If publishing fails, local history is kept: fix the reported error
and run `mash index` again. To keep publishing new chats, run a
[watcher](synchronization.md) on each computer.

## Search

[Search across your machines](search-across-machines.md) shows how to choose a
machine in the dashboard, search from the terminal, and check for missing chats.

## Manage machines from any computer

Every computer that joined can see the server's machines in Machines: which
have joined, which are waiting, and when each last published. It reads them
with its own credential, which can never change anything.

To add, remove or replace machines from another computer, give it an admin
key. On the server machine, run:

```sh
mash-server --server-config /absolute/path/to/server.toml --add-admin laptop
```

It prints the key once; the server keeps only its hash. On the computer you
manage from, open Machines, choose **Manage this server**, and paste the key.
Mash checks it with the server before saving it in `~/.mash/admin.env`,
readable only by you. A dashboard other people can reach without signing in
never uses it. The server records each admin action with the key's name and
the computer it came from, shown under Recent admin actions. Too many wrong
keys make the server refuse every admin request for a minute.

Run `--add-admin` with the same name to replace a key, and
`--revoke-admin NAME` to stop accepting one, for example for a lost laptop.

## Remove a machine, or let a rebuilt one publish again

The server ties each node ID to the local index that first published under it,
so another machine cannot overwrite that history by reusing the name. A machine
whose index was deleted and rebuilt is refused for the same reason: publishing
fails with "shared node ownership or revision changed". Restoring the index
from a [backup](upgrades.md#replace-an-index-that-cannot-be-upgraded) with
`--keep-identity` keeps the name. Otherwise, retire the name on the server.
To remove a computer for good, stop publishing from that computer and retire
its history before revoking its credential. Uninstalling Mash on a computer
leaves the server's copy in place.

For a machine added with a join string, use Machines in Mash with an admin
key: **Remove** stops its credential at once and keeps what it published, and
**Delete what it published** then deletes it while the server runs, once you
type the machine's name. The rest of this section is the terminal route, and
the only one for a machine listed in `server.toml` itself.

Stop `mash-server` and keep its Qdrant running. Run this with the same server
configuration and environment the service uses:

```sh
# On the server machine, with mash-server stopped.
mash-server --server-config /absolute/path/to/server.toml --retire-node personal/laptop
```

**This permanently deletes everything that machine published:** its chats and
records on the server, their search data, and its shared attachments. The
command prints how many chats and messages it removed and exits without
serving. Other machines and the retired machine's own local history are not
touched.

If you are rebuilding a computer’s index, start `mash-server` again, then run
`mash index` on that computer to publish from its current index. If you are
removing the computer, revoke its credential before restarting the server.

### Revoke or replace a computer’s credential

For a machine added with a join string, choose **Replace credential** or
**Remove** in Machines with an admin key. Either takes effect without a
restart; a replaced credential comes with a new join string, shown once. The
computer is told its credential was revoked the next time it connects.

For a computer added by hand, on the server, find the `[[nodes]]` block with that computer’s `group_id` and
`node_id`. For a computer added with `--add-node`, the block is in the file
next to your server configuration, usually `server.toml.nodes`. For a computer
added by hand, it is in `server.toml`. Remove the block and restart the server
to revoke access. If this was the last computer in the configuration, leave
the server stopped. Skip the retirement command to keep the published files
on the server.

To replace a lost or exposed join string, remove the old block, add the same
group and node again, and restart the server. Join from the same computer
with the new string. It can still publish under the same name if it kept its
local index.

Before installing a new Mash release, read
[upgrades and backups](upgrades.md#upgrade-a-shared-index).
