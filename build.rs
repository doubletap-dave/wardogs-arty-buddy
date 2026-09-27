fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icons/arty-buddy.ico");
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target != "windows" {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set("CompanyName", "Ghostweasel Labs");
    res.set("ProductName", "Wardogs Arty Buddy");
    res.set("FileDescription", "Wardogs Arty Buddy");
    res.set("InternalName", "wardogs-arty-buddy");
    res.set("OriginalFilename", "wardogs-arty-buddy.exe");
    res.set(
        "LegalCopyright",
        "Copyright (c) 2026 doubletap-dave. WARDOGS is not covered by this project's license.",
    );
    res.set_icon("assets/icons/arty-buddy.ico");
    res.compile()
        .expect("windows version resource failed to compile");
}
