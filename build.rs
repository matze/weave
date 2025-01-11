fn main() {
    println!("cargo:rerun-if-changed=input.css");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/render.rs");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let result = std::process::Command::new("tailwindcss")
        .args([
            "--input",
            &format!("{manifest_dir}/input.css"),
            "--output",
            &format!("{out_dir}/app.css"),
            "--minify",
        ])
        .output();

    let output = match result {
        Ok(output) => output,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => panic!("tailwindcss binary not found"),
            _ => panic!("failed to run tailwindcss: {err}"),
        },
    };

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("cargo:warning=Failed building CSS");
        println!("cargo:warning={error}");
    }
}
