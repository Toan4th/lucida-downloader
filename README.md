# lucida-downloader

a multithreaded client for downloading music for free with
[lucida](https://lucida.to/).

<a href="https://brainmade.org/">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://brainmade.org/white-logo.svg">
    <img alt="Brainmade mark" src="https://brainmade.org/black-logo.svg">
  </picture>
</a>

## interactive TUI (this fork)

If you run `lucida` without any command line arguments, it will boot into an **interactive Terminal User Interface (TUI)** featuring nested dropdown menus:

1. **Download Album(s)**: Prompt to paste one or multiple album URLs.
2. **Configure Download Location**:
   - **Local**: Paste a local directory path (autocleans escaped spaces, brackets, or quotes).
   - **Network Share**: Paste a path and auto-discover the network share SMB/AFP URL to enable **auto-mounting** on macOS.
   - **Custom**: Opens a native macOS Finder folder picker dialog in the foreground.
3. **Cloudflare Bypass Settings**: View clearance cookie validity status, trigger browser auto-fetch, or enter a cookie manually.
4. **UI & Performance Settings**: Toggle progress bars, colors, or output formatting.

---

## custom features

- **Automatic Network Share Mounting**: If your download location is set to a network share (like `/Volumes/Belmont/...`) and it is not currently mounted, the tool will automatically trigger macOS AppleScript to mount it before starting downloads.
- **Pasted Path Normalization**: Normalizes pasted paths with escaped spaces (e.g. `/path/to/my\ folder/`), surrounding quotes, or brackets.
- **Finder Integration**: Invokes a native Finder directory picker dialog on macOS using AppleScript.
- **Cloudflare Bypass Automation**: Automatically fetch `cf_clearance` cookies via Puppeteer browser automation (using `lucida --fetch-cf` or automatically triggered when receiving a 403 error).

---

## installation

To install this custom fork with the browser automation and TUI features enabled:

1. **Install dependencies (required for Cloudflare bypass)**:
   - Make sure **Node.js** and **Google Chrome** are installed on your system.
   - Run `npm install` in this project directory to set up the browser automation packages.

2. **Compile and install**:
   ```bash
   cargo install --path . --features fetch-cf
   ```

---

## usage

### TUI mode (recommended)
Simply run the executable without flags to access the interactive menu:
```bash
lucida
```

### CLI mode
You can also run directly via CLI to bypass the menu:
```bash
lucida "https://play.qobuz.com/album/..."
```

### config configuration directly
Update any setting in `config.toml` from the CLI:
```bash
lucida --set KEY=VALUE
```
*Supported Keys*: `output`, `mount`, `user_agent`, `cf_clearance`, `show_progress`, `colored_output`.

---

## cli options

```
Usage: lucida [OPTIONS] [URLS]...

Arguments:
  [URLS]...  URLs to download

Options:
  -f, --file <FILE>                    files to read URLs from
  -o, --output <OUTPUT>                custom path to download to
      --force                          overwrite already downloaded files
      --group-singles                  place all artist's singles in a "Singles" directory
      --album-year <ALBUM_YEAR>        use "<album> (year)" or "(year) <album>" directory name [possible values: append, prepend]
      --flatten-directories            use "<artist> - <album>" format instead of nested "<artist>/<album>" directories
      --country <COUNTRY>              country to use accounts from [default: auto]
      --no-metadata                    disable metadata embedding by lucida
      --private                        hide tracks from recent downloads on lucida
      --album-workers <ALBUM_WORKERS>  amount of albums to download simultaneously [default: 1]
      --track-workers <TRACK_WORKERS>  amount of tracks to download simultaneously for each album [default: 4]
      --skip-tracks                    skip downloading tracks in the album
      --skip-cover                     skip downloading album cover
      --cf-clearance <CF_CLEARANCE>    set the cf_clearance cookie and the User-Agent header
      --user-agent <USER_AGENT>        the User-Agent header to use

  [Custom Fork Options]
      --config                         show current configuration settings
      --set <KEY=VALUE>                update a configuration setting directly
      --fetch-cf                       automatically fetch cf-clearance cookie using browser automation
      --setup                          run interactive first-time setup

  -h, --help                           Print help
```

> [!NOTE]  
> remember to support your favorite artists!
