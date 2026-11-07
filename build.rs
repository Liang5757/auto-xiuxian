use std::{env, fs, path::PathBuf, process::Command};

fn resource_compiler() -> PathBuf {
    if let Some(rc) = env::var_os("RC") {
        return rc.into();
    }
    let kits = PathBuf::from(
        env::var_os("ProgramFiles(x86)").expect("Windows SDK requires ProgramFiles(x86)"),
    )
    .join("Windows Kits/10/bin");
    let arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "arm64",
        Ok("x86") => "x86",
        _ => "x64",
    };
    let mut versions: Vec<_> = fs::read_dir(kits)
        .expect("Install the Windows SDK or set RC to rc.exe")
        .filter_map(|entry| {
            entry
                .ok()
                .map(|entry| entry.path().join(arch).join("rc.exe"))
        })
        .filter(|path| path.is_file())
        .collect();
    versions.sort();
    versions
        .pop()
        .expect("Windows SDK resource compiler rc.exe was not found")
}

fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    println!("cargo:rerun-if-env-changed=RC");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let icon = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("assets/app.ico");
    let source = out.join("app.rc");
    let resource = out.join("app.res");
    fs::write(
        &source,
        format!(
            "1 ICON \"{}\"\n",
            icon.display().to_string().replace('\\', "/")
        ),
    )
    .expect("write icon resource");
    let status = Command::new(resource_compiler())
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(&source)
        .status()
        .expect("run Windows resource compiler");
    assert!(
        status.success(),
        "application icon resource compilation failed"
    );
    println!("cargo:rustc-link-arg-bin=xiuxian={}", resource.display());
    if env::var_os("CARGO_FEATURE_ADMIN").is_some() {
        println!("cargo:rustc-link-arg-bin=xiuxian=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=xiuxian=/MANIFESTUAC:level='requireAdministrator' uiAccess='false'"
        );
    }
}
