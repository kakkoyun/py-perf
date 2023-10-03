extern crate bindgen;

use std::env;
use std::path::PathBuf;

use bindgen::callbacks::{DeriveInfo, ParseCallbacks};
use libbpf_cargo::{Error, SkeletonBuilder};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

const PYTHON_STACK_SOURCE: &str = "./src/bpf/pyperf.bpf.c";
const PYTHON_STACK_HEADER: &str = "./src/bpf/pyperf.h";
const PYTHON_STACK_SKELETON: &str = "./src/bpf/pyperf.rs";

#[derive(Debug)]
struct BuildCallbacks;

impl ParseCallbacks for BuildCallbacks {
    fn add_derives(&self, derive_info: &DeriveInfo) -> Vec<String> {
        if derive_info.name == "PythonVersionOffsets" || derive_info.name.starts_with("Py") {
            vec![
                "Serialize".into(),
                "Deserialize".into(),
                "PartialEq".into(),
                "Eq".into(),
                "Hash".into(),
            ]
        } else if derive_info.name == "Stack" {
            vec!["PartialEq".into(), "Eq".into()]
        } else {
            vec![]
        }
    }

    // Copied from bindgen::CargoCallbacks, to tell cargo to invalidate
    // the built crate whenever any of the included header files changed.
    fn include_file(&self, filename: &str) {
        println!("cargo:rerun-if-changed={filename}");
    }
}

fn main() {
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(PYTHON_STACK_HEADER)
        .derive_default(true)
        .parse_callbacks(Box::new(BuildCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_out_file = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_out_file)
        .expect("Couldn't write bindings!");

    // Add Serde includes.
    let mut contents = String::new();
    File::open(&bindings_out_file)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    let new_contents = format!("use serde::{{Serialize, Deserialize}};\n{contents}");
    File::create(&bindings_out_file)
        .unwrap()
        .write_all(new_contents.as_bytes())
        .unwrap();

    let skel = Path::new(PYTHON_STACK_SKELETON);
    match SkeletonBuilder::new()
        .source(PYTHON_STACK_SOURCE)
        .clang_args("-Wextra -Wall -Werror")
        .debug(true)
        .build_and_generate(skel)
    {
        Ok(_) => {}
        Err(err) => match err {
            Error::Build(msg) => {
                panic!("Error running SkeletonBuilder for py-perf = Build: {msg:?}");
            }
            Error::Generate(msg) => {
                panic!("Error running SkeletonBuilder for py-perf = Generate: {msg:?}");
            }
        },
    }

    // Turn off some clippy warnings in the generated BPF skeleton.
    let mut contents = String::new();
    File::open(skel)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    let new_contents = format!("#![allow(clippy::derive_partial_eq_without_eq)]\n{contents}");
    File::create(skel)
        .unwrap()
        .write_all(new_contents.as_bytes())
        .unwrap();

    println!("cargo:rerun-if-changed={PYTHON_STACK_SOURCE}");
    println!("cargo:rerun-if-changed={PYTHON_STACK_HEADER}");
}
