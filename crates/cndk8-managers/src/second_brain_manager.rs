use std::fs::OpenOptions;
use std::io::{self, Write};

pub struct SecondBrain;
pub struct SecondBrainManager;

impl SecondBrainManager {
    pub fn append_to_brain(text: &str, format: SecondBrainSupportedFormats) -> io::Result<()> {
        let brain_location = get_brain_location();
        let mut file = OpenOptions::new()
            .append(true)
            .open(brain_location)
            .unwrap();
        file.write_all(text.as_bytes())
            .expect("failed to write/append to brain");
        Ok(())
    }
}

// Add any other necessary functions or types here
fn get_brain_location() -> String {
    match std::env::var("BRAIN_LOCATION") {
        Ok(location) => location,
        Err(_) => panic!("Please set the BRAIN_LOCATION environment variable"),
    }
}

pub enum SecondBrainSupportedFormats {
    Markdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_to_brain() {
        let result =
            SecondBrainManager::append_to_brain("test", SecondBrainSupportedFormats::Markdown);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_brain_location() {
        unsafe {
            std::env::set_var("BRAIN_LOCATION", "test_brain_location");
        }
        let location = get_brain_location();
        assert_eq!(location, "test_brain_location");
    }

    #[test]
    fn test_append_to_brain_no_space() {
        unsafe {
            std::env::set_var("BRAIN_LOCATION", "test.md");
        }
        let result =
            SecondBrainManager::append_to_brain("test", SecondBrainSupportedFormats::Markdown);
        assert!(result.is_err());
    }
}
