"""Stage release assets and replace (or add) one skill's entry in index.json.

Usage: update_index.py <name> <version> <tag> <out_dir> <assets_dir>

<out_dir>/<platform>/ holds each platform's install tree, and
<out_dir>/<platform>.executables lists (one per line) the relative paths that
get "executable": true (all others false). Every file is copied flat into <assets_dir> as
<dir with "/" as ".">.<stem>-<platform><ext>, ready for `gh release upload`.
"""

import hashlib
import json
import os
import shutil
import sys
from pathlib import Path

INDEX = Path("index.json")


def platform_files(platform_dir: Path, assets: Path, base_url: str) -> list:
    platform = platform_dir.name
    marker = platform_dir.with_name(f"{platform}.executables")
    executables = (
        {line.strip() for line in marker.read_text().splitlines() if line.strip()}
        if marker.exists()
        else set()
    )

    files = []
    for path in sorted(platform_dir.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(platform_dir).as_posix()
        dir_, _, file_name = rel.rpartition("/")
        # SKILL.md -> SKILL-windows-x86_64.md, remider.exe -> remider-windows-x86_64.exe
        stem, ext = os.path.splitext(file_name)
        asset = ".".join(p for p in (dir_.replace("/", "."), f"{stem}-{platform}{ext}") if p)
        shutil.copyfile(path, assets / asset)

        data = path.read_bytes()
        entry = {
            "dir": dir_,
            "fileName": file_name,
            "url": f"{base_url}/{asset}",
            "sha256": hashlib.sha256(data).hexdigest(),
            "size": len(data),
            "executable": rel in executables,
        }
        files.append(entry)
    return files


def main() -> None:
    name, version, tag, out_dir, assets_dir = sys.argv[1:6]
    out, assets = Path(out_dir), Path(assets_dir)
    assets.mkdir(parents=True, exist_ok=True)
    base_url = f"https://github.com/{os.environ['GITHUB_REPOSITORY']}/releases/download/{tag}"

    platforms = {
        p.name: {"files": platform_files(p, assets, base_url)}
        for p in sorted(out.iterdir())
        if p.is_dir()
    }
    if not platforms:
        sys.exit(f"no platform folders for {name} in {out}")

    index = (
        json.loads(INDEX.read_text(encoding="utf-8"))
        if INDEX.exists()
        else {"schema": 2, "skills": []}
    )
    skill = {"name": name, "version": version, "platforms": platforms}
    skills = index["skills"]
    for i, s in enumerate(skills):
        if s["name"] == name:
            skills[i] = skill
            break
    else:
        skills.append(skill)

    INDEX.write_text(json.dumps(index, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"index.json: {name} {version} -> {', '.join(platforms)}")


if __name__ == "__main__":
    main()
