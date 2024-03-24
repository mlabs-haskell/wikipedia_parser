use clap::{Parser, ValueEnum};

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use wikipedia_parser::extractors;
use wikipedia_parser::par_file::ParFile;
use wikipedia_parser::xml_parser::XMLParser;

#[derive(Parser, Debug)]
struct Args {
    /// Input file to read data from
    #[arg(short, long)]
    input_file: String,
    /// Path to the data file to write to
    #[arg(long)]
    output_data_file: String,
    /// Path to the index file to write to
    #[arg(long)]
    output_index_file: String,
    /// Number of threads to use for reading the input file
    #[arg(long, default_value_t = 16u64)]
    input_file_threads: u64,
    /// The extractor to run
    #[arg(short, long)]
    extractor: Extractor,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Extractor {
    // Extract links graph
    Links,
    // Extract contents
    Contents,
}

const PAR_FILE_BLOCK_SIZE: usize = 100 * 1024 * 1024;
const PAR_FILE_QUEUE_SIZE: u64 = 1;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let extractor = match args.extractor {
        Extractor::Links => extractors::links::extract,
        Extractor::Contents => extractors::wikitext::extract,
    };

    let input_filename = args.input_file;

    let input_file = File::open(&input_filename)?;
    let input_file_size = input_file.metadata()?.len();

    let input_par_file = ParFile::new(
        input_filename,
        PAR_FILE_BLOCK_SIZE as _,
        PAR_FILE_QUEUE_SIZE,
        args.input_file_threads,
    );

    let input_file_reader = BufReader::with_capacity(PAR_FILE_BLOCK_SIZE, input_par_file);

    let data_file = args.output_data_file;
    let index_file = args.output_index_file;

    ensure_parent_folder_exists(&data_file);
    ensure_parent_folder_exists(&index_file);

    let xml_parser = XMLParser::new(
        data_file,
        index_file,
        extractor,
        input_file_reader,
        input_file_size,
    )?;
    xml_parser.parse_xml()?;

    Ok(())
}

fn ensure_parent_folder_exists(filename: &str) {
    let path = Path::new(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .expect(&format!("Failed to create folder for: {}", filename));
    }
}
