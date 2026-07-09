use std::process::Command;
use tempfile::TempDir;

fn guac() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_guac"));
    cmd.env_remove("GROQ_API_KEY");
    cmd
}

fn run(args: &[&str], home: &std::path::Path) -> (String, String, bool) {
    let output = guac()
        .args(args)
        .env("GUAC_HOME", home)
        .output()
        .expect("failed to execute guac");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();
    (stdout, stderr, success)
}

#[test]
fn test_init_creates_memory_repo() {
    let dir = TempDir::new().unwrap();
    let (out, err, ok) = run(&["init"], dir.path());
    assert!(ok, "init failed: {}", err);
    assert!(out.contains("Initialized GUAC memory repository"));
    assert!(dir.path().join("memory/.git").is_dir());
}

#[test]
fn test_kg_set_and_get() {
    let dir = TempDir::new().unwrap();
    run(&["init"], dir.path());

    let (out, err, ok) = run(&["kg", "set", "user.name", "Rory"], dir.path());
    assert!(ok, "kg set failed: {}", err);
    assert!(out.contains("Set user.name = Rory"));

    let (out, _, ok) = run(&["kg", "get", "user.name"], dir.path());
    assert!(ok);
    assert!(out.contains("Rory"));
}

#[test]
fn test_branch_create_and_list() {
    let dir = TempDir::new().unwrap();
    run(&["init"], dir.path());

    let (out, err, ok) = run(&["branch", "create", "firebac"], dir.path());
    assert!(ok, "branch create failed: {}", err);
    assert!(out.contains("Created and switched to branch 'firebac'"));

    let (out, _, ok) = run(&["branch", "list"], dir.path());
    assert!(ok);
    assert!(out.contains("* firebac"));
    assert!(out.contains("main"));
}

#[test]
fn test_chat_local_mode() {
    let dir = TempDir::new().unwrap();
    run(&["init"], dir.path());

    let mut child = guac()
        .arg("chat")
        .env("GUAC_HOME", dir.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn chat");

    {
        let mut stdin = child.stdin.take().unwrap();
        std::io::Write::write_all(&mut stdin, b"hello\n/quit\n").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "chat failed: {}", stdout);
    assert!(stdout.contains("GUAC chat on branch 'main'"));
    assert!(stdout.contains("Groq API key not set"));

    // Verify a commit was made
    let (log, _, ok) = run(&["status"], dir.path());
    assert!(ok);
    assert!(log.contains("memory:"));
}

#[test]
fn test_fs_write_and_read() {
    let dir = TempDir::new().unwrap();
    run(&["init"], dir.path());

    let (out, err, ok) = run(
        &["fs", "write", "/memory/projects/firebac/warfare.yaml", "mode: tactics"],
        dir.path(),
    );
    assert!(ok, "fs write failed: {}", err);
    assert!(out.contains("Wrote /memory/projects/firebac/warfare.yaml"));

    let (out, _, ok) = run(&["fs", "read", "/memory/projects/firebac/warfare.yaml"], dir.path());
    assert!(ok);
    assert!(out.contains("mode: tactics"));
}

#[test]
fn test_recall_ranks_messages() {
    let dir = TempDir::new().unwrap();
    run(&["init"], dir.path());

    // Add a chat message via fs write to conversation file to avoid interactivity
    let conv_path = dir.path().join("memory/conversations/main/chat.yaml");
    std::fs::create_dir_all(conv_path.parent().unwrap()).unwrap();
    std::fs::write(
        &conv_path,
        r#"branch: main
messages:
- role: user
  content: hello Rory
  timestamp: 1781908532
  scores:
    importance: 0.5
    novelty: 0.5
    recency: 1.0
    repetition: 0.0
"#,
    )
    .unwrap();

    let (out, err, ok) = run(&["recall", "Rory"], dir.path());
    assert!(ok, "recall failed: {}", err);
    assert!(out.contains("hello Rory"));
}
