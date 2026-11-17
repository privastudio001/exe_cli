use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Commands understood by the addon. Serializes to
/// `{"type": "<snake_case name>", "params": {<variant fields>}}`.
///
/// `Option` fields that are `None` are omitted so the addon's own defaults apply.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum BlenderCommand {
    // --- Connection ---
    /// Liveness check; touches no bpy data. Returns `{"pong": true}`.
    Ping,
    /// Addon/protocol/Blender versions and supported commands.
    GetAddonInfo,

    // --- Observe ---
    /// Scene summary; lists only the first 10 objects.
    GetSceneInfo,
    /// Compact snapshot of the whole scene (up to 4000 objects).
    GetWorldStateSnapshot,
    /// Search objects, materials and collections by name.
    ListSceneItems {
        #[serde(skip_serializing_if = "Option::is_none")]
        query: Option<String>,
        /// Addon default 30, max 100.
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
    /// Details of a single object.
    GetObjectInfo { name: String },
    /// Save the 3D viewport as an image file (written by Blender).
    GetViewportScreenshot {
        filepath: String,
        /// Longest side in pixels; addon default 800.
        #[serde(skip_serializing_if = "Option::is_none")]
        max_size: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<ImageFormat>,
    },

    // --- Act ---
    /// Run Python with `bpy` in scope; returns captured stdout.
    ExecuteCode { code: String },
    /// Export the scene, the selection, or named objects to GLB/FBX.
    ExportScene {
        filepath: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<ExportFormat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        object_names: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        selection_only: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        apply_modifiers: Option<bool>,
    },

    // --- Docs ---
    /// Structured lookup of bpy types, properties, methods and operators.
    BpyApiLookup { query: String },
    /// Properties and socket layout of a node type.
    DescribeNodeType {
        bl_idname: String,
        /// e.g. `{"data_type": "RGBA"}`
        #[serde(skip_serializing_if = "Option::is_none")]
        property_overrides: Option<Value>,
    },

    // --- Telemetry ---
    GetTelemetryConsent,
    SetTelemetryConsent { consent: bool },
    /// Manual edits by the user since the last drain (needs consent).
    DrainHumanActivity,
}

#[derive(Debug, Clone, Copy, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Glb,
    Fbx,
}

/// The addon upper-cases this into Blender's `file_format`, so it must be
/// `png` or `jpeg` (`jpg` is not a Blender format).
#[derive(Debug, Clone, Copy, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Jpeg,
}

/// Raw reply from the MCP for Blender addon socket. The addon only ever sends
/// these two shapes; any other `status` fails to deserialize (protocol error).
#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum BlenderResponse {
    /// `{"status": "success", "result": ...}`.
    ///
    /// Not necessarily a success: many handlers report failure as
    /// `result: {"error": "...", "did_you_mean"?: [...], "attempts"?: [...]}`.
    Success {
        #[serde(default)]
        result: Value,
    },
    /// `{"status": "error", "message": "..."}`, from a raised exception or an
    /// unknown command. For `execute_code`, `message` is itself a JSON string
    /// `{"exception_type", "message", "traceback"}`.
    Error { message: String },
}
