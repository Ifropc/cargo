//! Tests for `lockfile-path` flag

use cargo_test_support::{basic_bin_manifest, cargo_test, main_file, project, Project};

fn run_with_path(p: &Project, command: &str, lockfile_path_argument: &str) {
    p.cargo(command)
        // .arg("--lockfile-path")
        // .arg(lockfile_path_argument)
        .run();
}

#[track_caller]
fn assert_lockfile_created(command: &str, lockfile_path_argument: &str) {
    let p = project()
        .file("Cargo.toml", &basic_bin_manifest("test_foo"))
        .file("src/main.rs", "fn main() {}")
        .build();

    run_with_path(&p, command, lockfile_path_argument);

    assert!(!p.root().join("Cargo.lock").is_file(), );
    assert!(p.root().join(lockfile_path_argument).is_file());
}

fn assert_bad_name() {
    // TODO: fail if name is not Cargo.lock -> make it unit test
}

fn assert_symlink_in_path() {
    // TODO: test path foo/bar/Cargo.lock where foo/bar is a symlink
}

fn assert_symlink_lockfile() {
    // TODO: test path foo/Cargo.lock where Cargo.lock is a symlink
}

fn assert_broken_symlink() {
    // TODO: test broken symlink path or a loop
}

fn assert_lockfile_override() {
    // TODO: test that when Cargo.lock exists, if path is
    // foo/Cargo.lock it's properly created and used
}

fn assert_readonly_dir_10096() {
    // TODO: test the following structure:
    // readonly/
    //   Cargo.toml
    //   src/
    // writeable/
    //   Cargo.lock
    // This test ensures that #10096 is fixed
}



#[cargo_test]
fn metadata_lockfile_created() {
    assert_lockfile_created("metadata", "mylockfile/Cargo.lock");
}
