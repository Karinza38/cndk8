use core::panic;
use rand::Rng;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use url::{ParseError, Url as UrlType};

trait UrlExtension {
    // UrlExtension should be able to dictate
    // how the shorten method behaves
    // basically have an in-memory implementation
    // and provide a way for other implementations to work in the same way
    // So we can take advantage of i.e PostgreSQL's domain types to do all the heavy lifting
    fn shorten(&mut self) -> Result<bool, ParseError>;
    //fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.shortcut.is_empty() {
            panic!("Not initialized");
        }
        write!(f, "{}", self.origin)
    }
}

impl From<Url> for String {
    fn from(value: Url) -> Self {
        value.origin.to_string()
    }
}

impl PartialEq<Url> for &str {
    fn eq(&self, other: &Url) -> bool {
        *self == other.origin.to_string().as_str()
    }
}

impl PartialEq<Url> for UrlType {
    fn eq(&self, other: &Url) -> bool {
        *self == other.origin
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Url {
    origin: UrlType,
    shortcut: String,
}

impl UrlExtension for Url {
    fn shorten(&mut self) -> Result<bool, ParseError> {
        self.shortcut = self.origin.host_str().unwrap().to_string();
        if !self.shortcut.is_empty() {
            Ok(true)
        } else {
            Err(ParseError::RelativeUrlWithoutBase)
        }
    }
}

#[derive(Debug, Clone)]
struct Link {
    id: u64,
    origin: UrlType,
    target: UrlType,
    created_at: DefaultInstant,
    updated_at: DefaultInstant,
}

// Define the LinkStore trait
trait LinkStore {
    fn get(&self, id: u64) -> Option<Link>;
    fn create(&mut self, link: Link) -> Result<(), String>;
    fn update(&mut self, id: u64, link: Link) -> Result<(), String>;
    fn delete(&mut self, id: u64) -> Result<(), String>;
}

// Implement the InMemoryLinkStore
#[derive(Debug, Default)]
struct InMemoryLinkStore {
    links: Arc<Mutex<HashMap<u64, Link>>>,
}

impl InMemoryLinkStore {
    fn new() -> Self {
        InMemoryLinkStore {
            links: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl LinkStore for InMemoryLinkStore {
    fn get(&self, id: u64) -> Option<Link> {
        self.links.lock().unwrap().get(&id).cloned()
    }

    fn create(&mut self, link: Link) -> Result<(), String> {
        self.links.lock().unwrap().insert(link.id, link);
        Ok(())
    }

    fn update(&mut self, id: u64, link: Link) -> Result<(), String> {
        if self.links.lock().unwrap().contains_key(&id) {
            self.links.lock().unwrap().insert(id, link);
            Ok(())
        } else {
            Err("Link not found".to_string())
        }
    }

    fn delete(&mut self, id: u64) -> Result<(), String> {
        if self.links.lock().unwrap().remove(&id).is_some() {
            Ok(())
        } else {
            Err("Link not found".to_string())
        }
    }
}

impl Default for Link {
    fn default() -> Self {
        let created_at = DefaultInstant::default();
        let updated_at = DefaultInstant::default();
        let id = rand::thread_rng().gen();
        Link {
            id,
            origin: UrlType::parse("https://example.com").unwrap(),
            target: UrlType::parse("https://example.com").unwrap(),
            created_at: DefaultInstant::default(),
            updated_at: DefaultInstant::default(),
        }
    }
}

#[derive(Debug)]
struct DefaultInstant {
    instant: Instant,
}

impl Default for DefaultInstant {
    fn default() -> Self {
        DefaultInstant {
            instant: Instant::now(),
        }
    }
}

impl Clone for DefaultInstant {
    fn clone(&self) -> DefaultInstant {
        Self {
            instant: self.instant,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::{Any, TypeId};
    use url::Url as UrlType;

    #[test]
    fn test_type_works() {
        let myurl = Url {
            origin: UrlType::parse("https://www.example.com").unwrap(),
            shortcut: String::from("example"),
        };

        assert_eq!(myurl.type_id(), TypeId::of::<Url>(), "Type not matched");
    }

    #[test]
    fn test_shorten() {
        let mut myurl = Url {
            origin: UrlType::parse("https://www.example.com").unwrap(),
            shortcut: String::from("example"),
        };
        let result: bool = match myurl.shorten() {
            Ok(val) => val,
            Err(e) => {
                println!("Error: {:#?}", e);
                false
            }
        };

        assert!(result);
    }

    #[test]
    fn test_shorten_new() {
        let myurl = Url {
            origin: UrlType::parse("https://www.example.com").unwrap(),
            shortcut: String::from("example"),
        };
        let expect = url::Url::parse("https://www.example.com").unwrap();

        assert!(expect == myurl, "got '{myurl}' instead of '{expect}'");

        let expect = url::Url::parse("https://www.example.co").unwrap();

        assert!(expect == myurl, "got '{myurl}' instead of '{expect}'");
    }

    #[test]
    fn test_link_store() {
        // this can be done and should be done thorugh the factory
        let mut linkstore = InMemoryLinkStore::new();
        assert_eq!(
            TypeId::of::<InMemoryLinkStore>(),
            linkstore.type_id(),
            "InMemoryLinkStore type does not match"
        );

        let link = Link::default();
        let id = match linkstore.create(link) {
            Ok(val) => val,
            Err(e) => {
                println!("Error: {:#?}", e);
                ()
            }
        };
        assert!(id != (), "id is not {:#?}, {:#?}", id, linkstore);
    }

    #[test]
    fn instant() {
        let instant = DefaultInstant::default();
        println!("what? {:#?}", instant);
    }
}
