# Developing Mash for Raycast

```sh
npm install
npm run dev     # opens the extension in Raycast
npm test        # the parsing and link tests
npm run lint
```

The extension uses npm, as Raycast's Store expects a `package-lock.json`.

## Before submitting to the Store

- **`author`**: James's Raycast username, `james_ko`. There is no `owner`:
  Raycast doesn't accept new public extensions under an organization yet, and
  its checks fail when a new public extension sets one.
- **Screenshots**: `metadata/mash-1.png` to `mash-5.png`, 2000 × 1250 PNG:
  recent chats, a search with matches, **Read Messages**, the action panel,
  and no matches. They were taken from made-up chats in a separate home
  folder (the **Mash Executable** preference pointed at a wrapper that sets
  `HOME`), never real history. Retake them the same way when the UI changes.

## The Store copy

The extension in [raycast/extensions](https://github.com/raycast/extensions)
leaves out what only this repository needs: the tests, `vitest` and its `test`
script, and this file. `packaging/raycast-store.sh` makes that copy, with its
own `package-lock.json`, ready to publish with `npx ray publish` from a clean
git checkout.
