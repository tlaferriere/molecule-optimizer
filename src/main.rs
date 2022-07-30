use std::fs::File;
use std::io;
use std::io::BufRead;
use std::string::String;

use clap::Parser;
use itertools::Itertools;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    example: String,
    #[clap(short, long)]
    print: bool,
}

fn main() {
    let cli = Cli::parse();
    let (t, n_atoms, h, edges) = read_prob(cli.example);
    println!("t = {t}, n_atoms = {n_atoms:?}, h = {h:?}, edges = {edges:#?}")
}

/// Read problem description
///
/// # Arguments
/// * path: path to file containing problem
///
/// # Returns
/// Result containing tuple of (t, nAtoms, H, edges)
/// * t: number of nodes
/// * nAtoms: number of each atom. shape = k
/// * H: energy of each atom combination. shape = k x k in a contiguous vector
/// * edges: available edges of target graph. shape = A x 2
/// The error type is ReadError
fn read_prob(path: String) -> (usize, Vec<u32>, Vec<i32>, Vec<(usize, usize)>) {
    let file = File::open(&path).expect("Could not open file.");
    let mut lines = io::BufReader::new(file)
        .lines()
        .enumerate()
        .map(|(n, line)| (n, line.expect(&format!("Error reading file at line {n}."))))
        .filter(|(_, line)| !line.is_empty());
    let (line_no, current_line) = lines.next().expect("File is empty.");
    let mut nums = current_line.splitn(3, ' ');
    let t = nums
        .next()
        .expect(&format!("t is missing at line {line_no}."))
        .parse::<usize>()
        .expect(&format!("Could not parse t at line {line_no}."));
    let k = nums
        .next()
        .expect(&format!("k is missing at line {line_no}."))
        .parse::<usize>()
        .expect(&format!("Could not parse k at line {line_no}."));
    let a = nums
        .next()
        .expect(&format!("a is missing at line {line_no}."))
        .parse::<usize>()
        .expect(&format!("Could not parse a at line {line_no}."));

    // let empty_line = |(n, cur_line): &(usize, io::Result<String>)| {
    //     let line: &str = cur_line
    //         .borrow();
    //     line.is_empty()
    // };
    let (line_no, current_line) = lines
        .next()
        .expect(&format!("Unexpected end of file after line {line_no}"));
    let n_atoms = current_line
        .splitn(k, ' ')
        .map(|s| {
            s.parse::<u32>().expect(&format!(
                "Unable to parse number of atoms at line {line_no}"
            ))
        })
        .collect();

    let (to_take, mut lines) = lines.tee();
    let (line_no, current_line) = to_take
        .take(k)
        .reduce(|(_, acc), (n, line)| (n, format!("{acc} {line}")))
        .expect(&format!("h not found at line {line_no}"));
    let h = current_line
        .splitn(k * k, ' ')
        .map(|s| {
            s.parse::<i32>()
                .expect(&format!("Unable to read an energy at line {line_no}."))
        })
        .collect();
    // Advance the iterator because take doesn't mutate it.
    _ = lines.nth(k - 1);

    let mut edges = lines
        .take(a)
        .map(|(line_no, current_line)| {
            let mut nums = current_line.splitn(2, " ");
            (
                nums.next()
                    .expect(&format!("Node missing at line {line_no}"))
                    .parse::<usize>()
                    .expect(&format!("Unable to parse node at line {line_no}")),
                nums.next()
                    .expect(&format!("Node missing at line {line_no}"))
                    .parse::<usize>()
                    .expect(&format!("Unable to parse node at line {line_no}")),
            )
        })
        .collect();

    (t, n_atoms, h, edges)
}
