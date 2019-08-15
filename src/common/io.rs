use path_clean; // Import path clean module

/// Get the path in which data is stored.
pub fn data_dir() -> String {
    path_clean::clean("./data") // Return path
}

/// Append a given string to the data dir.
pub fn format_data_dir(s: &str) -> String {
    path_clean::clean(&format!("{}/{}", data_dir(), s)) // Return dir
}

/// Get the path in which database files are stored.
pub fn db_dir() -> String {
    format_data_dir("db") // Return dir
}

/// Append a given string to the db dir.
pub fn format_db_dir(s: &str) -> String {
    path_clean::clean(&format!("{}/{}", db_dir(), s)) // Return dir
}

/// Get the path in which config files are stored.
pub fn config_dir() -> String {
    format_data_dir("config") // Return dir
}

/// Append a given string to the config dir.
pub fn format_config_dir(s: &str) -> String {
    path_clean::clean(&format!("{}/{}", config_dir(), s)) // Return dir
}

/// Get the path in which account files are stored.
pub fn keystore_dir() -> String {
    format_data_dir("keystore") // Return dir
}

/// Append a given string to the keystore dir.
pub fn format_keystore_dir(s: &str) -> String {
    path_clean::clean(&format!("{}/{}", keystore_dir(), s)) // Return dir
}
