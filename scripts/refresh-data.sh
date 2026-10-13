#!/usr/bin/env bash
#
# Rebuild the bundled game-data tables in crates/lootsmith-core/assets/data/
# from their upstream sources, and regenerate the derived rarity.json.
#
# Sources
#   * gibbed/Borderlands2Dumps  — the 17 Gibbed dump tables (verbatim copies;
#     the dumps are pretty-printed UTF-8 and carry a BOM, which the loaders
#     strip). Pinned to a commit so the refresh is reproducible.
#   * thekeogh/borderlens      — curated per-item rarities, via gen-rarity.py.
#   * BLCM/OpenBLCMM-Data      — pool tiers, via gen-rarity.py.
#
# Usage
#   scripts/refresh-data.sh                 # tables + rarity
#   scripts/refresh-data.sh --tables-only   # just the 17 Gibbed tables
#
# Environment overrides
#   BORDERLANDS2DUMPS_REPO   default gibbed/Borderlands2Dumps
#   BORDERLANDS2DUMPS_REF    default the pinned commit below
#   BLCM_DUMPS               default /tmp/opencode/blcm/bl2data.jar
#   BORDERLENS_DIR           default /tmp/opencode/borderlens
#
# After it runs, review `git status` and `git diff --stat`, then run the suite.

set -euo pipefail

REPO="${BORDERLANDS2DUMPS_REPO:-gibbed/Borderlands2Dumps}"
# Pinned master commit (2019-06-22); the committed tables match this revision.
REF="${BORDERLANDS2DUMPS_REF:-08c9a885e5939cc62ca82e9277e2f95580bf7850}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/crates/lootsmith-core/assets/data"
RAW="https://raw.githubusercontent.com/$REPO/$REF"

TABLES_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --tables-only) TABLES_ONLY=1 ;;
    -h|--help) sed -n '2,30p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }

# source (URL-escaped) -> bundled file name, for the 17 Gibbed tables.
TABLES=(
  "Asset%20Library%20Manager.json|asset-library-manager.json"
  "Item%20Balance.json|item-balance.json"
  "Weapon%20Balance.json|weapon-balance.json"
  "Weapon%20Types.json|weapon-types.json"
  "Item%20Parts.json|item-parts.json"
  "Weapon%20Parts.json|weapon-parts.json"
  "Item%20Part%20Lists.json|item-part-lists.json"
  "Weapon%20Part%20Lists.json|weapon-part-lists.json"
  "Item%20Balance%20Part%20Lists.json|item-balance-part-lists.json"
  "Weapon%20Balance%20Part%20Lists.json|weapon-balance-part-lists.json"
  "Item%20Name%20Parts.json|item-name-parts.json"
  "Weapon%20Name%20Parts.json|weapon-name-parts.json"
  "Items.json|items.json"
  "Missions.json|missions.json"
  "Travel%20Stations.json|travel-stations.json"
  "Customizations.json|customizations.json"
  "Downloadable%20Contents.json|downloadable-contents.json"
)

mkdir -p "$OUT"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "Fetching 17 tables from $REPO@$REF"
for entry in "${TABLES[@]}"; do
  src="${entry%%|*}"
  dst="${entry##*|}"
  # Keep the original Gibbed name in WORK (gen-rarity.py reads those).
  name="$(printf '%s' "$src" | sed 's/%20/ /g')"
  curl -fsSL --retry 3 "$RAW/$src" -o "$WORK/$name"
  cp "$WORK/$name" "$OUT/$dst"
  printf '  %-40s -> %s\n' "$name" "$dst"
done
echo "Wrote $((${#TABLES[@]})) tables to ${OUT#"$ROOT"/}"

if [ "$TABLES_ONLY" = 1 ]; then
  echo "Skipping rarity.json (--tables-only)."
else
  BLCM_DUMPS="${BLCM_DUMPS:-/tmp/opencode/blcm/bl2data.jar}"
  BORDERLENS_DIR="${BORDERLENS_DIR:-/tmp/opencode/borderlens}"
  if [ ! -e "$BLCM_DUMPS" ] || [ ! -d "$BORDERLENS_DIR" ]; then
    echo "Rarity sources missing (set BLCM_DUMPS and BORDERLENS_DIR); skipping rarity.json." >&2
  else
    echo "Regenerating rarity.json (gen-rarity.py)"
    python3 "$ROOT/scripts/gen-rarity.py" \
      --blcm "$BLCM_DUMPS" \
      --borderlens "$BORDERLENS_DIR" \
      --dumps "$WORK/Asset Library Manager.json" \
      --out "$ROOT/crates/lootsmith-core/assets/rarity.json"
    cp "$ROOT/crates/lootsmith-core/assets/rarity.json" "$OUT/rarity.json"
  fi
fi

echo
echo "Done. Review:"
echo "  git status --short"
echo "  git diff --stat"
echo "  LOOTSMITH_REQUIRE_FIXTURES=1 cargo test --workspace"
