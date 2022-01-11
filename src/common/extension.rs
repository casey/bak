use std::ffi::{OsStr, OsString};
pub static DEFAULT_EXTENSION: &str = "bak";

/// An extension, which may or may not be interpreted as a format string.
///
/// For example, if saving the file `a.ufo` while it and `a.bak.1.ufo` exists and the Extension is
/// a format string `bak{}.ufo`, then `a.bak.2.ufo` is the next file that will be created. If the
/// extension is `bak.ufo`, then `a.bak.ufo.1` would be created.
///
/// Two format string `{}` means "start at 0" (default). `{+}` means "start at 1". To simplify the
/// implementation and keep dependencies low, the last provided is `{++}` (start at 2). No other
/// offset makes logical sense than 0 (programmer's view), 1 (Lua programmer's view), or 2
/// (Microsoft view).
///
/// Remember: only ``Extension``'s created via `new_format_str` have formatting applied.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Extension {
    /// Extension w/o leading .
    pub inner: OsString,
    /// Whether the extension is literal or not
    is_format_str: bool,
    /// Zero (or otherwise) pad offset with n characters (e.g. '1' ⇒ '001')
    pub(crate) n_pad: u8,
    /// Whether to prepend period to format string (default yes)
    pub(crate) n_prepend_period: bool,
}

impl Default for Extension {
    fn default() -> Self {
        Self {
            inner: DEFAULT_EXTENSION.into(),
            is_format_str: false,
            n_pad: 0,
            n_prepend_period: true,
        }
    }
}

impl Extension {
    /// Make a new plain extension regardless of `{}` or `{+}` etc. content
    pub fn new_plain(inner: OsString) -> Self {
        Self { inner, ..Default::default() }
    }
    /// Make a formatted extension. See ``Extension``.
    pub fn new_format_str(inner: OsString) -> Self {
        let ret = Self { inner, is_format_str: true, ..Default::default() };
        ret.valid_or_panic();
        ret
    }
}

impl Extension {
    pub(crate) fn valid_or_panic(&self) {
        if !self.clone().is_valid() {
            panic!("Multiple format strings aren't supported");
        }
    }

    fn count_formatters(&self) -> usize {
        ["{}", "{+}", "{++}"].iter().map(|fmt| self.inner.to_string_lossy().matches(fmt).count()).sum()
    }

    fn is_valid(&mut self) -> bool {
        let count = self.count_formatters();
        self.is_format_str = count != 0;
        count == 0 || count == 1
    }
}

impl Extension {
    /// Is this a format string?
    pub fn is_format_str(&mut self) -> bool {
        self.is_format_str && self.is_valid()
    }

    /// Set offset 0-padding amount.
    pub fn offset_n_pad(mut self, n_pad: u8) -> Self {
        self.n_pad = n_pad;
        self
    }

    /// Don't prepend period to this extension's format string. E.g., " ({})" will become "(2)" not
    /// "(.2)"
    pub fn no_prepend_period_to_n(mut self) -> Self {
        self.n_prepend_period = false;
        self
    }
}

impl AsRef<Extension> for Extension {
    fn as_ref(&self) -> &Extension {
        &self
    }
}

impl<'a> Into<&'a OsStr> for &'a Extension {
    fn into(self) -> &'a OsStr {
        self.inner.as_ref()
    }
}

impl Into<Extension> for OsString {
    fn into(self) -> Extension {
        Extension::new_plain(self)
    }
}

impl Into<Extension> for &OsStr {
    fn into(self) -> Extension {
        Extension::new_plain(self.to_owned())
    }
}
