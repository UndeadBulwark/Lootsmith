#!/usr/bin/env python3
"""Rebuild the embedded skill board (`assets/skills/skills.json`) from the
game-derived table (`assets/skills/skill-source.json`).

`skill-source.json` is the authority for the tree layout (branch -> tier ->
column), the save object path (`id`, e.g. `GD_Siren_Skills.Motion.Ward`), the
display name, description and max rank of every skill, extracted from the
installed Borderlands 2 packages. It fixes two classes of bug the community
skill-calculator data had: leet display names (`0ptics` vs object `Optics`) and
skills parked in the wrong branch (`Resourceful`/`Crisis Management`).

The committed PNGs stay in the repo; each is re-attached to its skill by display
name (with a tiny alias table for spelling drift between the two sources).
Portraits, branch colours and the action-skill banner icon are carried over too.

Usage: scripts/gen-skill-paths.py
"""
import json
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "crates", "lootsmith-gui", "assets", "skills")
SOURCE = os.path.join(OUT, "skill-source.json")
BOARD = os.path.join(OUT, "skills.json")

# Source display name -> the display name the committed icon was catalogued
# under in the previous board (they match exactly except for these).
ICON_ALIASES = {
    "Unf0reseen": "Unf0rseen",
    "Yippee Ki Yay": "Yippie Ki Yay",
    "Out of Bubblegum": "All Out of Bubblegum",
}


def norm(name):
    return "".join(ch for ch in name.lower() if ch.isalnum())


def main():
    source = json.load(open(SOURCE))
    board = json.load(open(BOARD))

    # name -> icon filename, from the previous board (icons were per skill).
    icon_by_name = {}
    for cls in board["classes"].values():
        for tree in cls["trees"]:
            for tier in tree["tiers"]:
                for node in tier["skills"]:
                    if node.get("icon"):
                        icon_by_name[norm(node["skill"])] = node["icon"]

    def icon_for(name):
        # The committed board may still carry the source spelling (first run) or
        # the previous, aliased one (subsequent runs); accept either.
        for candidate in (ICON_ALIASES.get(name, name), name):
            icon = icon_by_name.get(norm(candidate))
            if icon:
                return icon
        return ""

    classes = {}
    for key, cls in source["classes"].items():
        old = board["classes"][key]
        old_trees = old["trees"]
        skills = cls["skills"]
        trees = []
        for bi, branch in enumerate(cls["branches"]):
            old_tree = old_trees[bi]
            tiers = []
            for ti, tier in enumerate(branch["tiers"]):
                hidden = set(tier.get("hidden", []))
                nodes = []
                for col, cell in enumerate(tier["cells"]):
                    if not cell or cell in hidden:
                        continue
                    skill = skills[cell]
                    nodes.append(
                        {
                            "skill": skill["name"],
                            "id": cell,
                            "max": skill["max"],
                            "col": col,
                            "icon": icon_for(skill["name"]),
                            "desc": skill["description"],
                        }
                    )
                tiers.append({"level": ti, "skills": nodes})
            trees.append(
                {
                    "name": old_tree["name"],
                    "color": old_tree["color"],
                    "tiers": tiers,
                }
            )
        action = skills[cls["action"]]
        classes[key] = {
            "name": old["name"],
            "class": old["class"],
            "page": old["page"],
            "portrait": old["portrait"],
            "action_skill": {
                "name": action["name"],
                "id": cls["action"],
                "icon": old["action_skill"]["icon"],
            },
            "trees": trees,
        }
        count = sum(len(t["skills"]) for tree in trees for t in tree["tiers"])
        print(f"{old['name']:>10}: {len(trees)} trees, {count} skills")

    data = {"version": 3, "classes": classes}
    with open(BOARD, "w") as fh:
        json.dump(data, fh, indent=1, sort_keys=True)
    print(f"wrote {BOARD}")


if __name__ == "__main__":
    main()
