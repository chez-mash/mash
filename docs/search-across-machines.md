# Search across your machines

With a shared index, you can find a chat from your workstation while working
on your laptop. Each computer keeps an index of its own chats and publishes
it to a Mash server you run.

- To search histories from several computers, join them to the same shared
  group. [Set up a shared index](shared-index.md#the-quick-way).
- To browse from a phone, pick a computer with the history you want.
  [Open that computer’s dashboard remotely](dashboard-access.md). It shows
  the history that computer can access, including shared history if configured.

## Try the demo

[Try the sample search](https://getmash.io/demo/#/search?q=invoice+exports+queue)
and open **Choose a queue for invoice exports**, a conversation from the
`workstation` machine. The result shows the machine beside the agent and opens
the original message in the conversation.

## Search in the dashboard

1. On a computer joined to your shared group, run `mash dash` and open the
   address it prints. Choose **chats** and search for a file, error, or phrase
   you remember.
2. To narrow the results, open the **machine** filter, select the machines
   you want, and choose **Apply filters**. The filter may show a machine name
   instead of **machine** if you already selected one.
3. Check the machine shown on the result, then open it to read the surrounding
   conversation. [Read a chat in the dashboard](dashboard.md#find-and-read-a-chat).

## Search from the terminal

Replace `authentication` with a word or phrase from your chats and
`workstation` with the name of a machine in your shared group.

```sh
mash search 'authentication'
mash search 'authentication' --shared
mash search 'authentication' --local
mash search 'authentication' --node workstation
```

| Choice | Where Mash searches |
| --- | --- |
| Default | This computer’s local index and published chats from the shared group. When the local index is available, Mash uses it for this computer’s chats. |
| `--shared` | Only the shared index, including any history this computer published |
| `--local` | Only this computer’s index, without contacting the server |
| `--node workstation` | Only results from the named machine |

The shared server answers requests for published history. You can search a
machine’s published chats while that machine is asleep.
If a default search returns `shared_error` alongside local matches, check the
shared connection before relying on those results for another computer.

## Open a result from another machine

In the dashboard, select the result to jump to its message. In the terminal,
copy `lookup.view.id` from the matching message and `lookup.view.shared` from
the same lookup. Use them as the message ID and the publishing machine ID
in the command below.

```sh
mash view MESSAGE_ID --shared NODE_ID --context 3
```

The command includes up to three messages on each side of the match.
[Read the original messages](search.md#read-the-original-messages).

## If a chat is missing

Check the computer where the agent saved the chat. It needs to index and
publish the chat before other machines can find it. Use `mash status --shared`
there to check what the server received. If publication is pending, follow
[the steps to refresh across machines](synchronization.md#across-machines).

After publishing finishes, choose **Check for updates** in the dashboard or reload
the search page if the button is missing. If the chat still doesn’t appear,
[check the chat list, search filters, and history range](find-missing-chat.md).
