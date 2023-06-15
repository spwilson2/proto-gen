pub mod codegen_csharp;
pub mod codegen_rust;
pub mod intern;
pub mod parse_tree;
pub mod parser;
pub mod serializable_tree;

use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::PathBuf,
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // Paths for input files.
    proto_files: Vec<PathBuf>,

    // Output formats
    #[arg(long)]
    rust_out: Option<PathBuf>,
    #[arg(long)]
    csharp_out: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    // Crawl input files
    for file in cli.proto_files.iter() {
        for entry in walkdir::WalkDir::new(file) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() || entry.file_type().is_symlink() {
                if entry.path().exists()
                    && entry.path().extension().is_some_and(|ext| ext == "proto")
                {
                    // Parse and generate for each input file.
                    let f = File::open(entry.path()).unwrap();
                    let reader = BufReader::new(f);
                    // Attempt parse. TODO/FIXME: Only supports ascii
                    let mut p =
                        crate::parser::Parser::new(reader.bytes().map(|b| char::from(b.unwrap())));
                    let res = p.parse();
                    let parse_tree = res.unwrap();
                    let serial_tree =
                        serializable_tree::SerializeTree::from_parse_tree(&parse_tree);

                    let mut opts = File::options();
                    let gen_file_opts = opts.create(true).write(true).truncate(true);
                    //
                    // TODO: User should supply an out-dir, not a specific file path.
                    //
                    if let Some(ref path) = cli.rust_out {
                        let f = gen_file_opts.clone().open(path).unwrap();
                        let mut writer = BufWriter::new(f);
                        codegen_rust::RustCodeGen::gen(&mut writer, &parse_tree, &serial_tree)
                            .unwrap();
                    }
                    if let Some(ref path) = cli.csharp_out {
                        let f = gen_file_opts.clone().open(path).unwrap();
                        let mut writer = BufWriter::new(f);
                        codegen_csharp::CsharpCodeGen::gen(&mut writer, &parse_tree, &serial_tree)
                            .unwrap();
                    }
                    println!("{}", entry.path().display());
                }
            }
        }
    }
}
