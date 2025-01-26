use std::path::PathBuf;

fn _generate_bindings() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=static=hsakmt");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg("-I/opt/rocm/include/hsakmt")
        .clang_arg("-I/opt/rocm/include")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from("./tmp");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    println!("cargo:rustc-link-lib=numa");

    println!("cargo:rustc-link-lib=drm");
    println!("cargo:rustc-link-lib=drm_amdgpu");

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=native=/opt/rocm/lib");
    println!("cargo:rustc-link-lib=static=hsakmt");

    // _generate_bindings();
}
