//! Like `load.rs` but a better format for inspecting component custom data.

use std::{collections::HashMap, env::args_os, fs::File, io::BufReader, process::exit};

use blotter::BlotterFile;

fn main() -> Result<(), blotter::error::Error> {
    let infile = args_os().nth(1).unwrap_or_else(usage);
    let mut reader = BufReader::new(File::open(infile)?);
    let blotter_file = BlotterFile::read(&mut reader)?.migrate();
    let name_map: HashMap<u16, &str> = blotter_file
        .component_types
        .iter()
        .map(|ty| (ty.numeric_id, ty.text_id.as_str()))
        .collect();
    for component in &blotter_file.components {
        let name = name_map
            .get(&component.type_id)
            .copied()
            .unwrap_or("(unknown)");
        let data = component
            .custom_data
            .as_ref()
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        println!();
        println!("{:?}", name);
        print!("    ({:03})", data.len());
        for byte in data {
            print!(" {:02x}", byte);
        }
        println!();
    }
    Ok(())
}

fn usage<T>() -> T {
    eprintln!("missing argument: input file name");
    exit(1);
}
