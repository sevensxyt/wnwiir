use std::{
    env, fmt,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

struct Args {
    flags: Flags,
    file_path: PathBuf,
}

impl Args {
    fn new() -> Result<Self, WcError> {
        let mut flags = Flags::new();
        let mut file_path = None;

        for arg in env::args().skip(1) {
            match arg.as_str() {
                "-l" => flags.lines = true,
                "-w" => flags.words = true,
                "-c" => flags.bytes = true,
                flag if flag.starts_with("-") => {
                    return Err(WcError::InvalidArg(flag.to_string()));
                }
                path => file_path = Some(PathBuf::from(path)),
            }
        }

        if flags.none_set() {
            flags.set_all();
        }

        let file_path = file_path.ok_or(WcError::NoFileProvided)?;

        Ok(Self { flags, file_path })
    }
}

struct Flags {
    lines: bool,
    words: bool,
    bytes: bool,
}

impl Flags {
    fn new() -> Self {
        Self {
            lines: false,
            words: false,
            bytes: false,
        }
    }
    pub fn none_set(&self) -> bool {
        !self.lines && !self.words && !self.bytes
    }

    pub fn set_all(&mut self) {
        self.lines = true;
        self.words = true;
        self.bytes = true;
    }
}

#[derive(Debug)]
enum WcError {
    NoFileProvided,
    Io(io::Error),
    InvalidArg(String),
}

impl fmt::Display for WcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFileProvided => write!(f, "Usage: wc [-l] [-w] [-c] <file_path>"),
            Self::Io(e) => write!(f, "Error reading file: {e}"),
            Self::InvalidArg(e) => write!(f, "Invalid arg: {e}"),
        }
    }
}

impl From<io::Error> for WcError {
    fn from(e: io::Error) -> Self {
        WcError::Io(e)
    }
}

fn main() {
    if let Err(e) = run() {
        eprint!("{e}");
    }
}

fn run() -> Result<(), WcError> {
    let Args { file_path, flags } = Args::new()?;

    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    let mut lines = 0;
    let mut words = 0;
    let bytes = if flags.bytes {
        fs::metadata(&file_path)?.len()
    } else {
        0
    };

    for line in reader.lines() {
        let line = line?;

        if flags.lines {
            lines += 1;
        }

        if flags.words {
            words += line.split_whitespace().count();
        }
    }

    if flags.lines {
        print!("{lines} ");
    }

    if flags.words {
        print!("{words} ");
    }

    if flags.bytes {
        print!("{bytes} ");
    }

    Ok(())
}
