/**
 * The desktop app download for the visitor's computer, as far as the browser
 * says, from the latest release's installers. Phones, tablets and computers
 * without an installer get none, and the button keeps pointing at the
 * downloads page.
 */
const latest = "https://github.com/chez-mash/mash/releases/latest/download/";

export interface Download {
  readonly system: string;
  readonly href: string;
}

interface Browser {
  readonly platform?: string;
  readonly userAgent?: string;
}

export function desktopDownload(browser: Browser): Download | undefined {
  const agent = (browser.userAgent ?? "").toLowerCase();
  const platform = (browser.platform ?? "").toLowerCase();
  // An iPad asks for the desktop site and calls itself a Mac; a touch screen gives it away.
  if (/iphone|ipad|android|mobile/.test(agent)) return undefined;
  const seen = `${platform} ${agent}`;
  if (seen.includes("win"))
    return { system: "Windows", href: `${latest}mash-desktop-windows-x64-setup.exe` };
  if (seen.includes("mac"))
    return { system: "macOS", href: `${latest}mash-desktop-macos-arm64.dmg` };
  // The Linux app is built for x86-64 only.
  if (seen.includes("linux") && /x86_64|amd64/.test(seen))
    return { system: "Linux", href: `${latest}mash-desktop-linux-x86_64.AppImage` };
  return undefined;
}
