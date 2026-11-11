# Formal Specification: Standard Library Cryptography & Encoding (`solix.crypto`)

## Document Metadata

* **Specification ID:** SPEC-0024
* **Scope:** Cryptographic Hashing, Base64 & Hexadecimal Encoding
* **Status:** Final Normative Standard
* **Source Path:** [`libraries/solixlib/src/crypto/`](file:///home/jehud/Projects/solix-language/libraries/solixlib/src/crypto/)

---

## 1. Cryptographic Hashing (`solix.crypto.Hash`)

The `Hash` class exposes cryptographic hash algorithms validated against official NIST/RFC vectors:

### 1.1 SHA-256 (FIPS 180-4)
* `static int8[] sha256(int8[] data_bytes)`: Returns raw 32-byte hash digest.
* `static String sha256_hex(String text_string)`: Returns 64-character lowercase hexadecimal hash.

### 1.2 SHA-1 (RFC 3174)
* `static int8[] sha1(int8[] data_bytes)`: Returns raw 20-byte hash digest.
* `static String sha1_hex(String text_string)`: Returns 40-character lowercase hexadecimal hash.

### 1.3 MD5 (RFC 1321)
* `static int8[] md5(int8[] data_bytes)`: Returns raw 16-byte hash digest.
* `static String md5_hex(String text_string)`: Returns 32-character lowercase hexadecimal hash.

### 1.4 Constant-Time Equality
* `static bool constant_time_equals(int8[] first_digest, int8[] second_digest)`: Compares digests in time proportional strictly to length, preventing timing attack side-channels.

---

## 2. Base64 Encoding (`solix.crypto.Base64`)

* `static String encode(int8[] raw_bytes)`: Standard Base64 string encoding (RFC 4648).
* `static int8[] decode(String base64_string)`: Decodes standard Base64 string.
* `static String encode_url_safe(int8[] raw_bytes)`: URL-safe encoding (`-` and `_` without padding).
* `static int8[] decode_url_safe(String url_safe_string)`

---

## 3. Hexadecimal Encoding (`solix.crypto.Hex`)

* `static String encode(int8[] raw_bytes)`: Returns 2-character hex per byte.
* `static int8[] decode(String hex_string)`: Parses hex string into byte array.
* `static bool is_valid(String hex_string)`: Validates that string contains only even-length hex digits.
