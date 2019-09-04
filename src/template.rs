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
    let munged_source = self.filename.to_string_lossy().into_owned();

    let munged_extension = extension.to_string_lossy().into_owned();

    let munged_siblings = self
      .directory
      .read_dir()?
      .map(|result| result.map(|entry| entry.file_name().to_string_lossy().into_owned()))
      .collect::<io::Result<Vec<String>>>()?;

    let mut present = munged_siblings
      .iter()
      // remove the source itself
      .filter(|sibling| **sibling != munged_source)
      // remove all siblings where source is not a prefix
      .filter(|sibling| sibling.starts_with(&munged_source))
      // remove everything but the prefix (".bak", ".bak.0", as well as non-candidate suffixes)
      .map(|sibling| &sibling[munged_source.len()..])
      // filter out suffixes that don't sttart with "."
      .filter(|suffix| suffix.starts_with("."))
      // filter out extensions that don't start with ".EXTENSION"
      .filter(|suffix| suffix[1..].starts_with(&munged_extension))
      // remove ".EXTENSION"
      .map(|suffix| &suffix[1 + munged_extension.len()..])
      // parse numeric extensions into indices
      // the empty extension (corresponding to ".EXTENSION") is 0
      // ".EXTENSION.0" is 1
      // ".EXTENSION.1" is 2
      // and so forth
      .flat_map(|n| {
        if n == "" {
          Some(0)
        } else if n.starts_with(".") {
          n[1..].parse::<u64>().ok().map(|n| n + 1)
        } else {
          None
        }
      })
      .collect::<Vec<u64>>();

    present.sort();

    let i = match present.last() {
      Some(i) => i + 1,
      None => 0,
    };

    let mut destination_filename = self.filename.clone();

    destination_filename.push(".");
    destination_filename.push(extension);

    if i > 0 {
      destination_filename.push(".");
      destination_filename.push(&(i - 1).to_string());
    }

    Ok(self.directory.join(destination_filename))
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
