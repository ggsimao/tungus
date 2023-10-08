use std::fs;
use std::path::Path;

pub fn read_from_file(path: &Path) -> String {
    fs::read_to_string(path).expect(&format!("Unable to read file {}", path.display())[..])
}
