# Ask Mashy

Mashy is an optional assistant in the dashboard. Use it to find decisions
in past chats, check where work stopped, or explain a passage you’re reading.
It uses a model service and API key you provide. [Set up Mashy](#set-up-mashy).

## Ask your first question

Open the dashboard and select the Mashy circle in the bottom-right corner.
Type a question and choose **Send**. Try “What did I work on yesterday?” or
“What did we decide about the restart test?” On a transcript page, try
“Summarise this chat.”

“Yesterday” and other dates use your browser’s clock. For a period such as
last week, Mashy looks at messages from that period. If it reads only part
of a long chat, check what the answer covers and ask a narrower question
to go further.

Choose a suggested follow-up or type your own. Use **Stop** to end an answer
in progress. On a wide screen, drag the top bar to move the panel and
double-click the bar to reset its position. On a phone, the panel fills the
screen.

## Follow sources

Open a chat or memory named in the answer to check its source. A link to a
specific message opens the transcript at that message. Quotes show who said
them and when.

Project, agent, and machine links open their pages in the dashboard. If an
answer offers **See all**, it opens the full search for the same words.
A command in an answer, such as `mash index`, has a **Copy** button so you
can run it yourself.

Mashy also reads built-in Mash guides and reports things like whether search
is ready or when your other machines last synced. A guide it names opens
on this site. Mashy’s tools are read-only.

## Ask about a passage

Select text in a transcript and choose **Ask Mashy**. The selection appears
under **About this passage**. Try “Was this changed later?”

The passage goes with each question while it’s pinned. Use **Remove** beside
the passage to ask about something else.

## Set up Mashy

On the computer running the dashboard, follow
[the model service and API key instructions](configuration.md#the-dashboard-assistant).
They show how to add an `[assistant]` section to your Mash configuration and
set the key in the dashboard’s environment. Use an OpenAI-compatible model
service with tool calls and streaming.

Start or restart the dashboard to load the updated environment. To change
the model later, open **settings**, choose a model from that service, and
select **Save to file**. If the service doesn’t list its models, type the
name it uses.

## Turn Mashy off

In the dashboard’s **settings**, switch **Mashy** off and choose **Save to file**.
You can also set `enabled = false` in the
[`[assistant]` configuration](configuration.md#the-dashboard-assistant).

## What Mashy sends

Your question, recent turns from the conversation with Mashy, any passage
you’ve pinned, and the chat text and memories Mashy looks up go to your
configured model service. By default, Mash masks recognizable credentials in
lookup results. [When history can leave your machine](privacy.md#when-history-can-leave-your-machine).

Mash keeps your Mashy conversation in the current tab while you move between
dashboard pages. Reloading or closing the tab clears it. Choose **Start over**
to clear it in place. The conversation stays out of your indexed chat history.

## Troubleshoot Mashy

If the Mashy circle is missing, check that `[assistant]` is configured,
Mashy is enabled, and the API key is set in the environment of the dashboard
process. If Mashy can’t find a recent conversation, first see whether it
appears in the dashboard’s [chat search](dashboard.md#find-and-read-a-chat).

For more detail on an answer that fails or misses the point, start the
dashboard with `MASH_TRACE=1` in its environment, then ask again. Mashy’s
trace entries record which lookups ran, how much they returned, model timing,
token counts, and why the answer stopped. The trace leaves out the text of
questions, answers, and history.
