use crate::common::*;

use std::fs::File;

pub(crate) fn tempdir(filenames: &[&str]) -> io::Result<tempfile::TempDir> {
  let dir = tempfile::tempdir()?;

  for filename in filenames {
    let path = dir.path().join(filename);

    File::create(path)?;
  }

  Ok(dir)
}
