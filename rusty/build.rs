use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};

extern crate bindgen;
extern crate cbindgen;

#[cfg(windows)]
const PICO_TOOLS_PATH: &'static str = concat!(env!("USERPROFILE"), r"\.pico-sdk");
#[cfg(unix)]
const PICO_TOOLS_PATH: &'static str = concat!(env!("HOME"), "/.pico-sdk");

const PICO_SDK_VERSION: &'static str = "1.5.1";
const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");

fn main() {
  let pico_tools_path = Path::new(PICO_TOOLS_PATH);
  assert!(pico_tools_path.exists());

  let pico_sdk_path = pico_tools_path.join("sdk").join(PICO_SDK_VERSION);
  let current_dir = env::current_dir().unwrap();
  let project_path = current_dir.parent().unwrap();
  let build_path = project_path.join("build");
  let output_file_path = project_path.join(format!("{PKG_NAME}.h"));

  println!("cargo:rerun-if-changed=wrapper.h");
  println!("cargo:rerun-if-changed=src/lib.rs");
  println!("cargo:rerun-if-changed={}", output_file_path.display());
  println!(
    "cargo:rerun-if-changed={}",
    build_path.join("generated").display()
  );

  if !build_path.exists() {
    fs::create_dir(build_path.as_path()).unwrap();
  }

  if !build_path.join("compile_commands.json").exists() {
    Command::new("cmake")
      .args(["..", "-G", "Ninja"])
      .current_dir(build_path.to_owned())
      .output()
      .unwrap();
  }

  let mut builder = bindgen::Builder::default()
    // .raw_line("#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]")
    .use_core()
    .generate_comments(true)
    .generate_inline_functions(true)
    .ctypes_prefix("crate::std::ffi")
    .disable_untagged_union()
    .prepend_enum_name(false)
    .layout_tests(false)
    .header("wrapper.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

  // C:\Users\kagan\.pico-sdk\toolchain\13_2_Rel1\arm-none-eabi\include
  builder = builder.clang_args([
    format!(
      "-I{}",
      pico_tools_path
        .join("toolchain")
        .join("13_2_Rel1")
        .join("arm-none-eabi")
        .join("include")
        .display()
    ),
    format!(
      "-I{}",
      build_path.join("generated").join("pico_base").display()
    ),
    format!(
      "-I{}",
      pico_sdk_path
        .join("src")
        .join("boards")
        .join("include")
        .display()
    ),
    format!("-I{}", project_path.display()),
  ]);
  builder = builder.clang_args(
    include_dir(pico_sdk_path.join("src").join("rp2_common"))
      .inspect(|rp2_common| {
        dbg!(rp2_common);
      })
      .expect("rp2_common read_dir call failed"),
  );
  builder = builder.clang_args(
    include_dir(pico_sdk_path.join("src").join("common"))
      .inspect(|common| {
        dbg!(common);
      })
      .expect("common read_dir call failed"),
  );
  builder = builder.clang_args(
    include_dir(pico_sdk_path.join("src").join("rp2040"))
      .inspect(|rp2040| {
        dbg!(rp2040);
      })
      .expect("rp2040 read_dir call failed"),
  );

  eprintln!("Builder: {builder:?}");

  let bindings = builder.generate().expect("Unable to generate bindings");
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");

  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  match cbindgen::generate(&crate_dir) {
    Ok(gen) => gen,
    Err(e) => match e {
      // Ignore syntax errors because those will be handled later on by cargo build.
      cbindgen::Error::ParseSyntaxError {
        crate_name: _,
        src_path: _,
        error: _,
      } => return,
      _ => panic!("{:?}", e),
    },
  }
  .write_to_file(output_file_path);
}

fn include_dir<P>(path: P) -> io::Result<Vec<String>>
where
  P: AsRef<Path>,
{
  let dir = path.as_ref().read_dir()?;
  let folders = dir.filter_map(Result::ok).filter(|entry| {
    entry.file_type().is_ok_and(|file_type| file_type.is_dir())
      && entry.path().join("include").exists()
  });

  Ok(
    folders
      .map(|entry| format!("-I{}", entry.path().join("include").display()))
      .collect(),
  )
}
