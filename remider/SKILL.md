---
name: remider
description: Set, list and delete reminders using the bundled command-line tool (remider). Use whenever the user asks to be reminded of something, or wants to see or cancel their reminders.
enable: true
---

## When to use
- The user asks to be reminded of something at a time ("remind me to call Bob at 3pm", "remind me in 2 hours").
- The user wants to see their reminders, or cancel one.

## Command
./remider {subcommand} {args}

{subcommand} and {args} are real CLI syntax, not free-form:
- `set "<MESSAGE>" --at <TIME>` — add a reminder. Both are required. Quote the message.
  You may not know today's date, so don't write dates yourself — let the tool work them out. Use one of these (24-hour time; quote values that contain a space):
  - `HH:MM` — the next time it is HH:MM (today if still ahead, otherwise tomorrow). Use this for "at 11", "at 3pm".
  - `"tomorrow HH:MM"` — tomorrow at that time.
  - `"+<N>d HH:MM"` — N days from today at that time ("in 3 days at 2pm" → `"+3d 14:00"`).
  - `+<N>m`, `+<N>h`, `+<N>d` — that long from now ("in 30 minutes" → `+30m`; one unit only, `+90m` not `+1h30m`).
  - `"YYYY-MM-DD HH:MM"` or `YYYY-MM-DD` (09:00) — only when the user states the full date themselves.
  If the user gives no time at all, ask rather than guessing. A time that is already past is rejected.
- `list` — show all reminders as a markdown table (ID, Due, Message), sorted by due time.
- `delete <ID>` — remove a reminder by its numeric ID. Run `list` first if you don't know the ID.

## Output
- `set` prints the new ID and the resolved due time (with date); confirm both to the user.
- Errors go to stderr as `error: ...` with a non-zero exit code (bad time format, a due time in the past, unknown ID, a local time skipped by DST). The past-time error shows the current time, which you can use to correct the value. Fix the arguments or tell the user; don't retry unchanged.

## Examples
| Intent | Command |
| --- | --- |
| Remind at 11 o'clock | `./remider set "Sleep" --at 11:00` |
| Remind tomorrow at 9:30 | `./remider set "Call Bob" --at "tomorrow 09:30"` |
| Remind in 3 days at 2pm | `./remider set "Pay rent" --at "+3d 14:00"` |
| Remind in 2 hours | `./remider set "Take out the laundry" --at +2h` |
| User gave a full date | `./remider set "Dentist" --at "2026-12-01 10:00"` |
| Show all reminders | `./remider list` |
| Cancel reminder #3 | `./remider delete 3` |
