mod client;
mod dto;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use serde_json::{Map, Value};

use client::BlenderClient;
use dto::{BlenderCommand, ExportFormat, ImageFormat};

#[derive(Parser)]
#[command(
    version,
    about = "Command-line client for the MCP for Blender addon",
    long_about = "Command-line client for the MCP for Blender addon.\n\n\
        Talks to the addon's socket server (default localhost:9876). \
        Results are printed to stdout as JSON; errors go to stderr with a \
        non-zero exit code."
)]
struct Cli {
    /// Host the Blender addon is listening on
    #[arg(long, global = true, default_value = "localhost")]
    host: String,

    /// Port the Blender addon is listening on
    #[arg(short, long, global = true, default_value_t = 9876)]
    port: u16,

    /// Connect/response timeout in seconds (raise it for slow scripts/exports)
    #[arg(long, global = true, default_value_t = 5.0)]
    timeout: f64,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // --- Connection ---
    /// Check that Blender is reachable and responding
    Ping,

    /// Show addon, protocol and Blender versions and supported commands
    Info,

    // --- Observe ---
    /// Snapshot of the whole scene: objects (transform, bbox, parent, modifiers,
    /// animation), selection, frame range, camera, lights. Rotation is in radians.
    #[command(after_help = "Example:\n  blender-cli scene")]
    Scene {
        /// Short summary only (name, type, location of the first 10 objects)
        #[arg(long)]
        brief: bool,
    },

    /// Search objects, materials and collections by name
    #[command(after_help = "Example:\n  blender-cli find cube -n 10")]
    Find {
        /// Text to match (case-insensitive); omit to list everything
        query: Option<String>,

        /// Maximum number of results (addon default 30, max 100)
        #[arg(short = 'n', long)]
        limit: Option<u32>,
    },

    /// Details of one object: transform, visibility, materials, bbox, mesh stats
    #[command(after_help = "Example:\n  blender-cli object Cube")]
    Object {
        /// Exact object name
        name: String,
    },

    /// Save the 3D viewport as an image and print where it was written
    #[command(after_help = "Renders the viewport as it is shown now (view angle, shading). \
        Saved next to the executable as screenshot-YYYYMMDD-HHMMSS.png (local time); \
        open the printed filepath to look at it.\n\n\
        Examples:\n  \
        blender-cli screenshot\n  \
        blender-cli screenshot --max-size 1200")]
    Screenshot {
        /// Longest side in pixels (addon default 800)
        #[arg(long)]
        max_size: Option<u32>,

        /// Image format (addon default: png)
        #[arg(short, long, value_enum)]
        format: Option<ImageFormat>,
    },

    // --- Act ---
    /// Run Python (bpy) code inside Blender
    #[command(
        group = clap::ArgGroup::new("src"),
        after_help = "Returns {\"executed\": true, \"result\": \"<print() output>\"}; \
            only `bpy` is pre-imported.\n\n\
            Examples:\n  \
            blender-cli exec -c 'import bpy; print(len(bpy.data.objects))'\n  \
            blender-cli exec -c \"for o in bpy.data.objects:\" -c \"    print(o.name)\"\n  \
            blender-cli exec script.py\n  \
            cat script.py | blender-cli exec"
    )]
    Exec {
        /// Python file to run ("-" or omitted = read from stdin)
        #[arg(group = "src")]
        file: Option<PathBuf>,

        /// Python code to run, given inline; repeat -c for multiple lines
        /// (each -c is one line, indentation kept)
        #[arg(short, long, group = "src")]
        code: Vec<String>,
    },

    /// Export the scene, the selection, or named objects to GLB/FBX
    #[command(after_help = "Example:\n  blender-cli export out.glb -o Cube -o Sphere")]
    Export {
        /// Output file (made absolute before sending, since Blender writes it)
        path: PathBuf,

        /// File format (addon default: glb)
        #[arg(short, long, value_enum)]
        format: Option<ExportFormat>,

        /// Object to export, with its children (repeatable)
        #[arg(short = 'o', long = "object")]
        objects: Vec<String>,

        /// Export only the current selection
        #[arg(long)]
        selected: bool,

        /// Keep modifiers unapplied (preserves rigs and shape keys)
        #[arg(long)]
        no_modifiers: bool,
    },

    // --- Docs ---
    /// Look up bpy types, properties, methods and operators
    #[command(after_help = "Examples:\n  \
        blender-cli api bpy.ops.mesh.primitive_cube_add\n  \
        blender-cli api ShaderNodeTexSky.sky_type")]
    Api {
        /// Type, Type.property, Type.method or bpy.ops.<category>.<op>
        query: String,
    },

    /// Describe a node type's properties and input/output sockets
    #[command(after_help = "Example:\n  blender-cli node ShaderNodeMix --set data_type=RGBA")]
    Node {
        /// Node bl_idname, e.g. ShaderNodeBsdfPrincipled
        bl_idname: String,

        /// Set a property before describing, as KEY=VALUE (repeatable)
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,
    },

    // --- Generic ---
    /// Call any addon command by name (for commands without a subcommand)
    #[command(after_help = "Example:\n  \
        blender-cli call get_polyhaven_status\n  \
        blender-cli call search_polyhaven_assets --params '{\"asset_type\":\"hdris\"}'")]
    Call {
        /// Addon command type, e.g. get_polyhaven_status
        command_type: String,

        /// Parameters as a JSON object
        #[arg(long, default_value = "{}")]
        params: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let request = build_request(cli.command)?;

    let timeout = Duration::from_secs_f64(cli.timeout);
    let mut client = BlenderClient::connect(&cli.host, cli.port, timeout)?;
    let result = client.send(&request)?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

/// Turn a subcommand into the `{"type": ..., "params": ...}` request to send.
fn build_request(command: Command) -> Result<Value> {
    let command = match command {
        Command::Call { command_type, params } => {
            let params: Value =
                serde_json::from_str(&params).context("--params must be a JSON object")?;
            return Ok(serde_json::json!({ "type": command_type, "params": params }));
        }

        Command::Ping => BlenderCommand::Ping,
        Command::Exec { file, code } => BlenderCommand::ExecuteCode { code: read_code(file, code)? },
        Command::Info => BlenderCommand::GetAddonInfo,
        Command::Scene { brief: true } => BlenderCommand::GetSceneInfo,
        Command::Scene { brief: false } => BlenderCommand::GetWorldStateSnapshot,
        Command::Find { query, limit } => BlenderCommand::ListSceneItems { query, limit },
        Command::Object { name } => BlenderCommand::GetObjectInfo { name },
        Command::Screenshot { max_size, format } => {
            // Always next to the executable (an absolute path, since Blender
            // writes the file), with a timestamp so shots don't overwrite each other.
            let ext = if matches!(format, Some(ImageFormat::Jpeg)) { "jpg" } else { "png" };
            let exe = std::env::current_exe().context("cannot locate blender-cli executable")?;
            let dir = exe.parent().context("executable has no parent directory")?;
            let time = chrono::Local::now().format("%Y%m%d-%H%M%S");
            let path = dir.join(format!("screenshot-{time}.{ext}"));
            BlenderCommand::GetViewportScreenshot {
                filepath: path.to_string_lossy().into_owned(),
                max_size,
                format,
            }
        }
        Command::Export { path, format, objects, selected, no_modifiers } => {
            let path = std::path::absolute(&path)
                .with_context(|| format!("invalid path {}", path.display()))?;
            BlenderCommand::ExportScene {
                filepath: path.to_string_lossy().into_owned(),
                format,
                object_names: (!objects.is_empty()).then_some(objects),
                selection_only: selected.then_some(true),
                apply_modifiers: no_modifiers.then_some(false),
            }
        }
        Command::Api { query } => BlenderCommand::BpyApiLookup { query },
        Command::Node { bl_idname, set } => BlenderCommand::DescribeNodeType {
            bl_idname,
            property_overrides: if set.is_empty() { None } else { Some(parse_kv(&set)?) },
        },
    };
    Ok(serde_json::to_value(command)?)
}

/// Parse KEY=VALUE pairs into a JSON object. Values that parse as JSON
/// (numbers, true/false, arrays, quoted strings) keep their type; anything
/// else is taken as a plain string, so `data_type=RGBA` works unquoted.
fn parse_kv(pairs: &[String]) -> Result<Value> {
    let mut map = Map::new();
    for pair in pairs {
        let (key, raw) = pair
            .split_once('=')
            .ok_or_else(|| anyhow!("expected KEY=VALUE, got '{pair}'"))?;
        let value = serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()));
        map.insert(key.to_string(), value);
    }
    Ok(Value::Object(map))
}

/// Code from -c (repeated -c = one line each), a file, or stdin (no file or "-").
fn read_code(file: Option<PathBuf>, code: Vec<String>) -> Result<String> {
    Ok(match (code, file) {
        (code, _) if !code.is_empty() => code.join("\n"),
        (_, Some(path)) if path.as_os_str() != "-" => fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?,
        _ => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).context("cannot read stdin")?;
            buf
        }
    })
}
