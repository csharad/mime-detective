//! The [`MimeDetective`](struct.MimeDetective.html) spies for the magic number of a file or buffer
//! and spits out strongly typed Mimes.

extern crate mime;
extern crate magic;

#[cfg(feature = "rocket_data")]
extern crate rocket;

use magic::{Cookie, flags, MagicError};
use std::path::Path;
use std::{error, fmt};
use mime::FromStrError;
use std::fs::File;
use std::io::{self, Read};

/// To detect the MimeType/ContentType using the magic library
pub struct MimeDetective {
    cookie: Cookie
}

impl MimeDetective {
    /// Initialize detective with magic database from `/usr/share/misc/magic.mgc`.
    ///
    /// Requires system to have libmagic installed
    pub fn new() -> Result<MimeDetective, DetectiveError> {
        let cookie = Cookie::open(flags::MIME_TYPE)?;
        cookie.load(&["/usr/share/misc/magic.mgc"])?;
        Ok(MimeDetective {
            cookie
        })
    }

    /// Detect Mime of a filepath
    pub fn detect_filepath<P: AsRef<Path>>(&self, filename: P) -> Result<mime::Mime, DetectiveError> {
        let mime_str = self.cookie.file(filename)?;
        let mime: mime::Mime = mime_str.parse()?;
        Ok(mime)
    }

    /// Detect Mime of a file
    pub fn detect_file(&self, file: &mut File) -> Result<mime::Mime, DetectiveError> {
        let mut buf: [u8; 2] = [0; 2];
        file.read_exact(&mut buf)?;
        self.detect_buffer(&buf)
    }

    /// Detect Mime of a buffer
    pub fn detect_buffer(&self, buffer: &[u8]) -> Result<mime::Mime, DetectiveError> {
        let mime_str = self.cookie.buffer(buffer)?;
        let mime: mime::Mime = mime_str.parse()?;
        Ok(mime)
    }

    /// Detect Mime for rocket::Data.
    ///
    /// Use `features = ["rocket_data"]`
    #[cfg(feature = "rocket_data")]
    pub fn detect_data(&self, data: &rocket::Data) -> Result<mime::Mime, DetectiveError> {
        self.detect_buffer(data.peek())
    }
}

/// Represents nested error of `magic` as well as parse and io errors
#[derive(Debug)]
pub enum DetectiveError {
    Magic(MagicError),
    Parse(FromStrError),
    IO(io::Error)
}

impl error::Error for DetectiveError {
    fn description(&self) -> &str {
        match *self {
            DetectiveError::Magic(ref err) => err.description(),
            DetectiveError::Parse(ref err) => err.description(),
            DetectiveError::IO(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DetectiveError::Magic(ref err) => err.cause(),
            DetectiveError::Parse(ref err) => err.cause(),
            DetectiveError::IO(ref err) => err.cause()
        }
    }
}

impl fmt::Display for DetectiveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DetectiveError::Magic(ref err) => write!(f, "MagicError: {}", err),
            DetectiveError::Parse(ref err) => write!(f, "MimeParseError: {}", err),
            DetectiveError::IO(ref err) => write!(f, "IOError: {}", err)
        }
    }
}

impl From<MagicError> for DetectiveError {
    fn from(err: MagicError) -> Self {
        DetectiveError::Magic(err)
    }
}

impl From<FromStrError> for DetectiveError {
    fn from(err: FromStrError) -> Self {
        DetectiveError::Parse(err)
    }
}

impl From<io::Error> for DetectiveError {
    fn from(err: io::Error) -> Self {
        DetectiveError::IO(err)
    }
}

#[cfg(test)]
mod tests {
    use super::MimeDetective;
    use mime;
    use std::fs::File;
    use std::io::Read;

    fn init() -> MimeDetective {
        MimeDetective::new().expect("mime db not found")
    }

    fn read_file() -> File {
        File::open("Cargo.toml").unwrap()
    }

    #[test]
    fn detect_filepath() {
        let detective = init();
        let mime = detective.detect_filepath("Cargo.toml").unwrap();
        assert_eq!(mime::TEXT_PLAIN, mime);
    }

    #[test]
    fn detect_file() {
        let detective = init();
        let mut file = read_file();
        let mime = detective.detect_file(&mut file).unwrap();
        assert_eq!(mime::TEXT_PLAIN, mime);
    }

    #[test]
    fn detect_buffer() {
        let detective = init();
        let mut file = read_file();
        let mut buf: [u8; 2] = [0; 2];
        file.read_exact(&mut buf).unwrap();
        let mime = detective.detect_buffer(&buf).unwrap();
        assert_eq!(mime::TEXT_PLAIN, mime);
    }
}