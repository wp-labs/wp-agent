use std::fs;

use super::{FileInputProcessor, TestSink, config, temp_dir};

/// 源文件缺失：inspect_path 应返回 NotFound，process 不上抛也不写检查点。
#[test]
fn missing_source_path_returns_io_error() {
    let root = temp_dir("missing-source");
    let source_path = root.join("does-not-exist.log");
    fs::create_dir_all(root.join("state")).expect("create state");
    let mut processor = FileInputProcessor::new(config(&root, &source_path), TestSink::default());

    let err = processor
        .process_once()
        .expect_err("missing source file must surface an io error");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

/// 源路径不可读（无权限）：非 root 下应得到 PermissionDenied；
/// root 会绕过权限位，跳过断言（避免 CI root 环境误报）。
#[cfg(unix)]
#[test]
fn unreadable_source_path_returns_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    if unsafe { libc::geteuid() } == 0 {
        eprintln!("running as root; skipping permission-denied assertion");
        return;
    }

    let root = temp_dir("unreadable-source");
    let source_path = root.join("app.log");
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::write(&source_path, "first\nsecond\n").expect("write log");
    fs::set_permissions(&source_path, fs::Permissions::from_mode(0o000)).expect("make unreadable");
    let mut processor = FileInputProcessor::new(config(&root, &source_path), TestSink::default());

    let err = processor
        .process_once()
        .expect_err("unreadable source file must surface an io error");
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);

    // 恢复权限以便临时目录可被清理。
    fs::set_permissions(&source_path, fs::Permissions::from_mode(0o644))
        .expect("restore permissions");
}
