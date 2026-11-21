#!/usr/bin/env python3
"""
Generate 3D Wavefront OBJ and MTL models for DOOM textured walls, floors, and ceilings.
Adheres strictly to GL-0011 (zero single-letter variables).
"""

import os


def generate_cube_obj(output_obj_path, material_filename, material_name):
    content = f"""mtllib {material_filename}
usemtl {material_name}
v -0.5 -0.5  0.5
v  0.5 -0.5  0.5
v  0.5  0.5  0.5
v -0.5  0.5  0.5
v -0.5 -0.5 -0.5
v  0.5 -0.5 -0.5
v  0.5  0.5 -0.5
v -0.5  0.5 -0.5
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
vn  0.0  0.0  1.0
vn  0.0  0.0 -1.0
vn  0.0  1.0  0.0
vn  0.0 -1.0  0.0
vn  1.0  0.0  0.0
vn -1.0  0.0  0.0
f 1/1/1 2/2/1 3/3/1
f 1/1/1 3/3/1 4/4/1
f 6/1/2 5/2/2 8/3/2
f 6/1/2 8/3/2 7/4/2
f 4/1/3 3/2/3 7/3/3
f 4/1/3 7/3/3 8/4/3
f 5/1/4 6/2/4 2/3/4
f 5/1/4 2/3/4 1/4/4
f 2/1/5 6/2/5 7/3/5
f 2/1/5 7/3/5 3/4/5
f 5/1/6 1/2/6 4/3/6
f 5/1/6 4/3/6 8/4/6
"""
    with open(output_obj_path, "w", encoding="utf-8") as file_stream:
        file_stream.write(content)


def generate_plane_obj(output_obj_path, material_filename, material_name, is_floor_plane):
    normal_elevation = 1.0 if is_floor_plane else -1.0
    content = f"""mtllib {material_filename}
usemtl {material_name}
v -0.5 0.0 -0.5
v  0.5 0.0 -0.5
v  0.5 0.0  0.5
v -0.5 0.0  0.5
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
vn 0.0 {normal_elevation} 0.0
"""
    if is_floor_plane:
        content += "f 1/1/1 2/2/1 3/3/1\nf 1/1/1 3/3/1 4/4/1\n"
    else:
        content += "f 1/1/1 3/3/1 2/2/1\nf 1/1/1 4/4/1 3/3/1\n"

    with open(output_obj_path, "w", encoding="utf-8") as file_stream:
        file_stream.write(content)


def generate_mtl(output_mtl_path, material_name, texture_relative_path):
    content = f"""newmtl {material_name}
map_Kd {texture_relative_path}
Ka 1.0 1.0 1.0
Kd 1.0 1.0 1.0
Ks 0.0 0.0 0.0
"""
    with open(output_mtl_path, "w", encoding="utf-8") as file_stream:
        file_stream.write(content)


def main():
    models_dir = "examples/doom/resources/models"
    os.makedirs(models_dir, exist_ok=True)

    cube_configs = [
        ("wall_brown", "WallBrown", "../textures/brown1.png"),
        ("wall_tech", "WallTech", "../textures/comp2.png"),
        ("wall_startan", "WallStartan", "../textures/startan1.png"),
        ("wall_step", "WallStep", "../textures/step2.png"),
        ("wall_exitsign", "WallExitSign", "../textures/exitsign.png"),
        ("wall_switch", "WallSwitch", "../textures/sw1brcom.png"),
        ("door_normal", "DoorNormal", "../textures/door3.png"),
        ("door_big", "DoorBig", "../textures/bigdoor2.png"),
        ("door_blue", "DoorBlue", "../textures/doorblu.png"),
        ("door_red", "DoorRed", "../textures/doorred.png"),
        ("door_yellow", "DoorYellow", "../textures/dooryel.png"),
    ]

    for model_name, mat_name, tex_path in cube_configs:
        obj_path = os.path.join(models_dir, f"{model_name}.obj")
        mtl_path = os.path.join(models_dir, f"{model_name}.mtl")
        generate_mtl(mtl_path, mat_name, tex_path)
        generate_cube_obj(obj_path, f"{model_name}.mtl", mat_name)

    plane_configs = [
        ("floor_flat", "FloorFlat", "../textures/floor4_8.png", True),
        ("ceil_flat", "CeilFlat", "../textures/ceil3_5.png", False),
        ("acid_flat", "AcidFlat", "../textures/nukage3.png", True),
    ]

    for model_name, mat_name, tex_path, is_floor_flag in plane_configs:
        obj_path = os.path.join(models_dir, f"{model_name}.obj")
        mtl_path = os.path.join(models_dir, f"{model_name}.mtl")
        generate_mtl(mtl_path, mat_name, tex_path)
        generate_plane_obj(obj_path, f"{model_name}.mtl", mat_name, is_floor_flag)

    print(f"Generated {len(cube_configs) + len(plane_configs)} 3D models in {models_dir}")


if __name__ == "__main__":
    main()
