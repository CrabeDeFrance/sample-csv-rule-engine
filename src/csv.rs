use std::{
    fs::File,
    io::{Error, ErrorKind, Result},
    path::Path,
};

pub struct Csv<T> {
    pub reader: csv::Reader<T>,
    pub headers: Vec<String>,
}

pub fn read_from_path(file_path: &Path) -> Result<Csv<File>> {
    // Build the CSV reader and iterate over each record.
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(file_path)
        .map_err(|e| {
            Error::new(
                ErrorKind::InvalidInput,
                format!("Can't read file {file_path:?}: {e}"),
            )
        })?;

    let headers = reader.headers()?.iter().map(|s| s.to_owned()).collect();

    Ok(Csv { reader, headers })
}

pub fn read_from_str(s: &str) -> Result<Csv<&[u8]>> {
    // Build the CSV reader and iterate over each record.
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(s.as_bytes());

    let headers = reader.headers()?.iter().map(|s| s.to_owned()).collect();

    Ok(Csv { reader, headers })
}
