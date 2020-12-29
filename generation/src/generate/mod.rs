use crate::generate::spec::AGNRSpec;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Range;
use std::path::Path;

pub mod spec;

pub fn generate_gnrs_json(
    lengths: Range<usize>,
    widths: Range<usize>,
    symmetric_only: bool,
    max_width: usize,
    filename: impl AsRef<Path>,
) {
    eprintln!("Generating AGNRs...");
    let all_agnrs = generate_all_agnrs(lengths, widths, symmetric_only, max_width);
    eprintln!("Done generating AGNRs");

    eprintln!("Converting AGNRs to JSON format...");
    let output = File::create(filename).unwrap();
    let mut output = BufWriter::new(output);

    for entry in all_agnrs.into_iter() {
        serde_json::to_writer(&mut output, &entry).unwrap();
        output.write_all(&[b'\n']).unwrap();
    }
    eprintln!("Done");
}

pub fn generate_all_agnrs(
    lengths: Range<usize>,
    widths: Range<usize>,
    symmetric_only: bool,
    max_width: usize,
) -> HashSet<AGNRSpec> {
    let mut total_db = HashSet::<AGNRSpec>::default();
    for l in lengths.clone() {
        let mut db = HashSet::default();
        for w in widths.clone() {
            let n_before = db.len();
            AGNRSpec::generate_all(w, l * 2, &mut db, symmetric_only, max_width);
            println!(
                "w: {:2} l: {:2} delta: {:6} total: {}",
                w,
                l * 2,
                db.len() - n_before,
                db.len()
            );
        }

        // check for GNRs that repeat (e.g. [A] -> [A, A])
        for s in lengths.start..l {
            if l % s == 0 {
                db.retain(|gnr| {
                    let beginning = &gnr.spec[0..(s * 2)];
                    let symm = beginning.repeat(l / s);
                    gnr.spec != symm
                });
            }
        }

        let n_before = total_db.len();
        total_db.extend(db.into_iter());
        println!(
            "l: {:2} delta: {:6} total: {}",
            l * 2,
            total_db.len() - n_before,
            total_db.len()
        );
    }

    total_db
}

pub fn generate_all_agnrs2(
    lengths: Range<usize>,
    widths: Range<usize>,
) -> HashSet<spec::temp::AGNRSpec> {
    let mut total_db = HashSet::<spec::temp::AGNRSpec>::default();
    for l in lengths.clone() {
        let mut db = HashSet::default();
        for w in widths.clone() {
            let n_before = db.len();
            spec::temp::AGNRSpec::generate_all(w, l * 2, &mut db);
            println!(
                "w: {:2} l: {:2} delta: {:6} total: {}",
                w,
                l * 2,
                db.len() - n_before,
                db.len()
            );
        }

        // check for GNRs that repeat (e.g. [A] -> [A, A])
        for s in lengths.start..l {
            if l % s == 0 {
                db.retain(|gnr| {
                    let beginning = &gnr.slices()[0..(s * 2)];
                    let symm = beginning.repeat(l / s);
                    gnr.slices() != symm
                });
            }
        }

        let n_before = total_db.len();
        total_db.extend(db.into_iter());
        println!(
            "l: {:2} delta: {:6} total: {}",
            l * 2,
            total_db.len() - n_before,
            total_db.len()
        );
    }

    total_db
}
