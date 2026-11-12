"""Regenerates src/npcs.rs (NPC positions + zaap to start from) from DofusDB.

DofusDB (https://dofusdb.fr) is a community database extracted from the game
files. The client data has no NPC spawn positions; quest objectives do: every
"go and see / bring to / show to / return to / report to <npc>" objective
names the NPC and the map it stands on. Same idea as other Dofus travel
tools, rebuilt here from DofusDB's public API only.

The zaap to start from is the one the game associates with the NPC's
subarea (`associatedZaapMapId`, as in the in-game travel suggestion), else
the nearest zaap of the same super area (world).

Run by hand when the game adds NPCs — never at build time (no network in
the build, cf. CLAUDE.md "Audit supply-chain"):

    python -I scripts/gen_npcs.py > src/npcs_table.rs

then paste the output over the `NPCS` table in src/npcs.rs.
"""
import json
import sys
import time
import urllib.parse
import urllib.request

sys.stdout.reconfigure(encoding="utf-8")

API = "https://api.dofusdb.fr"
HEADERS = {"User-Agent": "ROrganizer-npc-list-builder"}
PAGE = 50
# Objective types whose first parameter is the NPC to reach.
NPC_OBJECTIVES = [1, 2, 3, 9, 12]


def get(path, params=None):
    url = API + path + ("?" + urllib.parse.urlencode(params, doseq=True) if params else "")
    for attempt in range(4):
        try:
            req = urllib.request.Request(url, headers=HEADERS)
            with urllib.request.urlopen(req, timeout=30) as r:
                return json.load(r)
        except Exception:
            time.sleep(2 * (attempt + 1))
    raise RuntimeError(f"failed: {url}")


def get_all(path, params=None):
    params = dict(params or {})
    out, skip = [], 0
    while True:
        page = get(path, {**params, "$limit": PAGE, "$skip": skip})
        out.extend(page["data"])
        skip += PAGE
        print(f"  {path}: {min(skip, page['total'])}/{page['total']}", file=sys.stderr)
        if skip >= page["total"]:
            return out
        time.sleep(0.15)


def rust_str(s):
    return json.dumps(s, ensure_ascii=False)


# Zaaps: map hints named "Zaap".
zaaps = get("/hints", {"$limit": 100, "name.fr": "Zaap"})["data"]

# Subareas (name, area, associated zaap) and areas (super area = world).
subareas = {
    s["id"]: s
    for s in get_all("/subareas", {"$select[]": ["id", "areaId", "name", "associatedZaapMapId"]})
}
areas = {a["id"]: a for a in get_all("/areas", {"$select[]": ["id", "superAreaId"]})}


def super_area(subarea_id):
    sub = subareas.get(subarea_id)
    area = areas.get(sub["areaId"]) if sub else None
    return area["superAreaId"] if area else None


def area_of(subarea_id):
    return (subareas.get(subarea_id) or {}).get("areaId")


zaap_by_map = {z["mapId"]: z for z in zaaps}
zaaps_by_area, zaaps_by_super = {}, {}
for z in zaaps:
    zaaps_by_area.setdefault(area_of(z["subareaId"]), []).append(z)
    zaaps_by_super.setdefault(super_area(z["subareaId"]), []).append(z)


def nearest(candidates, x, y):
    return min(candidates, key=lambda z: (z["x"] - x) ** 2 + (z["y"] - y) ** 2, default=None)


def zaap_for(x, y, subarea_id):
    """Nearest zaap of the NPC's region (keeps islands on their own zaaps),
    else the zaap the game associates with the subarea, else the nearest of
    the same world. The associated zaap alone can be far: Forêt d'Amakna
    [3,12] points to Rivage sufokien [10,22] while [-1,13] is 4 maps away."""
    z = nearest(zaaps_by_area.get(area_of(subarea_id), []), x, y)
    if z:
        return z
    sub = subareas.get(subarea_id) or {}
    return zaap_by_map.get(sub.get("associatedZaapMapId")) or nearest(
        zaaps_by_super.get(super_area(subarea_id), []), x, y
    )


# NPC positions from quest objectives.
spots = set()
objectives = get_all("/quest-objectives", {"typeId[$in][]": NPC_OBJECTIVES, "mapId[$gt]": 0})
for o in objectives:
    m = o.get("map")
    npc = (o.get("parameters") or {}).get("parameter0")
    if m and npc:
        spots.add((npc, m["posX"], m["posY"], m["subAreaId"]))

names = {
    n["id"]: n["name"].get("fr", "").strip()
    for n in get_all("/npcs", {"$select[]": ["id", "name"]})
}

# Dungeon, event and some indoor maps carry coordinates unrelated to the
# world map (Plantala's Esprit du Bois at [10,10], 40 maps from any Pandala
# zaap): /travel cannot reach them. Real far-off places stay under 20 maps
# (Pandamonium, Pénates du Corbac), so anything beyond this is dropped.
MAX_ZAAP_DISTANCE = 25

rows = set()
for npc, x, y, sub in spots:
    name = names.get(npc, "")
    z = zaap_for(x, y, sub)
    if not name or not z or "{" in name:
        continue
    if (z["x"] - x) ** 2 + (z["y"] - y) ** 2 > MAX_ZAAP_DISTANCE**2:
        continue
    area = (subareas.get(sub) or {}).get("name", {}).get("fr", "")
    rows.add((name, area, x, y, z["x"], z["y"]))

rows = sorted(rows, key=lambda r: (r[0].lower(), r[1].lower(), r[2], r[3]))
print(f"{len(spots)} positions, {len(rows)} NPC entries", file=sys.stderr)
print("pub const NPCS: &[Npc] = &[")
for name, area, x, y, zx, zy in rows:
    print(
        f"    Npc {{ name: {rust_str(name)}, area: {rust_str(area)}, "
        f"x: {x}, y: {y}, zaap: ({zx}, {zy}) }},"
    )
print("];")
