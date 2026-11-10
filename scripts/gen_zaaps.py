"""Regenerates the zaap table in src/zaaps.rs from DofusDB.

DofusDB (https://dofusdb.fr) is a community database extracted from the game
files. A zaap is a map hint named "Zaap"; its territory is the hint's subarea.
Run by hand when Ankama adds zaaps — never at build time (no network in the
build, cf. CLAUDE.md "Audit supply-chain").

    python -I scripts/gen_zaaps.py > zaaps_table.rs

Then paste the output over the `ZAAPS` table in src/zaaps.rs.
"""
import json
import sys
import urllib.request

# Accented names: never let the Windows console code page pick the encoding.
sys.stdout.reconfigure(encoding="utf-8")

API = "https://api.dofusdb.fr"
HEADERS = {"User-Agent": "ROrganizer-zaap-list-builder"}


def get(path):
    req = urllib.request.Request(API + path, headers=HEADERS)
    with urllib.request.urlopen(req, timeout=20) as r:
        return json.load(r)


def rust_str(s):
    return json.dumps(s, ensure_ascii=False)


hints = get("/hints?$limit=100&name.fr=Zaap")["data"]
rows = []
for h in hints:
    sub = get(f"/subareas/{h['subareaId']}")
    area = get(f"/areas/{sub['areaId']}")
    rows.append((sub["name"]["fr"], area["name"]["fr"], h["x"], h["y"]))
rows.sort(key=lambda r: r[0].lower())

print("pub const ZAAPS: &[Zaap] = &[")
for name, area, x, y in rows:
    print(f"    Zaap {{ name: {rust_str(name)}, area: {rust_str(area)}, x: {x}, y: {y} }},")
print("];")
