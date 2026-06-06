# Lucida Downloader Browser Agents

## Overview

The lucida-downloader tool supports automatic cf-clearance cookie fetching using a Node.js-based browser automation to bypass Cloudflare protection on lucida.to.

## System Requirements

### Required Software
- **Node.js**: Required for running the cf-clearance fetcher script
- **Google Chrome**: Required browser for Cloudflare challenge solving
- **npm packages**: Installed automatically in the project directory

### Installation

#### 1. Node.js (if not installed)
```bash
# macOS using Homebrew
brew install node

# Or download from:
# https://nodejs.org/
```

#### 2. Google Chrome
```bash
# macOS using Homebrew
brew install --cask google-chrome

# Or download directly from:
# https://google.com/chrome/
```

### NPM Dependencies
The following packages are installed in the project directory:
- `puppeteer`
- `puppeteer-extra`
- `puppeteer-extra-plugin-stealth`

These are installed automatically when you first run `npm install` in the project directory.

## Browser Automation Behavior

### Headful Mode
The cf-clearance fetcher uses **headful mode** (visible browser window) to reliably solve Cloudflare challenges:
- **Visible**: Opens a browser window to solve Cloudflare challenges
- **Interactive**: May require user intervention for CAPTCHA challenges
- **Automatic**: Navigates to lucida.to and waits for Cloudflare challenges to complete
- **Extraction**: Automatically extracts cf_clearance cookies when available

### Chrome Path
The tool uses Google Chrome at:
- `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`

## Cookie Management

### Automatic Features
- **Persistent Storage**: Cookies saved to `~/.config/lucida/config.toml`
- **Timestamp Tracking**: Records when cookies were fetched
- **Validity Status**: Marks cookies as valid/invalid
- **Auto-Refresh**: Invalidates cookies on `--fetch-cf` request

### Configuration File Structure
```toml
[cloudflare]
cf_clearance = "..."
user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36..."
cf_clearance_timestamp = 1703123456
cf_clearance_valid = true
```

## Usage Commands

### Basic Cookie Fetching
```bash
# Fetch fresh cf-clearance cookie
lucida --fetch-cf

# Force refresh existing cookie
lucida --refresh-cf

# Check current cookie status
lucida --config
```

### Download with Auto-Fetch
```bash
# Automatically fetch cookie then download
lucida --fetch-cf <qobuz-urls>

# Note: --fetch-cf must come before URLs
```

## Retry Logic

### Automatic Retry Strategy
- **Infinite Retries**: Continues until success or Ctrl+C
- **Progressive Backoff**: 5s → 10s → 15s → 30s (max)
- **Clear Progress**: Shows attempt count and errors
- **Graceful Cancellation**: Ctrl+C stops the process cleanly

### Error Handling
- **Browser Failures**: Retries with increasing delays
- **Network Issues**: Automatic retry with backoff
- **Cookie Not Found**: Continues retry until successful
- **Chrome Missing**: Clear error message with installation instructions

## Security Features

### User Agent Consistency
- **Realistic**: Uses authentic Chrome User-Agent string
- **Consistent**: Same User-Agent saved with cookies
- **Updated**: Matches Chrome version for compatibility

### Cloudflare Challenge Handling
- **Automatic Wait**: 15 seconds for challenge completion
- **Smart Detection**: Waits for page stability
- **Cookie Extraction**: Finds cf-clearance cookie automatically
- **Fallback**: Graceful error handling on failures

## Cross-Platform Notes

### macOS
- **Chrome Path**: `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`
- **Permissions**: May need to grant browser access (System Preferences)
- **Sandbox**: `--no-sandbox` required for headless operation

### Linux
- **Chrome Path**: `/usr/bin/google-chrome` or `/usr/bin/chromium`
- **Dependencies**: May require `libgtk-3-0` and related libraries
- **Server Compatibility**: Headless operation works on servers

### Windows
- **Chrome Path**: `C:\Program Files\Google\Chrome\Application\chrome.exe`
- **Permissions**: May need administrator access for automation
- **Path Detection**: Auto-detects from standard locations

## Troubleshooting

### Chrome Not Found
```bash
# Check if Chrome is installed
ls "/Applications/Google Chrome.app"

# Install Chrome with Homebrew
brew install --cask google-chrome
```

### Node.js Not Found
```bash
# Check if Node.js is installed
node --version

# Install Node.js with Homebrew
brew install node
```

### Permission Issues (macOS)
```bash
# Grant browser access if prompted
# System Preferences → Security & Privacy → Privacy → Full Disk Access
# Or: Accessibility permissions if required
```

### Cookie Fails to Fetch
```bash
# Ensure Google Chrome is installed
# Ensure Node.js is installed
# Try running the fetch script manually:
node /Users/twan/.cargo/bin/lucida-downloader/scripts/fetch-cf.js
```

### Cloudflare Errors
```bash
# Force refresh cookies
lucida --refresh-cf

# Check cookie validity
lucida --config

# Clear and retry
lucida --fetch-cf
```

## Performance Notes

### Resource Usage
- **Memory**: ~200-400MB for Chrome browser instance
- **CPU**: Moderate during challenge solving
- **Network**: Only requests to lucida.to
- **Disk**: Minimal (config file updates only)

### Speed
- **Startup**: 2-5 seconds to launch Chrome
- **Navigation**: 3-10 seconds to load lucida.to
- **Challenge**: 10-30 seconds for Cloudflare completion (may vary)
- **Total**: ~20-60 seconds per successful fetch

### Note
The browser window will be visible during the fetch process to reliably solve Cloudflare challenges.

## Development Notes

### Building with Browser Support
```bash
# Build with cf-clearance fetching feature
cargo build --features fetch-cf

# Install with feature enabled
cargo install --git https://github.com/jelni/lucida-downloader --features fetch-cf
```

### Feature Flags
- **fetch-cf**: Enables headless Chrome automation
- **default**: Minimal dependencies (no browser automation)
- **Optional**: Lightweight without Chrome dependency

## Security and Privacy

### Data Storage
- **Local Only**: All data stored locally in config file
- **No Telemetry**: No data sent to external services
- **Cookie Management**: User controls cookie lifecycle
- **Privacy**: Browser automation is fully local

### Browser Isolation
- **Temporary**: Browser instance exists only during fetch
- **Sandboxed**: Headless Chrome runs with restrictions
- **Clean**: No browsing history or cache persistence
- **Secure**: No user data between sessions

---

*This agent documentation covers the browser automation capabilities of lucida-downloader version 0.7.0+.*