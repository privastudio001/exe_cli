---
name: simple-web-fetch
description: Fetch a web page and return its content as clean readable text using the bundled command-line tool (simple-web-fetch), not curl or wget. Use whenever the user gives a URL or asks to read a web page.
enable: true
---

## When to use
When the user wants to read the content of a specific URL or web page. Returns clean text, not raw HTML.

## Command
simple-web-fetch.exe {args}

{args} are real CLI flags, not free-form syntax:
- `--url <URL>` is always required — the page to fetch.
- `--max <N>` is optional — only add it when the user asks to cap/limit the amount of text returned (e.g. "keep it short", "just the first part", "limit to 2000 characters"). Omit it otherwise; don't guess a number the user didn't ask for.

## Examples
| Intent | Command |
| --- | --- |
| Fetch page | `simple-web-fetch.exe --url https://example.com` |
| Limit output size | `simple-web-fetch.exe --url https://example.com/article --max 4000` |