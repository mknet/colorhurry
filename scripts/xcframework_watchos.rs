#!/usr/bin/env -S cargo +nightly -Zscript
---
[package]
edition = "2024"
---
//! Add watchOS slices to the xcframework that `boltffi pack apple` makes.
//!
//! This script exists only until BoltFFI supports watchOS. When it does, move
//! this work into `boltffi pack apple` and delete this file.
//!
//! Run it after `boltffi pack apple` (`just ffi-apple` does both). The script:
//!
//! 1. builds the FFI staticlib for `aarch64-apple-watchos` and
//!    `aarch64-apple-watchos-sim` with the repo toolchain (stable),
//! 2. builds it for `arm64_32-apple-watchos` (older watches) with nightly and
//!    `-Zbuild-std`, because that target is tier 3,
//! 3. merges the two device libraries into one with `lipo`,
//! 4. rebuilds the xcframework with every existing slice and the two watchOS
//!    slices,
//! 5. adds watchOS and macOS to `platforms:` in the generated `Package.swift`.
//!
//! The script needs the nightly `rust-src` component. If it is missing, the
//! script prints the command that installs it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

const PACKAGE: &str = "color-hurry-ffi";
const WATCH_DEVICE: &str = "aarch64-apple-watchos";
const WATCH_SIM: &str = "aarch64-apple-watchos-sim";
const WATCH_DEVICE_32: &str = "arm64_32-apple-watchos";

/// One library in the xcframework, as its `Info.plist` lists it.
struct Slice {
    library: PathBuf,
    headers: PathBuf,
}

fn main() {
    // Under `-Zscript`, CARGO_MANIFEST_DIR is the directory of this script.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let root = root.canonicalize().unwrap_or(root);
    let shared = root.join("apps/apple/generated/Shared");

    check_rust_src();

    let xcframework = find_xcframework(&shared);
    let slices = read_slices(&xcframework);
    let Some(first) = slices.first() else {
        fail("the xcframework has no slices other than watchOS");
    };
    let lib_name = first.library.file_name().expect("library path has a file name").to_owned();
    let headers = first.headers.clone();

    // rustup sets RUSTUP_TOOLCHAIN for this process, because of `+nightly`.
    // Remove it, so that rust-toolchain.toml selects the toolchain again.
    for target in [WATCH_DEVICE, WATCH_SIM] {
        run(
            &format!("build {target}"),
            Command::new("cargo")
                .env_remove("RUSTUP_TOOLCHAIN")
                .current_dir(&root)
                .args(["build", "--release", "-p", PACKAGE, "--lib", "--target", target]),
        );
    }
    run(
        &format!("build {WATCH_DEVICE_32}"),
        Command::new("cargo")
            .env_remove("RUSTUP_TOOLCHAIN")
            .current_dir(&root)
            .args(["+nightly", "build", "-Zbuild-std=std,panic_abort", "--release"])
            .args(["-p", PACKAGE, "--lib", "--target", WATCH_DEVICE_32]),
    );

    let built = |target: &str| root.join("target").join(target).join("release").join(&lib_name);
    let fat_dir = root.join("target/watchos-universal/release");
    fs::create_dir_all(&fat_dir).unwrap_or_else(|e| fail(&format!("create {}: {e}", fat_dir.display())));
    let fat = fat_dir.join(&lib_name);
    run(
        "merge the watchOS device libraries",
        Command::new("lipo")
            .arg("-create")
            .arg(built(WATCH_DEVICE_32))
            .arg(built(WATCH_DEVICE))
            .arg("-output")
            .arg(&fat),
    );

    // xcodebuild does not overwrite a path that exists, and the output name
    // must end in `.xcframework`.
    let staged = xcframework.with_extension("new.xcframework");
    remove_dir(&staged);
    let mut create = Command::new("xcodebuild");
    create.arg("-create-xcframework");
    for slice in &slices {
        create.arg("-library").arg(&slice.library).arg("-headers").arg(&slice.headers);
    }
    for library in [&fat, &built(WATCH_SIM)] {
        create.arg("-library").arg(library).arg("-headers").arg(&headers);
    }
    create.arg("-output").arg(&staged);
    run("create the xcframework", &mut create);

    remove_dir(&xcframework);
    fs::rename(&staged, &xcframework)
        .unwrap_or_else(|e| fail(&format!("move {} to {}: {e}", staged.display(), xcframework.display())));
    println!("xcframework_watchos: updated {}", xcframework.display());

    // BoltFFI zips the xcframework it made. The zip no longer matches the new
    // xcframework, and `Package.swift` points at the xcframework by path.
    let zip = xcframework.with_extension("xcframework.zip");
    if zip.exists() {
        fs::remove_file(&zip).unwrap_or_else(|e| fail(&format!("remove {}: {e}", zip.display())));
    }

    add_platforms(&shared.join("Package.swift"));
}

/// Stops with the install command if nightly has no `rust-src`.
fn check_rust_src() {
    let output = Command::new("rustc")
        .args(["+nightly", "--print", "sysroot"])
        .output()
        .unwrap_or_else(|e| fail(&format!("run rustc +nightly: {e}")));
    if !output.status.success() {
        fail("the nightly toolchain is not installed: run `rustup toolchain install nightly`");
    }
    let sysroot = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let lock = Path::new(&sysroot).join("lib/rustlib/src/rust/library/Cargo.lock");
    if !lock.exists() {
        fail(
            "the nightly toolchain has no rust-src, which -Zbuild-std needs. Run:\n\n    \
             rustup component add rust-src --toolchain nightly\n",
        );
    }
}

/// Returns the one xcframework in `dir`.
fn find_xcframework(dir: &Path) -> PathBuf {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| {
        fail(&format!("read {}: {e}. Run `boltffi pack apple` first", dir.display()))
    });
    let found: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "xcframework"))
        .filter(|path| !path.to_string_lossy().ends_with(".new.xcframework"))
        .collect();
    match found.as_slice() {
        [one] => one.clone(),
        [] => fail(&format!("no xcframework in {}. Run `boltffi pack apple` first", dir.display())),
        _ => fail(&format!("more than one xcframework in {}", dir.display())),
    }
}

/// Reads the library and headers path of each slice from `Info.plist`.
///
/// The function skips watchOS slices, so that the script can run more than
/// once on the same xcframework.
fn read_slices(xcframework: &Path) -> Vec<Slice> {
    let plist = xcframework.join("Info.plist");
    let count: usize = plist_value(&plist, "AvailableLibraries")
        .and_then(|count| count.parse().ok())
        .unwrap_or_else(|| fail(&format!("no AvailableLibraries in {}", plist.display())));
    let mut slices = Vec::new();
    for i in 0..count {
        let key = |name: &str| {
            plist_value(&plist, &format!("AvailableLibraries.{i}.{name}"))
                .unwrap_or_else(|| fail(&format!("no {name} for slice {i} in {}", plist.display())))
        };
        if key("SupportedPlatform") == "watchos" {
            continue;
        }
        let dir = xcframework.join(key("LibraryIdentifier"));
        slices.push(Slice {
            library: dir.join(key("LibraryPath")),
            headers: dir.join(key("HeadersPath")),
        });
    }
    slices
}

/// Returns one value from a property list. For an array, `plutil` returns
/// the number of items.
fn plist_value(plist: &Path, key_path: &str) -> Option<String> {
    let output = Command::new("plutil")
        .args(["-extract", key_path, "raw", "-o", "-"])
        .arg(plist)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Adds watchOS and macOS to `platforms:` in `Package.swift`, if that list
/// exists and does not have them.
fn add_platforms(manifest: &Path) {
    let Ok(text) = fs::read_to_string(manifest) else {
        println!("xcframework_watchos: no {}, so no platforms to add", manifest.display());
        return;
    };
    let Some(start) = text.find("platforms: [").map(|i| i + "platforms: [".len()) else {
        return;
    };
    let end = start + text[start..].find(']').unwrap_or_else(|| fail("`platforms: [` has no `]`"));
    let list = &text[start..end];

    let missing: Vec<&str> = [(".watchOS(", ".watchOS(.v10)"), (".macOS(", ".macOS(.v13)")]
        .into_iter()
        .filter(|(prefix, _)| !list.contains(prefix))
        .map(|(_, entry)| entry)
        .collect();
    if missing.is_empty() {
        return;
    }

    let body = list.trim_end();
    let tail = &list[body.len()..];
    let mut new_list = body.to_owned();
    if list.contains('\n') {
        // One entry on each line, each with a comma at the end.
        let indent: String = list
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| line.chars().take_while(|c| c.is_whitespace()).collect())
            .unwrap_or_default();
        for entry in &missing {
            new_list.push_str(&format!("\n{indent}{entry},"));
        }
    } else {
        for entry in &missing {
            if !new_list.trim_end_matches(',').is_empty() {
                new_list = format!("{}, ", new_list.trim_end_matches(','));
            }
            new_list.push_str(entry);
        }
    }
    new_list.push_str(tail);

    let text = format!("{}{}{}", &text[..start], new_list, &text[end..]);
    fs::write(manifest, text).unwrap_or_else(|e| fail(&format!("write {}: {e}", manifest.display())));
    println!("xcframework_watchos: added {} to {}", missing.join(", "), manifest.display());
}

/// Runs `command` and stops the script if it fails.
fn run(step: &str, command: &mut Command) {
    println!("xcframework_watchos: {step}");
    match command.status() {
        Ok(status) if status.success() => {}
        Ok(status) => fail(&format!("{step}: {status}")),
        Err(e) => fail(&format!("{step}: {e}")),
    }
}

fn remove_dir(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path).unwrap_or_else(|e| fail(&format!("remove {}: {e}", path.display())));
    }
}

fn fail(message: &str) -> ! {
    eprintln!("xcframework_watchos: error: {message}");
    exit(1);
}
