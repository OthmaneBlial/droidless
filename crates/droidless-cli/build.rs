fn main() {
    println!("cargo:rerun-if-changed=src/macos.m");
    println!("cargo:rerun-if-changed=../droidless-runtime/native/macos-font.h");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("src/macos.m")
            .flag("-fobjc-arc")
            .flag("-Wall")
            .flag("-Wextra")
            .compile("droidless_appkit");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=QuartzCore");
    }
}
