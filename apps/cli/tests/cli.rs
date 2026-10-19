use std::process::Command;

#[test]
fn demo_prints_a_plain_language_decision_brief() {
    let output = Command::new(env!("CARGO_BIN_EXE_opti-cli"))
        .arg("demo")
        .output()
        .expect("demo command should start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("demo output should be UTF-8");

    assert!(stdout.contains("Decision brief:"));
    assert!(stdout.contains("100 croissants"));
    assert!(stdout.contains("$250.00"));
    assert!(stdout.contains("remaining"));
    assert!(stdout.contains("not a guarantee"));
}
