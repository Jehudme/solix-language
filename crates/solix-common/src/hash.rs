//! Self-contained, high-throughput FIPS 180-4 SHA-256 implementation.
//!
//! Provides cryptographic byte, string, and streaming file checksum calculation
//! without external third-party dependencies, adhering strictly to `#![forbid(unsafe_code)]`.

#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// FIPS 180-4 initial hash state values (first 32 bits of fractional parts of square roots of first 8 primes).
const INITIAL_HASH_VALUES: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// FIPS 180-4 round constants (first 32 bits of fractional parts of cube roots of first 64 primes).
const ROUND_CONSTANTS: [u32; 64] = [
    0x428a_2f98, 0x7137_4491, 0xb5c0_fbcf, 0xe9b5_dba5, 0x3956_c25b, 0x59f1_11f1, 0x923f_82a4,
    0xab1c_5ed5, 0xd807_aa98, 0x1283_5b01, 0x2431_85be, 0x550c_7dc3, 0x72be_5d74, 0x80de_b1fe,
    0x9bdc_06a7, 0xc19b_f174, 0xe49b_69c1, 0xefbe_4786, 0x0fc1_9dc6, 0x240c_a1cc, 0x2de9_2c6f,
    0x4a74_84aa, 0x5cb0_a9dc, 0x76f9_88da, 0x983e_5152, 0xa831_c66d, 0xb003_27c8, 0xbf59_7fc7,
    0xc6e0_0bf3, 0xd5a7_9147, 0x06ca_6351, 0x1429_2967, 0x27b7_0a85, 0x2e1b_2138, 0x4d2c_6dfc,
    0x5338_0d13, 0x650a_7354, 0x766a_0abb, 0x81c2_c92e, 0x9272_2c85, 0xa2bf_e8a1, 0xa81a_664b,
    0xc24b_8b70, 0xc76c_51a3, 0xd192_e819, 0xd699_0624, 0xf40e_3585, 0x106a_a070, 0x19a4_c116,
    0x1e37_6c08, 0x2748_774c, 0x34b0_bcb5, 0x391c_0cb3, 0x4ed8_aa4a, 0x5b9c_ca4f, 0x682e_6ff3,
    0x748f_82ee, 0x78a5_636f, 0x84c8_7814, 0x8cc7_0208, 0x90be_fffa, 0xa450_6ceb, 0xbef9_a3f7,
    0xc671_78f2,
];

/// Incremental SHA-256 state accumulator conforming to FIPS 180-4.
#[derive(Debug, Clone)]
pub struct Sha256Hasher {
    state: [u32; 8],
    buffer: [u8; 64],
    buffer_length: usize,
    total_bytes: u64,
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256Hasher {
    /// Creates a new `Sha256Hasher` initialized to standard FIPS 180-4 constants.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: INITIAL_HASH_VALUES,
            buffer: [0u8; 64],
            buffer_length: 0,
            total_bytes: 0,
        }
    }

    /// Ingests input bytes into the ongoing hash computation.
    pub fn update(&mut self, mut input_slice: &[u8]) {
        self.total_bytes = self.total_bytes.wrapping_add(input_slice.len() as u64);

        if self.buffer_length > 0 {
            let space_remaining = 64 - self.buffer_length;
            if input_slice.len() >= space_remaining {
                self.buffer[self.buffer_length..64].copy_from_slice(&input_slice[..space_remaining]);
                let full_block = self.buffer;
                self.process_block(&full_block);
                self.buffer_length = 0;
                input_slice = &input_slice[space_remaining..];
            } else {
                let current_length = self.buffer_length;
                self.buffer[current_length..current_length + input_slice.len()]
                    .copy_from_slice(input_slice);
                self.buffer_length += input_slice.len();
                return;
            }
        }

        while input_slice.len() >= 64 {
            let (block, remaining) = input_slice.split_at(64);
            self.process_block(block);
            input_slice = remaining;
        }

        if !input_slice.is_empty() {
            self.buffer[..input_slice.len()].copy_from_slice(input_slice);
            self.buffer_length = input_slice.len();
        }
    }

    fn process_block(&mut self, block: &[u8]) {
        let mut schedule = [0u32; 64];

        for (schedule_index, schedule_slot) in schedule.iter_mut().take(16).enumerate() {
            let offset = schedule_index * 4;
            *schedule_slot = u32::from_be_bytes([
                block[offset],
                block[offset + 1],
                block[offset + 2],
                block[offset + 3],
            ]);
        }

        for schedule_index in 16..64 {
            let value_minus_fifteen = schedule[schedule_index - 15];
            let small_sigma_zero = value_minus_fifteen.rotate_right(7)
                ^ value_minus_fifteen.rotate_right(18)
                ^ (value_minus_fifteen >> 3);

            let value_minus_two = schedule[schedule_index - 2];
            let small_sigma_one = value_minus_two.rotate_right(17)
                ^ value_minus_two.rotate_right(19)
                ^ (value_minus_two >> 10);

            schedule[schedule_index] = schedule[schedule_index - 16]
                .wrapping_add(small_sigma_zero)
                .wrapping_add(schedule[schedule_index - 7])
                .wrapping_add(small_sigma_one);
        }

        let mut register_a = self.state[0];
        let mut register_b = self.state[1];
        let mut register_c = self.state[2];
        let mut register_d = self.state[3];
        let mut register_e = self.state[4];
        let mut register_f = self.state[5];
        let mut register_g = self.state[6];
        let mut register_h = self.state[7];

        for round_index in 0..64 {
            let big_sigma_one = register_e.rotate_right(6)
                ^ register_e.rotate_right(11)
                ^ register_e.rotate_right(25);
            let choice = (register_e & register_f) ^ ((!register_e) & register_g);
            let temporary_sum_one = register_h
                .wrapping_add(big_sigma_one)
                .wrapping_add(choice)
                .wrapping_add(ROUND_CONSTANTS[round_index])
                .wrapping_add(schedule[round_index]);

            let big_sigma_zero = register_a.rotate_right(2)
                ^ register_a.rotate_right(13)
                ^ register_a.rotate_right(22);
            let majority =
                (register_a & register_b) ^ (register_a & register_c) ^ (register_b & register_c);
            let temporary_sum_two = big_sigma_zero.wrapping_add(majority);

            register_h = register_g;
            register_g = register_f;
            register_f = register_e;
            register_e = register_d.wrapping_add(temporary_sum_one);
            register_d = register_c;
            register_c = register_b;
            register_b = register_a;
            register_a = temporary_sum_one.wrapping_add(temporary_sum_two);
        }

        self.state[0] = self.state[0].wrapping_add(register_a);
        self.state[1] = self.state[1].wrapping_add(register_b);
        self.state[2] = self.state[2].wrapping_add(register_c);
        self.state[3] = self.state[3].wrapping_add(register_d);
        self.state[4] = self.state[4].wrapping_add(register_e);
        self.state[5] = self.state[5].wrapping_add(register_f);
        self.state[6] = self.state[6].wrapping_add(register_g);
        self.state[7] = self.state[7].wrapping_add(register_h);
    }

    /// Completes the hash calculation and returns the raw 32-byte digest array.
    #[must_use]
    pub fn finalize(mut self) -> [u8; 32] {
        let bit_length = self.total_bytes.wrapping_mul(8);

        self.buffer[self.buffer_length] = 0x80;
        self.buffer_length += 1;

        if self.buffer_length > 56 {
            for fill_index in self.buffer_length..64 {
                self.buffer[fill_index] = 0;
            }
            let full_block = self.buffer;
            self.process_block(&full_block);
            self.buffer_length = 0;
        }

        for fill_index in self.buffer_length..56 {
            self.buffer[fill_index] = 0;
        }

        self.buffer[56..64].copy_from_slice(&bit_length.to_be_bytes());
        let final_block = self.buffer;
        self.process_block(&final_block);

        let mut output_digest = [0u8; 32];
        for (state_index, &state_word) in self.state.iter().enumerate() {
            let offset = state_index * 4;
            output_digest[offset..offset + 4].copy_from_slice(&state_word.to_be_bytes());
        }

        output_digest
    }

    /// Completes the hash calculation and formats the digest as a canonical 64-character lowercase hex string.
    #[must_use]
    pub fn finalize_hex(self) -> String {
        let digest_bytes = self.finalize();
        format_hex_digest(&digest_bytes)
    }
}

/// Computes the SHA-256 digest of arbitrary in-memory bytes, returning a 32-byte array.
#[must_use]
pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

/// Computes the SHA-256 digest of arbitrary in-memory bytes, returning a 64-character lowercase hex string.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256Hasher::new();
    hasher.update(data);
    hasher.finalize_hex()
}

/// Computes the SHA-256 checksum of an on-disk file by streaming chunks into `Sha256Hasher`.
///
/// # Errors
/// Returns an [`io::Error`] if the file cannot be opened or read.
pub fn sha256_file(file_path: impl AsRef<Path>) -> io::Result<String> {
    let mut file_handle = File::open(file_path)?;
    let mut hasher = Sha256Hasher::new();
    let mut read_buffer = [0u8; 65536];

    loop {
        let bytes_read = file_handle.read(&mut read_buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&read_buffer[..bytes_read]);
    }

    Ok(hasher.finalize_hex())
}

/// Formats a 32-byte digest into a 64-character lowercase hexadecimal string.
#[must_use]
pub fn format_hex_digest(digest: &[u8; 32]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut hex_string = String::with_capacity(64);
    for &byte_value in digest {
        let high_nibble = (byte_value >> 4) as usize;
        let low_nibble = (byte_value & 0x0f) as usize;
        hex_string.push(HEX_CHARS[high_nibble] as char);
        hex_string.push(HEX_CHARS[low_nibble] as char);
    }
    hex_string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty_string_nist_vector() {
        let expected_digest = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let computed_digest = sha256_hex(b"");
        assert_eq!(computed_digest, expected_digest);
    }

    #[test]
    fn test_sha256_abc_nist_vector() {
        let expected_digest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let computed_digest = sha256_hex(b"abc");
        assert_eq!(computed_digest, expected_digest);
    }

    #[test]
    fn test_sha256_long_alphabet_nist_vector() {
        let input_text = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let expected_digest = "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";
        let computed_digest = sha256_hex(input_text);
        assert_eq!(computed_digest, expected_digest);
    }

    #[test]
    fn test_sha256_streaming_file() {
        let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
        let sample_file = temp_dir.path().join("sample.bin");
        std::fs::write(&sample_file, b"Hello Solix SHA-256 Checksum World!")
            .expect("Failed to write sample file");

        let direct_hex = sha256_hex(b"Hello Solix SHA-256 Checksum World!");
        let file_hex = sha256_file(&sample_file).expect("Failed to hash file");
        assert_eq!(file_hex, direct_hex);
    }
}
