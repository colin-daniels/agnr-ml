use agnr_gen::generate::generate_agnrs_to_json;
use clap::{Clap, ValueHint};
use std::path::PathBuf;

/// Generate armchair graphene nanoribbon (AGNR) structures by brute force.
/// Outputs all unique AGNR specifications in json format.
#[derive(Clap)]
#[clap(version = "0.1", author = "Colin Daniels <colin.r.daniels@gmail.com>")]
struct Opts {
    /// Maximum AGNR length to generate, in # of hexagonal unit cells
    #[clap(long)]
    max_len: usize,
    /// Maximum AGNR width to generate, in # of atoms
    #[clap(long)]
    max_width: usize,
    /// Output filename
    #[clap(short, long, parse(from_os_str), value_hint = ValueHint::FilePath)]
    output: PathBuf,
    /// Only generate symmetric AGNRs (translation, reflection, rotation, etc)
    #[clap(short, long)]
    symmetric_only: bool,
}

fn main() {
    let opts: Opts = Opts::parse();

    generate_agnrs_to_json(
        1..(opts.max_len + 1),
        2..(opts.max_width + 1),
        opts.symmetric_only,
        opts.max_width,
        opts.output,
    );
}
