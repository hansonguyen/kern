use std::process::Command;

fn ktype() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ktype"))
}

#[test]
fn version_long_flag() {
    let out = ktype().arg("--version").output().unwrap();
    assert!(out.status.success(), "ktype --version exited non-zero");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected = format!("ktype {}", env!("CARGO_PKG_VERSION"));
    assert!(stdout.contains(&expected), "stdout was: {stdout}");
}

#[test]
fn version_short_flag() {
    let out = ktype().arg("-V").output().unwrap();
    assert!(out.status.success(), "ktype -V exited non-zero");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected = format!("ktype {}", env!("CARGO_PKG_VERSION"));
    assert!(stdout.contains(&expected), "stdout was: {stdout}");
}

#[test]
fn help_long_flag() {
    let out = ktype().arg("--help").output().unwrap();
    assert!(out.status.success(), "ktype --help exited non-zero");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ktype"), "stdout was: {stdout}");
    assert!(stdout.contains("--version"), "stdout was: {stdout}");
}

#[test]
fn help_short_flag() {
    let out = ktype().arg("-h").output().unwrap();
    assert!(out.status.success(), "ktype -h exited non-zero");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ktype"), "stdout was: {stdout}");
    assert!(stdout.contains("--version"), "stdout was: {stdout}");
}
