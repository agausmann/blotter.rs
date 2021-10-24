use std::{env::args_os, fs::File, io::BufReader, process::exit};

use blotter::BlotterFile;

fn main() -> Result<(), blotter::Error> {
    let infile = args_os().nth(1).unwrap_or_else(usage);
    let mut reader = BufReader::new(File::open(infile)?);
    let blotter_file = BlotterFile::read(&mut reader)?;

    let mut buffer = Vec::new();
    blotter_file.write(&mut buffer)?;

    let mut reader = buffer.as_slice();
    let second_blotter_file = BlotterFile::read(&mut reader)?;
    eprintln!("{:#?}", second_blotter_file);
    Ok(())
}

fn usage<T>() -> T {
    eprintln!("missing argument: input file name");
    exit(1);
}
