use quick_xml::Result;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::collections::BTreeSet;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use crate::extractors::wikitext::tree::Tree;

const NUM_THREADS: usize = 64;

pub struct WorkQueue {
    thread_pool: ThreadPool,
    processing_articles: Arc<Mutex<BTreeSet<usize>>>,
    active_threads: Arc<AtomicUsize>,
}

impl WorkQueue {
    pub fn new() -> Self {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .unwrap();
        Self {
            thread_pool,
            processing_articles: Arc::new(Mutex::new(BTreeSet::new())),
            active_threads: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn queue<F>(
        &mut self,
        article_id: usize,
        text: Vec<u8>,
        title: String,
        text_processor: F,
        root_dir: String,
    ) where
        F: Fn(&[u8]) -> String + Sync + Send + 'static,
    {
        let processing_articles = self.processing_articles.clone();
        let active_threads = self.active_threads.clone();

        // Pause until we have some free threads
        loop {
            let active_threads = active_threads.load(Ordering::Relaxed);
            if active_threads < 10 * NUM_THREADS {
                break;
            }

            sleep(Duration::from_millis(50));
        }

        // Increase the number of active threads
        active_threads.fetch_add(1, Ordering::Relaxed);

        let root_dir = root_dir.clone();

        self.thread_pool.spawn(move || {
            // Add article to the list of articles being actively processed
            {
                let mut processing_articles = processing_articles.lock().unwrap();
                processing_articles.insert(article_id);

                if article_id % 10_000 == 0 {
                    println!("Processing the following files: {:?}", *processing_articles);
                }
            }

            // Process the text
            let text = (text_processor)(&text);

            // Write the text to a file
            write_file(&root_dir, &title, &text, article_id).unwrap();

            // Remove the article from the list of articles being processed
            {
                let mut processing_articles = processing_articles.lock().unwrap();
                processing_articles.remove(&article_id);
            }

            // Decrement number of active threads
            active_threads.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

fn write_file(root_dir: &str, title: &str, text: &str, article_id: usize) -> Result<()> {
    // Figure out where to write the file
    let filename = format!("{}_{}", article_id, title);
    let filename = filename.replace(|c: char| !c.is_alphanumeric(), "_");
    let filename: String = filename.chars().take(100).collect();
    let mut sub_dir: String = filename.chars().take(4).collect();
    if sub_dir.ends_with(|c: char| !c.is_numeric()) {
        sub_dir = "0000".to_string();
    }
    let path = format!("{}/{}/{}.json", root_dir, &sub_dir, &filename);

    // Create the directories that will contain the file
    let path = Path::new(&path);
    let prefix = path.parent().unwrap();
    create_dir_all(prefix)?;

    // Convert text to JSON
    let tree = Tree::from_string(title, text);
    let text = serde_json::to_string_pretty(&tree).unwrap();

    // Write the file
    let mut file = File::create(path);
    if let Ok(f) = file.as_mut() {
        f.write_all(text.as_bytes())?;
    } else {
        panic!("Could not write file: {}", filename);
    }

    Ok(())
}
