#[macro_export]
macro_rules! create_regex {
    ($patterns:expr) => {
        $patterns
            .iter()
            .map(|p| Regex::new(&p).unwrap())
            .collect::<Vec<Regex>>()
    };
}
