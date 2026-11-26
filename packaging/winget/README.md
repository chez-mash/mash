# winget

The Mash CLI is published to the Windows Package Manager as `MashHQ.Mash`.
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

After a release, the `winget` job of `release-check.yml` runs those first two
commands on x64 and ARM64 runners, runs both programs and uninstalls them, so
check its result for the tag before submitting. Started by hand from a branch
(`gh workflow run release-check.yml --ref BRANCH -f tag=vX.Y.Z`), it tries
that branch's templates against the published release.

The first submission creates the package; later ones only add a version
directory. The desktop app will be a separate package once its installers are
signed.
