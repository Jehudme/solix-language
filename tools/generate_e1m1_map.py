#!/usr/bin/env python3
"""
Extract authentic E1M1: Hangar level geometry, monsters, pickups, and doors
from doom1.wad and generate examples/doom/src/map_data.slx as well as
level_floors.obj/mtl and level_ceilings.obj/mtl.
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


def polygon_area(point_list):
    total_points = len(point_list)
    if total_points < 3:
        return 0.0
    accumulated_area = 0.0
    for point_idx in range(total_points):
        next_idx = (point_idx + 1) % total_points
        accumulated_area += (
            point_list[point_idx][0] * point_list[next_idx][1]
            - point_list[next_idx][0] * point_list[point_idx][1]
        )
    return accumulated_area * 0.5


def point_in_triangle(
    point_x, point_y, vert_ax, vert_ay, vert_bx, vert_by, vert_cx, vert_cy
):
    def edge_side(pos1_x, pos1_y, pos2_x, pos2_y, pos3_x, pos3_y):
        return (pos1_x - pos3_x) * (pos2_y - pos3_y) - (pos2_x - pos3_x) * (
            pos1_y - pos3_y
        )

    delta_side1 = edge_side(
        point_x, point_y, vert_ax, vert_ay, vert_bx, vert_by
    )
    delta_side2 = edge_side(
        point_x, point_y, vert_bx, vert_by, vert_cx, vert_cy
    )
    delta_side3 = edge_side(
        point_x, point_y, vert_cx, vert_cy, vert_ax, vert_ay
    )

    has_negative = (
        (delta_side1 < -1e-6) or (delta_side2 < -1e-6) or (delta_side3 < -1e-6)
    )
    has_positive = (
        (delta_side1 > 1e-6) or (delta_side2 > 1e-6) or (delta_side3 > 1e-6)
    )
    return not (has_negative and has_positive)


def triangulate_simple_polygon(polygon_vertices):
    remaining_points = list(polygon_vertices)
    if polygon_area(remaining_points) < 0:
        remaining_points.reverse()

    emitted_triangles = []
    iteration_guard = 0
    max_iterations = len(remaining_points) * 12

    while len(remaining_points) > 2 and iteration_guard < max_iterations:
        iteration_guard += 1
        num_points = len(remaining_points)
        triangle_clipped = False

        for current_idx in range(num_points):
            prev_idx = (current_idx - 1 + num_points) % num_points
            next_idx = (current_idx + 1) % num_points

            prev_pt = remaining_points[prev_idx]
            curr_pt = remaining_points[current_idx]
            next_pt = remaining_points[next_idx]

            cross_prod = (curr_pt[0] - prev_pt[0]) * (
                next_pt[1] - prev_pt[1]
            ) - (curr_pt[1] - prev_pt[1]) * (next_pt[0] - prev_pt[0])
            if cross_prod <= 1e-6:
                continue

            point_inside_triangle = False
            for test_idx in range(num_points):
                if test_idx in (prev_idx, current_idx, next_idx):
                    continue
                test_pt = remaining_points[test_idx]
                if point_in_triangle(
                    test_pt[0],
                    test_pt[1],
                    prev_pt[0],
                    prev_pt[1],
                    curr_pt[0],
                    curr_pt[1],
                    next_pt[0],
                    next_pt[1],
                ):
                    point_inside_triangle = True
                    break

            if not point_inside_triangle:
                emitted_triangles.append((prev_pt, curr_pt, next_pt))
                remaining_points.pop(current_idx)
                triangle_clipped = True
                break

        if not triangle_clipped:
            emitted_triangles.append(
                (
                    remaining_points[0],
                    remaining_points[1],
                    remaining_points[2],
                )
            )
            remaining_points.pop(1)

    return emitted_triangles


def merge_polygon_holes(loops_points):
    if not loops_points:
        return []

    sorted_loops = sorted(
        loops_points, key=lambda loop: abs(polygon_area(loop)), reverse=True
    )
    outer_polygon = list(sorted_loops[0])
    if polygon_area(outer_polygon) < 0:
        outer_polygon.reverse()

    hole_loops = sorted_loops[1:]
    for hole_loop in hole_loops:
        hole_points = list(hole_loop)
        if polygon_area(hole_points) > 0:
            hole_points.reverse()

        best_distance = 1e18
        best_outer_idx = 0
        best_hole_idx = 0

        for outer_idx, outer_vertex in enumerate(outer_polygon):
            for hole_idx, hole_vertex in enumerate(hole_points):
                distance_sq = (outer_vertex[0] - hole_vertex[0]) ** 2 + (
                    outer_vertex[1] - hole_vertex[1]
                ) ** 2
                if distance_sq < best_distance:
                    best_distance = distance_sq
                    best_outer_idx = outer_idx
                    best_hole_idx = hole_idx

        rotated_hole = (
            hole_points[best_hole_idx:] + hole_points[:best_hole_idx]
        )
        merged_points = (
            outer_polygon[: best_outer_idx + 1]
            + rotated_hole
            + [rotated_hole[0], outer_polygon[best_outer_idx]]
            + outer_polygon[best_outer_idx + 1 :]
        )
        outer_polygon = merged_points

    return triangulate_simple_polygon(outer_polygon)


def find_sector_for_coordinates(
    target_coord_x, target_coord_y, linedef_tuples, vertex_tuples, sidedef_sectors
):
    closest_distance = 1e18
    best_sector_index = 0

    for vert1_index, vert2_index, side1_index, side2_index in linedef_tuples:
        vert1_pos_x, vert1_pos_y = vertex_tuples[vert1_index]
        vert2_pos_x, vert2_pos_y = vertex_tuples[vert2_index]

        delta_x = vert2_pos_x - vert1_pos_x
        delta_y = vert2_pos_y - vert1_pos_y
        line_length_sq = delta_x * delta_x + delta_y * delta_y

        if line_length_sq == 0:
            distance_sq = (target_coord_x - vert1_pos_x) ** 2 + (
                target_coord_y - vert1_pos_y
            ) ** 2
            side_index = 0
        else:
            projection_param = max(
                0.0,
                min(
                    1.0,
                    (
                        (target_coord_x - vert1_pos_x) * delta_x
                        + (target_coord_y - vert1_pos_y) * delta_y
                    )
                    / line_length_sq,
                ),
            )
            projected_x = vert1_pos_x + projection_param * delta_x
            projected_y = vert1_pos_y + projection_param * delta_y
            distance_sq = (target_coord_x - projected_x) ** 2 + (
                target_coord_y - projected_y
            ) ** 2

            cross_product = delta_x * (target_coord_y - vert1_pos_y) - delta_y * (
                target_coord_x - vert1_pos_x
            )
            side_index = 0 if cross_product < 0 else 1

        if distance_sq < closest_distance:
            candidate_sector = -1
            if side_index == 0 and side1_index != -1:
                candidate_sector = sidedef_sectors[side1_index]
            elif side_index == 1 and side2_index != -1:
                candidate_sector = sidedef_sectors[side2_index]
            elif side1_index != -1:
                candidate_sector = sidedef_sectors[side1_index]

            if candidate_sector != -1:
                closest_distance = distance_sq
                best_sector_index = candidate_sector

    return best_sector_index


def assemble_directed_loops(directed_segments):
    unused_segments = list(directed_segments)
    assembled_loops = []

    while unused_segments:
        first_segment = unused_segments.pop(0)
        current_loop = [first_segment[0], first_segment[1]]

        while True:
            found_continuation = False
            for segment_idx, (start_vert, end_vert) in enumerate(
                unused_segments
            ):
                if start_vert == current_loop[-1]:
                    unused_segments.pop(segment_idx)
                    if end_vert != current_loop[0]:
                        current_loop.append(end_vert)
                    found_continuation = True
                    break
            if not found_continuation or current_loop[-1] == current_loop[0]:
                break

        assembled_loops.append(current_loop)

    return assembled_loops


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
        header_bytes = file_handle.read(12)
        ident_magic, num_lumps, dir_offset = struct.unpack(
            "<4sII", header_bytes
        )
        file_handle.seek(dir_offset)
        lumps_dictionary = {}
        for lump_idx in range(num_lumps):
            entry_record = file_handle.read(16)
            file_position, lump_size, lump_name = struct.unpack(
                "<II8s", entry_record
            )
            name_string = lump_name.decode("ascii", errors="ignore").rstrip(
                "\x00"
            )
            lumps_dictionary[name_string] = (file_position, lump_size)

        def read_named_lump(lump_label):
            position_offset, byte_size = lumps_dictionary[lump_label]
            file_handle.seek(position_offset)
            return file_handle.read(byte_size)

        vertex_raw_data = read_named_lump("VERTEXES")
        linedef_raw_data = read_named_lump("LINEDEFS")
        sidedef_raw_data = read_named_lump("SIDEDEFS")
        sector_raw_data = read_named_lump("SECTORS")
        thing_raw_data = read_named_lump("THINGS")

    num_vertices = len(vertex_raw_data) // 4
    vertices_list = [
        struct.unpack("<hh", vertex_raw_data[v_idx * 4 : (v_idx + 1) * 4])
        for v_idx in range(num_vertices)
    ]

    num_sectors = len(sector_raw_data) // 26
    sectors_list = []
    for sector_index in range(num_sectors):
        floor_h, ceil_h, floor_pic, ceil_pic, light_val, spec_tag, sector_tag = (
            struct.unpack(
                "<hh8s8shhh",
                sector_raw_data[sector_index * 26 : (sector_index + 1) * 26],
            )
        )
        sectors_list.append(
            {
                "floor": floor_h,
                "ceil": ceil_h,
                "floor_pic": floor_pic.decode("ascii", errors="ignore")
                .rstrip("\x00")
                .lower(),
                "ceil_pic": ceil_pic.decode("ascii", errors="ignore")
                .rstrip("\x00")
                .lower(),
                "light": light_val,
                "special": spec_tag,
                "tag": sector_tag,
            }
        )

    num_sidedefs = len(sidedef_raw_data) // 30
    sidedefs_list = []
    sidedef_sectors_map = []
    for sidedef_index in range(num_sidedefs):
        offset_x, offset_y, upper_tex, lower_tex, middle_tex, sector_ident = (
            struct.unpack(
                "<hh8s8s8sh",
                sidedef_raw_data[sidedef_index * 30 : (sidedef_index + 1) * 30],
            )
        )
        sidedefs_list.append(
            {
                "offset_x": offset_x,
                "offset_y": offset_y,
                "upper": upper_tex.decode("ascii", errors="ignore")
                .rstrip("\x00")
                .lower(),
                "lower": lower_tex.decode("ascii", errors="ignore")
                .rstrip("\x00")
                .lower(),
                "middle": middle_tex.decode("ascii", errors="ignore")
                .rstrip("\x00")
                .lower(),
                "sector": sector_ident,
            }
        )
        sidedef_sectors_map.append(sector_ident)

    num_linedefs = len(linedef_raw_data) // 14
    linedef_tuples_list = []
    walls_list = []
    scale_factor = 0.03125
    door_counter = 0

    sector_directed_segments = {
        sector_idx: [] for sector_idx in range(num_sectors)
    }

    for linedef_index in range(num_linedefs):
        vert1_idx, vert2_idx, line_flags, spec_code, line_tag, side1_idx, side2_idx = (
            struct.unpack(
                "<hhhhhhh",
                linedef_raw_data[linedef_index * 14 : (linedef_index + 1) * 14],
            )
        )
        linedef_tuples_list.append(
            (vert1_idx, vert2_idx, side1_idx, side2_idx)
        )

        start_pos_x, start_pos_y = vertices_list[vert1_idx]
        end_pos_x, end_pos_y = vertices_list[vert2_idx]

        side1_record = sidedefs_list[side1_idx]
        sec1_record = sectors_list[side1_record["sector"]]
        sector_directed_segments[side1_record["sector"]].append(
            (vert1_idx, vert2_idx)
        )

        is_door = spec_code in (1, 26, 27, 28, 31, 32, 33, 34, 117, 118)
        current_door_id = -1
        if is_door:
            current_door_id = door_counter
            door_counter = door_counter + 1

        if side2_idx == -1:  # One-sided solid wall
            wall_texture = (
                side1_record["middle"]
                if side1_record["middle"] != "-"
                else "brown1"
            )
            walls_list.append(
                {
                    "x1": round(start_pos_x * scale_factor, 2),
                    "z1": round(-start_pos_y * scale_factor, 2),
                    "x2": round(end_pos_x * scale_factor, 2),
                    "z2": round(-end_pos_y * scale_factor, 2),
                    "floor": round(sec1_record["floor"] * scale_factor, 2),
                    "ceil": round(sec1_record["ceil"] * scale_factor, 2),
                    "texture_id": resolve_texture_id(wall_texture),
                    "is_door": False,
                    "door_id": -1,
                    "key": 0,
                    "is_solid": True,
                }
            )
        else:  # Two-sided portal
            side2_record = sidedefs_list[side2_idx]
            sec2_record = sectors_list[side2_record["sector"]]
            sector_directed_segments[side2_record["sector"]].append(
                (vert2_idx, vert1_idx)
            )

            # Lower wall if sec2 floor is higher
            if sec2_record["floor"] > sec1_record["floor"]:
                wall_texture = (
                    side1_record["lower"]
                    if side1_record["lower"] != "-"
                    else side2_record["lower"]
                )
                if wall_texture == "-":
                    wall_texture = "step2"
                walls_list.append(
                    {
                        "x1": round(start_pos_x * scale_factor, 2),
                        "z1": round(-start_pos_y * scale_factor, 2),
                        "x2": round(end_pos_x * scale_factor, 2),
                        "z2": round(-end_pos_y * scale_factor, 2),
                        "floor": round(sec1_record["floor"] * scale_factor, 2),
                        "ceil": round(sec2_record["floor"] * scale_factor, 2),
                        "texture_id": resolve_texture_id(wall_texture),
                        "is_door": False,
                        "door_id": -1,
                        "key": 0,
                        "is_solid": (sec2_record["floor"] - sec1_record["floor"])
                        * scale_factor
                        > 0.8,
                    }
                )
            elif sec1_record["floor"] > sec2_record["floor"]:
                wall_texture = (
                    side2_record["lower"]
                    if side2_record["lower"] != "-"
                    else side1_record["lower"]
                )
                if wall_texture == "-":
                    wall_texture = "step2"
                walls_list.append(
                    {
                        "x1": round(start_pos_x * scale_factor, 2),
                        "z1": round(-start_pos_y * scale_factor, 2),
                        "x2": round(end_pos_x * scale_factor, 2),
                        "z2": round(-end_pos_y * scale_factor, 2),
                        "floor": round(sec2_record["floor"] * scale_factor, 2),
                        "ceil": round(sec1_record["floor"] * scale_factor, 2),
                        "texture_id": resolve_texture_id(wall_texture),
                        "is_door": False,
                        "door_id": -1,
                        "key": 0,
                        "is_solid": (sec1_record["floor"] - sec2_record["floor"])
                        * scale_factor
                        > 0.8,
                    }
                )

            # Upper wall if sec1 ceil is higher
            if sec1_record["ceil"] > sec2_record["ceil"]:
                wall_texture = (
                    side1_record["upper"]
                    if side1_record["upper"] != "-"
                    else side2_record["upper"]
                )
                if wall_texture == "-":
                    wall_texture = "brown1"
                walls_list.append(
                    {
                        "x1": round(start_pos_x * scale_factor, 2),
                        "z1": round(-start_pos_y * scale_factor, 2),
                        "x2": round(end_pos_x * scale_factor, 2),
                        "z2": round(-end_pos_y * scale_factor, 2),
                        "floor": round(sec2_record["ceil"] * scale_factor, 2),
                        "ceil": round(sec1_record["ceil"] * scale_factor, 2),
                        "texture_id": resolve_texture_id(wall_texture),
                        "is_door": False,
                        "door_id": -1,
                        "key": 0,
                        "is_solid": False,
                    }
                )
            elif sec2_record["ceil"] > sec1_record["ceil"]:
                wall_texture = (
                    side2_record["upper"]
                    if side2_record["upper"] != "-"
                    else side1_record["upper"]
                )
                if wall_texture == "-":
                    wall_texture = "brown1"
                walls_list.append(
                    {
                        "x1": round(start_pos_x * scale_factor, 2),
                        "z1": round(-start_pos_y * scale_factor, 2),
                        "x2": round(end_pos_x * scale_factor, 2),
                        "z2": round(-end_pos_y * scale_factor, 2),
                        "floor": round(sec1_record["ceil"] * scale_factor, 2),
                        "ceil": round(sec2_record["ceil"] * scale_factor, 2),
                        "texture_id": resolve_texture_id(wall_texture),
                        "is_door": False,
                        "door_id": -1,
                        "key": 0,
                        "is_solid": False,
                    }
                )

            # Door definition
            if is_door:
                door_texture = (
                    side1_record["middle"]
                    if side1_record["middle"] != "-"
                    else (
                        side2_record["middle"]
                        if side2_record["middle"] != "-"
                        else "bigdoor2"
                    )
                )
                key_required = (
                    1
                    if spec_code in (26, 32)
                    else (
                        2
                        if spec_code in (28, 34)
                        else (3 if spec_code in (27, 33) else 0)
                    )
                )
                walls_list.append(
                    {
                        "x1": round(start_pos_x * scale_factor, 2),
                        "z1": round(-start_pos_y * scale_factor, 2),
                        "x2": round(end_pos_x * scale_factor, 2),
                        "z2": round(-end_pos_y * scale_factor, 2),
                        "floor": round(
                            min(sec1_record["floor"], sec2_record["floor"])
                            * scale_factor,
                            2,
                        ),
                        "ceil": round(
                            max(sec1_record["ceil"], sec2_record["ceil"])
                            * scale_factor,
                            2,
                        ),
                        "texture_id": resolve_texture_id(door_texture),
                        "is_door": True,
                        "door_id": current_door_id,
                        "key": key_required,
                        "is_solid": True,
                    }
                )

    # Parse THINGS
    num_things = len(thing_raw_data) // 10
    things_list = []
    for thing_index in range(num_things):
        thing_x, thing_y, thing_yaw, thing_kind, thing_flags = struct.unpack(
            "<hhhhh", thing_raw_data[thing_index * 10 : (thing_index + 1) * 10]
        )
        things_list.append(
            (thing_x, thing_y, thing_yaw, thing_kind, thing_flags)
        )

    player_spawn = (3.0, 6.25, 8.5, 0.0)
    for thing in things_list:
        if thing[3] == 1:
            player_sector_idx = find_sector_for_coordinates(
                thing[0],
                thing[1],
                linedef_tuples_list,
                vertices_list,
                sidedef_sectors_map,
            )
            player_floor_elevation = round(
                sectors_list[player_sector_idx]["floor"] * scale_factor, 2
            )
            player_spawn = (
                round(thing[0] * scale_factor, 2),
                player_floor_elevation,
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

    monsters_list = []
    pickups_list = []
    for thing in things_list:
        thing_kind = thing[3]
        thing_sec_idx = find_sector_for_coordinates(
            thing[0],
            thing[1],
            linedef_tuples_list,
            vertices_list,
            sidedef_sectors_map,
        )
        thing_floor_elevation = round(
            sectors_list[thing_sec_idx]["floor"] * scale_factor, 2
        )

        if thing_kind in monster_mapping:
            monsters_list.append(
                (
                    monster_mapping[thing_kind],
                    round(thing[0] * scale_factor, 2),
                    thing_floor_elevation,
                    round(-thing[1] * scale_factor, 2),
                )
            )
        elif thing_kind in pickup_mapping:
            pickups_list.append(
                (
                    pickup_mapping[thing_kind],
                    round(thing[0] * scale_factor, 2),
                    thing_floor_elevation,
                    round(-thing[1] * scale_factor, 2),
                )
            )

    # 1. Generate 3D Sector Floor and Ceiling Meshes
    models_directory = "examples/doom/resources/models"
    os.makedirs(models_directory, exist_ok=True)

    floor_obj_lines = ["mtllib level_floors.mtl\n", "vn 0.0 1.0 0.0\n"]
    ceil_obj_lines = ["mtllib level_ceilings.mtl\n", "vn 0.0 -1.0 0.0\n"]

    floor_materials_set = set()
    ceil_materials_set = set()

    floor_vert_counter = 1
    ceil_vert_counter = 1

    for sector_num in range(num_sectors):
        sec_info = sectors_list[sector_num]
        sector_loops = assemble_directed_loops(
            sector_directed_segments[sector_num]
        )
        if not sector_loops:
            continue
        loops_vertex_coords = [
            [vertices_list[v_idx] for v_idx in loop_indices]
            for loop_indices in sector_loops
            if len(loop_indices) >= 3
        ]
        if not loops_vertex_coords:
            continue

        sector_triangles = merge_polygon_holes(loops_vertex_coords)
        if not sector_triangles:
            continue

        sec_floor_y = round(sec_info["floor"] * scale_factor, 3)
        sec_ceil_y = round(sec_info["ceil"] * scale_factor, 3)
        floor_pic_name = sec_info["floor_pic"]
        ceil_pic_name = sec_info["ceil_pic"]

        # Floor Triangles
        floor_obj_lines.append(f"usemtl {floor_pic_name}\n")
        floor_materials_set.add(floor_pic_name)
        for tri_p1, tri_p2, tri_p3 in sector_triangles:
            v1_x, v1_z = round(tri_p1[0] * scale_factor, 3), round(
                -tri_p1[1] * scale_factor, 3
            )
            v2_x, v2_z = round(tri_p2[0] * scale_factor, 3), round(
                -tri_p2[1] * scale_factor, 3
            )
            v3_x, v3_z = round(tri_p3[0] * scale_factor, 3), round(
                -tri_p3[1] * scale_factor, 3
            )

            uv1_u, uv1_v = round(v1_x / 2.0, 3), round(v1_z / 2.0, 3)
            uv2_u, uv2_v = round(v2_x / 2.0, 3), round(v2_z / 2.0, 3)
            uv3_u, uv3_v = round(v3_x / 2.0, 3), round(v3_z / 2.0, 3)

            floor_obj_lines.append(f"v {v1_x} {sec_floor_y} {v1_z}\n")
            floor_obj_lines.append(f"v {v2_x} {sec_floor_y} {v2_z}\n")
            floor_obj_lines.append(f"v {v3_x} {sec_floor_y} {v3_z}\n")
            floor_obj_lines.append(f"vt {uv1_u} {uv1_v}\n")
            floor_obj_lines.append(f"vt {uv2_u} {uv2_v}\n")
            floor_obj_lines.append(f"vt {uv3_u} {uv3_v}\n")
            floor_obj_lines.append(
                f"f {floor_vert_counter}/{floor_vert_counter}/1 {floor_vert_counter+1}/{floor_vert_counter+1}/1 {floor_vert_counter+2}/{floor_vert_counter+2}/1\n"
            )
            floor_vert_counter += 3

        # Ceiling Triangles
        ceil_obj_lines.append(f"usemtl {ceil_pic_name}\n")
        ceil_materials_set.add(ceil_pic_name)
        for tri_p1, tri_p2, tri_p3 in sector_triangles:
            v1_x, v1_z = round(tri_p1[0] * scale_factor, 3), round(
                -tri_p1[1] * scale_factor, 3
            )
            v2_x, v2_z = round(tri_p2[0] * scale_factor, 3), round(
                -tri_p2[1] * scale_factor, 3
            )
            v3_x, v3_z = round(tri_p3[0] * scale_factor, 3), round(
                -tri_p3[1] * scale_factor, 3
            )

            uv1_u, uv1_v = round(v1_x / 2.0, 3), round(v1_z / 2.0, 3)
            uv2_u, uv2_v = round(v2_x / 2.0, 3), round(v2_z / 2.0, 3)
            uv3_u, uv3_v = round(v3_x / 2.0, 3), round(v3_z / 2.0, 3)

            ceil_obj_lines.append(f"v {v1_x} {sec_ceil_y} {v1_z}\n")
            ceil_obj_lines.append(f"v {v2_x} {sec_ceil_y} {v2_z}\n")
            ceil_obj_lines.append(f"v {v3_x} {sec_ceil_y} {v3_z}\n")
            ceil_obj_lines.append(f"vt {uv1_u} {uv1_v}\n")
            ceil_obj_lines.append(f"vt {uv2_u} {uv2_v}\n")
            ceil_obj_lines.append(f"vt {uv3_u} {uv3_v}\n")
            ceil_obj_lines.append(
                f"f {ceil_vert_counter}/{ceil_vert_counter}/1 {ceil_vert_counter+2}/{ceil_vert_counter+2}/1 {ceil_vert_counter+1}/{ceil_vert_counter+1}/1\n"
            )
            ceil_vert_counter += 3

    floors_obj_filepath = os.path.join(models_directory, "level_floors.obj")
    with open(floors_obj_filepath, "w", encoding="utf-8") as file_stream:
        file_stream.writelines(floor_obj_lines)

    ceilings_obj_filepath = os.path.join(models_directory, "level_ceilings.obj")
    with open(ceilings_obj_filepath, "w", encoding="utf-8") as file_stream:
        file_stream.writelines(ceil_obj_lines)

    def write_mtl_material_file(material_filepath, materials_collection):
        with open(material_filepath, "w", encoding="utf-8") as file_stream:
            for material_name in sorted(materials_collection):
                texture_filename = f"{material_name}.png"
                texture_check_path = os.path.join(
                    "examples/doom/resources/textures", texture_filename
                )
                if not os.path.exists(texture_check_path):
                    texture_filename = "floor4_8.png"
                file_stream.write(f"newmtl {material_name}\n")
                file_stream.write(f"map_Kd ../textures/{texture_filename}\n")
                file_stream.write("Ka 1.0 1.0 1.0\n")
                file_stream.write("Kd 1.0 1.0 1.0\n")
                file_stream.write("Ks 0.0 0.0 0.0\n\n")

    write_mtl_material_file(
        os.path.join(models_directory, "level_floors.mtl"), floor_materials_set
    )
    write_mtl_material_file(
        os.path.join(models_directory, "level_ceilings.mtl"), ceil_materials_set
    )

    # 2. Generate Solix source code
    total_walls = len(walls_list)
    chunk_size = 50
    total_chunks = (total_walls + chunk_size - 1) // chunk_size

    output_path = "examples/doom/src/map_data.slx"
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    with open(output_path, "w", encoding="utf-8") as out_file:
        out_file.write(
            f"""package doom;

public class MapDataE1M1 {{
    public static float32 SPAWN_X = {player_spawn[0]};
    public static float32 SPAWN_Y = {player_spawn[1]};
    public static float32 SPAWN_Z = {player_spawn[2]};
    public static float32 SPAWN_YAW = {player_spawn[3]};

    public static int32 TOTAL_WALLS = {total_walls};
    public static int32 TOTAL_MONSTERS = {len(monsters_list)};
    public static int32 TOTAL_PICKUPS = {len(pickups_list)};

    public static void populate_level(LevelMap level_map) {{
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
                wall_entry = walls_list[wall_idx]
                is_door_str = "true" if wall_entry["is_door"] else "false"
                is_solid_str = "true" if wall_entry["is_solid"] else "false"
                out_file.write(
                    f"        walls[{wall_idx}] = new WallSegment({wall_entry['x1']}, {wall_entry['z1']}, {wall_entry['x2']}, {wall_entry['z2']}, {wall_entry['floor']}, {wall_entry['ceil']}, {wall_entry['texture_id']}, {is_door_str}, {wall_entry['door_id']}, {is_solid_str}, {wall_entry['key']});\n"
                )
            out_file.write("    }\n\n")

        # Monster population method
        out_file.write(
            "    public static void populate_monsters(EnemyMonster[] enemies) {\n"
        )
        for monster_index, (m_kind, m_x, m_y, m_z) in enumerate(monsters_list):
            out_file.write(
                f"        enemies[{monster_index}] = new EnemyMonster({m_kind}, {m_x}, {m_y}, {m_z});\n"
            )
        out_file.write("    }\n\n")

        # Pickup population method
        out_file.write(
            "    public static void populate_pickups(PickupItem[] pickups) {\n"
        )
        for pickup_index, (p_kind, p_x, p_y, p_z) in enumerate(pickups_list):
            out_file.write(
                f"        pickups[{pickup_index}] = new PickupItem({p_kind}, {p_x}, {p_y}, {p_z});\n"
            )
        out_file.write("    }\n")
        out_file.write("}\n")

    print(
        f"Generated {output_path} with {total_walls} walls, {len(monsters_list)} monsters, {len(pickups_list)} pickups."
    )


if __name__ == "__main__":
    main()
