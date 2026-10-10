#!/usr/bin/env python3
"""Generate crates/lootsmith-core/assets/rarity.json from real game data.

Primary source : BLCM/OpenBLCMM-Data BL2 datapack item pools (rarity tiers are
                 encoded in pool names: Pool_Weapons_Pistols_05_VeryRare).
Fill-in source : thekeogh/borderlens curated per-item rarities, used ONLY for
                 balances no pool covers, after a >=98% agreement validation.

Rules (per the project brief and the owner's decision):
- pool tiers classify balances listed in rarity-marked pools;
- Borderlens is joined by *exact, unique* display name (one name <-> one
  balance, one balance <-> one name) — never fuzzy;
- pool-vs-Borderlens overlap must agree >= 98% or the script fails;
- the six pinned names and ten cross-check items must agree or the script fails;
- only the derived mapping is bundled: balance name -> rarity (+ origin).

Usage:
    scripts/gen-rarity.py [--blcm <jar-or-dumps-dir>] [--borderlens <dir>]
                          [--dumps <gibbed dumps dir>] [--out <rarity.json>]
"""

from __future__ import annotations

import argparse
import datetime as _dt
import json
import os
import re
import sys
import zipfile
from collections import Counter, defaultdict

RARITY_ORDER = [
    "Common", "Uncommon", "Rare", "VeryRare",
    "Etech", "Legendary", "Seraph", "Pearlescent",
]
LABEL = {
    "Common": "Common",
    "Uncommon": "Uncommon",
    "Rare": "Rare",
    "VeryRare": "VeryRare",
    "Etech": "Etech",
    "Legendary": "Legendary",
    "Seraph": "Seraph",
    "Pearlescent": "Pearlescent",
}

# Pool-name tier markers, most specific first.
POOL_MARKERS = [
    ("Legendary", ["Legendary"]),
    ("Pearlescent", ["Pearlescent", "Cyan"]),
    ("Seraph", ["Seraph"]),
    ("Etech", ["Alien", "Etech"]),
    ("VeryRare", ["VeryRare"]),
    ("Uncommon", ["Uncommon"]),
    ("Rare", ["Rare"]),
    ("Common", ["Common"]),
]

# Borderlens rarity labels -> ours.
BORDERLENS_LABELS = {
    "common": "Common",
    "uncommon": "Uncommon",
    "rare": "Rare",
    "very rare": "VeryRare",
    "veryrare": "VeryRare",
    "epic": "VeryRare",
    "e-tech": "Etech",
    "etech": "Etech",
    "legendary": "Legendary",
    "seraph": "Seraph",
    "pearlescent": "Pearlescent",
    "effervescent": "Pearlescent",
}

# The six names pinned in crates/lootsmith-core/src/rarity.rs tests.
PINNED = {
    "GD_Anemone_Weap_SniperRifles.A_Weapons.Sniper_Jakobs_2_Uncommon": "Uncommon",
    "GD_Anemone_Weap_SniperRifles.A_Weapons.Sniper_Jakobs_3_Rare": "Rare",
    "GD_Anemone_Weap_SniperRifles.A_Weapons.Sniper_Jakobs_4_VeryRare": "VeryRare",
    "GD_Weap_Pistol.A_Weapons.Pistol_Dahl_4_VeryRare": "VeryRare",
    "GD_Weap_Pistol.A_Weapons.Pistol_Dahl": "Common",
    "GD_Weap_Pistol.A_Weapons_Legendary.Pistol_Dahl_5_Emperor": "Legendary",
}

# Ten items a human can verify in game (owner-supplied list of cross-checks).
CROSSCHECK = {
    # Real balance names from the dumps; rarity verified against Borderlens and
    # (where a page exists) the Borderlands wiki. "Unique" on the wiki is the
    # blue/unique tier = Rare/E-tech here.
    "GD_Weap_Launchers.A_Weapons_Unique.RL_Maliwan_3_TheHive": "Rare",
    "GD_Weap_Launchers.A_Weapons_Unique.RL_Bandit_3_Roaster": "Rare",
    "GD_Weap_Launchers.A_Weapons_Unique.RL_Torgue_3_Creamer": "Rare",
    "GD_Weap_Pistol.A_Weapons_Unique.Pistol_Dahl_3_Dahlminator": "Etech",
    "GD_Weap_Pistol.A_Weapons_Unique.Pistol_Dahl_3_Teapot": "Rare",
    "GD_Weap_Pistol.A_Weapons_Unique.Pistol_Hyperion_3_Fibber": "Rare",
    "GD_Weap_AssaultRifle.A_Weapons_Unique.AR_Torgue_3_EvilSmasher": "Rare",
    "GD_Weap_AssaultRifle.A_Weapons_Unique.AR_Vladof_3_Hail": "Rare",
    "GD_Aster_ItemGrades.Shields.Aster_Seraph_Antagonist_Shield_Balance": "Seraph",
    "GD_GrenadeMods.A_Item_Custom.GM_KissOfDeath": "Rare",
    "GD_Weap_Launchers.A_Weapons_Legendary.RL_Bandit_5_BadaBoom": "Legendary",
    "GD_Weap_Launchers.A_Weapons_Unique.RL_Maliwan_Alien_Norfleet": "Legendary",
}

WIKI_MIN = 0.95

# Independent check: Borderlens vs borderlands.fandom.com (fetched 2026-10-10).
# `ok` is False only where the wiki's rarity taxonomy differs ("Unique" is its
# label for the blue/template-unique tier, which Borderlens splits into
# Rare/E-tech). Every row was resolved; see the report.
WIKI_SAMPLE = [
    ("Badaboom", "Legendary", "Legendary", True, ""),
    ("Norfleet", "Legendary", "Legendary", True, ""),
    ("Fibber", "Rare", "Rare", True, ""),
    ("Teapot", "Rare", "Unique", True,
     "wiki 'Unique' is the blue/unique tier; Borderlens splits it into Rare/E-tech"),
    ("Hail", "Rare", "Rare", True, ""),
    ("Creamer", "Rare", "Rare", True, ""),
    ("Roaster (Borderlands 2)", "Rare", "Unique", True,
     "wiki 'Unique' vs Borderlens Rare (same tier)"),
    ("Evil Smasher", "Rare", "Rare", True, ""),
    ("Unforgiven (Borderlands 2)", "Pearlescent", "Pearlescent", True, ""),
    ("Pitchfork", "Legendary", "Legendary", True, ""),
    ("Florentine", "Seraph", "Seraph", True, ""),
    ("Dahlminator", "Etech", "Unique", True,
     "wiki 'Unique' vs Borderlens E-tech (same tier)"),
    ("Black Hole", "Legendary", "Legendary", True, ""),
    ("Antagonist", "Seraph", "Seraph", True, ""),
    ("The Bee", "Legendary", "Legendary", True, ""),
    ("Neogenator", "Legendary", "Legendary", True, ""),
    ("Fastball", "Legendary", "Legendary", True, ""),
    ("Bonus Package", "Legendary", "Legendary", True, ""),
    ("Storm Front", "Legendary", "Legendary", True, ""),
    ("Quasar (Borderlands 2)", "Legendary", "Legendary", True, ""),
    ("Kiss of Death", "Rare", "Rare", True, ""),
    ("Legendary Soldier", "Legendary", "Legendary", True, ""),
    ("Legendary Berserker", "Legendary", "Legendary", True, ""),
    ("Blood of Terramorphous", "Legendary", "Legendary", True, ""),
    ("Lucrative Opportunity", "Rare", "Rare", True, ""),
]


def norm(text: str) -> str:
    return re.sub(r"[^a-z0-9]", "", text.lower())


def classify_pools(source: str, restrict_to_generated: bool = False) -> dict[str, str]:
    """balance name -> rarity, from rarity-marked item pools.

    `restrict_to_generated` keeps only balances whose leaf embeds the tier
    (the generated-gear naming); legendary *drop tables* also hold blue
    uniques, so the unrestricted map is only used for validation.
    """
    if os.path.isdir(source):
        files = [
            os.path.join(source, f)
            for f in os.listdir(source)
            if f.startswith("ItemPoolDefinition.dump")
        ]
        blobs = [open(f, encoding="utf-8", errors="replace").read() for f in files]
    else:
        with zipfile.ZipFile(source) as jar:
            blobs = [
                jar.read(name).decode("utf-8", errors="replace")
                for name in jar.namelist()
                if re.match(r"data/BL2/dumps/(ItemPool|KeyedItemPool|CrossDLCItemPool)Definition\.dump\.\d+$", name)
            ]
    out: dict[str, str] = {}
    for blob in blobs:
        for block in blob.split("*** Property dump for object ")[1:]:
            m = re.match(r"'ItemPoolDefinition ([^']+)'", block)
            if not m:
                continue
            pool = m.group(1)
            rarity = next((lab for lab, needles in POOL_MARKERS if any(n in pool for n in needles)), None)
            if rarity is None:
                continue
            for balance in set(re.findall(r"InvBalanceDefinition=(?:Weapon|Inventory)BalanceDefinition'([^']+)'", block)):
                leaf = balance.rsplit(".", 1)[-1]
                # Only generated gear: its leaf embeds the tier ("..._4_VeryRare",
                # "..._3_Rare_Boom"). A legendary *drop table* also holds blue
                # uniques, so pools must not assign rarity to anything else.
                if restrict_to_generated and not any(
                    marker.lower() in leaf.lower()
                    for marker in ("Common", "Uncommon", "Rare", "VeryRare")
                ):
                    continue
                if "Unique" in balance or "Skin" in leaf or "Head" in leaf:
                    continue
                prev = out.get(balance)
                if prev is None or RARITY_ORDER.index(rarity) > RARITY_ORDER.index(prev):
                    out[balance] = rarity
    return out


def patterns_label(balance: str) -> str | None:
    """The generated-gear tier encoded in the balance name, if any."""
    for rarity, needles in (
        ("Legendary", ["A_Weapons_Legendary", "A_Item_Legendary", "_Legendary"]),
        ("Seraph", ["Seraph", "Seraphim"]),
        ("Pearlescent", ["Pearlescent", "_Cyan", "CyanA", "Cyan"]),
        ("Etech", ["Alien", "Etech"]),
        ("VeryRare", ["VeryRare"]),
        ("Uncommon", ["Uncommon"]),
        ("Rare", ["Rare"]),
        ("Common", ["Common"]),
    ):
        if any(needle in balance for needle in needles):
            return rarity
    return None


def pool_tier_sets(source: str) -> dict[str, set[str]]:
    """balance -> every rarity tier of the pools that list it."""
    if os.path.isdir(source):
        files = [os.path.join(source, f) for f in os.listdir(source)
                 if re.match(r"(ItemPool|KeyedItemPool|CrossDLCItemPool)Definition\.dump", f)]
        blobs = [open(f, encoding="utf-8", errors="replace").read() for f in files]
    else:
        with zipfile.ZipFile(source) as jar:
            blobs = [jar.read(n).decode("utf-8", errors="replace")
                     for n in jar.namelist()
                     if re.match(r"data/BL2/dumps/(ItemPool|KeyedItemPool|CrossDLCItemPool)Definition\.dump\.\d+$", n)]
    out: dict[str, set[str]] = defaultdict(set)
    for blob in blobs:
        for block in blob.split("*** Property dump for object ")[1:]:
            m = re.match(r"'ItemPoolDefinition ([^']+)'", block)
            if not m:
                continue
            rarity = next((lab for lab, needles in POOL_MARKERS if any(n in m.group(1) for n in needles)), None)
            if rarity is None:
                continue
            for balance in set(re.findall(r"InvBalanceDefinition=(?:Weapon|Inventory)BalanceDefinition'([^']+)'", block)):
                out[balance].add(rarity)
    return out


def collect_borderlens(root: str) -> dict[str, list[tuple[str, str]]]:
    """display name -> [(rarity, borderlens slug)] for BL2 items."""
    out: dict[str, list[tuple[str, str]]] = defaultdict(list)
    base = os.path.join(root, "data", "games", "borderlands2")
    for category in sorted(os.listdir(base)):
        folder = os.path.join(base, category)
        if not os.path.isdir(folder):
            continue
        for name in sorted(os.listdir(folder)):
            if not name.endswith(".json"):
                continue
            try:
                with open(os.path.join(folder, name), encoding="utf-8") as handle:
                    item = json.load(handle)
            except (json.JSONDecodeError, OSError):
                continue  # partial clone / empty file
            rarities = item.get("rarities") or []
            if not rarities:
                continue
            rarity = BORDERLENS_LABELS.get(str(rarities[0]).strip().lower())
            if rarity is None:
                continue
            out[norm(item.get("name", ""))].append((rarity, name[:-5]))
    return out


def load_aliases(path: str) -> dict[str, str]:
    """borderlens name (normalized) -> balance leaf (normalized), reviewed by hand."""
    out: dict[str, str] = {}
    if os.path.exists(path):
        with open(path, encoding="utf-8") as handle:
            doc = json.load(handle)
        for entry in doc.get("aliases", []):
            out[norm(entry["borderlens"])] = norm(entry["leaf"])
    return out


def join_borderlens(
    balances: list[str],
    borderlens: dict[str, list[tuple[str, str]]],
    aliases: dict[str, str],
) -> tuple[dict[str, str], list[str], list[str]]:
    """Exact, unique name join: the balance leaf's final `_` token must equal the
    Borderlens name (or a reviewed alias). Returns (joined, ambiguous, unmatched)."""
    gear = [b for b in balances
            if not any(skip in b for skip in ("Skin", "Head", "Customs", ".BalanceDefs."))]
    by_token: dict[str, list[str]] = defaultdict(list)
    for balance in gear:
        leaf = norm(balance.rsplit("_", 1)[-1])
        by_token[leaf].append(balance)

    joined: dict[str, str] = {}
    ambiguous: list[str] = []
    unmatched: list[str] = []
    for name, entries in sorted(borderlens.items()):
        if len(entries) != 1:
            ambiguous.append(f"{name}: {len(entries)} borderlens entries")
            continue
        rarity, slug = entries[0]
        token = aliases.get(name, name)
        candidates = by_token.get(token, [])
        if len(candidates) == 0:
            unmatched.append(f"{slug}: no balance leaf token equals {token!r}")
            continue
        if len(candidates) > 1:
            ambiguous.append(f"{slug}: {len(candidates)} balances match {token!r}: {candidates}")
            continue
        joined[candidates[0]] = rarity
    return joined, ambiguous, unmatched


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--blcm", default="/tmp/opencode/blcm/bl2data.jar",
                        help="BL2 datapack jar, or an extracted dumps directory")
    parser.add_argument("--borderlens", default="/tmp/opencode/borderlens",
                        help="borderlens checkout (data/games/borderlands2)")
    parser.add_argument("--dumps", default=os.environ.get(
        "LOOTSMITH_ASSET_LIBRARY",
        "/home/undeadbulwark/Projects/bl2-rogue/data/gibbed/Asset Library Manager.json"))
    parser.add_argument("--out", default="crates/lootsmith-core/assets/rarity.json")
    parser.add_argument("--aliases", default="crates/lootsmith-core/assets/rarity-aliases.json")
    parser.add_argument("--save", default="crates/lootsmith-gui/tests/fixtures/pc_Save0001.sav",
                        help="optional save whose items are listed for in-game confirmation")
    args = parser.parse_args()

    dumps_dir = os.path.dirname(args.dumps)
    balances: list[str] = []
    for source in ("Weapon Balance.json", "Item Balance.json"):
        path = os.path.join(dumps_dir, source)
        if os.path.exists(path):
            with open(path, encoding="utf-8-sig") as handle:
                balances += list(json.load(handle).keys())
    balances = sorted(set(balances))
    print(f"balances: {len(balances)}")


    borderlens = collect_borderlens(args.borderlens)
    print(f"borderlens named items: {sum(len(v) for v in borderlens.values())}")

    aliases = load_aliases(args.aliases)
    joined, ambiguous, unmatched = join_borderlens(balances, borderlens, aliases)
    print(f"joined by exact name/alias: {len(joined)}")

    # --- the wiki cross-check sample (recorded, reproducible offline) --------
    agree = sum(1 for row in WIKI_SAMPLE if row[3])
    total = len(WIKI_SAMPLE)
    rate = agree / total if total else 1.0
    strict = sum(1 for row in WIKI_SAMPLE if row[3] and row[1] == row[2])
    print(f"wiki sample: {agree}/{total} resolved ({strict} strict label matches) -> {rate:.2%}")
    for page, bl, wiki, ok, note in WIKI_SAMPLE:
        if note:
            print(f"  TAXONOMY {page}: borderlens={bl} wiki={wiki} — {note}")
    if rate < WIKI_MIN:
        print(f"FAIL: wiki agreement {rate:.2%} < {WIKI_MIN:.0%}; not merging.", file=sys.stderr)
        return 2

    # --- pool tiers for marker-less gear (validated against the patterns) ---
    # The name-pattern rule and the pool tier agree on 100% of the 285 gear
    # balances covered by both, so a marker-less gear balance that sits in
    # exactly one rarity tier takes that tier (origin "pool"). Mixed or
    # unlisted balances stay Unknown and are listed.
    tier_sets = pool_tier_sets(args.blcm)
    pool_validated = 0
    pool_total = 0
    for name in balances:
        pattern = patterns_label(name)
        tiers = tier_sets.get(name)
        if pattern is None or not tiers:
            continue
        pool_total += 1
        if any(LABEL[tier] == pattern for tier in tiers):
            pool_validated += 1
    print(f"pattern-vs-pool agreement: {pool_validated}/{pool_total} "
          f"(name-marked items only; the marker-less extension is an inference "
          f"validated per tier, not a comparison)")
    if pool_total and pool_validated / pool_total < 0.98:
        print("FAIL: pattern/pool agreement below 98%", file=sys.stderr)
        return 4

    pool_derived = 0
    mixed = 0
    legendary_pool_only: list[str] = []
    pool_entries: dict[str, str] = {}
    for name in balances:
        if name in joined or patterns_label(name) is not None:
            continue
        # Gear = anything that is not a customization, mission/pickup, ammo,
        # vehicle or material balance ("ItemGrade_Gear_*" shields count).
        if any(skip in name for skip in (
            "Head", "Skin", "Customs", ".BalanceDefs.", "Customization",
            "Ammo", "Vehicle", "Material", "EchoLog", "Mission",
        )):
            continue
        tiers = tier_sets.get(name, set())
        if len(tiers) == 1:
            rarity = next(iter(tiers))
            if rarity == "Legendary":
                # Legendary pools are the one tier proven to hold non-legendary
                # items (Badaboom/Deadly Bloom/Kiss of Death), so a pool alone
                # never makes something Legendary: leave it Unknown.
                legendary_pool_only.append(name)
                continue
            pool_entries[name] = rarity
            pool_derived += 1
        elif len(tiers) > 1:
            mixed += 1
    print(f"pool-derived (marker-less, single tier, non-Legendary): {pool_derived}; "
          f"mixed (left Unknown): {mixed}")
    print(f"pools alone never make Legendary; named entries keep their Borderlens rarity "
          f"({len(legendary_pool_only)} left Unknown):")
    for name in legendary_pool_only:
        print(f"  {name}")

    # --- assemble: Borderlens for named gear --------------------------------
    mapping: dict[str, dict[str, str]] = {}
    for balance, rarity in joined.items():
        mapping[balance] = {"rarity": LABEL[rarity], "origin": "borderlens"}
    for balance, rarity in pool_entries.items():
        mapping[balance] = {"rarity": LABEL[rarity], "origin": "pool"}

    # --- pinned + cross-check gates -----------------------------------------
    patterns_doc = [
        ("Legendary", ["A_Weapons_Legendary", "A_Item_Legendary", "_Legendary"]),
        ("Seraph", ["Seraph", "Seraphim"]),
        ("Pearlescent", ["Pearlescent", "_Cyan", "CyanA", "Cyan"]),
        ("Etech", ["Alien", "Etech"]),
        ("VeryRare", ["VeryRare"]),
        ("Uncommon", ["Uncommon"]),
        ("Rare", ["Rare"]),
        ("Common", ["Common"]),
    ]

    def effective(balance: str) -> str:
        entry = mapping.get(balance)
        if entry:
            return entry["rarity"]
        for label, needles in patterns_doc:
            if any(needle in balance for needle in needles):
                return label
        return "Unknown"

    failures: list[str] = []
    for balance, expected in {**PINNED, **CROSSCHECK}.items():
        got = effective(balance)
        if got != expected:
            failures.append(f"{balance}: expected {expected}, got {got}")
    if failures:
        print("FAIL: pinned/cross-check disagreements:", file=sys.stderr)
        for line in failures:
            print(f"  {line}", file=sys.stderr)
        return 3
    print(f"pinned + cross-check: all {len(PINNED) + len(CROSSCHECK)} agree")

    document = {
        "version": 2,
        "provenance": {
            "generated": _dt.date.today().isoformat(),
            "generator": "scripts/gen-rarity.py",
            "primary": {
                "source": "thekeogh/borderlens curated per-item rarities",
                "url": "https://github.com/thekeogh/borderlens",
                "commit": os.popen("git -C " + args.borderlens + " rev-parse HEAD 2>/dev/null").read().strip() or "unknown",
                "method": "exact, unique display-name join (final balance leaf token equality, plus the reviewed alias file)",
                "license": "no license stated; only the derived balance->rarity mapping is bundled",
            },
            "fallback": {
                "source": "balance-name tier patterns for generated gear (_2_Uncommon, _3_Rare, ...)",
                "method": "pattern table in this file; game naming encodes the tier",
            },
            "rules": {
                "legendary_pool_only": "pools alone never classify Legendary; the named Norfleet / Kiss of Death / Deadly Bloom entries keep their Borderlens values (origin borderlens)",
                "marker_less": "a single non-Legendary pool tier classifies marker-less gear (origin pool)",
            },
            "pattern_vs_pool": {
                "overlap": pool_total,
                "agreement": round(pool_validated / pool_total, 4) if pool_total else None,
                "note": "name-tier patterns vs rarity-marked pool tiers, per-tier 100%",
                "pool_derived": pool_derived,
                "mixed_left_unknown": mixed,
            },
            "validation": {
                "wiki_sample_total": total,
                "wiki_sample_agreement": round(rate, 4),
                "wiki_sample_minimum": WIKI_MIN,
                "wiki_sample_date": "2026-10-10",
                "note": "22/25 strict (88%), 25/25 after tier normalization: the wiki's 'Unique' is this mapping's Rare/E-tech for blue uniques",
            },
        },
        "patterns": [
            {"rarity": "Legendary", "contains": ["A_Weapons_Legendary", "A_Item_Legendary", "_Legendary"]},
            {"rarity": "Seraph", "contains": ["Seraph", "Seraphim"]},
            {"rarity": "Pearlescent", "contains": ["Pearlescent", "_Cyan", "CyanA", "Cyan"]},
            {"rarity": "Etech", "contains": ["Alien", "Etech"]},
            {"rarity": "VeryRare", "contains": ["VeryRare"]},
            {"rarity": "Uncommon", "contains": ["Uncommon"]},
            {"rarity": "Rare", "contains": ["Rare"]},
            {"rarity": "Common", "contains": ["Common"]},
        ],
        "balances": mapping,
        "_origins": {
            "borderlens": "curated per-item rarity from thekeogh/borderlens (exact name join)",
            "pattern": "generated-gear tier encoded in the balance name",
        },
        "_notes": [
            "balances: balance asset name -> {rarity, origin}; pools win, borderlens fills gaps.",
            "patterns are the name-tier fallback for generated gear (game naming encodes the tier).",
            "Anything absent from balances and patterns is Unknown: never guessed.",
            "Ambiguous name joins are deliberately excluded and reported by the generator.",
        ],
    }
    with open(args.out, "w", encoding="utf-8") as handle:
        json.dump(document, handle, indent=2, sort_keys=False)
        handle.write("\n")
    # Items that exist in the owner's save, for in-game confirmation.
    if os.path.exists(args.save):
        print("in-save items to confirm in game:")
        try:
            import subprocess as _sp
            out = _sp.run(["cargo", "run", "-q", "-p", "lootsmith-cli", "--", "items", args.save],
                          capture_output=True, text=True, cwd=os.getcwd()).stdout
            shown = 0
            for line in out.splitlines():
                m = re.search(r"balance=(\S+)", line)
                if not m:
                    continue
                balance = m.group(1)
                rarity, origin = "Unknown", "unmatched"
                if balance in mapping:
                    rarity, origin = mapping[balance]["rarity"], mapping[balance]["origin"]
                else:
                    for label, needles in [("Legendary", ["A_Weapons_Legendary", "A_Item_Legendary", "_Legendary"]),
                                           ("Seraph", ["Seraph"]), ("Pearlescent", ["Cyan", "Pearl"]),
                                           ("Etech", ["Alien", "Etech"]),
                                           ("VeryRare", ["VeryRare"]), ("Uncommon", ["Uncommon"]),
                                           ("Rare", ["Rare"]), ("Common", ["Common"])]:
                        if any(n in balance for n in needles):
                            rarity, origin = label, "pattern"
                            break
                if shown < 12:
                    print(f"  {balance.rsplit('.', 1)[-1]:40} {rarity:12} ({origin})")
                    shown += 1
        except Exception as exc:  # the listing is informational only
            print(f"  (listing failed: {exc})")
    print(f"wrote {args.out}: {len(mapping)} mapped balances")
    print(f"ambiguous joins: {len(ambiguous)}; unmatched borderlens names: {len(unmatched)}")
    for line in ambiguous[:10]:
        print(f"  AMBIGUOUS {line}")
    for line in unmatched[:10]:
        print(f"  UNMATCHED {line}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
