fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // libgit2-sys 0.16 does not declare advapi32, which is where libgit2 gets
    // its SID helpers (fs_path.c), registry lookups (sysdir.c) and CryptoAPI
    // entry points (hash/win32.c, rand.c). The MSVC toolchain happens to pull
    // advapi32 in transitively, so this only shows up as a link failure when
    // building against the x86_64-pc-windows-gnu target.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
}
