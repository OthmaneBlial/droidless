fn main() {
    println!("cargo:rerun-if-changed=native/macos-font.h");
    println!("cargo:rerun-if-changed=native/macos-text.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("native/macos-text.m")
            .flag("-fobjc-arc")
            .flag("-Wall")
            .flag("-Wextra")
            .compile("droidless_text");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
}
