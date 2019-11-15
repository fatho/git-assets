use std::error::Error;
use std::fmt;
use std::io;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum CliErrorKind {
    /// No store path has been specified, but the command was not run within a git repository.
    NotInGitRepo,
    /// Could not access the data store due to some underlying error.
    StoreAccess,
    /// A referenced content file was not found
    NoSuchContent,
    /// The store is in an inconsistent state
    Inconsistent,
    /// An unexpected error occurred.
    UnexpectedError,
}

#[derive(Debug)]
pub struct CliError {
    kind: CliErrorKind,
    source: Option<Box<dyn Error + 'static>>,
}

impl CliError {
    pub fn with_source(kind: CliErrorKind, source: Box<dyn Error + 'static>) -> Self {
        Self {
            kind,
            source: Some(source),
        }
    }

    pub fn store_access<E: Error + 'static>(source: E) -> Self {
        Self::with_source(CliErrorKind::StoreAccess, Box::new(source))
    }

    pub fn no_such_content<E: Error + 'static>(source: E) -> Self {
        Self::with_source(CliErrorKind::NoSuchContent, Box::new(source))
    }

    pub fn kind(&self) -> CliErrorKind {
        self.kind
    }

}

impl fmt::Display for CliErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            CliErrorKind::NotInGitRepo => "No store path has been specified, but the command was not run within a git repository.",
            CliErrorKind::StoreAccess => "Could not access the data store due to some underlying error.",
            CliErrorKind::NoSuchContent => "A referenced content file was not found.",
            CliErrorKind::Inconsistent => "The store is in an inconsistent state.",
            CliErrorKind::UnexpectedError => "An unexpected error occurred.",
        };
        f.write_str(msg)
    }
}

impl Error for CliErrorKind {}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(source) = self.source.as_ref() {
            write!(f, "\nSource: {}", source)?;
        }
        Ok(())
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(Box::as_ref)
    }
}

impl From<CliErrorKind> for CliError {
    fn from(kind: CliErrorKind) -> Self {
        CliError {
            kind,
            source: None,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::with_source(CliErrorKind::UnexpectedError, Box::new(err))
    }
}
