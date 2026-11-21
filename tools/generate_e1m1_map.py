#!/usr/bin/env python3
"""
Extract authentic E1M1: Hangar level geometry, monsters, pickups, and doors
from doom1.wad and generate examples/doom/src/map_data.slx.
Adheres strictly to GL-0011 (zero single-letter variables).
"""

import math
import os
import struct


def resolve_texture_id(texture_name):
    lowered_name = texture_name.lower()
    if "doorblu" in lowered_name:
        return 8
    if "doorred" in lowered_name:
        return 9
    if "dooryel" in lowered_name:
        return 10
    if "bigdoor" in lowered_name:
        return 7
    if "door" in lowered_name:
        return 6
    if "exit" in lowered_name:
        return 4
    if "step" in lowered_name:
        return 3
    if "startan" in lowered_name:
        return 2
    if "comp" in lowered_name or "tek" in lowered_name:
        return 1
    if "sw1" in lowered_name or "switch" in lowered_name:
        return 5
    return 0


def main():
    candidate_wads = [
        "examples/doom/resources/doom1.wad",
        "/tmp/doom-remake-eef/src/resources/wads/doom1.wad",
        "/tmp/doom_remake/wads/doom1.wad",
    ]
    wad_path = None
    for candidate in candidate_wads:
        if os.path.exists(candidate):
            wad_path = candidate
            break

    if not wad_path:
        raise FileNotFoundError("Could not find doom1.wad in any search location.")

    with open(wad_path, "rb") as file_handle:
        header = file_handle.read(12)
        ident, num_lumps, dir_offset = struct.unpack("<4sII", header)
        file_handle.seek(dir_offset)
        lumps = {}
        for lump_index in range(num_lumps):
            entry = file_handle.read(16)
            file_pos, size, name = struct.unpack("<II8s", entry)
            name_str = name.decode("ascii", errors="ignore").rstrip("\x00")
            lumps[name_str] = (file_pos, size)

        def read_lump(lump_name):
            pos, sz = lumps[lump_name]
            file_handle.seek(pos)
            return file_handle.read(sz)

        vertex_data = read_lump("VERTEXES")
        linedef_data = read_lump("LINEDEFS")
        sidedef_data = read_lump("SIDEDEFS")
        sector_data = read_lump("SECTORS")
        thing_data = read_lump("THINGS")

    num_vertices = len(vertex_data) // 4
    vertices = [
        struct.unpack("<hh", vertex_data[idx * 4 : (idx + 1) * 4])
        for idx in range(num_vertices)
    ]

    num_sectors = len(sector_data) // 26
    sectors = []
    for sector_index in range(num_sectors):
        fh, ch, fp, cp, ll, st, tg = struct.unpack(
            "<hh8s8shhh", sector_data[sector_index * 26 : (sector_index + 1) * 26]
        )
        sectors.append(
            {
                "floor": fh,
                "ceil": ch,
                "floor_pic": fp.decode("ascii", errors="ignore")
                .rstrip("\x00")
                .lower(),
                "ceil_pic": cp.decode("ascii", errors="ignore")
                .rstrip("\x00")
                .lower(),
                "light": ll,
                "special": st,
                "tag": tg,
            }
        )

    num_sidedefs = len(sidedef_data) // 30
    sidedefs = []
    for sidedef_index in range(num_sidedefs):
        ox, oy, ut, lt, mt, sec_id = struct.unpack(
            "<hh8s8s8sh", sidedef_data[sidedef_index * 30 : (sidedef_index + 1) * 30]
        )
        sidedefs.append(
            {
                "offset_x": ox,
                "offset_y": oy,
                "upper": ut.decode("ascii", errors="ignore").rstrip("\x00").lower(),
                "lower": lt.decode("ascii", errors="ignore").rstrip("\x00").lower(),
                "middle": mt.decode("ascii", errors="ignore").rstrip("\x00").lower(),
                "sector": sec_id,
            }
        )

    num_linedefs = len(linedef_data) // 14
    walls = []
    scale_factor = 0.03125
    door_counter = 0

    for linedef_index in range(num_linedefs):
        v1_idx, v2_idx, flags, spec, tag, s1_idx, s2_idx = struct.unpack(
            "<hhhhhhh", linedef_data[linedef_index * 14 : (linedef_index + 1) * 14]
        )
        start_coord_x, start_coord_y = vertices[v1_idx]
        end_coord_x, end_coord_y = vertices[v2_idx]

        side1 = sidedefs[s1_idx]
        sec1 = sectors[side1["sector"]]
        is_door = spec in (1, 26, 27, 28, 31, 32, 33, 34, 117, 118)
        current_door_id = -1
        if is_door:
            current_door_id = door_counter
            door_counter = door_counter + 1

        if s2_idx == -1:  # One-sided solid wall
            tex = side1["middle"] if side1["middle"] != "-" else "brown1"
            walls.append(
                {
                    "x1": round(start_coord_x * scale_factor, 2),
                    "z1": round(-start_coord_y * scale_factor, 2),
                    "x2": round(end_coord_x * scale_factor, 2),
                    "z2": round(-end_coord_y * scale_factor, 2),
                    "floor": round(sec1["floor"] * scale_factor, 2),
                    "ceil": round(sec1["ceil"] * scale_factor, 2),
                    "texture_id": resolve_texture_id(tex),
                    "is_door": False,
                    "door_id": -1,
                    "key": 0,
                    "is_solid": True,
                }
            )
        else:  # Two-sided portal
            side2 = sidedefs[s2_idx]
            sec2 = sectors[side2["sector"]]
            if sec2["floor"] > sec1["floor"]:
                tex = side1["lower"] if side1["lower"] != "-" else side2["lower"]
                if tex == "-":
                    tex = "step2"
                walls.append(
                    {
                        "x1": round(start_coord_x * scale_factor, 2),
                        "z1": round(-start_coord_y * scale_factor, 2),
                        "x2": round(end_coord_x * scale_factor, 2),
                        "z2": round(-end_coord_y * scale_factor, 2),
                        "floor": round(sec1["floor"] * scale_factor, 2),
                        "ceil": round(sec2["floor"] * scale_factor, 2),
                        "texture_id": resolve_texture_id(tex),
                        "is_door": False,
                        "door_id": -1,
                        "key": 0,
                        "is_solid": (sec2["floor"] - sec1["floor"]) * scale_factor
                        > 0.8,
                    }
                )
            elif sec1["floor"] > sec2["floor"]:
                tex = side2["lower"] if side2["lower"] != "-" else side1["lower"]
                if tex == "-":
                    tex = "step2"
                walls.append(
                    {
                        "x1": round(start_coord_x * scale_factor, 2),
                        "z1": round(-start_coord_y * scale_factor, 2),
                        "x2": round(end_coord_x * scale_factor, 2),
                        "z2": round(-end_coord_y * scale_factor, 2),
                        "floor": round(sec2["floor"] * scale_factor, 2),
                        "ceil": round(sec1["floor"] * scale_factor, 2),
                        "texture_id": resolve_texture_id(tex),
                        "is_door": False,
                        "door_id": -1,
                        "key": 0,
                        "is_solid": (sec1["floor"] - sec2["floor"]) * scale_factor
                        > 0.8,
                    }
                )
            if sec1["ceil"] > sec2["ceil"]:
                tex = side1["upper"] if side1["upper"] != "-" else side2["upper"]
                if tex == "-":
                    tex = "brown1"
                walls.append(
                    {
                        "x1": round(start_coord_x * scale_factor, 2),
                        "z1": round(-start_coord_y * scale_factor, 2),
                        "x2": round(end_coord_x * scale_factor, 2),
                        "z2": round(-end_coord_y * scale_factor, 2),
                        "floor": round(sec2["ceil"] * scale_factor, 2),
                        "ceil": round(sec1["ceil"] * scale_factor, 2),
                        "texture_id": resolve_texture_id(tex),
                        "is_door": False,
                        "door_id": -1,
                        "key": 0,
                        "is_solid": False,
                    }
                )
            if is_door:
                tex = (
                    side1["middle"]
                    if side1["middle"] != "-"
                    else (
                        side2["middle"] if side2["middle"] != "-" else "bigdoor2"
                    )
                )
                key_required = (
                    1
                    if spec in (26, 32)
                    else (2 if spec in (28, 34) else (3 if spec in (27, 33) else 0))
                )
                walls.append(
                    {
                        "x1": round(start_coord_x * scale_factor, 2),
                        "z1": round(-start_coord_y * scale_factor, 2),
                        "x2": round(end_coord_x * scale_factor, 2),
                        "z2": round(-end_coord_y * scale_factor, 2),
                        "floor": round(
                            min(sec1["floor"], sec2["floor"]) * scale_factor, 2
                        ),
                        "ceil": round(
                            max(sec1["ceil"], sec2["ceil"]) * scale_factor, 2
                        ),
                        "texture_id": resolve_texture_id(tex),
                        "is_door": True,
                        "door_id": current_door_id,
                        "key": key_required,
                        "is_solid": True,
                    }
                )

    # Parse THINGS
    num_things = len(thing_data) // 10
    things = []
    for thing_index in range(num_things):
        th_x, th_y, th_ang, th_type, th_flags = struct.unpack(
            "<hhhhh", thing_data[thing_index * 10 : (thing_index + 1) * 10]
        )
        things.append((th_x, th_y, th_ang, th_type, th_flags))

    player_spawn = (3.0, 8.5, 0.0)
    for thing in things:
        if thing[3] == 1:
            player_spawn = (
                round(thing[0] * scale_factor, 2),
                round(-thing[1] * scale_factor, 2),
                float(thing[2]),
            )
            break

    monster_mapping = {3004: 0, 9: 1, 3001: 2, 3002: 3}
    pickup_mapping = {
        2001: 0,
        2002: 1,
        2005: 2,
        2007: 3,
        2008: 4,
        2011: 5,
        2012: 6,
        2014: 7,
        2015: 8,
        2018: 9,
        5: 10,
        13: 11,
        6: 12,
        2035: 13,
    }

    monsters = []
    pickups = []
    for thing in things:
        th_type = thing[3]
        if th_type in monster_mapping:
            monsters.append(
                (
                    monster_mapping[th_type],
                    round(thing[0] * scale_factor, 2),
                    round(-thing[1] * scale_factor, 2),
                )
            )
        elif th_type in pickup_mapping:
            pickups.append(
                (
                    pickup_mapping[th_type],
                    round(thing[0] * scale_factor, 2),
                    round(-thing[1] * scale_factor, 2),
                )
            )

    # Generate Solix source code
    total_walls = len(walls)
    chunk_size = 50
    total_chunks = (total_walls + chunk_size - 1) // chunk_size

    output_path = "examples/doom/src/map_data.slx"
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    with open(output_path, "w", encoding="utf-8") as out_file:
        out_file.write(
            """package doom;

public class MapDataE1M1 {
    public static float32 SPAWN_X = """
            + str(player_spawn[0])
            + """;
    public static float32 SPAWN_Y = 0.0;
    public static float32 SPAWN_Z = """
            + str(player_spawn[1])
            + """;
    public static float32 SPAWN_YAW = """
            + str(player_spawn[2])
            + """;

    public static int32 TOTAL_WALLS = """
            + str(total_walls)
            + """;
    public static int32 TOTAL_MONSTERS = """
            + str(len(monsters))
            + """;
    public static int32 TOTAL_PICKUPS = """
            + str(len(pickups))
            + """;

    public static void populate_level(LevelMap level_map) {
        level_map.walls = new WallSegment[MapDataE1M1::TOTAL_WALLS];
        level_map.wall_count = MapDataE1M1::TOTAL_WALLS;

        level_map.enemies = new EnemyMonster[MapDataE1M1::TOTAL_MONSTERS];
        level_map.enemy_count = MapDataE1M1::TOTAL_MONSTERS;

        level_map.pickups = new PickupItem[MapDataE1M1::TOTAL_PICKUPS];
        level_map.pickup_count = MapDataE1M1::TOTAL_PICKUPS;

        level_map.player_spawn_x = MapDataE1M1::SPAWN_X;
        level_map.player_spawn_y = MapDataE1M1::SPAWN_Y;
        level_map.player_spawn_z = MapDataE1M1::SPAWN_Z;
        level_map.player_spawn_yaw = MapDataE1M1::SPAWN_YAW;

"""
        )
        for chunk_index in range(total_chunks):
            out_file.write(
                f"        MapDataE1M1::populate_walls_chunk_{chunk_index}(level_map.walls);\n"
            )

        out_file.write(
            """        MapDataE1M1::populate_monsters(level_map.enemies);\n"""
        )
        out_file.write(
            """        MapDataE1M1::populate_pickups(level_map.pickups);\n"""
        )
        out_file.write("    }\n\n")

        # Chunk methods for walls
        for chunk_index in range(total_chunks):
            start_wall_index = chunk_index * chunk_size
            end_wall_index = min(start_wall_index + chunk_size, total_walls)
            out_file.write(
                f"    public static void populate_walls_chunk_{chunk_index}(WallSegment[] walls) {{\n"
            )
            for wall_idx in range(start_wall_index, end_wall_index):
                wall = walls[wall_idx]
                is_door_str = "true" if wall["is_door"] else "false"
                is_solid_str = "true" if wall["is_solid"] else "false"
                out_file.write(
                    f"        walls[{wall_idx}] = new WallSegment({wall['x1']}, {wall['z1']}, {wall['x2']}, {wall['z2']}, {wall['floor']}, {wall['ceil']}, {wall['texture_id']}, {is_door_str}, {wall['door_id']}, {is_solid_str}, {wall['key']});\n"
                )
            out_file.write("    }\n\n")

        # Monster population method
        out_file.write(
            "    public static void populate_monsters(EnemyMonster[] enemies) {\n"
        )
        for monster_index, (m_kind, m_x, m_z) in enumerate(monsters):
            out_file.write(
                f"        enemies[{monster_index}] = new EnemyMonster({m_kind}, {m_x}, 0.0, {m_z});\n"
            )
        out_file.write("    }\n\n")

        # Pickup population method
        out_file.write(
            "    public static void populate_pickups(PickupItem[] pickups) {\n"
        )
        for pickup_index, (p_kind, p_x, p_z) in enumerate(pickups):
            out_file.write(
                f"        pickups[{pickup_index}] = new PickupItem({p_kind}, {p_x}, 0.0, {p_z});\n"
            )
        out_file.write("    }\n")
        out_file.write("}\n")

    print(
        f"Generated {output_path} with {total_walls} walls, {len(monsters)} monsters, {len(pickups)} pickups."
    )


if __name__ == "__main__":
    main()
