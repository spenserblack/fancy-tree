//! Provides extensions for [`Path`].
use std::ffi::OsStr;
use std::path::Path;

/// Extends behavior for [`Path`].
pub trait PathExt {
    /// Gets both the file stem and the file extension.
    ///
    /// See [`Path::file_stem`]
    fn split_at_extension(&self) -> (Option<&OsStr>, Option<&OsStr>);

    /// Gets the two extensions of a path if both exist.
    ///
    /// This can be helpful to identify files like `*.tar.gz`, for example.
    fn double_extension(&self) -> Option<(&OsStr, &OsStr)>;
}

impl PathExt for Path {
    fn split_at_extension(&self) -> (Option<&OsStr>, Option<&OsStr>) {
        (self.file_stem(), self.extension())
    }

    fn double_extension(&self) -> Option<(&OsStr, &OsStr)> {
        let (file_stem, suffix_ext) = self.split_at_extension();
        file_stem
            .map(Path::new)
            .and_then(|file_stem| file_stem.extension())
            .and_then(|prefix_ext| suffix_ext.map(|suffix_ext| (prefix_ext, suffix_ext)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("foo.txt", Some("foo"), Some("txt"))]
    #[case("foo", Some("foo"), None)]
    fn test_split_at_extension<P>(
        #[case] path: P,
        #[case] expected_file_stem: Option<&str>,
        #[case] expected_extension: Option<&str>,
    ) where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let expected_file_stem = expected_file_stem.map(OsStr::new);
        let expected_extension = expected_extension.map(OsStr::new);

        assert_eq!(
            (expected_file_stem, expected_extension),
            path.split_at_extension()
        );
    }

    #[rstest]
    #[case::no_extension("foo", None)]
    #[case::one_extension("foo.tar", None)]
    #[case::two_extensions("foo.tar.gz", Some(("tar", "gz")))]
    #[case::three_extensions("foo.sh.tar.gz", Some(("tar", "gz")))]
    fn test_double_extension<P>(#[case] path: P, #[case] expected: Option<(&str, &str)>)
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let expected = expected.map(|(prefix, suffix)| (OsStr::new(prefix), OsStr::new(suffix)));

        assert_eq!(expected, path.double_extension());
    }
}
