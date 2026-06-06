# lucida-downloader

a multithreaded client for downloading music for free with
[lucida](https://lucida.to/).

<a href="https://brainmade.org/">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://brainmade.org/white-logo.svg">
    <img alt="Brainmade mark" src="https://brainmade.org/black-logo.svg">
  </picture>
</a>

## custom features (this fork)

This fork introduces several improvements to configuration, UI, and Cloudflare bypass capability:

1. **Persistent Configuration**: Saved at `~/.config/lucida/config.toml` (managed via `lucida --setup`).
2. **Cloudflare Bypass Automation**: Automatically fetch `cf_clearance` cookies via Puppeteer browser automation (using `lucida --fetch-cf`).
3. **Rich Progress UI**: Styled status messages and detailed download speed / track progress bars.
4. **Error Handling**: Friendly warnings for unsupported services (like Amazon Music) and automated prompts on Cloudflare 403 errors.

## installation

To install this custom fork with the browser automation features enabled:

1. **Install dependencies (required for Cloudflare bypass)**:
   - Make sure **Node.js** and **Google Chrome** are installed on your system.
   - Run `npm install` in this project directory to set up the browser automation packages.

2. **Compile and install**:
   ```bash
   cargo install --path . --features fetch-cf
   ```
   *Or from git:*
   ```bash
   cargo install --git https://github.com/Toan4th/lucida-downloader.git --features fetch-cf
   ```

## usage

- find the albums you want to download on https://play.qobuz.com/ (requires an
  account, but provides superior experience) or https://www.qobuz.com/shop

- run
  ```
  lucida <urls>
  ```

### cloudflare bypass setup

If you run into Cloudflare `403 Forbidden` errors:
```bash
lucida --fetch-cf <urls>
```
This will open a Google Chrome browser window, wait for you to solve the Cloudflare challenge, automatically extract the `cf_clearance` cookie and User-Agent, save them to your configuration file, and resume downloading.

### interactive configuration

To run the interactive first-time setup:
```bash
lucida --setup
```

To display your current configuration settings:
```bash
lucida --config
```

### cli options

```
Usage: lucida [OPTIONS] [URLS]...

Arguments:
  [URLS]...  URLs to download

Options:
  -f, --file <FILE>                    files to read URLs from
  -o, --output <OUTPUT>                custom path to download to
      --force                          overwrite already downloaded files
      --group-singles                  place all artist's singles in a "Singles" directory. their covers will not be downloaded
      --album-year <ALBUM_YEAR>        use "<album> (year)" or "(year) <album>" directory name [possible values: append, prepend]
      --flatten-directories            use "<artist> - <album>" format instead of nested "<artist>/<album>" directories
      --country <COUNTRY>              country to use accounts from [default: auto]
      --no-metadata                    disable metadata embedding by lucida
      --private                        hide tracks from recent downloads on lucida
      --album-workers <ALBUM_WORKERS>  amount of albums to download simultaneously [default: 1]
      --track-workers <TRACK_WORKERS>  amount of tracks to download simultaneously for each album [default: 4]
      --skip-tracks                    skip downloading tracks in the album
      --skip-cover                     skip downloading album cover
      --cf-clearance <CF_CLEARANCE>    set the cf_clearance cookie and the User-Agent header if Cloudflare is blocking your requests
      --user-agent <USER_AGENT>        the User-Agent header to use

  [Custom Fork Options]
      --config                         show current configuration settings
      --set-output <PATH>              update default output directory
      --set-user-agent <USER_AGENT>    update User-Agent header
      --update-cf <CF_CLEARANCE>       update Cloudflare clearance cookie
      --fetch-cf                       automatically fetch cf-clearance cookie using browser automation
      --refresh-cf                     force refresh cf-clearance even if existing is valid
      --setup                          run interactive first-time setup

  -h, --help                           Print help
```

> [!NOTE]  
> remember to support your favorite artists!
