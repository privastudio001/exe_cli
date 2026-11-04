"""Replace (or add) one skill's entry in index.json from the files in a release folder.

Usage: update_index.py <name> <version> <tag> <out_dir>
"""

import hashlib
import json
import os
import sys
from pathlib import Path

# platform key -> executable extension
PLATFORMS = {"windows-x86_64": ".exe", "macos-aarch64": ""}

INDEX = Path("index.json")


def entry(path: Path, base_url: str) -> dict:
    data = path.read_bytes()
    return {
        "url": f"{base_url}/{path.name}",
        "sha256": hashlib.sha256(data).hexdigest(),
        "size": len(data),
    }


def main() -> None:
    name, version, tag, out_dir = sys.argv[1:5]
    out = Path(out_dir)
    base_url = f"https://github.com/{os.environ['GITHUB_REPOSITORY']}/releases/download/{tag}"

    platforms = {}
    for platform, ext in PLATFORMS.items():
        exe = out / f"{name}-{platform}{ext}"
        if not exe.exists():
            continue
        skill_md = out / f"SKILL-{platform}.md"
        if not skill_md.exists():
            skill_md = out / "SKILL.md"
        platforms[platform] = {
            "bin": entry(exe, base_url),
            "skill_md": entry(skill_md, base_url),
        }
    if not platforms:
        sys.exit(f"no binaries for {name} in {out}")

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
