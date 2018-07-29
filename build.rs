extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let cosmosis_inc = env::var("COSMOSIS_INC").expect("COSMOSIS_INC should be defined");
    let manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Link to `libcosmosis.so`
    println!("cargo:rustc-link-search=native={}", cosmosis_inc);
    println!("cargo:rustc-link-lib=dylib=cosmosis");

    // Make sure to regenerate bindings if COSMOSIS_INC changes, or if the
    // wrapper header file changes
    println!("cargo:rerun-if-env-changed=COSMOSIS_INC");
    println!("cargo:rerun-if-changed={}", cosmosis_inc);
    println!("cargo:rerun-if-changed={}", manifest_path.join("wrapper.h").to_str().unwrap());

    let bindings = bindgen::Builder::default()
         .clang_arg(format!("-I{}", cosmosis_inc))
         .enable_cxx_namespaces()
         .header("wrapper.h")
         .whitelist_type("DATABLOCK_STATUS")
         .rustified_enum("DATABLOCK_STATUS")
         .whitelist_type("c_datablock")
         .whitelist_type("datablock_type_t")
         .rustified_enum("datablock_type_t")
         /* Creation/destruction */
         .whitelist_function("make_c_datablock")
         .whitelist_function("destroy_c_datablock")
         .whitelist_function("clone_c_datablock")
         /* Basic Section and value access */
         .whitelist_function("c_datablock_has_section")
         .whitelist_function("c_datablock_get_section_name")
         .whitelist_function("c_datablock_num_sections")
         .whitelist_function("c_datablock_delete_section")
         .whitelist_function("c_datablock_copy_section")
         .whitelist_function("c_datablock_has_value")
         .whitelist_function("c_datablock_get_value_name")
         .whitelist_function("c_datablock_num_values")
         .whitelist_function("c_datablock_get_type")
         .whitelist_function("c_datablock_get_array_length")
         /* Simple getters */
         .whitelist_function("c_datablock_get_int")
         .whitelist_function("c_datablock_get_bool")
         .whitelist_function("c_datablock_get_double")
         .whitelist_function("c_datablock_get_complex")
         .whitelist_function("c_datablock_get_string")
         /* Simple putters */
         .whitelist_function("c_datablock_put_int")
         .whitelist_function("c_datablock_put_bool")
         .whitelist_function("c_datablock_put_double")
         .whitelist_function("c_datablock_put_complex")
         .whitelist_function("c_datablock_put_string")
         /* Simple replacement */
         .whitelist_function("c_datablock_replace_int")
         .whitelist_function("c_datablock_replace_bool")
         .whitelist_function("c_datablock_replace_double")
         .whitelist_function("c_datablock_replace_complex")
         .whitelist_function("c_datablock_replace_string")
         /* Getting 1D arrays */
         .whitelist_function("c_datablock_get_int_array_1d_preallocated")
         .whitelist_function("c_datablock_get_double_array_1d_preallocated")
         .whitelist_function("c_datablock_get_complex_array_1d_preallocated")
         /* Putting 1D arrays */
         .whitelist_function("c_datablock_put_int_array_1d")
         .whitelist_function("c_datablock_put_double_array_1d")
         .whitelist_function("c_datablock_put_complex_array_1d")
         /* Replacing 1D arrays */
         .whitelist_function("c_datablock_replace_int_array_1d")
         .whitelist_function("c_datablock_replace_double_array_1d")
         .whitelist_function("c_datablock_replace_complex_array_1d")
         /* TODO: Neglecting higher-dimensional arrays */
         .generate()
         .expect("Error generating bindings");

    bindings.write_to_file(manifest_path.join("src/_raw_cosmosis_bindings.rs"))
            .expect("Error writing bindings");
}
