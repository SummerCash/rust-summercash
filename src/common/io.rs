use path_clean; // Import path clean module

/// Get the path in which data is stored.
pub fn data_dir() -> String {
    path_clean::clean("./data") // Return path
}

/// Get the path in which database files are stored.
pub fn db_dir() -> String {
    let mut raw_data_dir = data_dir().clone(); // Set raw data dir
    raw_data_dir.push_str("/db"); // Add /db to path

    return path_clean::clean(&raw_data_dir); // Return db dir
}
