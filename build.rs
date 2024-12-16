fn main() -> std::io::Result<()> {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "resources.gresource",
    );

    if cfg!(target_os = "windows") {
        use std::{env, fs, path::PathBuf, process::Command};

        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        fs::copy(manifest_dir.join("icon.ico"), out_dir.join("icon.ico")).unwrap();
        fs::write(out_dir.join("icon.rc"), "icon ICON icon.ico").unwrap();

        Command::new("windres")
            .current_dir(&out_dir)
            .arg("icon.rc")
            .arg("icon.lib")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        println!(
            "cargo:rustc-link-search={}",
            out_dir.into_os_string().into_string().unwrap()
        );
        println!("cargo:rustc-link-lib=icon");
    }

    Ok(())
}
