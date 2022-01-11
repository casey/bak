use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) struct Template {
  source: PathBuf,
  directory: PathBuf,
  filename: OsString,
}

impl Template {
  pub(crate) fn new(path: &Path) -> io::Result<Template> {
    let source = path.canonicalize()?;

    let components = source.components().collect::<Vec<Component>>();

    assert!(!components.is_empty());

    if components.len() == 1 {
      return Err(io::Error::new(io::ErrorKind::InvalidInput, Error::Root));
    }

    let directory = components[..components.len() - 1]
      .iter()
      .collect::<PathBuf>();

    let filename = if let Component::Normal(filename) = components[components.len() - 1] {
      filename.to_owned()
    } else {
      return Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        Error::UnexpectedFinalPathComponent,
      ));
    };

    Ok(Template {
      source,
      directory,
      filename,
    })
  }

  pub(crate) fn source(&self) -> &Path {
    &self.source
  }

  fn make_candidate_for_n(&self, extension: &mut Extension, n: u128) -> PathBuf {
    let mut filename: PathBuf = self.filename.clone().into();
    let is_format_str = extension.is_format_str();
    let format_n = |n: u128| (['.'].iter().map(|c|*c)).chain(
            std::iter::repeat('0').take((extension.n_pad as isize - n.to_string().len() as isize).try_into().unwrap_or(0))
        ).chain(
            n.to_string().chars()
        ).collect::<String>();

    let extension_str = if n > 0 {
        if is_format_str {
            extension.inner.to_string_lossy().replace("{}", &format_n(n - 1)).replace("{+}", &format_n(n)).replace("{++}", &format_n(n + 2))
        } else {
            let mut ext: String = extension.inner.to_string_lossy().to_string();
            ext.push_str(&format_n(n - 1));
            ext
        }
    } else {
        if is_format_str {
            let replacement = if extension.n_prepend_period {
                "."
            } else {
                ""
            };
            extension.inner.to_string_lossy().replace("{}", replacement).replace("{+}", replacement).replace("{++}", replacement)
        } else {
            extension.inner.to_string_lossy().to_string()
        }
    };
    filename.set_extension(extension_str);
    filename = self.directory.join(PathBuf::from(filename));
    filename
  }

  pub(crate) fn destination<'a>(&self, extension: impl Into<&'a Extension>) -> io::Result<PathBuf> {
    let mut extension = extension.into().clone();
    extension.valid_or_panic();
    for n in 0u128.. {
      let candidate = self.make_candidate_for_n(&mut extension, n);

      match fs::symlink_metadata(&candidate) {
        Ok(_) => continue,
        Err(io_error) => {
          if io_error.kind() == io::ErrorKind::NotFound {
            return Ok(candidate);
          } else {
            return Err(io_error);
          }
        }
      }
    }

    unreachable!();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::testing;

  use std::ffi::OsStr;
  use std::fs;

  #[test]
  fn root() {
    assert_eq!(
      Template::new("/".as_ref()).unwrap_err().kind(),
      io::ErrorKind::InvalidInput
    );
  }

  #[test]
  fn simple() -> io::Result<()> {
    let filename = "foo";

    let tempdir = testing::tempdir(&[filename])?;

    let tempdir_path = tempdir.path().canonicalize()?;

    let path = tempdir_path.join(filename);

    let template = Template::new(&path)?;

    assert_eq!(template.source, path);
    assert_eq!(template.directory, tempdir_path);
    assert_eq!(template.filename, OsStr::new("foo"));

    Ok(())
  }

  #[test]
  fn dot() -> io::Result<()> {
    let tempdir = tempfile::tempdir()?;

    let tempdir_path = tempdir.path().canonicalize()?;

    let foo = tempdir_path.join("foo");

    fs::create_dir(&foo)?;

    let path = foo.join(".");

    let have = Template::new(&path)?;

    assert_eq!(have.source, path);
    assert_eq!(have.directory, tempdir_path);
    assert_eq!(have.filename, OsStr::new("foo"));

    Ok(())
  }

  #[test]
  fn dotdot() -> io::Result<()> {
    let tempdir = tempfile::tempdir()?;

    let tempdir_path = tempdir.path().canonicalize()?;

    let foo = tempdir_path.join("foo");

    fs::create_dir(&foo)?;

    let path = foo.join("..");

    let have = Template::new(&path)?;

    assert_eq!(have.source, tempdir_path);
    assert_eq!(have.directory, tempdir_path.parent().unwrap());
    assert_eq!(have.filename, tempdir_path.file_name().unwrap());

    Ok(())
  }
}
