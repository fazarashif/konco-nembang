# Installing Konco Nembang

Konco Nembang runs on **Windows 10 (version 1809 or later) and Windows 11**, 64-bit.

## 1. Download

1. Open the [latest release](https://github.com/fazarashif/konco-nembang/releases/latest).
2. Under **Assets**, download the file ending in **`_x64-setup.exe`**
   (for example `Konco.Nembang_0.1.0_x64-setup.exe`).

Only download the installer from this GitHub page. Other files in the release (`.sig`, `latest.json`) are used by the built-in updater and are not needed to install.

## 2. Get past the SmartScreen warning

The installer is not code-signed yet, so Windows does not recognise the publisher. You will probably see one of these:

**Browser download warning** (Edge/Chrome may say the file "isn't commonly downloaded"):

1. Open the downloads list.
2. Choose **Keep** (in Edge: click **⋯** next to the file → **Keep** → **Show more** → **Keep anyway**).

**"Windows protected your PC"** (blue SmartScreen window when you run the installer):

1. Click **More info**.
2. Check that the app name is `Konco.Nembang_…_x64-setup.exe` and the publisher is **Unknown publisher**.
3. Click **Run anyway**.

If you would rather verify the file first: right-click it → **Properties**. If there is an **Unblock** checkbox at the bottom of the **General** tab, you can tick it and click **OK**, then run the installer.

## 3. Install

1. The installer runs for your user account only, so it does not ask for administrator rights.
2. Click **Next**, choose a folder (the default is fine), then **Install**.
3. If Microsoft Edge WebView2 is missing (rare on Windows 11), the installer downloads it automatically. This needs an internet connection.
4. Leave **Run Konco Nembang** ticked and click **Finish**.

## 4. First run

- Konco Nembang lives in the **system tray** (bottom-right, next to the clock). If you don't see it, click the **^** arrow to show hidden icons. You can drag the icon onto the taskbar to keep it visible.
- **Left-click** the tray icon to open **Settings**. **Right-click** it for the menu: show/hide the overlay, move/resize it, search lyrics, quit.
- Play a song in **Spotify**, **YouTube Music** or **YouTube** in your browser. The lyrics appear at the bottom of your screen.
- To move or resize the overlay, press **Ctrl+Alt+L**, drag it, then press **Ctrl+Alt+L** again to lock it. Making the overlay bigger also makes the text bigger.
- To start Konco Nembang with Windows, turn on **Settings → General → Start with Windows**.

### Default shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Alt+L` | Move / resize the overlay (press again to lock) |
| `Ctrl+Alt+H` | Show / hide the overlay |
| `Ctrl+Alt+S` | Open Settings |
| `Ctrl+Alt+=` | Show lyrics earlier (this track) |
| `Ctrl+Alt+-` | Show lyrics later (this track) |
| `Ctrl+Alt+0` | Reset this track's offset |

You can change them in **Settings → Shortcuts**.

## Updating

Konco Nembang checks for updates a few seconds after it starts. When a new version is out, you'll see a short message on the overlay. Open **Settings → About → Install and restart**. Updates are signed, and the app refuses any update that isn't.

You can also download and run the newer installer from the releases page; it upgrades the existing install and keeps your settings.

## Uninstalling

**Settings (Windows) → Apps → Installed apps → Konco Nembang → Uninstall.**

Your settings, lyrics cache and offsets stay in `%APPDATA%\com.konconembang.desktop`. Delete that folder too if you want to remove everything.

## Troubleshooting

**The overlay doesn't show over my game.**
The game is probably in *exclusive fullscreen*. Switch it to **borderless** or **windowed** mode in the game's video settings. Konco Nembang does not hook into games, so it cannot draw over exclusive fullscreen.

**No lyrics appear.**
- Check that the song is playing (the tray icon tooltip shows the current track).
- Some songs are not in the [LRCLIB](https://lrclib.net) database yet. Open **Search lyrics…** from the tray to look for another version or import your own `.lrc` file.
- Lyrics are fetched from the internet the first time a song plays. After that they are cached and work offline.

**The lyrics are ahead of or behind the singing.**
Music videos often have an intro the album version doesn't. Press `Ctrl+Alt+=` or `Ctrl+Alt+-` until it matches; the offset is remembered for that track.

**A shortcut doesn't work.**
Another app may already use it. Settings shows a warning when that happens. Pick a different combination in **Settings → Shortcuts**.

**My antivirus flags the installer.**
Unsigned installers sometimes trigger false positives. Make sure you downloaded it from this repository's releases page. If you're unsure, you can build it yourself from source (see the README).
