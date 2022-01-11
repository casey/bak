pub(crate) use std::{
  convert::TryInto,
  ffi::OsString,
  fmt::{self, Display, Formatter},
  fs, io,
  path::{Component, Path, PathBuf},
};

pub(crate) use crate::{error::Error, template::Template};

mod extension;
pub use self::extension::Extension;
pub use self::extension::DEFAULT_EXTENSION;
