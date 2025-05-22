# A sqruff extension for Zed

This extension provides language support for [sqruff].

## Installation

Open the Zed extensions page, search for `sqruff`, and install the extension.

## Configuration

By default, Zed uses Prettier as the SQL formatter.

To fix this, use sqruff’s LSP formatter by adding to your `settings.json`:

```json
  "languages": {
    "SQL": {
      "language_servers": ["sqruff"],
      "formatter": { "language_server": { "name": "sqruff" } }
    }
  }
```
