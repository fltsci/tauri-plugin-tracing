const COMMANDS: &[&str] = &["log"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        // .global_api_script_path("./dist-js/api-iife.js")
        .ios_path("ios")
        .build();
}
