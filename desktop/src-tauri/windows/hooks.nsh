; What the Windows installer does beyond Tauri's template: the app's `mash`
; command. The app itself does the work (src/cli/windows.rs), because it
; edits the user's PATH in the registry, and an NSIS string is too short to
; hold a long PATH safely.

; After the files are in place, on a first install and on every install over
; an earlier one: copy the app's mash.exe into bin\ and put that folder at the
; end of the user's PATH. The app does nothing when the person removed the
; command with Install Command-Line Tool…. A failure never fails the install;
; the same menu item puts the command in place later.
;
; Also register the silent uninstall command beside Tauri's UninstallString
; (written to the same key just before this hook), which `winget uninstall
; --silent` and other tools run; the uninstaller deletes the key, value and
; all.
!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "${UNINSTKEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --command-line-tool installed' $0
!macroend

; Before the files go: take the command and its PATH entry away. Not when the
; uninstaller runs as a step of an update, where the install that follows
; refreshes the command.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} $UpdateMode <> 1
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --command-line-tool uninstalling' $0
  ${EndIf}
!macroend
