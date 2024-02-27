use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Arg is invalid {path}"))]
    InvalidArg { path: String },
    #[snafu(display("Io error {source}"))]
    Io { source: std::io::Error },
    #[snafu(display("Strip prefix {source}"))]
    StripPrefix { source: std::path::StripPrefixError },
    #[snafu(display("Pattern {source}"))]
    Pattern { source: glob::PatternError },
    #[snafu(display("Glob {source}"))]
    Glob { source: glob::GlobError },
    #[snafu(display("Compression is invalid {compression}"))]
    InvalidCompression { compression: String },
    #[snafu(display("Snappy {source}"))]
    Snappy { source: snap::Error },
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io { source: err }
    }
}

impl From<snap::Error> for Error {
    fn from(err: snap::Error) -> Self {
        Error::Snappy { source: err }
    }
}
