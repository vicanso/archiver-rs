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
}
