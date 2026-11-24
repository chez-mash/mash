# winget

The Mash CLI is published to the Windows Package Manager as `ChezMash.Mash`.
Each release fills these templates from its `SHA256SUMS` (the release
workflow's `winget` job uploads the result as the `winget-manifests`
artifact), and the manifests are then submitted to
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) by a
maintainer. Nothing is submitted automatically.

To submit a release, download the artifact and check it, then open the pull
request with [wingetcreate](https://github.com/microsoft/winget-create):

```powershell
winget validate --manifest .\winget-manifests
winget install --manifest .\winget-manifests   # needs LocalManifestFiles enabled
wingetcreate submit .\winget-manifests
```

The first submission creates the package; later ones only add a version
directory. The desktop app will be a separate package once its installers are
signed.
