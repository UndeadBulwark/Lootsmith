#!/usr/bin/env bash
# Build the milestone review bundle. Spec: docs/PLAN.md §5.
#
# Usage: scripts/review-bundle.sh [milestone-label]
# Output: artifacts/bundles/lootsmith-review-<short-hash>.zip (newest 5 kept).
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$(pwd)
OUT="$ROOT/artifacts/bundles"
mkdir -p "$OUT"
MILESTONE="${1:-unspecified}"
MAX_BYTES=$((5 * 1024 * 1024))

HASH=$(git rev-parse --short HEAD)
SUBJ=$(git log -1 --format=%s)
CDATE=$(git log -1 --format=%ci)
EPOCH=$(git log -1 --format=%ct)
ZIP="$OUT/lootsmith-review-$HASH.zip"

CONV=$(command -v magick || command -v convert)
have() { command -v "$1" >/dev/null 2>&1; }

echo "== preflight =="
if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: working tree not clean — commit everything first"; git status --short; exit 1
fi

PREV=$(for z in "$OUT"/lootsmith-review-*.zip; do
  [ -e "$z" ] || continue
  h=$(basename "$z" .zip | sed 's/^lootsmith-review-//')
  [ "$h" = "$HASH" ] && continue
  git cat-file -e "$h^{commit}" 2>/dev/null || continue
  echo "$(git show -s --format=%ct "$h") $z"
done | sort -rn | head -1 | awk '{print $2}')
PREVHASH=""
if [ -n "$PREV" ]; then
  PREVHASH=$(basename "$PREV" .zip | sed 's/^lootsmith-review-//')
fi
# Optional explicit "since" hash (2nd arg) overrides the discovered previous bundle.
SINCE="${2:-}"
if [ -n "$SINCE" ]; then PREVHASH="$SINCE"; fi
echo "HEAD=$HASH  milestone=$MILESTONE  previous=${PREVHASH:-none}"

echo "== gate: tests / clippy / fmt =="
TESTOUT=$(mktemp); CLIPOUT=$(mktemp); FMTOUT=$(mktemp); CONTRAST=$(mktemp)
set +e
LOOTSMITH_REQUIRE_FIXTURES=1 cargo test --workspace >"$TESTOUT" 2>&1; TESTRC=$?
cargo clippy --workspace --all-targets -- -D warnings >"$CLIPOUT" 2>&1; CLIPRC=$?
cargo fmt --all -- --check >"$FMTOUT" 2>&1; FMTRC=$?
cargo test -p lootsmith-gui --test contrast -- --nocapture >"$CONTRAST" 2>&1
set -e
if [ "$TESTRC" != 0 ]; then echo "ERROR: tests failed"; tail -40 "$TESTOUT"; exit 1; fi
if [ "$CLIPRC" != 0 ]; then echo "ERROR: clippy failed"; tail -40 "$CLIPOUT"; exit 1; fi
if [ "$FMTRC" != 0 ]; then echo "ERROR: fmt check failed"; tail -40 "$FMTOUT"; exit 1; fi

TESTLINE=$(grep -E '^test result:' "$TESTOUT" | awk '{p+=$4;f+=$6;i+=$8} END{printf "passed=%d failed=%d ignored=%d", p, f, i}')
echo "tests: $TESTLINE"

echo "== render screenshots (current code) =="
SHOTS=$(mktemp -d)
cargo run -q -p lootsmith-gui --example bundle_shots -- "$SHOTS" >/dev/null 2>&1
SHOTCOUNT=$(find "$SHOTS" -name '*.png' | wc -l)
[ "$SHOTCOUNT" -gt 0 ] || { echo "ERROR: no screenshots rendered"; exit 1; }

STAGE=$(mktemp -d)
mkdir -p "$STAGE/screenshots"
# Quantise (flat pixel art -> PNG8) to fit the size budget.
for f in "$SHOTS"/*.png; do
  "$CONV" "$f" -strip -colors 64 PNG8:"$STAGE/screenshots/$(basename "$f")"
done
SS="$STAGE/screenshots"

# Reference mockups (quantised).
for t in dark light; do
  "$CONV" "$ROOT/docs/screenshots/mockup-backpack-$t.png" -strip -colors 64 PNG8:"$SS/mockup_backpack_$t.png"
done

# Backpack side by side: mockup | mine 100% | mine 200%.
for t in dark light; do
  montage "$SS/mockup_backpack_$t.png" "$SS/backpack_${t}_1x.png" "$SS/backpack_${t}_2x.png" \
    -resize 420x -tile 3x1 -geometry +6+6 -background '#14100C' "$SS/backpack_sidebyside_$t.png"
done

# A contact sheet per theme (all 13 screens at 1x).
for t in dark light; do
  montage "$SS"/*_"$t"_1x.png -resize 300x -tile 4x -geometry +4+4 \
    -background '#14100C' "$SS/contact_$t.png"
done

# Final quantise pass: every image in the bundle is a flat PNG8.
for f in "$SS"/*.png; do
  "$CONV" "$f" -strip -colors 64 PNG8:"$f.tmp"
  mv "$f.tmp" "$f"
done

echo "== source + diff =="
mkdir -p "$STAGE/src"
while IFS= read -r -d '' f; do
  case "$f" in
    crates/*/assets/*|*.png|*.zip|*.bak|*.old.png) continue ;;
  esac
  mkdir -p "$STAGE/src/$(dirname "$f")"
  cp "$f" "$STAGE/src/$f"
done < <(git -c core.quotePath=false ls-files -z)
if [ -n "$PREVHASH" ]; then
  git diff --no-color "$PREVHASH..HEAD" -- . ':(exclude)*/assets/*' ':(exclude)*.png' > "$STAGE/diff.patch" || true
else
  : > "$STAGE/diff.patch"
fi

echo "== README =="
RATCHET=$(sed -n '/ratchet table/,/test result/p' "$CONTRAST" | grep -E '^  [A-Z]' || true)
COMMITS=$( [ -n "$PREVHASH" ] && git log --oneline "$PREVHASH..HEAD" || echo "(no previous bundle)" )
DELETED=$( [ -n "$PREVHASH" ] && git log --diff-filter=D --name-only --format= "$PREVHASH..HEAD" | grep -vE '^$' || echo "(none)" )
{
  echo "Lootsmith review bundle"
  echo "======================="
  echo "HEAD:      $HASH  $SUBJ"
  echo "Date:      $CDATE"
  echo "Milestone: $MILESTONE"
  echo
  echo "Tests:     $TESTLINE   (LOOTSMITH_REQUIRE_FIXTURES=1 cargo test --workspace)"
  echo "Clippy:    clean (-D warnings)"
  echo "Fmt:       clean (--check)"
  echo
  echo "Commits since the previous bundle (${PREVHASH:-none}):"
  echo "$COMMITS"
  echo
  echo "Files deleted since the previous bundle:"
  echo "$DELETED"
  echo
  echo "Contrast ratchet (per screen, [dark, light] failing runs):"
  echo "$RATCHET"
  echo
  echo "Known differences from the mockup / open issues:"
  echo "  1. Skills still the old UCP calculator (M5 rebuild pending)."
  echo "  2. Non-Backpack screens still use the old card/section style (M4.1+)."
  echo "  3. Snapshot baselines are provisional until M6."
  echo "  4. Not verified: in-game load of migrated screens."
} > "$STAGE/README.txt"

echo "== checks =="
grep -q "$HASH" "$STAGE/README.txt" || { echo "ERROR: README hash != HEAD"; exit 1; }

python3 - "$SS" <<'PY'
import sys, os
from PIL import Image
ss = sys.argv[1]
bad = []
for n in sorted(os.listdir(ss)):
    if not n.endswith(".png"):
        continue
    im = Image.open(os.path.join(ss, n)).convert("RGB")
    w, h = im.size
    px = im.load()
    xs = list(range(0, w, max(1, w // 64))) + [w - 1]
    ys = list(range(0, h, max(1, h // 64))) + [h - 1]
    border = [(x, 0) for x in xs] + [(x, h - 1) for x in xs] + [(0, y) for y in ys] + [(w - 1, y) for y in ys]
    for (x, y) in border:
        if px[x, y] == (0, 0, 0):
            bad.append(f"{n}@({x},{y})")
            break
if bad:
    print("ERROR: pure-black corner/edge pixels:", bad)
    sys.exit(1)
print("corners/edges: no pure black")
PY

# Screenshots must be newer than the HEAD commit.
OLD=$(find "$SHOTS" -name '*.png' ! -newermt "@$EPOCH" | wc -l)
[ "$OLD" -eq 0 ] || { echo "ERROR: $OLD screenshots are older than HEAD"; exit 1; }

# No stray review renders or backups.
if find "$STAGE" -path '*tests/snapshots/review*' -o -name '*.old.png' | grep -q .; then
  echo "ERROR: review renders or *.old.png present"; exit 1
fi

echo "== zip =="
python3 - "$STAGE" "$ZIP" <<'PY'
import sys, os, zipfile
stage, zip_path = sys.argv[1], sys.argv[2]
if os.path.exists(zip_path):
    os.remove(zip_path)
with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for root, _dirs, files in os.walk(stage):
        for f in files:
            p = os.path.join(root, f)
            z.write(p, os.path.relpath(p, stage))
    names = z.namelist()
    assert not any("tests/snapshots/review" in n or n.endswith(".old.png") for n in names), "forbidden files in zip"
print(os.path.getsize(zip_path))
PY
SIZE=$(stat -c %s "$ZIP")
if [ "$SIZE" -ge "$MAX_BYTES" ]; then echo "ERROR: zip is $SIZE bytes (>= 5 MB)"; rm -f "$ZIP"; exit 1; fi

# Keep the newest 5 bundles.
ls -t "$OUT"/lootsmith-review-*.zip | tail -n +6 | while read -r old; do rm -f "$old"; done

echo
echo "Bundle: $ZIP"
echo "Size:   $(( SIZE / 1024 )) KiB  (limit $(( MAX_BYTES / 1024 )) KiB)"
