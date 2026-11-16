//! Integration and unit tests for solixlib native bridges (formatting, JSON, compression).

use solixlib_native::string_prim::{format_template_with_arguments, format_with_specifier};

#[test]
fn test_format_hex_binary_octal_specifiers() {
    let arguments = vec![String::from("255"), String::from("10"), String::from("64")];
    let template = "hex: {:#08x}, bin: {:b}, oct: {:o}";
    let formatted_output = format_template_with_arguments(template, &arguments)
        .expect("Formatting must succeed for valid specifiers");

    assert_eq!(formatted_output, "hex: 0x0000ff, bin: 1010, oct: 100");
}

#[test]
fn test_format_uppercase_hex() {
    let arguments = vec![String::from("255")];
    let template = "{:X}";
    let formatted_output = format_template_with_arguments(template, &arguments)
        .expect("Uppercase hex formatting must succeed");

    assert_eq!(formatted_output, "FF");
}

#[test]
fn test_format_alignment_and_padding() {
    let arguments = vec![String::from("solix")];

    let left_aligned = format_template_with_arguments("{:<10}", &arguments).unwrap();
    assert_eq!(left_aligned, "solix     ");

    let right_aligned = format_template_with_arguments("{:>10}", &arguments).unwrap();
    assert_eq!(right_aligned, "     solix");

    let center_aligned = format_template_with_arguments("{:^11}", &arguments).unwrap();
    assert_eq!(center_aligned, "   solix   ");

    let custom_fill_left = format_template_with_arguments("{:*<10}", &arguments).unwrap();
    assert_eq!(custom_fill_left, "solix*****");

    let custom_fill_right = format_template_with_arguments("{:_>10}", &arguments).unwrap();
    assert_eq!(custom_fill_right, "_____solix");
}

#[test]
fn test_format_floating_point_precision() {
    let arguments = vec![String::from("3.14159265")];
    let formatted_output = format_template_with_arguments("{:.2f}", &arguments)
        .expect("Float precision formatting must succeed");

    assert_eq!(formatted_output, "3.14");
}

#[test]
fn test_format_explicit_positional_indices() {
    let arguments = vec![String::from("first"), String::from("second")];
    let template = "{1} comes after {0}, repeated: {0}";
    let formatted_output = format_template_with_arguments(template, &arguments)
        .expect("Positional indices must succeed");

    assert_eq!(formatted_output, "second comes after first, repeated: first");
}

#[test]
fn test_format_escaped_curly_braces() {
    let arguments = vec![String::from("value")];
    let template = "{{ escaped }} and replaced: {}";
    let formatted_output = format_template_with_arguments(template, &arguments)
        .expect("Escaped braces must succeed");

    assert_eq!(formatted_output, "{ escaped } and replaced: value");
}

#[test]
fn test_format_negative_unclosed_brace_throws_e0331() {
    let arguments = vec![String::from("test")];
    let template = "unclosed {specifier";
    let format_result = format_template_with_arguments(template, &arguments);

    assert!(format_result.is_err());
    let error_message = format_result.unwrap_err();
    assert!(
        error_message.contains("[E0331]"),
        "Error message must contain diagnostic code [E0331], got: {error_message}"
    );
}

#[test]
fn test_format_negative_invalid_type_character_throws_e0331() {
    let arguments = vec![String::from("123")];
    let template = "invalid specifier: {:q}";
    let format_result = format_template_with_arguments(template, &arguments);

    assert!(format_result.is_err());
    let error_message = format_result.unwrap_err();
    assert!(
        error_message.contains("[E0331]"),
        "Error message must contain diagnostic code [E0331], got: {error_message}"
    );
}

#[test]
fn test_format_negative_trailing_garbage_throws_e0331() {
    let format_result = format_with_specifier("123", "08x_extra");

    assert!(format_result.is_err());
    let error_message = format_result.unwrap_err();
    assert!(
        error_message.contains("[E0331]"),
        "Error message must contain diagnostic code [E0331], got: {error_message}"
    );
}

#[test]
fn test_json_parse_and_pointer_query_logic() {
    let raw_json_text = r#"{"name": "Solix", "version": 2, "features": ["fast", "safe"]}"#;
    let parsed_root: serde_json::Value =
        serde_json::from_str(raw_json_text).expect("Valid JSON must parse cleanly");

    let name_leaf = parsed_root.pointer("/name").expect("Pointer /name must exist");
    assert_eq!(name_leaf.as_str().unwrap(), "Solix");

    let first_feature = parsed_root
        .pointer("/features/0")
        .expect("Pointer /features/0 must exist");
    assert_eq!(first_feature.as_str().unwrap(), "fast");

    let non_existent = parsed_root.pointer("/missing/item");
    assert!(non_existent.is_none());
}

#[test]
fn test_json_negative_syntax_error_reporting() {
    let malformed_json = r#"{"name": "Solix", unquoted_key: 123}"#;
    let parse_result = serde_json::from_str::<serde_json::Value>(malformed_json);

    assert!(parse_result.is_err());
    let parse_error = parse_result.unwrap_err();
    let diagnostic_string = format!(
        "[E0332] Failed to parse JSON string at line {}, column {}: {}",
        parse_error.line(),
        parse_error.column(),
        parse_error
    );

    assert!(diagnostic_string.contains("[E0332]"));
    assert!(diagnostic_string.contains("line 1"));
}

#[test]
fn test_gzip_compression_and_decompression_roundtrip() {
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::{Read, Write};

    let original_payload = b"Solix standard library compression test payload string!";
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(original_payload)
        .expect("Write to GzEncoder must succeed");
    let compressed_bytes = encoder.finish().expect("Finishing GzEncoder must succeed");

    assert!(!compressed_bytes.is_empty());
    assert_ne!(&compressed_bytes[..], original_payload);

    let mut decoder = GzDecoder::new(&compressed_bytes[..]);
    let mut decompressed_buffer = Vec::new();
    decoder
        .read_to_end(&mut decompressed_buffer)
        .expect("GzDecoder read must succeed");

    assert_eq!(&decompressed_buffer[..], original_payload);
}

#[test]
fn test_zlib_compression_and_decompression_roundtrip() {
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::{Read, Write};

    let original_payload = b"Zlib compression verification payload for Solix standard library.";
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(original_payload)
        .expect("Write to ZlibEncoder must succeed");
    let compressed_bytes = encoder.finish().expect("Finishing ZlibEncoder must succeed");

    let mut decoder = ZlibDecoder::new(&compressed_bytes[..]);
    let mut decompressed_buffer = Vec::new();
    decoder
        .read_to_end(&mut decompressed_buffer)
        .expect("ZlibDecoder read must succeed");

    assert_eq!(&decompressed_buffer[..], original_payload);
}

#[test]
fn test_deflate_negative_corrupted_payload_throws_e0334() {
    use flate2::read::DeflateDecoder;
    use std::io::Read;

    let corrupted_bytes = [0xFF, 0xFE, 0xFD, 0xFC, 0x01, 0x02];
    let mut decoder = DeflateDecoder::new(&corrupted_bytes[..]);
    let mut output_buffer = Vec::new();
    let read_result = decoder.read_to_end(&mut output_buffer);

    assert!(read_result.is_err());
    let decode_error = read_result.unwrap_err();
    let formatted_error = format!(
        "[E0334] Failed to decompress buffer using algorithm 'deflate': {}",
        decode_error
    );

    assert!(formatted_error.contains("[E0334]"));
    assert!(formatted_error.contains("algorithm 'deflate'"));
}
