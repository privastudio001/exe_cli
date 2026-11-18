## data flow
```
llm agent -> blender-cli -> TCP socket -> Blender Add-on
```
## setup Blender
1. Install `addon.py` in Blender (Edit > Preferences > Add-ons > Install from Disk).
2. In the 3D Viewport press `N`, open the **MCP for Blender** tab and click Connect (default `localhost:9876`).
## run
```
cargo run -- ping
cargo run -- scene --brief
```
## build
```
cargo build --release
```
## exe test
macOS:
```
./target/release/blender-cli ping
./target/release/blender-cli exec -c 'print(bpy.app.version_string)'
```
Windows:
```
.\target\release\blender-cli.exe ping
.\target\release\blender-cli.exe exec -c "print(bpy.app.version_string)"
```
