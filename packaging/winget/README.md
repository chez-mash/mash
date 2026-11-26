# winget

Mash is two packages in the Windows Package Manager:

| Package | Installs | Release asset | Command once published |
|---|---|---|---|
| `MashHQ.Mash` | The desktop app, which carries its own copy of `mash` | `mash-desktop-windows-{x64,arm64}-setup.exe` (Tauri's NSIS installer, per user) | `winget install MashHQ.Mash` |
| `MashHQ.Mash.CLI` | The command-line programs `mash` and `mash-server`, on `PATH` | `mash-{x86_64,aarch64}-pc-windows-msvc.zip` (portable) | `winget install MashHQ.Mash.CLI` |

Each has its templates in a directory named for it. A release fills them from
its `SHA256SUMS` with `python3 packaging/render.py --winget TAG SHA256SUMS
OUTPUT` (the release workflow's `winget` job uploads the result as the
`winget-manifests` artifact, one directory per package), and the manifests are
then submitted to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs)
by a maintainer. Nothing is submitted automatically.

To submit a release, download the artifact and check it, then open one pull
request per package with [wingetcreate](https://github.com/microsoft/winget-create):

```powershell
winget validate --manifest .\winget-manifests\MashHQ.Mash
winget install --manifest .\winget-manifests\MashHQ.Mash   # needs LocalManifestFiles enabled
wingetcreate submit .\winget-manifests\MashHQ.Mash
```

and the same three for `.\winget-manifests\MashHQ.Mash.CLI`. In winget-pkgs
the app's manifests live in `manifests/m/MashHQ/Mash/VERSION/` and the
command line's in `manifests/m/MashHQ/Mash/CLI/VERSION/`: an identifier's
dots are its directories.

After a release, the `winget` job of `release-check.yml` does this on x64 and
ARM64 runners for both packages, so check its result for the tag before
submitting. For the app it validates, installs silently, compares what the
installer registered with the manifest, runs the app's engine, uninstalls and
looks for anything left, then installs once more the way a plain
`winget install` does (showing progress). For the command line it validates,
installs, runs both programs from a new shell's `PATH` and uninstalls.
Started by hand from a branch
(`gh workflow run release-check.yml --ref BRANCH -f tag=vX.Y.Z`), it tries
that branch's templates against the published release.

## The app's manifest

The installer writes its registration to the current user's
`Software\Microsoft\Windows\CurrentVersion\Uninstall\Mash` key. The manifest's
`ProductCode` is that key's name, and `AppsAndFeaturesEntries` repeats its
`DisplayName` and `Publisher`; winget matches an installed copy to the package
by them, for `winget list` and `winget upgrade`. The `winget` job fails when
they stop agreeing, for example after `productName` or the bundle's publisher
changes in `desktop/src-tauri/tauri.conf.json`.

`/S` installs silently and `/P` shows only the progress bar; neither starts
the app. The app does not yet put a `mash` command on `PATH`, and the app's
package does not depend on `MashHQ.Mash.CLI`: its description names that
package instead.

The first submission of a package creates it; later ones only add a version
directory.
