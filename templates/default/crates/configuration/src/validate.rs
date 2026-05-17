pub(crate) fn validate_configuration(config: &crate::schema::Config) -> anyhow::Result<()> {
    /*
        @ Validate Configuration
    */
    ensure_not_blank(&config.primary.env, "primary.env")?;
    ensure_not_blank(&config.primary.name, "primary.name")?;
    ensure_not_blank(&config.store.url, "store.url")?;
    ensure_not_blank(&config.auth.secret, "auth.secret")?;
    ensure_not_blank(&config.auth.base_url, "auth.base_url")?;
    ensure_not_blank(&config.auth.path_prefix, "auth.path_prefix")?;
    if config.auth.secret.len() < 32 {
        anyhow::bail!("auth.secret must be at least 32 characters long");
    }
    Ok(())
}

/*
    @ Helper Functions
*/
fn ensure_not_blank(value: &str, field: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} cannot be blank");
    }
    Ok(())
}
