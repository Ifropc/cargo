//! Tests for `lockfile-path` flag

use std::fs;

use snapbox::str;

use cargo_test_support::paths::CargoPathExt;
use cargo_test_support::{
    basic_bin_manifest, cargo_test, project, symlink_supported, Execs, ProjectBuilder,
};

const VALID_LOCKFILE: &str = r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "test_foo"
version = "0.5.0"
"#;

fn basic_project() -> ProjectBuilder {
    return project()
        .file("Cargo.toml", &basic_bin_manifest("test_foo"))
        .file("src/main.rs", "fn main() {}");
}

fn make_basic_command(execs: &mut Execs, lockfile_path_argument: String) -> &mut Execs {
    return execs
        .masquerade_as_nightly_cargo(&["unstable-options"])
        .arg("-Zunstable-options")
        .arg("--lockfile-path")
        .arg(lockfile_path_argument);
}

fn lockfile_must_exist(command: &str) -> bool {
    return command == "pkgid";
}

fn assert_lockfile_created(command: &str, make_execs: impl Fn(&mut Execs, String) -> &mut Execs) {
    if lockfile_must_exist(command) {
        return;
    }

    let lockfile_path_argument = "mylockfile/Cargo.lock";
    let p = basic_project().build();

    for _ in 1..=2 {
        make_execs(&mut p.cargo(command), lockfile_path_argument.to_string()).run();
        assert!(!p.root().join("Cargo.lock").is_file());
        assert!(p.root().join(lockfile_path_argument).is_file());
    }

    p.root()
        .join(lockfile_path_argument)
        .parent()
        .unwrap()
        .rm_rf();
    make_execs(&mut p.cargo(command), lockfile_path_argument.to_string()).run();
    assert!(!p.root().join("Cargo.lock").is_file());
    assert!(p.root().join(lockfile_path_argument).is_file());
}

fn assert_lockfile_read(command: &str, make_execs: impl Fn(&mut Execs, String) -> &mut Execs) {
    let lockfile_path_argument = "mylockfile/Cargo.lock";
    let p = basic_project()
        .file("mylockfile/Cargo.lock", VALID_LOCKFILE)
        .build();

    make_execs(&mut p.cargo(command), lockfile_path_argument.to_string()).run();

    assert!(!p.root().join("Cargo.lock").is_file());
    assert!(p.root().join(lockfile_path_argument).is_file());
}

fn assert_lockfile_override(command: &str, make_execs: impl Fn(&mut Execs, String) -> &mut Execs) {
    if lockfile_must_exist(command) {
        return;
    }

    let lockfile_path_argument = "mylockfile/Cargo.lock";
    let p = basic_project()
        .file("Cargo.lock", "This is an invalid lock file!")
        .build();

    make_execs(&mut p.cargo(command), lockfile_path_argument.to_string()).run();

    assert!(p.root().join(lockfile_path_argument).is_file());
}

fn assert_symlink_in_path(command: &str, make_execs: impl Fn(&mut Execs, String) -> &mut Execs) {
    if !symlink_supported() || lockfile_must_exist(command) {
        return;
    }

    let dst = "dst";
    let src = "somedir/link";
    let lockfile_path_argument = format!("{src}/Cargo.lock");

    let p = basic_project().symlink_dir(dst, src).build();

    fs::create_dir(p.root().join("dst"))
        .unwrap_or_else(|e| panic!("could not create directory {}", e));
    assert!(p.root().join(src).is_dir());

    make_execs(&mut p.cargo(command), lockfile_path_argument.to_string()).run();

    assert!(p.root().join(format!("{src}/Cargo.lock")).is_file());
    assert!(p.root().join(lockfile_path_argument).is_file());
    assert!(p.root().join(dst).join("Cargo.lock").is_file());
}

fn assert_symlink_lockfile(command: &str, make_execs: impl Fn(&mut Execs, String) -> &mut Execs) {
    if !symlink_supported() {
        return;
    }

    let lockfile_path_argument = "dst/Cargo.lock";
    let src = "somedir/link";
    let lock_body = VALID_LOCKFILE;

    let p = basic_project()
        .file(lockfile_path_argument, lock_body)
        .symlink(lockfile_path_argument, src)
        .build();

    assert!(p.root().join(src).is_file());

    make_execs(&mut p.cargo(command), lockfile_path_argument.to_string()).run();

    assert!(!p.root().join("Cargo.lock").is_file());
}

fn assert_broken_symlink(command: &str, make_execs: impl Fn(&mut Execs, String) -> &mut Execs) {
    if !symlink_supported() {
        return;
    }

    let invalid_dst = "invalid_path";
    let src = "somedir/link";
    let lockfile_path_argument = format!("{src}/Cargo.lock");

    let p = basic_project().symlink_dir(invalid_dst, src).build();
    assert!(!p.root().join(src).is_dir());

    make_execs(&mut p.cargo(command), lockfile_path_argument.to_string())
        .with_status(101)
        .with_stderr_data(str![[r#"
[ERROR] Failed to create lockfile-path parent directory somedir/link

Caused by:
  File exists (os error 17)

"#]])
        .run();
}

fn assert_loop_symlink(command: &str, make_execs: impl Fn(&mut Execs, String) -> &mut Execs) {
    if !symlink_supported() {
        return;
    }

    let loop_link = "loop";
    let src = "somedir/link";
    let lockfile_path_argument = format!("{src}/Cargo.lock");

    let p = basic_project()
        .symlink_dir(loop_link, src)
        .symlink_dir(src, loop_link)
        .build();
    assert!(!p.root().join(src).is_dir());

    make_execs(&mut p.cargo(command), lockfile_path_argument.to_string())
        .with_status(101)
        .with_stderr_data(str![[r#"
[ERROR] Failed to fetch lock file's parent path metadata somedir/link

Caused by:
  Too many levels of symbolic links (os error 40)

"#]])
        .run();
}

/////////////////////
//// Generic tests
/////////////////////

macro_rules! tests {
    ($name: ident, $command:expr, $f:expr) => {
        #[cfg(test)]
        mod $name {
            use super::*;

            #[cargo_test(nightly, reason = "--lockfile-path is unstable")]
            fn test_lockfile_created() {
                assert_lockfile_created($command, $f);
            }

            #[cargo_test(nightly, reason = "--lockfile-path is unstable")]
            fn test_lockfile_read() {
                assert_lockfile_read($command, $f);
            }

            #[cargo_test(nightly, reason = "--lockfile-path is unstable")]
            fn test_lockfile_override() {
                assert_lockfile_override($command, $f);
            }

            #[cargo_test(nightly, reason = "--lockfile-path is unstable")]
            fn test_symlink_in_path() {
                assert_symlink_in_path($command, $f);
            }

            #[cargo_test(nightly, reason = "--lockfile-path is unstable")]
            fn test_symlink_lockfile() {
                assert_symlink_lockfile($command, $f);
            }

            #[cargo_test(nightly, reason = "--lockfile-path is unstable")]
            fn test_broken_symlink() {
                assert_broken_symlink($command, $f);
            }

            #[cargo_test(nightly, reason = "--lockfile-path is unstable")]
            fn test_loop_symlink() {
                assert_loop_symlink($command, $f);
            }
        }
    };

    ($name: ident, $command:expr) => {
        tests!($name, $command, make_basic_command);
    };
}

fn make_add_command(execs: &mut Execs, lockfile_path_argument: String) -> &mut Execs {
    return make_basic_command(execs, lockfile_path_argument).arg("hello");
}

fn make_clean_command(execs: &mut Execs, lockfile_path_argument: String) -> &mut Execs {
    return make_basic_command(execs, lockfile_path_argument)
        .arg("--package")
        .arg("test_foo");
}

fn make_fix_command(execs: &mut Execs, lockfile_path_argument: String) -> &mut Execs {
    return make_basic_command(execs, lockfile_path_argument)
        .arg("--package")
        .arg("test_foo")
        .arg("--allow-no-vcs");
}

fn make_remove_command(execs: &mut Execs, lockfile_path_argument: String) -> &mut Execs {
    return make_basic_command(execs, lockfile_path_argument).arg("hello");
}

// tests!(add, "add", make_add_command); // TODO: works with hello, but better to make a local crate
tests!(bench, "bench");
tests!(build, "build");
tests!(check, "check");
tests!(clean, "clean", make_clean_command);
tests!(doc, "doc");
tests!(fetch, "fetch");
// tests!(fix, "fix", make_fix_command); // TODO: check why creates lockfile in a wrong place
tests!(generate_lockfile, "generate-lockfile");
tests!(metadata, "metadata");
// tests!(package, "package"); // TODO: check why lockfile is not generated
tests!(pkgid, "pkgid");
// tests!(publish, "publish");  // TODO: test registry
// tests!(remove, "remove", make_remove_command);  // TODO: modify TOML file with dependency
tests!(run, "run");
tests!(rustc, "rustc");
tests!(rustdoc, "rustdoc");
tests!(test, "test");
tests!(tree, "tree");
tests!(update, "update");
tests!(vendor, "vendor");
