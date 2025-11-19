/// Parse and validate a global variable name for IIFE bundles.
///
/// Ensures the name is a valid JavaScript identifier:
/// - Must start with a letter, underscore, or dollar sign
/// - Can contain letters, numbers, underscores, or dollar signs
/// - Cannot be empty
///
/// # Examples
///
/// Valid identifiers: MyLibrary, _internal, $jquery, lib123
/// Invalid identifiers: 123abc, my-lib, my.lib, ""
///
/// # Errors
///
/// Returns an error message if the identifier is invalid.
pub fn parse_global(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("Global name cannot be empty".to_string());
    }

    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return Err(format!(
            "Global name must start with a letter, underscore, or dollar sign: '{}'",
            s
        ));
    }

    for c in s.chars() {
        if !c.is_alphanumeric() && c != '_' && c != '$' {
            return Err(format!(
                "Global name can only contain letters, numbers, underscores, or dollar signs: '{}'",
                s
            ));
        }
    }

    Ok(s.to_string())
}
