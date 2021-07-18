use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;

fn main() -> io::Result<()> {
    set_env()?;

    Ok(())
}

fn set_env() -> io::Result<()> {
    let path = "build.env";
    println!("cargo:rerun-if-changed={}", path);

    for (key, value) in BufReader::new(File::open(path)?)
        .lines()
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .flat_map(|line| line.split_once('='))
    {
        println!("cargo:rustc-env={}={}", key, value);
    }
    Ok(())
}
