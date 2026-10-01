use std::process::Command;

#[test]
fn cli_lays_out_and_starts_guest_animation_before_advancing_time() {
    let output = Command::new(env!("CARGO_BIN_EXE_droidless"))
        .args([
            "run",
            "--headless",
            "--ephemeral",
            "--click",
            "Start layout animation",
            "--advance-ms",
            "50",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/generated/scheduling.apk"
            ),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(tree["view"]["text"], "Layout animation");
    assert_eq!(tree["rect"]["x"], 50.0);
    assert_eq!(tree["view"]["alpha"], 0.5);
}

#[test]
fn cli_advanced_frame_runs_before_the_next_back_callback() {
    let output = Command::new(env!("CARGO_BIN_EXE_droidless"))
        .args([
            "run",
            "--headless",
            "--ephemeral",
            "--click",
            "Start layout animation",
            "--advance-ms",
            "50",
            "--back",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/generated/scheduling.apk"
            ),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(tree["view"]["text"], "Advanced frame observed");
    assert_eq!(tree["rect"]["x"], 50.0);
}
