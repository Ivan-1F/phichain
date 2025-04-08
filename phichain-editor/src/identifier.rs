use bevy::prelude::Deref;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smallvec::SmallVec;
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Deref, Hash, PartialEq, Eq, Clone)]
pub struct Identifier(SmallVec<[String; 6]>);

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id: &str = Deserialize::deserialize(deserializer)?;
        Ok(id.into())
    }
}

pub trait IntoIdentifier {
    fn into_identifier(self) -> Identifier;
}

// TODO: optimize
impl<T: Into<Identifier>> IntoIdentifier for T {
    fn into_identifier(self) -> Identifier {
        self.into()
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.join("."))
    }
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self(value.split('.').map(|x| x.to_owned()).collect())
    }
}

impl FromStr for Identifier {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

#[allow(dead_code)]
impl Identifier {
    pub fn push(&mut self, name: String) {
        assert!(!name.contains('.'));
        self.0.push(name);
    }

    pub fn push_dotted(&mut self, names: &str) {
        self.0.append(
            &mut names
                .split('.')
                .map(|s| s.to_owned())
                .collect::<SmallVec<[String; 6]>>(),
        )
    }

    pub fn pop(&mut self) -> Option<String> {
        self.0.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn test_serialization() {
        assert_eq!(
            serde_json::to_string(&Identifier::from("a.b.c")).unwrap(),
            "\"a.b.c\""
        );
    }

    #[test]
    fn test_deserialization() {
        assert_eq!(
            serde_json::from_str::<Identifier>("\"a.b.c\"").unwrap(),
            Identifier::from("a.b.c")
        );
    }

    #[test]
    fn test_construction() {
        let expected: SmallVec<[String; 6]> =
            smallvec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
        assert_eq!(Identifier::from("a.b.c").0, expected);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Identifier::from("a.b.c")), "a.b.c".to_owned());
    }

    #[test]
    fn test_push() {
        let mut identifier = Identifier::from("a.b.c");
        identifier.push("d".to_owned());
        assert_eq!(identifier, Identifier::from("a.b.c.d"));
    }

    #[test]
    #[should_panic]
    fn test_push_with_dot() {
        let mut identifier = Identifier::from("a.b.c");
        identifier.push("d.e.f".to_owned());
    }

    #[test]
    fn test_push_dotted() {
        let mut identifier = Identifier::from("a.b.c");
        identifier.push_dotted("d.e.f.g");
        assert_eq!(identifier, Identifier::from("a.b.c.d.e.f.g"));
    }

    #[test]
    fn test_pop() {
        let mut identifier = Identifier::from("a.b.c");
        assert_eq!(identifier.pop(), Some("c".to_owned()));
        assert_eq!(identifier, Identifier::from("a.b"));
    }
}
