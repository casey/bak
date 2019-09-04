# bak

[![crates.io](https://img.shields.io/crates/v/bak.svg)](https://crates.io/crates/bak) [![docs](https://docs.rs/bak/badge.svg)](http://docs.rs/bak)

`bak` is a Rust library for safely moving files out of the way.

The API has a few methods, but the one to start with is
`bak::move_aside(PATH)`.

`move_aside("foo")` will move the file or directory "foo" to
"foo.bak", if there isn't already something there. If there is
already a file called "foo.bak", it will move it to "foo.bak.0", and
so on.

`move_aside()` returns an `io::Result<PathBuf>` containing the path
to the renamed file.

You can call `move_aside_with_extension(PATH, EXTENSION)` if you'd
like to use an extension other than "bak". To see where a file would
be moved without actually moving it, call `destination_path(PATH)`
or `destination_with_extension(PATH, EXTENSION)`.

`bak` skips holes in sequences of backup files. For exmaple, if you
call `bak::move_aside("foo")`, and "foo.bak.12" exists, bak will
move "foo" to "foo.bak.13".

## caveats

- If `bak` is in the middle of renaming a file from `foo` to
  `foo.bak`, and another process or thread concurrently creates a
  file called `foo.bak`, `bak` will silently overwrite the newly
  created `foo.bak` with `foo`. This is because `bak` uses
  `std::fs::rename`, which clobbers destination files.
