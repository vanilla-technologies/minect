use std::{
    env,
    ffi::OsStr,
    fs::{copy, create_dir_all, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};
use walkdir::WalkDir;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    set_build_env();

    remove_license_header_from_templates(&out_dir);
}

fn set_build_env() {
    let path = "build.env";
    println!("cargo:rerun-if-changed={}", path);
    if let Ok(file) = File::open(path) {
        for (key, value) in BufReader::new(file)
            .lines()
            .map(|line| line.unwrap())
            .collect::<Vec<_>>()
            .iter()
            .flat_map(|line| line.split_once('='))
        {
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
}

fn remove_license_header_from_templates(out_dir: impl AsRef<Path>) {
    let in_dir = "src/datapack";
    println!("cargo:rerun-if-changed={}", in_dir);

    for entry in WalkDir::new(&in_dir) {
        let entry = entry.unwrap();
        let in_path = entry.path();
        let out_path = out_dir.as_ref().join(in_path);
        let file_type = entry.file_type();
        if file_type.is_dir() {
            println!("Creating dir {}", out_path.display());
            create_dir_all(out_path).unwrap();
        } else if file_type.is_file() {
            println!("Creating file {}", out_path.display());
            if in_path.extension() == Some(OsStr::new("mcfunction")) {
                let reader = BufReader::new(File::open(in_path).unwrap());
                let mut writer = BufWriter::new(File::create(out_path).unwrap());
                for line in reader
                    .lines()
                    .skip_while(|line| line.as_ref().ok().filter(|l| l.starts_with('#')).is_some())
                    .skip_while(|line| line.as_ref().ok().filter(|l| l.is_empty()).is_some())
                {
                    writer.write_all(line.unwrap().as_bytes()).unwrap();
                    writer.write_all(&[b'\n']).unwrap();
                }
            } else {
                copy(in_path, out_path).unwrap();
            }
        }
    }
}
