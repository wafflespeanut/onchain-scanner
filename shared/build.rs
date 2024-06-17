use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const AWS_REGIONS_LIST: &str = "../aws_regions.txt";
const IGNORED_POOLS_LIST: &str = "../ignored_pools.txt";

fn lines_to_vec(file: &str) -> Vec<String> {
    let bytes = fs::read(file).unwrap();
    String::from_utf8(bytes)
        .unwrap()
        .split("\n")
        .filter_map(|l| {
            let l = l.trim();
            if l == "" {
                None
            } else {
                Some(l.into())
            }
        })
        .collect()
}

fn vec_to_const(fd: &mut File, const_name: &str, vec: Vec<String>) {
    fd.write(
        format!(
            "pub const {}: [&str; {}] = {:?};\n",
            const_name,
            vec.len(),
            vec
        )
        .as_bytes(),
    )
    .unwrap();
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build.rs");
    let mut fd = fs::File::create(dest_path).expect("creating file");

    vec_to_const(&mut fd, "AWS_REGIONS", lines_to_vec(AWS_REGIONS_LIST));
    println!("cargo::rerun-if-changed={}", AWS_REGIONS_LIST);
    vec_to_const(&mut fd, "IGNORED_POOLS", lines_to_vec(IGNORED_POOLS_LIST));
    println!("cargo::rerun-if-changed={}", IGNORED_POOLS_LIST);

    println!("cargo::rerun-if-changed=build.rs");
}
