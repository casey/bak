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

  pub(crate) fn destination(&self, extension: &OsStr) -> io::Result<PathBuf> {
    let filename_with_extension = {
      let mut filename = self.filename.clone();
      filename.push(".");
      filename.push(extension);
      filename
    };

    for n in 0u128.. {
      let candidate_filename = {
        let mut candidate_filename = filename_with_extension.clone();

        if n > 0 {
          candidate_filename.push(".");
          candidate_filename.push((n - 1).to_string());
        }

        candidate_filename
      };

      let candidate = self.directory.join(candidate_filename);

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
