#!/usr/bin/env python3
"""Generate the embedded skill-tree data + icons for the Lootsmith skill calculator.

Parses the six Borderlands 2 skill-calculator class pages (UCP/"patch" variant) and
downloads the skill icons. Output is committed so the app and its builds are offline.

Usage: scripts/gen-skill-data.py
"""
import json
import os
import re
import subprocess
import sys

BASE = "https://bl2skills.com/patch"
ICON_BASE = "https://bl2skills.com/icons"
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "crates", "lootsmith-gui", "assets", "skills")
ICONS = os.path.join(OUT, "icons")
PORTRAITS = os.path.join(OUT, "portraits")

CLASSES = [
    ("gunzerker", "Salvador", "Gunzerker", "GD_Mercenary.Character.CharClass_Mercenary"),
    ("commando", "Axton", "Commando", "GD_Soldier.Character.CharClass_Soldier"),
    ("assassin", "Zer0", "Assassin", "GD_Assassin.Character.CharClass_Assassin"),
    ("siren", "Maya", "Siren", "GD_Siren.Character.CharClass_Siren"),
    ("mechromancer", "Gaige", "Mechromancer", "GD_Tulip_Mechromancer.Character.CharClass_Mechromancer"),
    ("psycho", "Krieg", "Psycho", "GD_Lilac_PlayerClass.Character.CharClass_LilacPlayerClass"),
]

TAG = re.compile(r"<[^>]+>")
WS = re.compile(r"\s+")


def strip_tags(html):
    return WS.sub(" ", TAG.sub(" ", html)).strip()


def fetch(url):
    return subprocess.check_output(["curl", "-sS", url], text=True)


def parse_class(page):
    html = fetch(f"{BASE}/{page}.html")
    trees = []
    # Each tree column starts at <div class="treewrapper ...">.
    for chunk in html.split('<div class="treewrapper')[1:]:
        m = re.match(r'\s*([a-z]+)"', chunk)
        color = m.group(1) if m else "green"
        legend = re.search(r"<legend>(.*?)</legend>", chunk, re.S)
        name = strip_tags(legend.group(1)) if legend else "Tree"
        tiers = []
        for tier_chunk in chunk.split('<div class="tier')[1:]:
            level = re.search(r'data-level="(\d+)"', tier_chunk)
            skills = []
            for skill_chunk in tier_chunk.split('<div class="skill')[1:]:
                maxm = re.search(r'data-max="(\d+)"', skill_chunk)
                icon = re.search(r'src="([^"]+)"', skill_chunk)
                h2 = re.search(r"<h2>(.*?)</h2>", skill_chunk, re.S)
                desc = re.search(r'<div class="description">(.*?)</div>', skill_chunk, re.S)
                if not h2:
                    continue
                desc_text = strip_tags(desc.group(1)) if desc else ""
                # Drop the leading name from the description.
                desc_text = desc_text.replace(strip_tags(h2.group(1)), "", 1).strip()
                skills.append({
                    "skill": strip_tags(h2.group(1)),
                    "max": int(maxm.group(1)) if maxm else 5,
                    "icon": os.path.basename(icon.group(1)) if icon else "",
                    "desc": desc_text,
                })
            tiers.append({"level": int(level.group(1)) if level else len(tiers), "skills": skills})
        trees.append({"name": name, "color": color, "tiers": tiers})
    return trees


def main():
    os.makedirs(ICONS, exist_ok=True)
    os.makedirs(PORTRAITS, exist_ok=True)
    data = {"version": 1, "classes": {}}
    icons = set()
    for page, name, cls, key in CLASSES:
        trees = parse_class(page)
        skills = sum(len(tier["skills"]) for tree in trees for tier in tree["tiers"])
        print(f"{name: >10} ({page}): {len(trees)} trees, {skills} skills", file=sys.stderr)
        if skills == 0:
            print(f"  !! no skills parsed for {page}", file=sys.stderr)
        for tree in trees:
            for tier in tree["tiers"]:
                for sk in tier["skills"]:
                    if sk["icon"]:
                        icons.add(sk["icon"])
        portrait = f"{page}Portrait.png"
        dest = os.path.join(PORTRAITS, portrait)
        if not os.path.exists(dest):
            try:
                with open(dest, "wb") as fh:
                    fh.write(subprocess.check_output(["curl", "-sS", f"https://bl2skills.com/{portrait}"]))
            except subprocess.CalledProcessError as exc:
                print(f"  !! portrait failed {portrait}: {exc}", file=sys.stderr)
        data["classes"][key] = {
            "name": name,
            "class": cls,
            "page": page,
            "portrait": portrait,
            "trees": trees,
        }

    print(f"downloading {len(icons)} icons…", file=sys.stderr)
    for icon in sorted(icons):
        dest = os.path.join(ICONS, icon)
        if os.path.exists(dest):
            continue
        try:
            with open(dest, "wb") as fh:
                fh.write(subprocess.check_output(["curl", "-sS", f"{ICON_BASE}/{icon}"]))
        except subprocess.CalledProcessError as exc:
            print(f"  !! icon failed {icon}: {exc}", file=sys.stderr)

    with open(os.path.join(OUT, "skills.json"), "w") as fh:
        json.dump(data, fh, indent=1, sort_keys=True)
    print(f"wrote {os.path.join(OUT, 'skills.json')}", file=sys.stderr)


if __name__ == "__main__":
    main()
