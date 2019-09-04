use crate::common::*;

#[derive(Debug)]
pub(crate) enum Error {
  Root,
  UnexpectedFinalPathComponent,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Root => write!(f, "cannot move aside root directory"),
      Self::UnexpectedFinalPathComponent => write!(f, ""),
    }
  }
}
impl std::error::Error for Error {}
