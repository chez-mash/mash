// Runs in the window before each page loads, the dashboard included.

// The dashboard shares mash:// links instead of its private address while
// this is set (dashboard/src/lib/shareLink.ts).
window.__MASH_DESKTOP__ = true;
