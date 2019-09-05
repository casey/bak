//! `bak` is a Rust library for safely moving files out of the way.
//!
//! The API has a few methods, but the one to start with is
//! `bak::move_aside(PATH)`.
//!
//! `move_aside("foo")` will move the file or directory "foo" to
//! "foo.bak", if there isn't already something there. If there is
//! already a file called "foo.bak", it will move it to "foo.bak.0", and
//! so on.
//!
//! `move_aside()` returns an `io::Result<PathBuf>` containing the path
//! to the renamed file.
//!
//! You can call `move_aside_with_extension(PATH, EXTENSION)` if you'd
//! like to use an extension other than "bak". To see where a file would
//! be moved without actually moving it, call `destination_path(PATH)`
//! or `destination_with_extension(PATH, EXTENSION)`.
//!
//! ## caveats
//!
//! - If `bak` is in the middle of renaming a file from `foo` to
//!   `foo.bak`, and another process or thread concurrently creates a
//!   file called `foo.bak`, `bak` will silently overwrite the newly
//!   created `foo.bak` with `foo`. This is because `bak` uses
//!   `std::fs::rename`, which clobbers destination files.
#![deny(missing_docs)]

mod common;
mod error;
mod template;

#[cfg(test)]
mod testing;

const DEFAULT_EXTENSION: &str = "bak";

use crate::common::*;

/// Move aside `path` using the default extension, "bak".
pub fn move_aside(path: impl AsRef<Path>) -> io::Result<PathBuf> {
  move_aside_with_extension(path, DEFAULT_EXTENSION)
}

/// Move aside `path` using `extension`.
pub fn move_aside_with_extension(
  path: impl AsRef<Path>,
  extension: impl AsRef<OsStr>,
) -> io::Result<PathBuf> {
  let template = Template::new(path.as_ref())?;

  let source = template.source();

  let destination = template.destination(extension.as_ref())?;

  fs::rename(source, &destination)?;

  Ok(destination)
}

/// Get the destination that `path` would be moved to by `move_aside(path)`
/// without actually moving it.
pub fn destination(path: impl AsRef<Path>) -> io::Result<PathBuf> {
  destination_with_extension(path, DEFAULT_EXTENSION)
}

/// Get the destination that `path` would be moved to by
/// `move_aside(path, extension)` without actually moving it.
pub fn destination_with_extension(
  path: impl AsRef<Path>,
  extension: impl AsRef<OsStr>,
) -> io::Result<PathBuf> {
  Template::new(path.as_ref())?.destination(extension.as_ref())
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::fs::File;

  macro_rules! test {
    {
      name:        $name:ident,
      files:       [$($file:expr),*],
      source:      $source:expr,
      extension:   $extension:expr,
      destination: $destination:expr,
    } => {
      #[test]
      fn $name() -> io::Result<()> {
        let mut files = Vec::new();
        $(
          {
            files.push(PathBuf::from($file));
          }
        )*;

        let source = PathBuf::from($source);

        let extension: Option<&OsStr> = $extension.map(|extension: &str| extension.as_ref());

        let desired_destination = PathBuf::from($destination);

        let tempdir = tempfile::tempdir()?;

        let base = tempdir.path();

        for file in &files {
          File::create(base.join(file))?;
        }

        let planned_destination = match extension {
          Some(extension) => destination_with_extension(base.join(&source), extension)?,
          None => destination(base.join(&source))?,
        };

        let planned_destination = planned_destination.strip_prefix(base.canonicalize()?).unwrap();

        assert_eq!(planned_destination, desired_destination);

        let actual_destination = match extension {
          Some(extension) => move_aside_with_extension(base.join(&source), extension)?,
          None => move_aside(base.join(&source))?,
        };

        let actual_destination = actual_destination.strip_prefix(base.canonicalize()?).unwrap();

        assert_eq!(actual_destination, desired_destination);

        let mut want = files.clone();
        want.retain(|file| file != &source);
        want.push(desired_destination);
        want.sort();

        let mut have = tempdir.path()
          .read_dir()?
          .map(|result| result.map(|entry| PathBuf::from(entry.file_name())))
          .collect::<io::Result<Vec<PathBuf>>>()?;
        have.sort();

        assert_eq!(have, want, "{:?} != {:?}", have, want);

        Ok(())
      }
    }
  }

  test! {
    name:        no_conflicts,
    files:       ["foo"],
    source:      "foo",
    extension:   None,
    destination: "foo.bak",
  }

  test! {
    name:        one_conflict,
    files:       ["foo", "foo.bak"],
    source:      "foo",
    extension:   None,
    destination: "foo.bak.0",
  }

  test! {
    name:        two_conflicts,
    files:       ["foo", "foo.bak", "foo.bak.0"],
    source:      "foo",
    extension:   None,
    destination: "foo.bak.1",
  }

  test! {
    name:        three_conflicts,
    files:       ["foo", "foo.bak", "foo.bak.0", "foo.bak.1"],
    source:      "foo",
    extension:   None,
    destination: "foo.bak.2",
  }

  test! {
    name:        no_conflicts_ext,
    files:       ["foo"],
    source:      "foo",
    extension:   Some("bar"),
    destination: "foo.bar",
  }

  test! {
    name:        one_conflict_ext,
    files:       ["foo", "foo.bar"],
    source:      "foo",
    extension:   Some("bar"),
    destination: "foo.bar.0",
  }

  test! {
    name:        two_conflicts_ext,
    files:       ["foo", "foo.bar", "foo.bar.0"],
    source:      "foo",
    extension:   Some("bar"),
    destination: "foo.bar.1",
  }

  test! {
    name:        three_conflicts_ext,
    files:       ["foo", "foo.bar", "foo.bar.0", "foo.bar.1"],
    source:      "foo",
    extension:   Some("bar"),
    destination: "foo.bar.2",
  }
}
