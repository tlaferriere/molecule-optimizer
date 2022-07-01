use clap::{AppSettings, Parser};
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read};

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
    let (t, nAtoms, h, edges) = match read_prob(cli.example) {
        Ok(x) => x,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
}

#[derive(Debug, Clone)]
struct ReadError {
    msg: String,
}

impl Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error reading problem definition: {}", self.msg)
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> Self {
        ReadError {
            msg: err.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for ReadError {
    fn from(err: std::num::ParseIntError) -> Self {
        ReadError {
            msg: err.to_string(),
        }
    }
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
fn read_prob(path: String) -> Result<(usize, Vec<u32>, Vec<i32>, Vec<(usize, usize)>), ReadError> {
    let file = File::open(&path)?;
    let mut lines = io::BufReader::new(file).lines();
    let (t, k, a) = if let Some(Ok(l)) = &lines.next() {
        let mut nums = l.splitn(3, ' ');
        (
            nums.next().unwrap().parse::<usize>()?,
            nums.next().unwrap().parse::<usize>()?,
            nums.next().unwrap().parse::<usize>()?,
        )
    } else {
        return Err(ReadError {
            msg: format!("file is empty: {}", path),
        });
    };

    let mut n_atoms = vec![0; k];
    let mut lines = lines.skip_while(|line| {
        if let Ok(li) = line {
            li.is_empty()
        } else {
            false
        }
    });
    if let Some(Ok(l)) = lines.next() {
        l.splitn(k, ' ').map(|s| s.parse::<u32>())
    } else {
        return Err(ReadError {
            msg: format!("unexpected end of file after reading t, k, a: {}", path),
        });
    };

    let mut h = vec![0; k * k];
    let mut last_idx: (usize, usize) = (0, 0);
    let mut lines = lines.skip_while(|line| {
        if let Ok(li) = line {
            li.is_empty()
        } else {
            false
        }
    });
    for (i, l) in lines.take(k).enumerate() {
        for (j, s) in l?.splitn(k, ' ').enumerate() {
            h[i * k + j] = s.parse()?;
            last_idx = (i, j);
        }
    }

    if last_idx != (k - 1, k - 1) {
        return Err(ReadError {
            msg: format!(
                "unexpected end of file while reading H: at line {}, column {}, in file: {}",
                last_idx.0, last_idx.1, path
            ),
        });
    }

    let mut edges = vec![(0, 0); a];
    let mut last_idx: (usize, usize) = (0, 0);
    for (i, l) in lines
        .skip_while(|l| l.is_ok() && l.unwrap().is_empty())
        .take(k)
        .enumerate()
    {
        for (j, s) in l?.splitn(k, ' ').enumerate() {
            h[i * k + j] = s.parse()?;
            last_idx = (i, j);
        }
    }

    if last_idx != (k - 1, k - 1) {
        return Err(ReadError {
            msg: format!(
                "unexpected end of file while reading H: at line {}, column {}, in file: {}",
                last_idx.0, last_idx.1, path
            ),
        });
    }

    Ok((t, n_atoms, h, edges))
}
