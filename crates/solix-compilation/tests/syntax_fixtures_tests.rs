//! Automated test harness for language syntax fixtures.

#![forbid(unsafe_code)]

mod common;

use common::LanguageFixture;
use solix_compilation::phases::lexing::lex_source;
use solix_compilation::phases::parser::parse_source;
use std::fs;
use std::path::Path;

#[test]
fn test_syntax_fixture_directories_exist() {
    let positive_syntax_directory = Path::new("tests/fixtures/syntax/positive");
    let negative_syntax_directory = Path::new("tests/fixtures/syntax/negative");

    assert!(
        positive_syntax_directory.exists(),
        "Expected positive syntax fixtures directory at 'tests/fixtures/syntax/positive'"
    );
    assert!(
        negative_syntax_directory.exists(),
        "Expected negative syntax fixtures directory at 'tests/fixtures/syntax/negative'"
    );
}

#[test]
fn test_all_positive_syntax_fixtures_lex_cleanly() {
    let positive_directory = Path::new("tests/fixtures/syntax/positive");
    let mut fixtures_count = 0;

    for entry_result in fs::read_dir(positive_directory).expect("Read positive syntax directory") {
        let entry = entry_result.expect("Valid directory entry");
        let path = entry.path();

        if path.extension().is_some_and(|extension| extension == "slx") {
            let fixture = LanguageFixture::load_from_path(&path)
                .unwrap_or_else(|error| panic!("Failed to read fixture '{:?}': {}", path, error));

            let result = lex_source(0, &fixture.source_content);
            assert!(
                result.is_ok(),
                "Positive syntax fixture '{:?}' failed to lex: {:?}",
                path,
                result.err()
            );

            fixtures_count += 1;
        }
    }

    assert!(
        fixtures_count >= 1,
        "Expected at least 1 positive fixture to be verified, found {}",
        fixtures_count
    );
}

#[test]
fn test_all_negative_syntax_fixtures_fail_compilation() {
    let negative_directory = Path::new("tests/fixtures/syntax/negative");
    let mut fixtures_count = 0;

    for entry_result in fs::read_dir(negative_directory).expect("Read negative syntax directory") {
        let entry = entry_result.expect("Valid directory entry");
        let path = entry.path();

        if path.extension().is_some_and(|extension| extension == "slx") {
            let fixture = LanguageFixture::load_from_path(&path)
                .unwrap_or_else(|error| panic!("Failed to read fixture '{:?}': {}", path, error));

            let lex_result = lex_source(0, &fixture.source_content);
            match lex_result {
                Ok(tokens) => {
                    let parse_result = parse_source(0, &tokens, &fixture.source_content);
                    assert!(
                        parse_result.is_err(),
                        "Negative syntax fixture '{:?}' was expected to fail parsing, but succeeded",
                        path
                    );
                    let errors = parse_result.unwrap_err();
                    assert!(
                        !errors.is_empty(),
                        "Expected at least one ParsingError from '{:?}'",
                        path
                    );
                }
                Err(errors) => {
                    assert!(
                        !errors.is_empty(),
                        "Expected at least one LexingError from '{:?}'",
                        path
                    );
                }
            }

            fixtures_count += 1;
        }
    }

    assert!(
        fixtures_count >= 1,
        "Expected at least 1 negative fixture to be verified, found {}",
        fixtures_count
    );
}
