// Copyright 2025 Tree xie.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Arg is invalid {path}"))]
    InvalidArg { path: String },
    #[snafu(display("Path is not exists {path}"))]
    PathNotExists { path: String },
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
    #[snafu(display("Lz4 decompress {source}"))]
    Lz4Decompress {
        source: lz4_flex::block::DecompressError,
    },
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

impl From<lz4_flex::block::DecompressError> for Error {
    fn from(err: lz4_flex::block::DecompressError) -> Self {
        Error::Lz4Decompress { source: err }
    }
}
