---
name: simple-duckduckgo-search
description: Search the web with DuckDuckGo and return titles, URLs and snippets using the bundled command-line tool (simple-duckduckgo-search). Use whenever the user asks about recent events or something you don't know.
enable: false
---

## When to use
When the user needs up-to-date information, or asks about something you are not sure about. Returns a short list of results; to read a full page, pass its URL to a web fetch tool.

## Command
simple-duckduckgo-search.exe {args}

{args} are real CLI flags, not free-form syntax:
- `--query "<QUERY>"` is always required — what to search for. Quote it.
- `--max <N>` is optional — number of results (default 3). Only add it when the user wants more or fewer results.

## Examples
| Intent | Command |
| --- | --- |
| Search | `simple-duckduckgo-search.exe --query "rust programming language"` |
| More results | `simple-duckduckgo-search.exe --query "tokyo weather this week" --max 8` |
