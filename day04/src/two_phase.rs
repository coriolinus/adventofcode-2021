//! A Reader which parses an input file into two output types.
//!
//! This reader operates on newline-separated inputs, equivalent to the
//! `*_newline_sep` functions in `aoclib::input`. The first collection of lines
//! is parsed into a single type. All subsequent lines collections are parsed
//! into another type, which might have no relation to the first.
//!
//! This should be considered for inclusion into `aoclib::input`.

use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, Cursor},
    path::Path,
    str::FromStr,
};

const TEST_DATA_FILENAME: &str = "TEST DATA";

fn is_new_field(buf: &str) -> bool {
    let patterns = ["\n\n", "\n\r\n"];
    patterns.iter().any(|pat| {
        buf.as_bytes()
            .iter()
            .rev()
            .zip(pat.as_bytes().iter())
            .all(|(b, p)| b == p)
    })
}

fn get_next_item<T>(
    buf: &mut String,
    line: &mut usize,
    reader: &mut impl BufRead,
    file_name: &impl Display,
) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    buf.clear();
    while buf.is_empty() || !is_new_field(&buf) {
        *line += 1;
        if reader.read_line(buf).ok()? == 0 {
            break;
        }
    }
    (!buf.is_empty())
        .then(|| match T::from_str(&buf) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("{}:{}: {} for {:?}", file_name, *line - 1, e, buf);
                None
            }
        })
        .flatten()
}

#[derive(Debug, thiserror::Error)]
pub enum TwoPhaseError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no first line")]
    NoFirstLine,
}

/// Parse the contents of the provided reader into a single instance of `A` and a stream of `B`.
///
/// Often [`parse_two_phase`] or [`parse_two_phase_str`] are more ergonomic.
///
/// Lines are batched into clusters separated by blank lines. Once a cluster has been collected,
/// it (and internal newlines) are parsed into an instance of the appropriate type.
///
/// As whitespace is potentially significant, it is not adjusted in any way before being handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
pub fn parse_two_phase_reader<'a, A, B, Reader, Filename>(
    mut reader: Reader,
    file_name: Filename,
) -> Result<(A, impl 'a + Iterator<Item = B>), TwoPhaseError>
where
    A: 'a + FromStr,
    <A as FromStr>::Err: Display,
    B: 'a + FromStr,
    <B as FromStr>::Err: Display,
    Reader: 'a + BufRead,
    Filename: 'a + Display,
{
    let mut buf = String::new();
    let mut line: usize = 0;

    let a = get_next_item(&mut buf, &mut line, &mut reader, &file_name)
        .ok_or(TwoPhaseError::NoFirstLine)?;

    Ok((
        a,
        std::iter::from_fn(move || get_next_item(&mut buf, &mut line, &mut reader, &file_name))
            .fuse(),
    ))
}

/// Parse the file at the specified path into a single instance of `A` and a stream of `B`.
///
/// Lines are batched into clusters separated by blank lines. Once a cluster has been collected,
/// it (and internal newlines) are parsed into an instance of the appropriate type.
///
/// As whitespace is potentially significant, it is not adjusted in any way before being handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
pub fn parse_two_phase<'a, A, B>(
    path: &'a Path,
) -> Result<(A, impl 'a + Iterator<Item = B>), TwoPhaseError>
where
    A: 'a + FromStr,
    <A as FromStr>::Err: Display,
    B: 'a + FromStr,
    <B as FromStr>::Err: Display,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_two_phase_reader(
        reader,
        path.file_name()
            .expect("File::open() didn't early return before now; qed")
            .to_string_lossy(),
    )
}

/// Parse the provided data into a single instance of `A` and a stream of `B`.
///
/// Lines are batched into clusters separated by blank lines. Once a cluster has been collected,
/// it (and internal newlines) are parsed into an instance of the appropriate type.
///
/// As whitespace is potentially significant, it is not adjusted in any way before being handed to the parser.
///
/// If any record cannot be parsed, this prints the parse error on stderr and stops iteration.
pub fn parse_two_phase_str<'a, A, B>(
    data: &'a str,
) -> Result<(A, impl 'a + Iterator<Item = B>), TwoPhaseError>
where
    A: 'a + FromStr,
    <A as FromStr>::Err: Display,
    B: 'a + FromStr,
    <B as FromStr>::Err: Display,
{
    parse_two_phase_reader(Cursor::new(data), TEST_DATA_FILENAME)
}

/// This adaptor plugs into any of the parse functions, splitting each line into a set of comma-separated items.
///
/// After splitting by commas but before parsing, leading and trailing whitespace is trimmed.
pub struct TrimmedCommaSep<T>(Vec<T>);

impl<T> FromStr for TrimmedCommaSep<T>
where
    T: FromStr,
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .map(str::trim)
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .map(TrimmedCommaSep)
    }
}

impl<T> IntoIterator for TrimmedCommaSep<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Into<Vec<T>> for TrimmedCommaSep<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}
