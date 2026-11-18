---
name: blender-cli
description: Inspect and control a running Blender scene with the blender-cli command-line tool — list objects, read an object's transform/materials/bbox, run bpy Python, export GLB/FBX, and look up bpy API or node sockets. Use whenever the user asks to see, create, modify, or export anything "in Blender", "in my scene", or in a 3D model/object. If the command fails with "cannot connect to Blender", the reply MUST tell the user to open Blender, press N in the 3D Viewport, open the "MCP for Blender" tab and click Connect (default localhost:9876).
enable: false
---

## When to use
blender-cli talks to the MCP for Blender addon running inside the user's open Blender.

Always query the live scene — never guess object names, locations or counts from memory, and never invent a result if a command comes back empty or fails; say so plainly instead.

## Command
./blender-cli {args}

{args} is one subcommand — apply these whenever the user's request matches:
| {args} | What it does |
| --- | --- |
| `ping` | Check that Blender is reachable. |
| `info` | Addon, protocol and Blender versions, and the commands the addon supports. |
| `scene` | Every object in the scene (name, type, transform, bbox, parent, modifiers, animation), plus selection, frame range, camera and lights. `rotation` is in radians. |
| `scene --brief` | Short summary: name, type and location of the first 10 objects only. |
| `find [text] [-n N]` | Search objects, materials and collections by partial name (case-insensitive). No text lists everything. `-n` caps results (default 30, max 100). Use it to get the exact name. |
| `object <name>` | One object by exact name: transform, visibility, materials, bbox, mesh stats. |
| `screenshot` | Save the 3D viewport as it is shown now (view angle, shading) to an image and print its `filepath`. Then open that file to see the result. Saved next to the blender-cli executable as `screenshot-YYYYMMDD-HHMMSS.png` (local time), so earlier shots are kept; `--max-size <px>` (default 800), `-f jpeg`. Use it to check how a model looks, not its numbers. |
| `exec -c '<python>'` | Run Python inside Blender. Only `bpy` is pre-imported; only `print()` output comes back, as `{"executed": true, "result": "<stdout>"}`. |
| `exec <file>` | Same, reading the code from a file. |
| `exec` (stdin) | Same, reading the code from stdin, e.g. a heredoc `exec <<'EOF' … EOF` for multi-line code. |
| `export <path>` | Export to GLB (default). `-f fbx` for FBX, `-o <name>` per object with its children (repeatable), `--selected` for the selection only, `--no-modifiers` to keep modifiers unapplied. |
| `api <query>` | Look up bpy: a type, `Type.property`, `Type.method` or `bpy.ops.<category>.<op>`. Use it before using an operator argument, property or enum value you are not sure of. |
| `node <bl_idname>` | A node type's properties and input/output sockets. `--set key=value` (repeatable) for mode-dependent nodes, e.g. `node ShaderNodeMix --set data_type=RGBA`. |
| `call <type> [--params '<json>']` | Send any other addon command by name with JSON params. |

Global options (any command): `--timeout <seconds>` (default 5; raise it for big scripts, exports, imports), `--host` (default localhost), `-p <port>` (default 9876).

Results print to stdout as JSON. Errors print to stderr with a non-zero exit code; a Python error shows its traceback.

Rules for `exec`:
- End the code with `print(json.dumps({...}))` to report what was done.
- Blender renames duplicates (`Cube` → `Cube.001`), so always print the real name.
- Never guess operator arguments, property names or enum values; check with `api` first.

## Examples
| Intent | Command |
| --- | --- |
| Is Blender connected | `./blender-cli ping` |
| What is in the scene | `./blender-cli scene` |
| Find objects named like "chair" | `./blender-cli find chair -n 5` |
| Details of the Cube | `./blender-cli object Cube` |
| See what the viewport shows | `./blender-cli screenshot` |
| Add a cube at (2, 0, 0) | `./blender-cli exec -c 'import bpy, json; bpy.ops.mesh.primitive_cube_add(location=(2,0,0)); print(json.dumps({"name": bpy.context.active_object.name}))'` |
| Move Cube up 1 unit | `./blender-cli exec -c 'import bpy; o = bpy.data.objects["Cube"]; o.location.z += 1; print(tuple(o.location))'` |
| Delete Cube | `./blender-cli exec -c 'import bpy; bpy.data.objects.remove(bpy.data.objects["Cube"]); print("deleted")'` |
| Arguments of an operator | `./blender-cli api bpy.ops.mesh.primitive_uv_sphere_add` |
| Sockets of the Principled BSDF | `./blender-cli node ShaderNodeBsdfPrincipled` |
| Export Cube to FBX | `./blender-cli export cube.fbx -f fbx -o Cube --timeout 60` |
