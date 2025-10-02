#[cfg(not(feature = "test"))]
mod tests {
    use mockcmd::Command;

    #[test]
    fn using_non_test_code() {
        let mut cmd = Command::new("file");
        cmd.current_dir("/");
        cmd.arg("etc");
        let output = cmd.output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"etc: directory\n");
    }
}
