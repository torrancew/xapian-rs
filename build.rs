fn main() -> miette::Result<()> {
    let local = std::path::PathBuf::from("cpp");
    let mut b = autocxx_build::Builder::new("src/ffi.rs", [&local]).build()?;
    b.flag_if_supported("-std=c++14")
        .flag_if_supported("-Wno-deprecated-declarations")
        .compile("xapian-rs");

    println!("cargo:rerun-if-changed=cpp/shim.h");
    println!("cargo:rerun-if-changed=src/ffi.rs");

    println!("cargo:rustc-link-lib=xapian");

    Ok(())
}
