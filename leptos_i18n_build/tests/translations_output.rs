use leptos_i18n_build::{Config, TranslationsInfos};
use std::path::{Path, PathBuf};

fn parse_fixtures() -> TranslationsInfos {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    TranslationsInfos::parse_at_dir(dir, Config::new("en").unwrap()).unwrap()
}

fn out_dir(test_name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test_name);
    if path.exists() {
        std::fs::remove_dir_all(&path).unwrap();
    }
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn written_translations_are_valid_json() {
    let out = out_dir("written_translations_are_valid_json");
    parse_fixtures()
        .get_translations()
        .write_to_dir(&out)
        .unwrap();

    let written = std::fs::read_to_string(out.join("en.json")).unwrap();
    let strings: Vec<String> = serde_json::from_str(&written).unwrap();
    assert!(strings.iter().any(|s| s.contains('\u{7}')));
    assert!(strings.iter().any(|s| s.contains('\u{7f}')));
    assert!(strings.iter().any(|s| s.contains('\u{301}')));
}

#[test]
fn generate_data_reports_io_errors() {
    let out = out_dir("generate_data_reports_io_errors");
    let mod_directory = out.join("baked_data");
    std::fs::write(&mod_directory, "not a directory").unwrap();

    let err = parse_fixtures().generate_data(mod_directory).unwrap_err();
    assert!(
        matches!(err.kind, icu_provider::DataErrorKind::Io(_)),
        "{err:?}"
    );
}
