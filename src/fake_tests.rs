mod tests {
    use crate::{was_command_executed, Command};

    #[test]
    fn using_test_code_unmocked() {
        let output = Command::new("echo")
            .arg("hello")
            .current_dir("$HOME")
            .output()
            .unwrap();

        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());

        assert!(was_command_executed(&["echo", "hello"], Some("$HOME")));
        assert!(!was_command_executed(&["echo", "hello", "world"], None));
    }

    #[test]
    fn using_test_code_mocked() {
        crate::mock("echo")
            .with_arg("world")
            .with_status(1)
            .with_stdout("WORLD")
            .with_stderr("failed")
            .register();

        let mut cmd = Command::new("echo");
        cmd.arg("world");
        let output = cmd.output().unwrap();
        assert!(!output.status.success());
        assert_eq!(output.stdout, b"WORLD");
        assert_eq!(output.stderr, b"failed");

        assert!(was_command_executed(&["echo", "world"], None));
    }

    #[test]
    fn git_mocks() {
        crate::mock("git")
            .with_arg("push")
            .with_stdout(b"Everything up-to-date")
            .register();

        crate::mock("git")
            .with_arg("add")
            .with_arg(".")
            .with_stdout(b"nothing added to commit but untracked files present")
            .register();

        fn run_git_add() {
            let output = Command::new("git").arg("add").arg(".").output().unwrap();
            assert_eq!(
                output.stdout,
                b"nothing added to commit but untracked files present",
            );
        }

        fn run_git_push() {
            let output = Command::new("git").arg("push").output().unwrap();
            assert_eq!(output.stdout, b"Everything up-to-date");
        }

        run_git_add();
        run_git_push();

        assert!(was_command_executed(&["git", "add", "."], None));
        assert!(was_command_executed(&["git", "push"], None));
        assert!(!was_command_executed(&["git"], None));
        assert!(!was_command_executed(&["git", "push", "--force"], None));
    }

    #[test]
    fn file_etc_mock() {
        crate::mock("file")
            .current_dir("/")
            .with_arg("etc")
            .with_stdout(b"etc: directory")
            .register();

        fn run_file_etc() {
            let output = Command::new("file")
                .current_dir("/")
                .arg("etc")
                .output()
                .unwrap();

            assert_eq!(output.stdout, b"etc: directory");
        }

        run_file_etc();

        assert!(was_command_executed(&["file", "etc"], Some("/")));
    }
}
