//! Opaque, globally-unique identifiers for chart objects.
//!
//! Ids are UUID v7 (time-ordered): unique across charts and sessions, so
//! cross-chart copy-paste and merges never need remapping. They are minted
//! once at object creation and never reused or reinterpreted.
//!
//! Each object kind has its own strongly-typed id ([`NoteId`], [`EventId`],
//! ...) sharing one global id space; [`ObjectId`] is the heterogeneous form.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;
use uuid::Uuid;

#[cfg(feature = "bevy")]
use bevy::ecs::reflect::ReflectComponent;

/// A chart data type whose identity is carried by a separate id component
pub trait HasId {
    type Id: Copy + Eq + Hash + Default + Serialize + for<'de> Deserialize<'de>;
}

/// A chart object paired with its id, used at the serialization boundary
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Identified<T: HasId> {
    pub id: T::Id,
    #[serde(flatten)]
    pub data: T,
}

impl<T: HasId> Identified<T> {
    pub fn new(data: T) -> Self {
        Self {
            id: T::Id::default(),
            data,
        }
    }
}

macro_rules! define_ids {
    ($($(#[$meta:meta])* $name:ident => $variant:ident),* $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
            #[serde(transparent)]
            #[repr(transparent)]
            #[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
            #[cfg_attr(feature = "bevy", component(immutable))]
            #[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
            #[cfg_attr(feature = "bevy", reflect(opaque, Component, Clone, PartialEq, Debug))]
            pub struct $name(Uuid);

            impl $name {
                pub fn new() -> Self {
                    Self(Uuid::now_v7())
                }

                pub fn from_uuid(uuid: Uuid) -> Self {
                    Self(uuid)
                }

                pub fn uuid(self) -> Uuid {
                    self.0
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self.0)
                }
            }

            impl fmt::Debug for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, concat!(stringify!($name), "({})"), self.0)
                }
            }

            impl From<$name> for ObjectId {
                fn from(id: $name) -> Self {
                    ObjectId::$variant(id)
                }
            }
        )*

        /// An id of any chart object kind
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum ObjectId {
            $($variant($name),)*
        }

        impl ObjectId {
            pub fn uuid(self) -> Uuid {
                match self {
                    $(ObjectId::$variant(id) => id.uuid(),)*
                }
            }
        }
    };
}

define_ids! {
    /// Id of a [`crate::note::Note`]
    NoteId => Note,
    /// Id of a [`crate::event::LineEvent`]
    EventId => Event,
    /// Id of a [`crate::line::Line`]
    LineId => Line,
    /// Id of a [`crate::curve_note_track::CurveNoteTrack`]
    CurveNoteTrackId => CurveNoteTrack,
    /// Id of a [`crate::bpm_list::BpmPoint`]
    BpmPointId => BpmPoint,
    /// Id of the open project session, carrying its editable metadata and offset.
    ProjectId => Project,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ids_are_unique() {
        assert_ne!(NoteId::new(), NoteId::new());
    }

    #[test]
    fn test_serde_roundtrip() {
        let id = NoteId::new();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: NoteId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_serializes_as_uuid_string() {
        let uuid = Uuid::now_v7();
        let id = EventId::from_uuid(uuid);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{uuid}\""));
    }

    #[test]
    fn test_object_id() {
        let note_id = NoteId::new();
        let object_id: ObjectId = note_id.into();
        assert_eq!(object_id, ObjectId::Note(note_id));
        assert_eq!(object_id.uuid(), note_id.uuid());
    }
}
