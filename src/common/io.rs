use path_clean; // Import path clean module

/// Get the path in which data is stored.
pub fn data_dir() -> String {
    path_clean::clean("./data") // Return path
}

/// Get the path in which database files are stored.
pub fn db_dir() -> String {
    let mut raw_data_dir = data_dir().clone(); // Set raw data dir
    raw_data_dir.push_str("/db"); // Add /db to path

    path_clean::clean(&raw_data_dir) // Return db dir
}

/// Append a given string to the db dir.
pub fn format_db_dir(s: &str) -> String {
    path_clean::clean(&format!("{}/{}", db_dir(), s)) // Return dir
}

/// Get the path in which config files are stored.
pub fn config_dir() -> String {
    let mut raw_data_dir = data_dir().clone(); // Set raw data dir
    raw_data_dir.push_str("/config"); // Add /config to path

    path_clean::clean(&raw_data_dir) // Return db dir
}

/// Append a given string to the config dir.
pub fn format_config_dir(s: &str) -> String {
    path_clean::clean(&format!("{}/{}", config_dir(), s)) // Return dir
}
