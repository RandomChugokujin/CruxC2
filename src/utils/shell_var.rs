use std::collections::HashMap;
use regex::Regex;

pub fn parse_var_def(def_str: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut split_by_equal = def_str.splitn(2, '=');
    let var_name = match split_by_equal.next(){
        Some(var_name) => var_name.to_string(),
        _none => return Err("No variable name specified".into())
    };
    let mut var_value = match split_by_equal.next(){
        Some(var_value) => var_value.to_string(),
        _none => return Err("No variable value specified".into())
    };

    // Check for Strings in the value
    if var_value.starts_with('\'') && var_value.ends_with('\'') {
        var_value.remove(0);
        var_value.pop();
    }
    else if var_value.starts_with('"') && var_value.ends_with('"') {
        // TODO: Character Escaping
        // TODO: variable resolution
        var_value.remove(0);
        var_value.pop();
    }
    return Ok((var_name, var_value));
}

pub fn variable_substitution(command: &str, var_map: &HashMap<String, String>) -> String {
    // Regex to match either $VAR or ${VAR}
    let re = Regex::new(r"\$(?:\{(\w+)\}|(\w+))").unwrap();

    let result = re.replace_all(command, |caps: &regex::Captures| {
        // Extract variable name from either capture group
        let var_name = caps.get(1).or_else(|| caps.get(2)).unwrap().as_str();

        // Lookup value from var_map first
        // If not found, restore original variable name to be passed to CruxAgent for env lookup
        var_map
            .get(var_name)
            .cloned()
            .unwrap_or(format!("${{{}}}", var_name))
    });
    result.to_string()
}

