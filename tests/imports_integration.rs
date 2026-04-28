use std::io::Write;
use std::path::Path;

use claude_usage_tracker::archive::imports;
use tempfile::TempDir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn build_openai_zip(path: &Path) {
    let convs = serde_json::json!([
        { "id": "c1", "title": "Hi", "create_time": 1.0,
          "mapping": { "n": { "id": "n", "parent": null, "children": [], "message": null } } },
        { "id": "c2", "title": "Bye", "create_time": 2.0,
          "mapping": {} }
    ]);
    let f = std::fs::File::create(path).unwrap();
    let mut w = ZipWriter::new(f);
    w.start_file("conversations.json", SimpleFileOptions::default())
        .unwrap();
    w.write_all(serde_json::to_string(&convs).unwrap().as_bytes())
        .unwrap();
    w.finish().unwrap();
}

fn build_anthropic_zip(path: &Path) {
    let payload = serde_json::json!({
        "conversations": [
            { "uuid": "abc", "name": "Conv A", "created_at": "2026-04-01T00:00:00Z",
              "chat_messages": [{"text": "hi"}] }
        ]
    });
    let f = std::fs::File::create(path).unwrap();
    let mut w = ZipWriter::new(f);
    w.start_file("conversations_data.json", SimpleFileOptions::default())
        .unwrap();
    w.write_all(serde_json::to_string(&payload).unwrap().as_bytes())
        .unwrap();
    w.finish().unwrap();
}

#[test]
fn openai_zip_round_trips_through_import() {
    let tmp = TempDir::new().unwrap();
    let archive_root = tmp.path().join("archive");
    let zip_path = tmp.path().join("openai.zip");
    build_openai_zip(&zip_path);

    let report = imports::import_zip(&archive_root, &zip_path).unwrap();
    assert_eq!(report.vendor, imports::detect::Vendor::OpenAI);
    assert_eq!(report.conversation_count, 2);
    assert!(report.root.join("original.zip").is_file());
    assert!(report.root.join("conversations").join("c1.json").is_file());
    assert!(report.root.join("conversations").join("c2.json").is_file());
    assert!(report.root.join("metadata.json").is_file());
}

#[test]
fn anthropic_zip_round_trips_through_import() {
    let tmp = TempDir::new().unwrap();
    let archive_root = tmp.path().join("archive");
    let zip_path = tmp.path().join("anthropic.zip");
    build_anthropic_zip(&zip_path);

    let report = imports::import_zip(&archive_root, &zip_path).unwrap();
    assert_eq!(report.vendor, imports::detect::Vendor::Anthropic);
    assert_eq!(report.conversation_count, 1);
    assert!(report.root.join("conversations").join("abc.json").is_file());
    let meta_bytes = std::fs::read(report.root.join("metadata.json")).unwrap();
    let meta: serde_json::Value = serde_json::from_slice(&meta_bytes).unwrap();
    assert_eq!(meta["vendor"], "anthropic");
    assert!(meta["schema_fingerprint"].is_string());
}

#[test]
fn unknown_zip_fails_clearly() {
    let tmp = TempDir::new().unwrap();
    let archive_root = tmp.path().join("archive");
    let zip_path = tmp.path().join("noise.zip");
    let f = std::fs::File::create(&zip_path).unwrap();
    let mut w = ZipWriter::new(f);
    w.start_file("README.txt", SimpleFileOptions::default())
        .unwrap();
    w.write_all(b"hi").unwrap();
    w.finish().unwrap();

    let result = imports::import_zip(&archive_root, &zip_path);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("not a recognised export")
    );
}
