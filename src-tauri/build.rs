fn main() {
    println!("cargo:rerun-if-env-changed=WINSTORE_UPDATE_FEED_URL");
    tauri_build::build()
}
