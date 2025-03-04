use mockcmd::Command;

fn run_git_add() {
    let output = Command::new("git").arg("add").arg(".").output().unwrap();

    if output.stdout != "" {
        eprintln!("Error! output didn't outputted the outpution");
    }
}
