//! Cryptographic hashing (SHA-256, SHA-1, MD5) and binary encoding (Base64, Hex) native functions.

#![allow(clippy::chunks_exact_to_as_chunks)]
#![allow(clippy::needless_range_loop)]

use crate::{
    alloc_solix_bytes, alloc_solix_chars, read_solix_bytes, read_solix_chars,
    register_symbol_variants, NativeCFunctionPtr, RegisterNativeCallback, SolixCValue,
};
use std::ffi::c_void;

// --- Hex Encoding & Decoding ---

fn hex_encode(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);
    for &byte_value in bytes {
        hex_string.push_str(&format!("{byte_value:02x}"));
    }
    hex_string
}

fn hex_decode(hex_str: &str) -> Option<Vec<u8>> {
    let clean_hex = hex_str.trim();
    if !clean_hex.len().is_multiple_of(2) {
        return None;
    }
    let mut output_bytes = Vec::with_capacity(clean_hex.len() / 2);
    let byte_slice = clean_hex.as_bytes();
    for chunk_index in (0..byte_slice.len()).step_by(2) {
        let high_char = byte_slice[chunk_index] as char;
        let low_char = byte_slice[chunk_index + 1] as char;
        let high_nibble = high_char.to_digit(16)?;
        let low_nibble = low_char.to_digit(16)?;
        output_bytes.push(((high_nibble << 4) | low_nibble) as u8);
    }
    Some(output_bytes)
}

// --- Base64 Encoding & Decoding ---

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(bytes: &[u8]) -> String {
    let mut encoded_string = String::new();
    let total_bytes = bytes.len();
    let mut byte_index = 0;

    while byte_index < total_bytes {
        let first_byte = bytes[byte_index];
        let second_byte = if byte_index + 1 < total_bytes {
            bytes[byte_index + 1]
        } else {
            0
        };
        let third_byte = if byte_index + 2 < total_bytes {
            bytes[byte_index + 2]
        } else {
            0
        };

        let triple_val =
            ((first_byte as u32) << 16) | ((second_byte as u32) << 8) | (third_byte as u32);

        encoded_string.push(BASE64_ALPHABET[((triple_val >> 18) & 0x3F) as usize] as char);
        encoded_string.push(BASE64_ALPHABET[((triple_val >> 12) & 0x3F) as usize] as char);

        if byte_index + 1 < total_bytes {
            encoded_string.push(BASE64_ALPHABET[((triple_val >> 6) & 0x3F) as usize] as char);
        } else {
            encoded_string.push('=');
        }

        if byte_index + 2 < total_bytes {
            encoded_string.push(BASE64_ALPHABET[(triple_val & 0x3F) as usize] as char);
        } else {
            encoded_string.push('=');
        }

        byte_index += 3;
    }

    encoded_string
}

fn base64_decode(base64_str: &str) -> Option<Vec<u8>> {
    let clean_str = base64_str.trim();
    let mut decoded_bytes = Vec::new();

    fn decode_char(character: char) -> Option<u32> {
        match character {
            'A'..='Z' => Some((character as u32) - ('A' as u32)),
            'a'..='z' => Some((character as u32) - ('a' as u32) + 26),
            '0'..='9' => Some((character as u32) - ('0' as u32) + 52),
            '+' => Some(62),
            '/' => Some(63),
            _ => None,
        }
    }

    let mut quartet = [0u32; 4];
    let mut quartet_count = 0;

    for character in clean_str.chars() {
        if character == '=' {
            break;
        }
        if character.is_whitespace() {
            continue;
        }
        let char_val = decode_char(character)?;
        quartet[quartet_count] = char_val;
        quartet_count += 1;

        if quartet_count == 4 {
            let triple_val =
                (quartet[0] << 18) | (quartet[1] << 12) | (quartet[2] << 6) | quartet[3];
            decoded_bytes.push(((triple_val >> 16) & 0xFF) as u8);
            decoded_bytes.push(((triple_val >> 8) & 0xFF) as u8);
            decoded_bytes.push((triple_val & 0xFF) as u8);
            quartet_count = 0;
        }
    }

    if quartet_count == 2 {
        let combined_val = (quartet[0] << 18) | (quartet[1] << 12);
        decoded_bytes.push(((combined_val >> 16) & 0xFF) as u8);
    } else if quartet_count == 3 {
        let combined_val = (quartet[0] << 18) | (quartet[1] << 12) | (quartet[2] << 6);
        decoded_bytes.push(((combined_val >> 16) & 0xFF) as u8);
        decoded_bytes.push(((combined_val >> 8) & 0xFF) as u8);
    }

    Some(decoded_bytes)
}

// --- SHA-256 Implementation (FIPS 180-4) ---

const K256: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

fn sha256_digest(input_bytes: &[u8]) -> [u8; 32] {
    let mut state = [
        0x6a09_e667u32,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    let bit_count = (input_bytes.len() as u64) * 8;
    let mut padded_bytes = input_bytes.to_vec();
    padded_bytes.push(0x80);
    while (padded_bytes.len() % 64) != 56 {
        padded_bytes.push(0x00);
    }
    padded_bytes.extend_from_slice(&bit_count.to_be_bytes());

    for chunk in padded_bytes.chunks_exact(64) {
        let mut words = [0u32; 64];
        for word_index in 0..16 {
            let offset = word_index * 4;
            words[word_index] = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for word_index in 16..64 {
            let s0 = words[word_index - 15].rotate_right(7)
                ^ words[word_index - 15].rotate_right(18)
                ^ (words[word_index - 15] >> 3);
            let s1 = words[word_index - 2].rotate_right(17)
                ^ words[word_index - 2].rotate_right(19)
                ^ (words[word_index - 2] >> 10);
            words[word_index] = words[word_index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[word_index - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for (word_index, &schedule_word) in words.iter().enumerate() {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K256[word_index])
                .wrapping_add(schedule_word);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut output_digest = [0u8; 32];
    for (state_index, &state_word) in state.iter().enumerate() {
        let offset = state_index * 4;
        output_digest[offset..offset + 4].copy_from_slice(&state_word.to_be_bytes());
    }
    output_digest
}

// --- SHA-1 Implementation (RFC 3174) ---

fn sha1_digest(input_bytes: &[u8]) -> [u8; 20] {
    let mut state = [
        0x6745_2301u32,
        0xefcd_ab89,
        0x98ba_dcfe,
        0x1032_5476,
        0xc3d2_e1f0,
    ];

    let bit_count = (input_bytes.len() as u64) * 8;
    let mut padded_bytes = input_bytes.to_vec();
    padded_bytes.push(0x80);
    while (padded_bytes.len() % 64) != 56 {
        padded_bytes.push(0x00);
    }
    padded_bytes.extend_from_slice(&bit_count.to_be_bytes());

    for chunk in padded_bytes.chunks_exact(64) {
        let mut words = [0u32; 80];
        for word_index in 0..16 {
            let offset = word_index * 4;
            words[word_index] = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for word_index in 16..80 {
            words[word_index] = (words[word_index - 3]
                ^ words[word_index - 8]
                ^ words[word_index - 14]
                ^ words[word_index - 16])
                .rotate_left(1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];

        for (word_index, &schedule_word) in words.iter().enumerate() {
            let (f, k) = match word_index {
                0..=19 => ((b & c) | ((!b) & d), 0x5a82_7999u32),
                20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
                _ => (b ^ c ^ d, 0xca62_c1d6),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(schedule_word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
    }

    let mut output_digest = [0u8; 20];
    for (state_index, &state_word) in state.iter().enumerate() {
        let offset = state_index * 4;
        output_digest[offset..offset + 4].copy_from_slice(&state_word.to_be_bytes());
    }
    output_digest
}

// --- MD5 Implementation (RFC 1321) ---

fn md5_digest(input_bytes: &[u8]) -> [u8; 16] {
    let mut state = [0x6745_2301u32, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];

    let bit_count = (input_bytes.len() as u64) * 8;
    let mut padded_bytes = input_bytes.to_vec();
    padded_bytes.push(0x80);
    while (padded_bytes.len() % 64) != 56 {
        padded_bytes.push(0x00);
    }
    padded_bytes.extend_from_slice(&bit_count.to_le_bytes());

    const SHIFTS: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];

    const TABLE_T: [u32; 64] = [
        0xd76a_a478,
        0xe8c7_b756,
        0x2420_70db,
        0xc1bd_ceee,
        0xf57c_0faf,
        0x4787_c62a,
        0xa830_4613,
        0xfd46_9501,
        0x6980_98d8,
        0x8b44_f7af,
        0xffff_5bb1,
        0x895c_d7be,
        0x6b90_1122,
        0xfd98_7193,
        0xa679_438e,
        0x49b4_0821,
        0xf61e_2562,
        0xc040_b340,
        0x265e_5a51,
        0xe9b6_c7aa,
        0xd62f_105d,
        0x0244_1453,
        0xd8a1_e681,
        0xe7d3_fbc8,
        0x21e1_cde6,
        0xc337_07d6,
        0xf4d5_0d87,
        0x455a_14ed,
        0xa9e3_e905,
        0xfcefa3f8,
        0x676f_02d9,
        0x8d2a_4c8a,
        0xfffa_3942,
        0x8771_f681,
        0x6d9d_6122,
        0xfde5_380c,
        0xa4be_ea44,
        0x4bde_cfa9,
        0xf6bb_4b60,
        0xbebf_bc70,
        0x289b_7ec6,
        0xeaa1_27fa,
        0xd4ef_3085,
        0x0488_1d05,
        0xd9d4_d039,
        0xe6db_99e5,
        0x1fa2_7cf8,
        0xc4ac_5665,
        0xf429_2244,
        0x432a_ff97,
        0xab94_23a7,
        0xfc93_a039,
        0x655b_59c3,
        0x8f0c_cc92,
        0xffef_f47d,
        0x8584_5dd1,
        0x6fa8_7e4f,
        0xfe2c_e6e0,
        0xa301_4314,
        0x4e08_11a1,
        0xf753_7e82,
        0xbd3a_f235,
        0x2ad7_d2bb,
        0xeb86_d391,
    ];

    for chunk in padded_bytes.chunks_exact(64) {
        let mut words = [0u32; 16];
        for word_index in 0..16 {
            let offset = word_index * 4;
            words[word_index] = u32::from_le_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];

        for step in 0..64 {
            let (f, g) = match step {
                0..=15 => ((b & c) | ((!b) & d), step),
                16..=31 => ((d & b) | ((!d) & c), (5 * step + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * step + 5) % 16),
                _ => (c ^ (b | (!d)), (7 * step) % 16),
            };

            let temp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(TABLE_T[step])
                    .wrapping_add(words[g])
                    .rotate_left(SHIFTS[step]),
            );
            a = temp;
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
    }

    let mut output_digest = [0u8; 16];
    for (state_index, &state_word) in state.iter().enumerate() {
        let offset = state_index * 4;
        output_digest[offset..offset + 4].copy_from_slice(&state_word.to_le_bytes());
    }
    output_digest
}

// --- FFI Bridge Wrappers ---

pub unsafe extern "C" fn crypto_native_base64_encode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, address) };
    let encoded = base64_encode(&input_bytes);
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &encoded) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_base64_encode_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let text = unsafe { read_solix_chars(vm_context, address) };
    let encoded = base64_encode(text.as_bytes());
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &encoded) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_base64_decode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let base64_text = unsafe { read_solix_chars(vm_context, address) };
    let decoded = base64_decode(&base64_text).unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_bytes(vm_context, &decoded) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_base64_decode_to_string(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let base64_text = unsafe { read_solix_chars(vm_context, address) };
    let decoded_bytes = base64_decode(&base64_text).unwrap_or_default();
    let text = String::from_utf8(decoded_bytes).unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &text) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_hex_encode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, address) };
    let encoded = hex_encode(&input_bytes);
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &encoded) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_hex_decode(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let hex_text = unsafe { read_solix_chars(vm_context, address) };
    let decoded_bytes = hex_decode(&hex_text).unwrap_or_default();
    let allocated_address = unsafe { alloc_solix_bytes(vm_context, &decoded_bytes) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_sha256(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, address) };
    let digest = sha256_digest(&input_bytes);
    let allocated_address = unsafe { alloc_solix_bytes(vm_context, &digest) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_sha256_hex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let input_text = unsafe { read_solix_chars(vm_context, address) };
    let digest = sha256_digest(input_text.as_bytes());
    let hex_representation = hex_encode(&digest);
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &hex_representation) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_sha1(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, address) };
    let digest = sha1_digest(&input_bytes);
    let allocated_address = unsafe { alloc_solix_bytes(vm_context, &digest) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_sha1_hex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let input_text = unsafe { read_solix_chars(vm_context, address) };
    let digest = sha1_digest(input_text.as_bytes());
    let hex_representation = hex_encode(&digest);
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &hex_representation) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_md5(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let input_bytes = unsafe { read_solix_bytes(vm_context, address) };
    let digest = md5_digest(&input_bytes);
    let allocated_address = unsafe { alloc_solix_bytes(vm_context, &digest) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe extern "C" fn crypto_native_md5_hex(
    vm_context: *mut c_void,
    args: *const SolixCValue,
    arg_count: u32,
    _out_error: *mut SolixCValue,
) -> SolixCValue {
    if arg_count == 0 || args.is_null() {
        return SolixCValue::null();
    }
    let address = unsafe { (*args.add(0)).as_ref() };
    let input_text = unsafe { read_solix_chars(vm_context, address) };
    let digest = md5_digest(input_text.as_bytes());
    let hex_representation = hex_encode(&digest);
    let allocated_address = unsafe { alloc_solix_chars(vm_context, &hex_representation) };
    SolixCValue::ref_val(allocated_address)
}

pub unsafe fn register_crypto_natives(collector: *mut c_void, callback: RegisterNativeCallback) {
    let base64_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Base64_native_encode",
                "solix_crypto_Base64_native_encode",
                "Base64::native_encode",
            ],
            crypto_native_base64_encode,
        ),
        (
            &[
                "Base64_native_encode_string",
                "solix_crypto_Base64_native_encode_string",
                "Base64::native_encode_string",
            ],
            crypto_native_base64_encode_string,
        ),
        (
            &[
                "Base64_native_decode",
                "solix_crypto_Base64_native_decode",
                "Base64::native_decode",
            ],
            crypto_native_base64_decode,
        ),
        (
            &[
                "Base64_native_decode_to_string",
                "solix_crypto_Base64_native_decode_to_string",
                "Base64::native_decode_to_string",
            ],
            crypto_native_base64_decode_to_string,
        ),
    ];

    for (variants, func) in base64_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let hex_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Hex_native_encode",
                "solix_crypto_Hex_native_encode",
                "Hex::native_encode",
            ],
            crypto_native_hex_encode,
        ),
        (
            &[
                "Hex_native_decode",
                "solix_crypto_Hex_native_decode",
                "Hex::native_decode",
            ],
            crypto_native_hex_decode,
        ),
    ];

    for (variants, func) in hex_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }

    let hash_mappings: &[(&[&str], NativeCFunctionPtr)] = &[
        (
            &[
                "Hash_native_sha256",
                "solix_crypto_Hash_native_sha256",
                "Hash::native_sha256",
            ],
            crypto_native_sha256,
        ),
        (
            &[
                "Hash_native_sha256_hex",
                "solix_crypto_Hash_native_sha256_hex",
                "Hash::native_sha256_hex",
            ],
            crypto_native_sha256_hex,
        ),
        (
            &[
                "Hash_native_sha1",
                "solix_crypto_Hash_native_sha1",
                "Hash::native_sha1",
            ],
            crypto_native_sha1,
        ),
        (
            &[
                "Hash_native_sha1_hex",
                "solix_crypto_Hash_native_sha1_hex",
                "Hash::native_sha1_hex",
            ],
            crypto_native_sha1_hex,
        ),
        (
            &[
                "Hash_native_md5",
                "solix_crypto_Hash_native_md5",
                "Hash::native_md5",
            ],
            crypto_native_md5,
        ),
        (
            &[
                "Hash_native_md5_hex",
                "solix_crypto_Hash_native_md5_hex",
                "Hash::native_md5_hex",
            ],
            crypto_native_md5_hex,
        ),
    ];

    for (variants, func) in hash_mappings {
        unsafe { register_symbol_variants(collector, callback, variants, *func) };
    }
}
