extern crate fuse;
extern crate libc;
extern crate ctrlc;
extern crate time;
#[macro_use]
extern crate clap;
extern crate rainbowcoat;

use std::ffi::OsStr;
use libc::ENOENT;
use std::time::Duration;
use time::Timespec;
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};
use clap::{App, Arg};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::io::Write;
use std::cmp::min;

fn dir_attr(ino: u64, size: u64) -> FileAttr {
    let current_time = time::get_time();

    FileAttr {
        ino,
        size,
        blocks: 0,
        atime: current_time,
        mtime: current_time,
        ctime: current_time,
        crtime: current_time,
        kind: FileType::Directory,
        perm: 0o755,
        nlink: 2,
        uid: 0,
        gid: 0,
        rdev: 0,
        flags: 0,
    }
}

fn file_attr(ino: u64, size: u64) -> FileAttr {
    let current_time = time::get_time();

    FileAttr {
        ino,
        size,
        blocks: 0,
        atime: current_time,
        mtime: current_time,
        ctime: current_time,
        crtime: current_time,
        kind: FileType::RegularFile,
        perm: 0o644,
        nlink: 0,
        uid: 0,
        gid: 0,
        rdev: 0,
        flags: 0,
    }
}

fn make_pretty_rainbow(text: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    rainbowcoat::Colors::configure(&mut buf, 5.0, 1.0, 0.0).write_all(text.as_bytes()).unwrap();
    buf
}

fn alexrc() -> Vec<u8> {
    make_pretty_rainbow("Ich hasse alles außer Sven.")
}

fn svenrc() -> Vec<u8> {
    make_pretty_rainbow(
        "
░░░░░░░░░▄░░░░░░░░░░░░░░▄
░░░░░░░░▌▒█░░░░░░░░░░░▄▀▒▌
░░░░░░░░▌▒▒█░░░░░░░░▄▀▒▒▒▐
░░░░░░░▐▄▀▒▒▀▀▀▀▄▄▄▀▒▒▒▒▒▐
░░░░░▄▄▀▒░▒▒▒▒▒▒▒▒▒█▒▒▄█▒▐
░░░▄▀▒▒▒░░░▒▒▒░░░▒▒▒▀██▀▒▌
░░▐▒▒▒▄▄▒▒▒▒░░░▒▒▒▒▒▒▒▀▄▒▒▌
░░▌░░▌█▀▒▒▒▒▒▄▀█▄▒▒▒▒▒▒▒█▒▐
░▐░░░▒▒▒▒▒▒▒▒▌██▀▒▒░░░▒▒▒▀▄▌
░▌░▒▄██▄▒▒▒▒▒▒▒▒▒░░░░░░▒▒▒▒▌
▌▒▀▐▄█▄█▌▄░▀▒▒░░░░░░░░░░▒▒▒▐
▐▒▒▐▀▐▀▒░▄▄▒▄▒▒▒▒▒▒░▒░▒░▒▒▒▒▌
▐▒▒▒▀▀▄▄▒▒▒▄▒▒▒▒▒▒▒▒░▒░▒░▒▒▐
░▌▒▒▒▒▒▒▀▀▀▒▒▒▒▒▒░▒░▒░▒░▒▒▒▌
░▐▒▒▒▒▒▒▒▒▒▒▒▒▒▒░▒░▒░▒▒▄▒▒▐
░░▀▄▒▒▒▒▒▒▒▒▒▒▒░▒░▒░▒▄▒▒▒▒▌
░░░░▀▄▒▒▒▒▒▒▒▒▒▒▄▄▄▀▒▒▒▒▄▀
░░░░░░▀▄▄▄▄▄▄▀▀▀▒▒▒▒▒▄▄▀
░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▀▀
"
)
}

fn tronjerc() -> Vec<u8> {
    make_pretty_rainbow(
        &(0..5000)
            .fold("".to_string(), |acc, _| acc.to_string() + "tronje ")
    )
}

fn read_end(data_size: u64, offset: u64, read_size: u32) -> usize {
    (offset + min(data_size - offset, read_size as u64)) as usize
}

struct SvenFS;

impl Filesystem for SvenFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let ttl = Timespec::new(1, 0);
        if parent == 1 && name.to_str() == Some(".alexrc") {
            reply.entry(&ttl, &file_attr(2, alexrc().len() as u64), 0);
        } else if parent == 1 && name.to_str() == Some(".svenrc") {
            reply.entry(&ttl, &file_attr(3, svenrc().len() as u64), 0);
        } else if parent == 1 && name.to_str() == Some(".tronjerc") {
            reply.entry(&ttl, &file_attr(4, tronjerc().len() as u64), 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let ttl = Timespec::new(1, 0);
        match ino {
            1 => reply.attr(&ttl, &dir_attr(1, 5)),
            2 => reply.attr(&ttl, &file_attr(2, alexrc().len() as u64)),
            3 => reply.attr(&ttl, &file_attr(3, svenrc().len() as u64)),
            4 => reply.attr(&ttl, &file_attr(4, tronjerc().len() as u64)),
            _ => reply.error(ENOENT),
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, reply: ReplyData) {
        if ino == 2 {
            let (data_buf, data_size) = (alexrc(), alexrc().len() as u64);
            let read_size = read_end(data_size, offset as u64, size);
            reply.data(&data_buf[offset as usize..read_size]);
        } else if ino == 3 {
            let (data_buf, data_size) = (svenrc(), svenrc().len() as u64);
            let read_size = read_end(data_size, offset as u64, size);
            reply.data(&data_buf[offset as usize..read_size]);
        } else if ino == 4 {
            let (data_buf, data_size) = (tronjerc(), tronjerc().len() as u64);
            let read_size = read_end(data_size, offset as u64, size);
            reply.data(&data_buf[offset as usize..read_size]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if ino == 1 {
            if offset == 0 {
                reply.add(1, 0, FileType::Directory, ".");
                reply.add(1, 1, FileType::Directory, "..");
                reply.add(2, 2, FileType::RegularFile, ".alexrc");
                reply.add(3, 3, FileType::RegularFile, ".svenrc");
                reply.add(4, 4, FileType::RegularFile, ".tronjerc");
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(Arg::with_name("MOUNTPOINT")
             .help("Choose the mountpoint")
             .required(true)
             .index(1))
        .get_matches();
    let mountpoint = matches.value_of("MOUNTPOINT").unwrap();
    let options = ["-o", "ro", "-o", "fsname=svenfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    unsafe {
        println!("Mounting to {}", mountpoint);
        let _fuse_handle = fuse::spawn_mount(SvenFS, &mountpoint, &options).unwrap();

        while running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }
        println!("Unmounting and exiting");
    }
}
