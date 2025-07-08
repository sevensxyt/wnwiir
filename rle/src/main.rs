use std::{
    env,
    fmt::{self, Write},
    fs::File,
    io::{self, BufRead, BufReader},
    iter::{once, repeat_n},
    num::ParseIntError,
    path::PathBuf,
};

struct Args {
    flags: Flags,
    file_path: PathBuf,
}

impl Args {
    fn new() -> Result<Self, RleError> {
        let mut flags = Flags::new();
        let mut file_path = None;

        for arg in env::args().skip(1) {
            match arg.as_str() {
                "-d" => flags.use_decoder = true,
                arg if arg.starts_with("-") => return Err(RleError::InvalidArg(arg.to_string())),
                path => file_path = Some(PathBuf::from(path)),
            }
        }

        let file_path = file_path.ok_or(RleError::NoFileProvided)?;

        Ok(Self { flags, file_path })
    }
}

struct Flags {
    pub use_decoder: bool,
}

impl Flags {
    fn new() -> Self {
        Self { use_decoder: false }
    }
}

enum RleError {
    NoFileProvided,
    Io(io::Error),
    Fmt(fmt::Error),
    InvalidArg(String),
    ParseCount(ParseIntError),
}

impl fmt::Display for RleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFileProvided => write!(f, "Usage: rle <file_path>"),
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Fmt(e) => write!(f, "fmt error: {}", e),
            Self::InvalidArg(arg) => write!(f, "Invalid arg: {}", arg),
            Self::ParseCount(e) => write!(f, "Error parsing number: {}", e),
        }
    }
}

impl From<io::Error> for RleError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<fmt::Error> for RleError {
    fn from(value: fmt::Error) -> Self {
        Self::Fmt(value)
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
    }
}

fn run() -> Result<(), RleError> {
    let Args { flags, file_path } = Args::new()?;
    let reader = BufReader::new(File::open(file_path)?);

    let result = if flags.use_decoder {
        decode(reader)?
    } else {
        encode(reader)?
    };

    println!("{}", result);
    Ok(())
}

const SENTINEL: char = '\0';

fn encode(reader: BufReader<File>) -> Result<String, RleError> {
    let mut result = String::new();
    let mut prev = SENTINEL;
    let mut count = 0;

    for line in reader.lines() {
        for c in line?.chars().chain(once(SENTINEL)) {
            match prev {
                SENTINEL => {
                    count = 1;
                    prev = c;
                }
                pc if c == pc => count += 1,
                pc => {
                    write!(result, "{}{}", pc, count)?;
                    count = 1;
                    prev = c;
                }
            }
        }
    }

    Ok(result)
}

fn decode(reader: BufReader<File>) -> Result<String, RleError> {
    let mut result = String::new();
    let mut count = String::new();
    let mut curr = SENTINEL;

    for line in reader.lines() {
        for ch in line?.chars().chain(once(SENTINEL)) {
            match curr {
                SENTINEL => curr = ch,
                _pchar if ch.is_ascii_digit() => {
                    count.push(ch);
                }
                pchar => {
                    let n = count.parse::<usize>().map_err(RleError::ParseCount)?;
                    result.extend(repeat_n(pchar, n));

                    curr = ch;
                    count.clear();
                }
            }
        }
    }

    Ok(result)
}
