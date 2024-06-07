use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

const AWS_REGIONS_LIST: &str = "../aws_regions.txt";

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

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build.rs");
    let mut fd = fs::File::create(dest_path).expect("creating file");
    let aws_list = lines_to_vec(AWS_REGIONS_LIST);
    fd.write(
        format!(
            "pub const AWS_REGIONS: [&str; {}] = {:?};\n",
            aws_list.len(),
            aws_list
        )
        .as_bytes(),
    )
    .unwrap();
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", AWS_REGIONS_LIST);
}
