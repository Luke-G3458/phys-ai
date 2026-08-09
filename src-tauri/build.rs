fn main() {
    // Tauri does not emit Cargo change tracking for bundle icons, so make icon
    // edits invalidate the native executable during development as well.
    println!("cargo:rerun-if-changed=icons/icon.png");
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=icons/icon.icns");

    tauri_build::build()
}
