# 跨设备搜索

如果笔记本电脑和台式机都保存了代理对话，共享索引可让你在任意一台电脑上找到这些记录。每台电脑都保留一份可搜索的本地副本，并将索引发布到你运行的同一台 Mash 服务器。

| 你想做什么 | 从哪里开始 |
| --- | --- |
| 在手机或其他浏览器上阅读某台电脑的对话 | [远程打开该电脑的网页面板](dashboard.md#open-it-from-your-phone-or-another-computer) |
| 搜索多台电脑已索引的对话 | [设置共享索引](#the-quick-way) |
| 仅搜索这台电脑上的历史记录 | [使用本地网页面板](dashboard.md#find-and-read-a-chat) |

要看看来自其他设备的搜索结果是什么样的，请在演示中搜索并打开工作站上的对话 **Choose a queue for invoice exports**。
[试用示例搜索](https://getmash.io/demo/#/search?q=invoice+exports+queue)

<span id="before-sharing"></span>

## 共享前

**发布操作会将已索引的对话和保留的附件发送到服务器。**
任何持有同一组令牌的电脑都可以搜索和阅读这些历史记录。请选择你信任的服务器和网络。开始发布前，请检查已索引的对话，包括 Mash 保留的较早记录。
[历史记录何时会离开你的设备](privacy.md#when-history-can-leave-your-machine)

<span id="what-runs-where"></span>

## 各设备运行什么

| 运行位置 | 运行内容 | 设置次数 |
| --- | --- | --- |
| 一台服务器 | `mash-server` 及其专用的 Qdrant 1.20+ 实例，两者都使用持久化存储 | 一次 |
| 每台为对话建立索引的电脑 | `mash` 及其本地索引和本地 Qdrant，详见 [安装说明](installation.md) | 每台电脑一次 |

服务器可以是你现有的一台电脑，也可以是另一台始终在线的主机。
`mash-server` 随 `mash`一同分发。请在服务器和每台电脑上运行相同的发行版本。

如果已有服务器，请向服务器管理员索取每台电脑各自的加入字符串。也可以使用服务器 URL、组 ID、电脑名称（即节点 ID）和该电脑自己的令牌来连接。 [连接每台电脑](#2-connect-each-computer)

<span id="the-quick-way"></span>

## 快速设置

如果服务器运行 Linux 或 macOS、已启动 Qdrant 且可以使用 Tailscale Serve，请采用此方法。将服务器和各台电脑接入你的 tailnet，并为该网络启用 MagicDNS 和 HTTPS 证书。
[启动 Qdrant](installation.md#start-the-local-search-service) · [在 Tailscale 中启用 HTTPS](https://tailscale.com/docs/how-to/set-up-https-certificates)

在服务器上，将笔记本电脑加入名为 `personal`的组。

```sh
mash-server --setup --add-node personal/laptop
```

此命令会将服务器配置写入 `~/.mash/server`，启动一个用户级服务，并输出服务的私有 HTTPS 地址和一个 `join` 值。要添加另一台电脑，请在服务器上用不同的名称重新运行命令，例如
`personal/desktop`。每台电脑都应使用不同的加入字符串。字符串包含该电脑的凭据，请通过私密渠道传递。使用已添加的名称重新运行设置命令会修复服务并沿用原有的注册信息。

<span id="join-and-review-each-computer"></span>

### 加入并检查每台电脑

在要连接的电脑上安装 Mash，并启动本地 Qdrant。检查期间，请暂停该电脑上的后台索引。然后运行以下命令，传入该电脑的完整加入字符串。

```sh
mash init --join 'JOIN_STRING_FROM_SERVER' --no-index --no-agents
```

加入时会保存服务器、组和节点的配置，并将凭据存入 `~/.mash/node.env`，只有你可以读取该文件。上述选项会跳过索引并保留现有的代理设置。直接运行 `mash init --join` 还会连接检测到的受支持代理，并跳过向导，直接开始索引和发布。

运行 `mash status --installation` 查看已配置的历史记录文件夹，并在本地网页面板中检查已索引的对话。准备好将这些历史记录发送到服务器后，请 [在每台电脑上发布并检查](#3-publish-and-check)。

此设置在服务器上使用关键词搜索，无需下载模型。如果主机没有 Tailscale，或服务器需要语义搜索，请使用下文的手动设置。要停止通过 `--setup` 创建的服务器，请运行
`mash-server --setup --remove`。该操作会保留配置和历史记录，供下次启动服务器时使用。

<span id="1-set-up-the-server"></span>

## 1. 设置服务器

在服务器上执行以下操作。

1. 启动服务器使用的 Qdrant。
   [`mash qdrant install`](installation.md#start-the-local-search-service)
   会在 `127.0.0.1`上运行一个实例，使用 6333（HTTP）和 6334（gRPC）端口。也可以使用
   [Docker 镜像](local-search.md#optional-qdrant-docker-image) 或现有的 Qdrant 1.20+ 服务。请仅允许本机访问 Qdrant 的端口。

2. 为每台电脑创建一个令牌。每个令牌须由 32 至 4096 个不含空格的可打印 ASCII 字符组成。示例命令如下。

   ```sh
   openssl rand -hex 32
   ```

3. 将以下内容保存为 `server.toml`。它列出允许连接的所有电脑，并且仅使用关键词搜索，因此服务器无需下载模型。

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

   此文件只记录环境变量名，凭据保存在环境变量中。请勿将令牌提交到 Git。

4. 设置这些变量并启动服务器。两个 Qdrant 环境变量必须指向 **同一个** Qdrant 实例。

   ```sh
   export MASH_QDRANT_URL=http://127.0.0.1:6333
   export MASH_QDRANT_GRPC_URL=http://127.0.0.1:6334
   export MASH_LAPTOP_TOKEN='the token you made for the laptop'
   export MASH_DESKTOP_TOKEN='the token you made for the desktop'
   mash-server --server-config /absolute/path/to/server.toml
   ```

   使用服务管理器让服务器持续运行，并为它设置相同的环境变量。

5. 让其他电脑可以访问服务器。 `listen = "127.0.0.1:8787"` 仅接受来自服务器本机的连接。可在服务前部署 HTTPS 反向代理，也可将
   `listen` 设为服务器的私有网络 IP 地址，并使用该地址作为服务器 URL。

此配置适用于新服务器。如果已有服务器，请先备份再更改配置。

<span id="2-connect-each-computer"></span>

## 2. 连接每台电脑

在每台需要发布或搜索共享历史记录的电脑上执行此操作。请先 [安装](installation.md) Mash，并确保本地 Qdrant 已在运行。

<span id="with-a-join-string"></span>

### 使用加入字符串

在服务器上注册每台电脑，确保组内的节点 ID 各不相同。请提供该电脑访问服务器时所用的地址。

```sh
mash-server --server-config /absolute/path/to/server.toml \
  --add-node personal/laptop --api-url https://mash.example.net
```

命令会输出一个 `join` 值，开头为 `mash-join.`。重启服务器以允许新电脑连接，然后 [让这台电脑加入并检查其配置和记录](#join-and-review-each-computer)。服务器会将凭据的哈希值保存到配置文件旁的另一个文件中，例如 `server.toml.nodes`。
[更换或撤销加入凭据](#revoke-or-replace-a-computers-credential)

<span id="by-hand"></span>

### 手动设置

你也可以自行设置令牌和配置。

先将这台电脑的令牌存入一个环境变量，例如
`MASH_NODE_TOKEN`。Mash 只保存变量名，因此运行 Mash 的各个环境都必须设置该变量，包括命令行 shell、
[监视程序](synchronization.md)以及启动 Mash MCP 服务器的代理客户端。

接下来可使用向导，也可直接编辑配置。

**通过向导设置。** 运行 `mash init --no-index --no-agents` （在终端中执行）。如果首先看到检查配置的界面，请选择 **Customize setup** （在 v0.1.1-alpha 中称为 **Review and edit settings**
）。当向导询问在哪儿搜索时，选择 **Connect to an existing shared server**。输入服务器 URL、组 ID、这台电脑的节点 ID 和存放令牌的变量名。检查历史记录文件夹，然后选择 **Set up without indexing**。下一步会将历史记录发送到服务器。

**手动设置。** 编辑由 `mash init` 创建的节点配置文件。保留其中的提供程序设置和本地 Qdrant 设置。在现有的 `[index]` 表中设置
`destination`，然后添加一个 `[shared]` 表，内容如下。

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

`group_id` 和 `node_id` 必须与服务器上同一个 `[[nodes]]` 条目中的对应值一致，环境变量也必须存放该条目的令牌。在台式机上，使用
`node_id = "desktop"` 以及台式机的令牌。

搜索设置必须与服务器匹配。对于上述使用关键词搜索的服务器，每台电脑都使用 `[views.build] representations = "lexical-only"` 和
`[views.query] intent = "exact-mention"`，这两项设置由 `mash init` 在未安装模型时写入。要使用语义搜索，请向服务器管理员索取匹配的模型设置。

<span id="3-publish-and-check"></span>

## 3. 发布并检查

在 **每台** 电脑上运行以下命令。

```sh
mash index
mash status --shared
```

`mash index` 会更新本地索引，然后发布它保留的历史记录。配置的日期范围决定 Mash 本次读取哪些历史记录文件。本地索引中已有的较早对话仍可能被发布。
`mash status --shared` 会显示服务器收到的内容，结果位于 `shared_index` 下。如果看到 `shared_error`，请检查服务器地址、连接和令牌。

一台电脑发布对话后，其他电脑才能看到这些记录。如果发布失败，本地历史记录会保留。请修复命令报告的错误，再运行一次 `mash index` 。要持续发布新的对话，请在每台电脑上运行
[监视程序](synchronization.md) 。

<span id="search"></span>

## 搜索

运行 `mash dash` 打开网页面板，转到 **chats**，然后搜索你记得在另一台电脑上聊过的内容。结果会显示它来自哪台设备。若要只看某台电脑的结果，请用设备筛选器选中该电脑，然后选择 **Apply filters**。
[在网页面板中阅读对话](dashboard.md#find-and-read-a-chat)

在终端中运行以下命令。

```sh
mash search 'authentication'            # this computer and the shared index
mash search 'authentication' --shared   # the shared index only, including this computer's published chats
mash search 'authentication' --local    # this computer only; the server is not contacted
```

添加 `--node desktop` 可只查看来自该节点的对话。共享历史记录的查询由服务器处理，因此台式机可以处于睡眠状态。要用手机访问，请在已配置为可搜索此组的电脑上
[开启网页面板的远程访问](dashboard.md#open-it-from-your-phone-or-another-computer)。

<span id="remove-a-machine-or-let-a-rebuilt-one-publish-again"></span>

## 移除设备，或让重建索引的设备重新发布

服务器会将每个节点 ID 绑定到首次以该 ID 发布的本地索引，避免其他设备复用该名称并覆盖原有历史记录。出于同样的原因，如果某台设备删除并重建了索引，发布时也会被拒绝，并出现错误“shared node ownership or revision changed”。从 [备份](../upgrades.md#replace-an-index-that-cannot-be-upgraded) 恢复索引时使用
`--keep-identity` 即可保留该名称。否则，请在服务器上注销该节点。要永久移除一台电脑，请先停止该电脑的发布，再清除它已发布的历史记录，最后撤销其凭据。从电脑上卸载 Mash 后，服务器上的副本仍会保留。

停止 `mash-server` ，并让它使用的 Qdrant 继续运行。使用与该服务相同的服务器配置和环境运行以下命令。

```sh
# On the server machine, with mash-server stopped.
mash-server --server-config /absolute/path/to/server.toml --retire-node personal/laptop
```

**这会永久删除该设备发布的所有内容，包括** 服务器上的对话和记录、对应的搜索数据以及共享附件。命令会输出删除的对话和消息数量，然后退出，不会启动服务。其他设备及已注销设备自身的本地历史记录均不受影响。

如果你正在重建某台电脑的索引，请重新启动 `mash-server` ，然后在该电脑上运行
`mash index` ，以从当前索引发布。如果你要移除这台电脑，请先撤销其凭据，再重启服务器。

<span id="revoke-or-replace-a-computers-credential"></span>

### 撤销或更换某台电脑的凭据

在服务器上，找到 `[[nodes]]` 配置块，其中包含该电脑对应的 `group_id` 和
`node_id`。如果电脑是通过 `--add-node` 添加的，该配置块位于服务器配置文件旁的文件中，通常是 `server.toml.nodes`。如果电脑是手动添加的，该配置块位于 `server.toml`。删除该配置块并重启服务器即可撤销访问权限。如果这是配置中的最后一台电脑，请保持服务器停止。若要将已发布的文件保留在服务器上，请跳过注销命令。

要更换丢失或泄露的加入字符串，请删除旧的配置块，重新添加相同的组和节点，然后重启服务器。使用新的加入字符串从同一台电脑加入。如果该电脑保留了本地索引，仍可用原来的名称发布。

安装新版 Mash 前，请阅读
[升级和备份](../upgrades.md#upgrade-a-shared-index)。

<!-- mash-docs-source: d53981430b20b1e97415d553067b960e398181e3c769b414ed3965a9f7296bd1 -->
