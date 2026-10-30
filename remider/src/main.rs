use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use chrono::{Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const DATA_FILE: &str = "remider.json";
const TIME_FMT: &str = "%Y-%m-%d %H:%M";

#[derive(Parser)]
#[command(name = "remider", version, about = "Simple reminder CLI; data is stored in remider.json next to the executable")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Add a reminder
    Set {
        /// Reminder message
        message: String,
        /// Due time: "HH:MM" (next occurrence), "tomorrow HH:MM", "+Nd HH:MM", relative "+30m" "+2h" "+1d",
        /// "YYYY-MM-DD HH:MM", or "YYYY-MM-DD" (09:00)
        #[arg(short, long)]
        at: String,
    },
    /// List all reminders
    List,
    /// Delete a reminder
    Delete {
        /// Reminder ID
        id: u32,
    },
    /// List reminders that are due
    Due,
}

#[derive(Serialize, Deserialize)]
struct Reminder {
    id: u32,
    message: String,
    /// Local time, formatted with TIME_FMT
    due: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Store {
    next_id: u32,
    reminders: Vec<Reminder>,
}

fn data_path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate executable: {e}"))?;
    let dir = exe.parent().ok_or("cannot locate executable directory")?;
    Ok(dir.join(DATA_FILE))
}

fn load(path: &PathBuf) -> Result<Store, String> {
    if !path.exists() {
        return Ok(Store { next_id: 1, reminders: Vec::new() });
    }
    let text = fs::read_to_string(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("failed to parse {}: {e}", path.display()))
}

fn save(path: &PathBuf, store: &Store) -> Result<(), String> {
    let text = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn parse_due(input: &str, now: NaiveDateTime) -> Result<NaiveDateTime, String> {
    let input = input.trim();
    // "HH:MM": today, or tomorrow if that time has already passed
    if let Ok(t) = NaiveTime::parse_from_str(input, "%H:%M") {
        let dt = now.date().and_time(t);
        return Ok(if dt <= now { dt + Duration::days(1) } else { dt });
    }
    // "today HH:MM", "tomorrow HH:MM", "+Nd HH:MM": N days from today at HH:MM
    if let Some((day, time)) = input.split_once(' ') {
        if let Ok(t) = NaiveTime::parse_from_str(time.trim(), "%H:%M") {
            let days = match day {
                "today" => Some(0),
                "tomorrow" => Some(1),
                _ => day.strip_prefix('+').and_then(|d| d.strip_suffix('d')).and_then(|n| n.parse::<i64>().ok()),
            };
            if let Some(n) = days {
                return Ok((now.date() + Duration::days(n)).and_time(t));
            }
        }
    }
    if let Some(rel) = input.strip_prefix('+') {
        let (num, unit) = rel.split_at(rel.len().saturating_sub(1));
        let n: i64 = num.parse().map_err(|_| format!("invalid relative time: {input}"))?;
        let delta = match unit {
            "m" => Duration::minutes(n),
            "h" => Duration::hours(n),
            "d" => Duration::days(n),
            _ => return Err(format!("relative time unit must be m/h/d: {input}")),
        };
        return Ok(now + delta);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(input, TIME_FMT) {
        return Ok(dt);
    }
    if let Ok(d) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return Ok(d.and_hms_opt(9, 0, 0).unwrap());
    }
    Err(format!("cannot parse time: {input} (e.g. \"15:00\", \"tomorrow 09:30\", \"+3d 14:00\", \"+2h\")"))
}

fn parse_stored(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, TIME_FMT).ok()
}

fn print_table(items: &[&Reminder]) {
    println!("| ID | Due | Message |");
    println!("|---:|---|---|");
    for r in items {
        // Escape pipes so they don't break the markdown table
        let message = r.message.replace('|', "\\|");
        println!("| {} | {} | {} |", r.id, r.due, message);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let path = data_path()?;
    let mut store = load(&path)?;
    let now = Local::now().naive_local();

    match cli.command {
        Command::Set { message, at } => {
            let due = parse_due(&at, now)?;
            // Make sure the local time exists (e.g. not skipped by a DST transition)
            if Local.from_local_datetime(&due).earliest().is_none() {
                return Err(format!("local time does not exist: {}", due.format(TIME_FMT)));
            }
            if due <= now {
                return Err(format!(
                    "due time is in the past: {} (now {})",
                    due.format(TIME_FMT),
                    now.format(TIME_FMT)
                ));
            }
            let id = store.next_id.max(1);
            store.next_id = id + 1;
            store.reminders.push(Reminder {
                id,
                message,
                due: due.format(TIME_FMT).to_string(),
                created_at: now.format(TIME_FMT).to_string(),
            });
            save(&path, &store)?;
            println!("Added reminder #{id}, due {}", due.format(TIME_FMT));
        }
        Command::List => {
            if store.reminders.is_empty() {
                println!("No reminders");
                return Ok(());
            }
            let mut items: Vec<&Reminder> = store.reminders.iter().collect();
            items.sort_by(|a, b| a.due.cmp(&b.due));
            print_table(&items);
        }
        Command::Delete { id } => {
            let before = store.reminders.len();
            store.reminders.retain(|r| r.id != id);
            if store.reminders.len() == before {
                return Err(format!("reminder #{id} not found"));
            }
            save(&path, &store)?;
            println!("Deleted reminder #{id}");
        }
        Command::Due => {
            let (mut items, rest): (Vec<Reminder>, Vec<Reminder>) = store
                .reminders
                .into_iter()
                .partition(|r| parse_stored(&r.due).is_some_and(|d| d <= now));
            if items.is_empty() {
                return Ok(());
            }
            // Due reminders are removed once they have been shown
            store.reminders = rest;
            save(&path, &store)?;
            items.sort_by(|a, b| a.due.cmp(&b.due));
            // Phrased for an LLM to relay to the user
            let noun = if items.len() == 1 { "reminder" } else { "reminders" };
            println!("Remind the user of {} due {noun}.", items.len());
            println!();
            for r in items {
                println!("- **{}** (due {}, #{})", r.message, r.due, r.id);
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
