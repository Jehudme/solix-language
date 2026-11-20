#!/usr/bin/env python3
"""
DOOM Asset Extractor: Extracts authentic textures, billboard sprites,
HUD weapon overlays, status bar elements, and PCM WAV audio from doom1.wad.
Uses pure Python standard library (zlib, struct, wave) for instantaneous extraction.
Adheres strictly to GL-0011 (zero single-letter variables).
"""

import os
import struct
import wave
import zlib


def write_png(image_width, image_height, raw_rgba_bytes, target_png_path):
    os.makedirs(os.path.dirname(target_png_path), exist_ok=True)

    def create_png_chunk(chunk_type_bytes, chunk_payload_bytes):
        chunk_length = len(chunk_payload_bytes)
        crc_checksum = zlib.crc32(chunk_type_bytes + chunk_payload_bytes) & 0xFFFFFFFF
        return (
            struct.pack(">I", chunk_length)
            + chunk_type_bytes
            + chunk_payload_bytes
            + struct.pack(">I", crc_checksum)
        )

    png_signature = b"\x89PNG\r\n\x1a\n"
    ihdr_payload = struct.pack(">IIBBBBB", image_width, image_height, 8, 6, 0, 0, 0)
    ihdr_chunk = create_png_chunk(b"IHDR", ihdr_payload)

    scanlines_buffer = bytearray()
    row_stride = image_width * 4
    for row_index in range(image_height):
        scanlines_buffer.append(0)  # Filter type: None
        row_start_offset = row_index * row_stride
        scanlines_buffer.extend(raw_rgba_bytes[row_start_offset : row_start_offset + row_stride])

    compressed_idat = zlib.compress(bytes(scanlines_buffer), level=6)
    idat_chunk = create_png_chunk(b"IDAT", compressed_idat)
    iend_chunk = create_png_chunk(b"IEND", b"")

    with open(target_png_path, "wb") as output_stream:
        output_stream.write(png_signature + ihdr_chunk + idat_chunk + iend_chunk)


def read_wad_directory(wad_file_path):
    with open(wad_file_path, "rb") as file_handle:
        wad_ident, total_lumps, dir_offset = struct.unpack("<4sII", file_handle.read(12))
        file_handle.seek(dir_offset)
        lump_records = {}
        ordered_lumps = []
        for lump_index in range(total_lumps):
            lump_pos, lump_size, lump_name_raw = struct.unpack("<II8s", file_handle.read(16))
            clean_name = lump_name_raw.split(b"\x00")[0].decode("ascii", errors="ignore").upper()
            lump_records[clean_name] = (lump_pos, lump_size)
            ordered_lumps.append((clean_name, lump_pos, lump_size))
        return lump_records, ordered_lumps


def decode_doom_picture(file_handle, lump_position, lump_byte_count, palette_bytes):
    if lump_byte_count < 8:
        return None
    file_handle.seek(lump_position)
    raw_data = file_handle.read(lump_byte_count)
    if len(raw_data) < 8:
        return None
    image_width, image_height, left_offset, top_offset = struct.unpack("<HHhh", raw_data[:8])
    if image_width == 0 or image_height == 0 or image_width > 2048 or image_height > 2048:
        return None

    header_required_size = 8 + image_width * 4
    if len(raw_data) < header_required_size:
        return None

    column_offsets = struct.unpack(f"<{image_width}I", raw_data[8:header_required_size])
    pixel_grid = bytearray([0] * (image_width * image_height * 4))

    for col_index in range(image_width):
        curr_offset = column_offsets[col_index]
        while curr_offset < len(raw_data):
            top_delta = raw_data[curr_offset]
            if top_delta == 255:
                break
            if curr_offset + 2 >= len(raw_data):
                break
            pixel_count = raw_data[curr_offset + 1]
            pixel_start = curr_offset + 3
            if pixel_start + pixel_count > len(raw_data):
                break
            for row_sub_index in range(pixel_count):
                coord_y = top_delta + row_sub_index
                if coord_y < image_height:
                    palette_index = raw_data[pixel_start + row_sub_index]
                    color_red = palette_bytes[palette_index * 3]
                    color_green = palette_bytes[palette_index * 3 + 1]
                    color_blue = palette_bytes[palette_index * 3 + 2]
                    dest_byte_offset = (coord_y * image_width + col_index) * 4
                    pixel_grid[dest_byte_offset] = color_red
                    pixel_grid[dest_byte_offset + 1] = color_green
                    pixel_grid[dest_byte_offset + 2] = color_blue
                    pixel_grid[dest_byte_offset + 3] = 255
            curr_offset += 4 + pixel_count

    return image_width, image_height, bytes(pixel_grid)


def extract_flats(file_handle, ordered_lumps, palette_bytes, output_directory):
    inside_flats_marker = False
    extracted_count = 0
    for clean_name, lump_pos, lump_size in ordered_lumps:
        if clean_name in ("F_START", "F1_START"):
            inside_flats_marker = True
            continue
        if clean_name in ("F_END", "F1_END"):
            inside_flats_marker = False
            continue
        if inside_flats_marker and lump_size == 4096:
            file_handle.seek(lump_pos)
            flat_bytes = file_handle.read(4096)
            flat_rgba = bytearray(64 * 64 * 4)
            for pixel_idx in range(4096):
                palette_idx = flat_bytes[pixel_idx]
                flat_rgba[pixel_idx * 4] = palette_bytes[palette_idx * 3]
                flat_rgba[pixel_idx * 4 + 1] = palette_bytes[palette_idx * 3 + 1]
                flat_rgba[pixel_idx * 4 + 2] = palette_bytes[palette_idx * 3 + 2]
                flat_rgba[pixel_idx * 4 + 3] = 255
            write_png(64, 64, bytes(flat_rgba), os.path.join(output_directory, f"{clean_name.lower()}.png"))
            extracted_count += 1
    return extracted_count


def extract_audio_sounds(file_handle, lump_records, output_directory):
    extracted_count = 0
    for clean_name, (lump_pos, lump_size) in lump_records.items():
        if clean_name.startswith("DS") and lump_size > 8:
            file_handle.seek(lump_pos)
            sound_header = file_handle.read(8)
            audio_format, sample_rate, sample_count = struct.unpack("<HHI", sound_header)
            if sample_count > 0 and sample_count <= (lump_size - 8):
                raw_pcm_samples = file_handle.read(sample_count)
                sound_file_path = os.path.join(output_directory, f"{clean_name.lower()}.wav")
                with wave.open(sound_file_path, "wb") as wave_file:
                    wave_file.setnchannels(1)
                    wave_file.setsampwidth(1)
                    wave_file.setframerate(sample_rate)
                    wave_file.writeframes(raw_pcm_samples)
                extracted_count += 1
    return extracted_count


def extract_composite_textures(file_handle, lump_records, palette_bytes, output_directory):
    if "PNAMES" not in lump_records or "TEXTURE1" not in lump_records:
        return 0

    pnames_pos, pnames_size = lump_records["PNAMES"]
    file_handle.seek(pnames_pos)
    total_pnames = struct.unpack("<I", file_handle.read(4))[0]
    pnames_list = []
    for pname_index in range(total_pnames):
        patch_entry_name = file_handle.read(8).split(b"\x00")[0].decode("ascii", errors="ignore").upper()
        pnames_list.append(patch_entry_name)

    tex_pos, tex_size = lump_records["TEXTURE1"]
    file_handle.seek(tex_pos)
    total_textures = struct.unpack("<I", file_handle.read(4))[0]
    texture_offsets = struct.unpack(f"<{total_textures}I", file_handle.read(4 * total_textures))

    cached_patches = {}

    def get_patch_pixels(patch_target_name):
        if patch_target_name in cached_patches:
            return cached_patches[patch_target_name]
        if patch_target_name not in lump_records:
            return None
        patch_pos, patch_sz = lump_records[patch_target_name]
        decoded_info = decode_doom_picture(file_handle, patch_pos, patch_sz, palette_bytes)
        cached_patches[patch_target_name] = decoded_info
        return decoded_info

    extracted_count = 0
    for texture_offset in texture_offsets:
        file_handle.seek(tex_pos + texture_offset)
        raw_tex_name, masked_flag, tex_w, tex_h, col_dir, patch_count = struct.unpack("<8siHHiH", file_handle.read(22))
        clean_texture_name = raw_tex_name.split(b"\x00")[0].decode("ascii", errors="ignore").upper()

        texture_rgba = bytearray([0] * (tex_w * tex_h * 4))

        for patch_sub_index in range(patch_count):
            origin_x, origin_y, pnames_index, step_dir, color_map = struct.unpack("<hhHHh", file_handle.read(10))
            if pnames_index < len(pnames_list):
                patch_id_name = pnames_list[pnames_index]
                patch_decoded = get_patch_pixels(patch_id_name)
                if patch_decoded is not None:
                    patch_width, patch_height, patch_rgba = patch_decoded
                    for row_pos in range(patch_height):
                        dest_y = origin_y + row_pos
                        if 0 <= dest_y < tex_h:
                            for col_pos in range(patch_width):
                                dest_x = origin_x + col_pos
                                if 0 <= dest_x < tex_w:
                                    src_offset = (row_pos * patch_width + col_pos) * 4
                                    alpha_val = patch_rgba[src_offset + 3]
                                    if alpha_val > 0:
                                        target_offset = (dest_y * tex_w + dest_x) * 4
                                        texture_rgba[target_offset] = patch_rgba[src_offset]
                                        texture_rgba[target_offset + 1] = patch_rgba[src_offset + 1]
                                        texture_rgba[target_offset + 2] = patch_rgba[src_offset + 2]
                                        texture_rgba[target_offset + 3] = 255

        write_png(
            tex_w,
            tex_h,
            bytes(texture_rgba),
            os.path.join(output_directory, f"{clean_texture_name.lower()}.png"),
        )
        extracted_count += 1

    return extracted_count


def extract_doom_sprites(file_handle, ordered_lumps, palette_bytes, base_directory):
    inside_sprites_marker = False
    extracted_count = 0

    sprites_directory = os.path.join(base_directory, "sprites")
    weapons_directory = os.path.join(base_directory, "weapons")
    hud_directory = os.path.join(base_directory, "hud")

    for clean_name, lump_pos, lump_size in ordered_lumps:
        if clean_name in ("S_START", "S1_START"):
            inside_sprites_marker = True
            continue
        if clean_name in ("S_END", "S1_END"):
            inside_sprites_marker = False
            continue

        is_hud_element = clean_name.startswith("ST") or clean_name.startswith("NUM")
        is_weapon_hud = any(clean_name.startswith(weapon_pfx) for weapon_pfx in ["PISG", "SHTG", "CHGG", "MISG", "PUNG"])
        is_relevant_sprite = inside_sprites_marker or is_hud_element or is_weapon_hud

        if is_relevant_sprite and lump_size > 8:
            picture_decoded = decode_doom_picture(file_handle, lump_pos, lump_size, palette_bytes)
            if picture_decoded is not None:
                img_width, img_height, rgba_buffer = picture_decoded
                if is_weapon_hud:
                    target_dir = weapons_directory
                elif is_hud_element:
                    target_dir = hud_directory
                else:
                    target_dir = sprites_directory

                dest_file_path = os.path.join(target_dir, f"{clean_name.lower()}.png")
                write_png(img_width, img_height, rgba_buffer, dest_file_path)
                extracted_count += 1

    return extracted_count


def main():
    wad_source_path = "/tmp/doom_remake/src/resources/wads/doom1.wad"
    destination_resources_root = "examples/doom/resources"

    print(f"Reading WAD file from {wad_source_path}...")
    lump_records, ordered_lumps = read_wad_directory(wad_source_path)

    if "PLAYPAL" not in lump_records:
        raise RuntimeError("PLAYPAL lump not found in WAD!")

    with open(wad_source_path, "rb") as file_handle:
        pal_pos, pal_size = lump_records["PLAYPAL"]
        file_handle.seek(pal_pos)
        palette_bytes = file_handle.read(768)

        print("Extracting flats (floors/ceilings)...")
        flats_count = extract_flats(
            file_handle,
            ordered_lumps,
            palette_bytes,
            os.path.join(destination_resources_root, "textures"),
        )
        print(f"Extracted {flats_count} flats.")

        print("Extracting composite wall textures...")
        textures_count = extract_composite_textures(
            file_handle,
            lump_records,
            palette_bytes,
            os.path.join(destination_resources_root, "textures"),
        )
        print(f"Extracted {textures_count} wall textures.")

        print("Extracting monster sprites, weapons, and HUD...")
        sprites_count = extract_doom_sprites(
            file_handle,
            ordered_lumps,
            palette_bytes,
            destination_resources_root,
        )
        print(f"Extracted {sprites_count} sprites/weapons/HUD graphics.")

        print("Extracting PCM WAV sound effects...")
        audio_count = extract_audio_sounds(
            file_handle,
            lump_records,
            os.path.join(destination_resources_root, "audio"),
        )
        print(f"Extracted {audio_count} WAV audio sound effects.")

    print("Authentic DOOM asset extraction completed successfully!")


if __name__ == "__main__":
    main()
