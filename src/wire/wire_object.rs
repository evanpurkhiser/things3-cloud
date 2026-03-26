use crate::wire::area::{AreaPatch, AreaProps};
use crate::wire::checklist::{ChecklistItemPatch, ChecklistItemProps};
use crate::wire::tags::{TagPatch, TagProps};
use crate::wire::task::{TaskPatch, TaskProps};
use crate::wire::tombstone::{CommandProps, TombstoneProps};
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::de::DeserializeOwned;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;
use strum::{Display, EnumString};

pub type WireItem = BTreeMap<String, WireObject>;

/// A single wire object entry keyed by UUID.
#[derive(Debug, Clone, PartialEq)]
pub struct WireObject {
    pub operation_type: OperationType,
    pub entity_type: Option<EntityType>,
    pub payload: Properties,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Properties {
    TaskCreate(TaskProps),
    TaskUpdate(TaskPatch),
    ChecklistCreate(ChecklistItemProps),
    ChecklistUpdate(ChecklistItemPatch),
    TagCreate(TagProps),
    TagUpdate(TagPatch),
    AreaCreate(AreaProps),
    AreaUpdate(AreaPatch),
    TombstoneCreate(TombstoneProps),
    CommandCreate(CommandProps),
    Delete,
    /// Known entity families we intentionally skip materializing in store state.
    Ignored(BTreeMap<String, Value>),
    /// Unknown/unsupported entity payload preserved for forward compatibility.
    Unknown(BTreeMap<String, Value>),
}

impl From<BTreeMap<String, Value>> for Properties {
    fn from(value: BTreeMap<String, Value>) -> Self {
        Self::Unknown(value)
    }
}

macro_rules! impl_properties_from {
    ($($source:ty => $variant:ident),+ $(,)?) => {
        $(
            impl From<$source> for Properties {
                fn from(value: $source) -> Self {
                    Self::$variant(value)
                }
            }
        )+
    };
}

impl_properties_from!(
    TaskProps => TaskCreate,
    TaskPatch => TaskUpdate,
    ChecklistItemProps => ChecklistCreate,
    ChecklistItemPatch => ChecklistUpdate,
    TagProps => TagCreate,
    TagPatch => TagUpdate,
    AreaProps => AreaCreate,
    AreaPatch => AreaUpdate,
    TombstoneProps => TombstoneCreate,
    CommandProps => CommandCreate,
);

impl WireObject {
    pub fn properties(&self) -> Result<Properties, serde_json::Error> {
        match &self.payload {
            Properties::Unknown(map) => WireObject::properties_from(
                self.operation_type,
                self.entity_type.as_ref(),
                map.clone(),
            ),
            other => Ok(other.clone()),
        }
    }

    pub fn properties_map(&self) -> BTreeMap<String, Value> {
        match &self.payload {
            Properties::TaskCreate(props) => to_map(props),
            Properties::TaskUpdate(patch) => to_map(patch),
            Properties::ChecklistCreate(props) => to_map(props),
            Properties::ChecklistUpdate(patch) => to_map(patch),
            Properties::TagCreate(props) => to_map(props),
            Properties::TagUpdate(patch) => to_map(patch),
            Properties::AreaCreate(props) => to_map(props),
            Properties::AreaUpdate(patch) => to_map(patch),
            Properties::TombstoneCreate(props) => to_map(props),
            Properties::CommandCreate(props) => to_map(props),
            Properties::Delete => BTreeMap::new(),
            Properties::Ignored(map) => map.clone(),
            Properties::Unknown(map) => map.clone(),
        }
    }

    pub fn create(entity_type: EntityType, payload: impl Into<Properties>) -> Self {
        let payload =
            Self::coerce_known_payload(OperationType::Create, &entity_type, payload.into());
        Self {
            operation_type: OperationType::Create,
            entity_type: Some(entity_type),
            payload,
        }
    }

    pub fn update(entity_type: EntityType, payload: impl Into<Properties>) -> Self {
        let payload =
            Self::coerce_known_payload(OperationType::Update, &entity_type, payload.into());
        Self {
            operation_type: OperationType::Update,
            entity_type: Some(entity_type),
            payload,
        }
    }

    pub fn delete(entity_type: EntityType) -> Self {
        Self {
            operation_type: OperationType::Delete,
            entity_type: Some(entity_type),
            payload: Properties::Delete,
        }
    }

    fn properties_from(
        operation_type: OperationType,
        entity_type: Option<&EntityType>,
        properties: BTreeMap<String, Value>,
    ) -> Result<Properties, serde_json::Error> {
        use EntityType::*;
        use Properties::*;
        let p = properties;

        fn parse<T: DeserializeOwned>(
            properties: BTreeMap<String, Value>,
        ) -> Result<T, serde_json::Error> {
            parse_props_from_map(properties)
        }

        let payload = match operation_type {
            OperationType::Delete => Delete,
            OperationType::Create => match entity_type {
                Some(Task3 | Task4 | Task6) => TaskCreate(parse(p)?),
                Some(ChecklistItem | ChecklistItem2 | ChecklistItem3) => ChecklistCreate(parse(p)?),
                Some(Tag3 | Tag4) => TagCreate(parse(p)?),
                Some(Area2 | Area3) => AreaCreate(parse(p)?),
                Some(Tombstone | Tombstone2) => TombstoneCreate(parse(p)?),
                Some(Command) => CommandCreate(parse(p)?),
                Some(Settings3 | Settings4 | Settings5) => Ignored(p),
                _ => Properties::Unknown(p),
            },
            OperationType::Update => match entity_type {
                Some(Task3 | Task4 | Task6) => TaskUpdate(parse(p)?),
                Some(ChecklistItem | ChecklistItem2 | ChecklistItem3) => ChecklistUpdate(parse(p)?),
                Some(Tag3 | Tag4) => TagUpdate(parse(p)?),
                Some(Area2 | Area3) => AreaUpdate(parse(p)?),
                Some(Settings3 | Settings4 | Settings5) => Ignored(p),
                _ => Properties::Unknown(p),
            },
            OperationType::Unknown(_) => Properties::Unknown(p),
        };

        Ok(payload)
    }

    fn coerce_known_payload(
        operation_type: OperationType,
        entity_type: &EntityType,
        payload: Properties,
    ) -> Properties {
        match payload {
            Properties::Unknown(map) => {
                match Self::properties_from(operation_type, Some(entity_type), map.clone()) {
                    Ok(parsed) => parsed,
                    Err(_) => Properties::Unknown(map),
                }
            }
            other => other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawWireObject {
    #[serde(rename = "t")]
    operation_type: OperationType,
    #[serde(rename = "e")]
    entity_type: Option<EntityType>,
    #[serde(rename = "p", default)]
    properties: BTreeMap<String, Value>,
}

impl Serialize for WireObject {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("WireObject", 3)?;
        state.serialize_field("t", &self.operation_type)?;
        state.serialize_field("e", &self.entity_type)?;
        state.serialize_field("p", &self.properties_map())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for WireObject {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawWireObject::deserialize(deserializer)?;
        let payload = WireObject::properties_from(
            raw.operation_type,
            raw.entity_type.as_ref(),
            raw.properties,
        )
        .map_err(serde::de::Error::custom)?;
        Ok(Self {
            operation_type: raw.operation_type,
            entity_type: raw.entity_type,
            payload,
        })
    }
}

fn parse_props_from_map<T: DeserializeOwned>(
    properties: BTreeMap<String, Value>,
) -> Result<T, serde_json::Error> {
    serde_json::from_value(Value::Object(
        properties
            .into_iter()
            .collect::<serde_json::Map<String, Value>>(),
    ))
}

fn to_map<T: Serialize>(value: &T) -> BTreeMap<String, Value> {
    match serde_json::to_value(value) {
        Ok(Value::Object(map)) => map.into_iter().collect(),
        _ => BTreeMap::new(),
    }
}

/// Operation type for wire field `t`.
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Display,
    EnumString,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(i32)]
#[serde(from = "i32", into = "i32")]
pub enum OperationType {
    /// Full snapshot/create (replace current object state for UUID).
    Create = 0,
    /// Partial update (merge `p` into existing properties).
    Update = 1,
    /// Deletion event.
    Delete = 2,

    /// Unknown operation value preserved for forward compatibility.
    #[num_enum(catch_all)]
    #[strum(disabled, to_string = "{0}")]
    Unknown(i32),
}

#[expect(
    clippy::derivable_impls,
    reason = "num_enum(catch_all) conflicts with #[default]"
)]
impl Default for OperationType {
    fn default() -> Self {
        Self::Create
    }
}

/// Entity type for wire field `e`.
///
/// Values are versioned by Things (for example `Task6`, `Area3`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
#[serde(from = "String", into = "String")]
pub enum EntityType {
    /// Task entity (legacy version).
    Task3,
    /// Task entity (legacy version).
    Task4,
    /// Task/project/heading entity (current observed version).
    Task6,

    /// Checklist item entity (legacy version).
    ChecklistItem,
    /// Checklist item entity (legacy version).
    ChecklistItem2,
    /// Checklist item entity (current observed version).
    ChecklistItem3,

    /// Tag entity (legacy version).
    Tag3,
    /// Tag entity (current observed version).
    Tag4,

    /// Area entity (legacy version).
    Area2,
    /// Area entity (current observed version).
    Area3,

    /// Settings entity.
    Settings3,
    Settings4,
    Settings5,

    /// Tombstone marker for deleted objects (legacy version).
    Tombstone,
    /// Tombstone marker for deleted objects.
    Tombstone2,

    /// One-shot command entity.
    Command,
    /// Unknown entity name preserved for forward compatibility.
    #[strum(default, to_string = "{0}")]
    Unknown(String),
}

impl From<String> for EntityType {
    fn from(value: String) -> Self {
        value.parse().unwrap_or(Self::Unknown(value))
    }
}

impl From<EntityType> for String {
    fn from(value: EntityType) -> Self {
        value.to_string()
    }
}
