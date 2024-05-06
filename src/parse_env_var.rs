pub fn parse_env_var<T: std::str::FromStr>(var_name: &str) -> T {
    dotenv::var(var_name)
        .ok()
        .and_then(|val| val.parse().ok())
        .expect(format!("{var_name} not found in environment variables").as_str())
}
