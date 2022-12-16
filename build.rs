use bindgen::callbacks::EnumVariantValue;

#[derive(Debug)]
struct ParseCallbacks {
    parent: bindgen::CargoCallbacks,
}

impl bindgen::callbacks::ParseCallbacks for ParseCallbacks {
    fn enum_variant_name(
        &self,
        _enum_name: Option<&str>,
        original_variant_name: &str,
        _variant_value: EnumVariantValue,
    ) -> Option<String> {
        if original_variant_name.starts_with("RABBITIZER_INSTR_ID_") {
            Some(original_variant_name.trim_start_matches("RABBITIZER_INSTR_ID_").to_string())
        } else if original_variant_name.starts_with("RAB_OPERAND_") {
            Some(original_variant_name.trim_start_matches("RAB_OPERAND_").to_string())
        } else {
            None
        }
    }

    fn include_file(&self, filename: &str) {
        self.parent.include_file(filename);
    }
}

fn main() {
    let c_paths: Vec<std::path::PathBuf> =
        glob::glob("rabbitizer/src/**/*.c").unwrap().filter_map(|g| g.ok()).collect();
    let h_paths: Vec<std::path::PathBuf> =
        glob::glob("rabbitizer/include/**/*.h").unwrap().filter_map(|g| g.ok()).collect();
    println!("cargo:rerun-if-changed=wrapper.h");
    for path in c_paths.iter().chain(&h_paths) {
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());
    }
    cc::Build::new()
        .files(c_paths)
        .include("rabbitizer/include")
        .warnings(false)
        .compile("rabbitizer");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Irabbitizer/include")
        .use_core()
        .ctypes_prefix("cty")
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .prepend_enum_name(false)
        .allowlist_function("Rabbitizer.*")
        .allowlist_var("Rabbitizer.*")
        .parse_callbacks(Box::new(ParseCallbacks { parent: bindgen::CargoCallbacks }));
    let result = bindings.generate().expect("Unable to generate bindings");
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    result.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings");
}
