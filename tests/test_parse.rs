#[cfg(test)]
mod parse {
    mod success {
        use std::fs;
        use std::path::PathBuf;

        use diff_fuse::diff::Diff;

        fn test_impl(diff_path: PathBuf) {
            let data = fs::read_to_string(diff_path).unwrap();
            let diff = Diff::from(data.parse().unwrap());
            assert_eq!(data, diff.to_string());
        }

        casegen::for_each_file!("tests/data/svn/parse/success/", test_impl);
    }
}
