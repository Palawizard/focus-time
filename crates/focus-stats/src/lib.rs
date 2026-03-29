pub const fn crate_name() -> &'static str {
    "focus-stats"
}

pub fn bootstrap_status() -> &'static str {
    "aggregation-pipeline-scaffolded"
}

#[cfg(test)]
mod tests {
    use super::bootstrap_status;

    #[test]
    fn exposes_bootstrap_status() {
        assert_eq!(bootstrap_status(), "aggregation-pipeline-scaffolded");
    }
}
