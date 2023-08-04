//! flatten [`toml::Table`]s into [`Vec`]s of `keya.keyb.keyc = value` strings
//!
//! Matches up with alacritty's expected `update` logic for `alacritty msg config`.
//! Note that alacritty actually expects the values to be YAML at the moment,
//! but the values are close enough that things fit fine for us.


/// flatten this table (who's key is `current_path`), appending results to res
fn subtable(table: &toml::Table, res: &mut Vec<String>, current_path: &str) {
    for (key, value) in table.iter() {
        match value {
            // basic values just become `key = value`
            toml::Value::String(_) |
            toml::Value::Integer(_) |
            toml::Value::Float(_) | 
            toml::Value::Boolean(_) |
            toml::Value::Datetime(_) => {
                if current_path.is_empty() {
                    res.push(format!("{key} = {value}"));
                } else {
                    res.push(format!("{current_path}.{key} = {value}"));
                }
            },
            // tables get mapped
            toml::Value::Table(tbl) => {
                if current_path.is_empty() {
                    subtable(tbl, res, key);
                } else {
                    subtable(tbl, res, &format!("{current_path}.{key}"));
                }
            },

            // arrays have no special handling for now, but because we expect the inside to be
            // YAML, we might wanna adapt in the future
            toml::Value::Array(_) => {
                // NB(directxman12): right now alacritty expects this to be serialized as yaml, but
                // really it should be toml in the future.  We just serialize as TOML, since for
                // basic values the syntax is close enough to be fine.

                // TODO(directxman12): special handling for this?
                if current_path.is_empty() {
                    res.push(format!("{key} = {value}"));
                } else {
                    res.push(format!("{current_path}.{key} = {value}"));
                }
            }
        }
    }
}

/// flatten settings as per the [module][`self`] docs
pub fn settings(raw: toml::Table) -> Vec<String> {
    let mut res = vec![];
    subtable(&raw, &mut res, "");
    res
}
