use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
    time::{Duration, SystemTime},
};

use clap::{Parser, ValueEnum};
use rayon::prelude::*;

use wikipedia_parser::extractors::links::Page;
use wikipedia_parser::par_file::ParFile;
use wikipedia_parser::progress::Progress;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    input_data_file: String,
    #[arg(long)]
    input_index_file: String,
    #[arg(long)]
    input_file_threads: u64,
    #[arg(short, long)]
    output_file: String,
    #[arg(short, long)]
    root_page: String,
    #[arg(short, long)]
    depth: f64,
    #[arg(short, long)]
    method: SubgraphMethod,
    #[arg(short, long)]
    fanout_factor: Option<f64>,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum SubgraphMethod {
    /// Get the links in a tree of fixed depth starting from the root page.
    DepthLimited,
    /// Start with X=depth.
    /// Subtract (number of links in the page)/(fanout factor) from X when descending to
    /// a child link.
    /// Stop when X <= 0.
    DepthLimitedWithFanOutFactor,
}

pub fn main() {
    let args = Args::parse();

    let index_list = read_index_file(&args.input_index_file);

    let index_map: HashMap<String, usize> = index_list
        .iter()
        .map(|i| (i.title.clone(), i.idx))
        .collect();

    let root_page_index = match index_map.get(&args.root_page) {
        Some(x) => *x,
        None => {
            eprintln!("Error: Root page not found");
            eprintln!("{}", &args.root_page);
            return;
        }
    };

    let graph = build_graph(
        &args.input_data_file,
        &index_list,
        &index_map,
        args.input_file_threads,
    );

    let mut list = HashSet::new();
    match args.method {
        SubgraphMethod::DepthLimited => {
            if args.fanout_factor.is_some() {
                eprintln!("Ignoring the fanout factor argument.");
                eprintln!("It's only used by the DepthLimitedWithFanOutFactor method");
            }
            depth_limited_subgraph(
                root_page_index,
                &index_list,
                &graph,
                args.depth.trunc() as _,
                &mut list,
            );
        }
        SubgraphMethod::DepthLimitedWithFanOutFactor => {
            let fanout_factor = match args.fanout_factor {
                Some(x) => x,
                None => {
                    eprintln!("Error: Fanout factor must be provided when using this method");
                    eprintln!("{}", &args.root_page);
                    return;
                }
            };
            depth_limited_subgraph_with_fanout_factor(
                root_page_index,
                &index_list,
                &graph,
                args.depth,
                fanout_factor,
                &mut list,
            );
        }
    }

    write_lines(list.iter().map(String::as_str), &args.output_file);
}

fn build_graph(
    filename: &str,
    index_list: &[PageIndex],
    index_map: &HashMap<String, usize>,
    threads: u64,
) -> HashMap<usize, Vec<usize>> {
    let file = File::open(filename).unwrap();
    let file_size = file.metadata().unwrap().len();
    let mut i = 0;

    let mut progress = Progress {
        total: file_size,
        rate_divider: 1024.0 * 1024.0,
        rate_unit: "MB/s",
        start: SystemTime::now(),
        window_length: Duration::from_secs(5),
        window_start: SystemTime::now(),
        window_count: 0,
    };

    let mut file = ParFile::new(filename.to_owned(), 40 * 1024 * 1024, 1, threads);
    let mut buf: Vec<u8> = Vec::new();

    let graph = index_list
        .iter()
        .map(|page_index| {
            i += 1;

            if i >= 1_000 {
                i = 0;
                let s = progress.progress(page_index.start_offset, SystemTime::now());
                print!("Progress: {}\r", s);
            }

            read_file_slice(
                &mut file,
                page_index.start_offset,
                page_index.end_offset.unwrap_or(file_size),
                &mut buf,
            );

            (page_index.idx, buf.clone())
        })
        .par_bridge()
        .map(|(idx, buf)| {
            let page: Page = serde_json::from_slice(&buf).unwrap();

            let link_idxs: Vec<_> = page
                .links
                .iter()
                .filter_map(|link| index_map.get(&link.target).map(|x| *x))
                .collect();
            (idx, link_idxs)
        })
        .collect();

    let s = progress.progress(file_size, SystemTime::now());
    println!("Progress: {}", s);
    println!();
    println!("Graph built");

    graph
}

#[derive(Debug)]
struct PageIndex {
    title: String,
    idx: usize,
    start_offset: u64,
    end_offset: Option<u64>,
}

fn read_index_file(filename: &str) -> Vec<PageIndex> {
    const BUF_SIZE: usize = 300 * 1024 * 1024;
    const BLOCK_SIZE: u64 = 4 * 1024 * 1024;
    const NUM_THREADS: u64 = 16;

    println!("Reading index ..");
    let mut index_buf = String::with_capacity(BUF_SIZE);

    let index_file = ParFile::new(filename.to_owned(), BLOCK_SIZE, 1, NUM_THREADS);
    BufReader::new(index_file)
        .read_to_string(&mut index_buf)
        .unwrap();

    println!("Processing index ..");
    let index_list: Vec<_> = index_buf
        .lines()
        // .enumerate()
        .map(|line| {
            let (start_offset, name) = line.split_once(":").unwrap();
            let start_offset: u64 = start_offset.parse().unwrap();
            let name = name.trim().to_owned();
            (name, start_offset)
        })
        .collect();

    println!("Got index list");
    let mut end_offsets: Vec<Option<u64>> = index_list
        .iter()
        .map(|(_, start_offset)| Some(*start_offset))
        .collect();

    println!("Got end offsets");
    end_offsets.push(None);

    let index_map: Vec<PageIndex> = index_list
        .into_iter()
        .zip(end_offsets[1..].into_iter())
        .enumerate()
        .map(|(i, ((name, start_offset), end_offset))| PageIndex {
            title: name,
            idx: i,
            start_offset,
            end_offset: *end_offset,
        })
        .collect();

    println!("Done");
    println!();

    index_map
}

pub fn read_file_slice(file: &mut impl Read, start: u64, end: u64, buf: &mut Vec<u8>) {
    // file.seek(SeekFrom::Start(start)).unwrap();
    buf.resize((end.checked_sub(start).expect("start > end")) as _, 0u8);
    file.read_exact(buf).unwrap();
}

pub fn read_page(file: &mut File, start: u64, end: Option<u64>, buf: &mut Vec<u8>) -> Page {
    file.seek(SeekFrom::Start(start)).unwrap();
    let end = end.unwrap_or_else(|| file.metadata().unwrap().len());
    buf.resize((end.checked_sub(start).expect("start > end")) as _, 0u8);
    file.read_exact(buf).unwrap();

    let page: Page = serde_json::from_slice(&buf).unwrap();
    page
}

pub fn write_lines<'a>(items: impl Iterator<Item = &'a str>, filename: &str) {
    let mut out_str = String::new();
    for item in items {
        out_str.push_str(&item);
        out_str.push('\n');
    }

    let output_path = Path::new(filename);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(filename, out_str).unwrap();
}

fn depth_limited_subgraph(
    root: usize,
    index_list: &[PageIndex],
    graph: &HashMap<usize, Vec<usize>>,
    depth: usize,
    list: &mut HashSet<String>,
) {
    if depth == 0 {
        return;
    }

    let links = graph.get(&root).unwrap();
    for link in links {
        if list.insert(index_list[*link].title.clone()) {
            depth_limited_subgraph(root, index_list, graph, depth - 1, list);
        }
    }
}

fn depth_limited_subgraph_with_fanout_factor(
    root: usize,
    index_list: &[PageIndex],
    graph: &HashMap<usize, Vec<usize>>,
    depth: f64,
    links_factor: f64,
    list: &mut HashSet<String>,
) {
    if depth <= 0.0 {
        return;
    }

    let links = graph.get(&root).unwrap();
    let x = links.len() as f64 / links_factor;
    for link in links {
        if list.insert(index_list[*link].title.clone()) {
            depth_limited_subgraph_with_fanout_factor(
                root,
                index_list,
                graph,
                depth - x,
                links_factor,
                list,
            );
        }
    }
}
