use std::{
    io::Read,
    sync::{atomic::Ordering, mpsc},
    thread,
    time::Duration,
};

use super::ParFile;

fn make_tempfile(name: &str) -> String {
    let tempdir = std::env::temp_dir();
    let tmpfile = tempdir.join(format!("rust-test-{}.tmp", name));
    let tmpfile_name = tmpfile.to_str().unwrap().to_owned();
    tmpfile_name
}

#[test]
fn read_1byte() {
    let filename = make_tempfile("read_1byte");
    std::fs::write(&filename, "12345").unwrap();

    let mut parfile = ParFile::new(filename, 1, 1, 1);
    let mut buf = [10u8];

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'1']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'2']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'3']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'4']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'5']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(buf, [b'5']);
    assert_eq!(n, 0);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(buf, [b'5']);
    assert_eq!(n, 0);
}

#[test]
fn read_1byte_2threads() {
    let filename = make_tempfile("read_1byte_2threads");
    std::fs::write(&filename, "12345").unwrap();

    let mut parfile = ParFile::new(filename, 1, 1, 2);
    let mut buf = [10u8];

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'1']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'2']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'3']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'4']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(n, 1);
    assert_eq!(buf, [b'5']);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(buf, [b'5']);
    assert_eq!(n, 0);

    let n = parfile.read(&mut buf).unwrap();
    assert_eq!(buf, [b'5']);
    assert_eq!(n, 0);
}

#[test]
fn read_to_end() {
    let src = "Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis.";

    let filename = make_tempfile("read_to_end");
    std::fs::write(&filename, src).unwrap();

    let mut parfile = ParFile::new(filename.clone(), 1, 1, 1);
    let mut out = String::new();
    parfile.read_to_string(&mut out).unwrap();

    assert_eq!(out, src);

    let mut parfile = ParFile::new(filename.clone(), 2, 1, 1);
    let mut out = String::new();
    parfile.read_to_string(&mut out).unwrap();

    assert_eq!(out, src);

    let mut parfile = ParFile::new(filename.clone(), 2, 2, 2);
    let mut out = String::new();
    parfile.read_to_string(&mut out).unwrap();

    assert_eq!(out, src);
}

#[test]
fn drop_test() {
    let (active_threads, parfile) = {
        let parfile = ParFile::new(String::from("/dev/zero"), 1, 1, 2);
        let active_threads = parfile.active_thread_count();
        thread::sleep(Duration::from_secs_f64(0.1));
        (active_threads, parfile)
    };
    thread::sleep(Duration::from_secs_f64(0.1));
    assert_ne!(active_threads.load(Ordering::Relaxed), 0);
    drop(parfile);
    thread::sleep(Duration::from_secs_f64(0.1));
    assert_eq!(active_threads.load(Ordering::Relaxed), 0);
}
