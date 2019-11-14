//! Tests for the command line interface of git-assets

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;

use sha2::Digest;

use git_assets;

const TEST_CONTENTS: &[u8] = b"this is a test\nand a second line";
const TEST_CONTENTS_REF: &[u8] =
    b"git-assets v1\nfbbeac4b21cc086bfd7ed8b9c7b99e014e436b8bb0069114054ca374e8e69b26\n";

/// Check that cleaning a file puts it into the correct place in the store.
#[test]
fn test_clean() {
    run_test("clean", |env| {
        let mut bin = env.run_test_command(&["clean"]);
        bin.stdin_send(TEST_CONTENTS);

        // Ensure program output is correct
        assert_eq!(bin.expect_success().as_slice(), TEST_CONTENTS_REF);

        // Ensure store is in a correct state:
        assert_empty_staging(env);
        assert_data_count(env, 1);
        assert_data_contents(env, TEST_CONTENTS);
    });
}

/// Check cleaning two files at about the same time.
#[test]
fn test_clean_double() {
    run_test("clean_double", |env| {
        let mut bin1 = env.run_test_command(&["clean"]);
        let mut bin2 = env.run_test_command(&["clean"]);

        bin1.stdin_send(TEST_CONTENTS);
        bin2.stdin_send(TEST_CONTENTS);
        bin1.stdin_close();
        bin2.stdin_close();
        let out1 = bin1.expect_success();
        let out2 = bin2.expect_success();

        // Ensure program output is correct
        assert_eq!(out1.as_slice(), TEST_CONTENTS_REF);
        assert_eq!(out2.as_slice(), TEST_CONTENTS_REF);

        // Ensure store is in a correct state:
        assert_empty_staging(env);
        assert_data_count(env, 1);
        assert_data_contents(env, TEST_CONTENTS);
    });
}

/// Check that a cleaned file can be smudged afterwards.
#[test]
fn test_clean_smudge() {
    run_test("clean_smudge", |env| {
        {
            let mut bin = env.run_test_command(&["clean"]);
            bin.stdin_send(TEST_CONTENTS);

            // Ensure program output is correct
            assert_eq!(bin.expect_success().as_slice(), TEST_CONTENTS_REF);
        }

        {
            let mut bin = env.run_test_command(&["smudge"]);
            bin.stdin_send(TEST_CONTENTS_REF);

            // Ensure program output is correct
            assert_eq!(bin.expect_success().as_slice(), TEST_CONTENTS);
        }

        // Ensure store is in a correct state:
        assert_empty_staging(env);
        assert_data_count(env, 1);
        assert_data_contents(env, TEST_CONTENTS);
    });
}

fn assert_empty_staging(env: &TestEnv) {
    assert_eq!(
        fs::read_dir(env.store_dir.join("staging")).unwrap().count(),
        0
    );
}

fn assert_data_count(env: &TestEnv, num_data_files: usize) {
    assert_eq!(
        fs::read_dir(env.store_dir.join("data")).unwrap().count(),
        num_data_files
    );
}

/// Assert that the given contents are stored in a data file with the corresponding hash as name.
fn assert_data_contents(env: &TestEnv, contents: &[u8]) {
    let hash = git_assets::hash::Sha256Hash::hash_bytes(contents);
    let actual = fs::read(env.store_dir.join("data").join(hash.to_hex_string())).unwrap();
    assert_eq!(actual.as_slice(), contents);
}

/// Simple interface for interacting with the child via stdin/stdout
struct TestChild {
    child: process::Child,
}

impl TestChild {
    /// Send input to the child's stdin. Panics if sending fails
    fn stdin_send(&mut self, input: &[u8]) {
        self.child
            .stdin
            .as_mut()
            .expect("stdin already closed")
            .write_all(input)
            .unwrap()
    }

    fn stdin_close(&mut self) {
        self.child.stdin = None;
    }

    fn wait_output(self) -> process::Output {
        self.child
            .wait_with_output()
            .expect("waiting on child failed")
    }

    /// Assert that the program returned successful and return its stdout.
    fn expect_success(self) -> Vec<u8> {
        let out = self.wait_output();
        assert!(out.status.success());
        out.stdout
    }
}

struct TestEnv {
    store_dir: PathBuf,
    bin: PathBuf,
}

impl TestEnv {
    fn new(name: &str) -> TestEnv {
        let bin = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("git-assets");

        eprintln!("git-assets binary: {}", bin.to_string_lossy());

        let process_id = std::process::id();
        let store_dir = std::env::temp_dir().join(format!("git-assets.{}.{}", name, process_id));

        if store_dir.exists() {
            panic!(
                "Previous test didn't clean up temporary store path: {}",
                store_dir.to_string_lossy()
            );
        }

        Self { store_dir, bin }
    }

    /// Build a test command with piped stdin/stdout and an initial `--store` argument.
    fn build_test_cmd(&self) -> process::Command {
        let mut cmd = process::Command::new(&self.bin);
        cmd.arg("--store")
            .arg(&self.store_dir)
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::inherit());
        cmd
    }

    fn run_test_command(&self, args: &[&str]) -> TestChild {
        let child = self
            .build_test_cmd()
            .args(args)
            .spawn()
            .expect("could not spawn child");
        TestChild { child }
    }

    fn remove_store(&self) {
        fs::remove_dir_all(&self.store_dir).expect("could not clean up temp store");
    }
}

/// Generate a temporary store directory name and call the closure.
/// This does not yet create the store directory.
fn run_test<F: FnOnce(&TestEnv) -> () + std::panic::UnwindSafe>(name: &str, callback: F) {
    let env = TestEnv::new(name);

    eprintln!("Test using store: {}", env.store_dir.to_string_lossy());

    callback(&env);

    env.remove_store();
}
