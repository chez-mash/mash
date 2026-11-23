# Open the dashboard on another device

Use the Mash dashboard from your phone without installing Mash on it. The
computer running Mash serves the dashboard over HTTPS through your private
Tailscale network, called a tailnet. Your tailnet's access rules still apply.
[Tailscale Serve](https://tailscale.com/docs/features/tailscale-serve)

## Open it from your phone

1. On the Linux or macOS computer with your indexed history, connect
   [Tailscale](https://tailscale.com/download) and enable HTTPS for your tailnet.
   Linux needs a systemd user session. If Mash is still running from an earlier
   `mash dash` command on port 8790, close it first or choose another port.
   [Enable HTTPS in Tailscale](https://tailscale.com/docs/how-to/set-up-https-certificates)
2. On that computer, run this in a terminal.

   ```sh
   mash dash remote
   ```

   Mash starts a user service, sets up the private HTTPS address, and prints
   the address and a one-time code. In an interactive terminal, it also shows
   a QR code. Look for `dashboard` and `code` in the output. You can close the
   terminal and leave the service running. Keep the host awake and connected
   when you need access.
3. On the phone, connect Tailscale to the same tailnet. Scan the QR code with
   the camera, or open the HTTPS address and enter the one-time code. On
   another computer, open the address in a browser while connected to that
   tailnet. The browser stays signed in for 30 days.

After signing in, open **chats** and search for a word you remember. You can
read the original conversation and open saved memories from that computer's
index. If the host can access a shared index, you can also read the history
it can access there. [Find and read a chat](dashboard.md#find-and-read-a-chat)

## Manage device access

Run these commands on the computer hosting the dashboard.

| To | Run |
| --- | --- |
| Issue a one-time code for another device | `mash dash remote --code` |
| Show the sign-in QR code again | `mash dash remote --qr` |
| Choose another port during setup | `mash dash remote --port 8791` |
| Stop serving and remove the service | `mash dash remote --remove` |

Keep the same port when you request a code, display the QR code, or remove
the service. For example, use `mash dash remote --code --port 8791` if you
set it up on 8791. Removing the service keeps your history and the saved
dashboard sign-in token. To move an existing service to another port, remove
it using the old port before you set up the new one.

A one-time code works once within ten minutes. Requesting a new one replaces
the previous code. The QR code contains the long-lived sign-in token, so
treat a photo of the code like a password. The token is in the URL fragment,
which the browser keeps out of the page request. The dashboard clears the
fragment from the browser address before it signs in.

## Serve a shared index

If the dashboard host already has a [shared-index connection](shared-index.md),
the dashboard can use its node token to read chats from other machines.
If you joined with `mash init --join`, Mash can read the saved token from
`~/.mash/node.env`. In v0.1.1-alpha and earlier, setup can report local-only
access even when that token works. Open a chat from another computer to check.
[Search across your machines](search-across-machines.md)

If you configured sharing by hand, set the node token variable in the shell
where you run `mash dash remote`. Mash saves the token for the service in
`~/.mash/dashboard.env`, readable only by your user. On macOS, the LaunchAgent
also stores the service environment in
`~/Library/LaunchAgents/dev.mash.dash.plist` and writes its output to
`~/.mash/dashboard.log`. On Linux, use `journalctl --user -u mash-dash.service`
to read the service log.

To host the dashboard on a machine with a shared-index connection and node
token, but no local history, run this on that host.

```sh
mash dash remote --shared-only
```

Chats, search and memories come from the shared index.

## Allow access from outside your tailnet

Only use public access if the device needs to connect without Tailscale.
This makes the sign-in page reachable from the public internet. Your
dashboard token and one-time codes protect the full history the dashboard can
read.

Enable Funnel for the host in your tailnet, then run this on the host.
Funnel allows ports 443, 8443 and 10000.
[Tailscale Funnel requirements](https://tailscale.com/docs/features/tailscale-funnel)

```sh
mash dash remote --public --port 8443
```

Open the printed address and enter the code, or scan the QR code. Run
`mash dash remote --remove --port 8443` on the same host to stop serving it.

## Set it up by hand

Use this approach on Windows or when you manage your own HTTPS reverse
proxy, such as Caddy. Keep Mash listening on loopback and put the HTTPS
proxy in front of it.

1. On the host, forward an HTTPS address to `http://127.0.0.1:8790`. With
   Tailscale, run `tailscale serve --bg 8790` and use the HTTPS address it
   prints. [Tailscale Serve command](https://tailscale.com/docs/reference/tailscale-cli/serve)
2. In the shell that will run Mash, set a private token of at least 32
   printable ASCII characters without spaces as the `MASH_DASH_TOKEN`
   environment variable. For example, generate it with a password manager.
3. On the host, start Mash with the name from the HTTPS address. Replace
   `laptop.your-tailnet.ts.net` with your own hostname.

   ```sh
   mash dash --allow-host laptop.your-tailnet.ts.net --access-token-env MASH_DASH_TOKEN
   ```

   Include the port in `--allow-host` if the browser address shows one.
   Repeat `--allow-host` to allow more than one name.
4. On the phone or other computer, open the HTTPS address and enter the
   token. Leave the host's Mash terminal running. Restarting Mash with a
   new token signs every browser out.

## Keep access private

The default `mash dash` listens on loopback without a sign-in. If you run
`mash dash --listen` with an address other devices can reach, use
`--access-token-env` to require sign-in. A reachable listener without a token
serves your history to anyone who can connect and prints a warning at startup.
`--allow-host` requires a token.

Ten wrong sign-in attempts during a minute pause sign-in for everyone until
that window ends. Prefer a private network. If an HTTPS proxy or tunnel
provider terminates encryption on your behalf, it can read the traffic it
forwards.
