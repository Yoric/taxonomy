//! Selectors for services and channels.
//!
//! The high-level API of Project Link always offers access by selectors, rather than by individual
//! services/channels. This allows operations such as sending a temperature to all heaters in the
//! living room (that's a selector), rather than needing to access every single heater one by one.

use api::services::*;
use io::parse::*;
use misc::util::ptr_eq;

use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashSet;

use serde::de::Deserialize;

fn merge<T>(mut a: HashSet<T>, b: &[T]) -> HashSet<T> where T: Hash + Eq + Clone {
    a.extend(b.iter().cloned());
    a
}

/// A selector for one or more services.
///
///
/// # Example
///
/// ```
/// use foxbox_taxonomy::selector::*;
/// use foxbox_taxonomy::services::*;
///
/// let selector = ServiceSelector::new()
///   .with_tags(vec![Id::<TagId>::new("entrance")])
///   .with_getters(vec![GetterSelector::new() /* can be more restrictive */]);
/// ```
///
/// # JSON
///
/// A selector is an object with the following fields:
///
/// - (optional) string `id`: accept only a service with a given id;
/// - (optional) array of string `tags`:  accept only services with all the tags in the array;
/// - (optional) array of objects `getters` (see `GetterSelector`): accept only services with
///    channels matching all the selectors in this array;
/// - (optional) array of objects `setters` (see `SetterSelector`): accept only services with
///    channels matching all the selectors in this array;
///
/// While each field is optional, at least one field must be provided.
///
/// ```
/// use foxbox_taxonomy::selector::*;
///
/// // A selector with all fields defined.
/// let json_selector = "{
///   \"id\": \"setter 1\",
///   \"tags\": [\"tag 1\", \"tag 2\"],
///   \"getters\": [{
///     \"kind\": \"Ready\"
///   }],
///   \"setters\": [{
///     \"tags\": [\"tag 3\"]
///   }]
/// }";
///
/// ServiceSelector::from_str(json_selector).unwrap();
///
/// // The following will be rejected because no field is provided:
/// let json_empty = "{}";
/// match ServiceSelector::from_str(json_empty) {
///   Err(ParseError::EmptyObject {..}) => { /* as expected */ },
///   other => panic!("Unexpected result {:?}", other)
/// }
/// ```
#[derive(Clone, Debug, Deserialize, Default)]
pub struct ServiceSelector {
    /// If `Exactly(id)`, return only the service with the corresponding id.
    pub id: Exactly<Id<ServiceId>>,

    ///  Restrict results to services that have all the tags in `tags`.
    pub tags: HashSet<Id<TagId>>,

    /// Restrict results to services that have all the getters in `getters`.
    pub features: Vec<SimpleFeatureSelector>,

    /// Make sure that we can't instantiate from another crate.
    private: (),
}

impl PartialEq for ServiceSelector {
    fn eq(&self, other: &Self) -> bool {
        // We always expect two ServiceSelectors to be distinct
        ptr_eq(self, other)
    }
}

impl Parser<ServiceSelector> for ServiceSelector {
    fn description() -> String {
        "ServiceSelector".to_owned()
    }
    fn parse(path: Path, source: &JSON, support: &DeserializeSupport) -> Result<Self, ParseError> {
        let id = try!(match path.push("id", |path| Exactly::take_opt(path, source, "id", support)) {
            None => Ok(Exactly::Always),
            Some(result) => result
        });
        let tags : HashSet<_> = match path.push("tags", |path| Id::take_vec_opt(path, source, "tags", support)) {
            None => HashSet::new(),
            Some(Ok(mut vec)) => vec.drain(..).collect(),
            Some(Err(err)) => return Err(err),
        };
        let features = match path.push("features", |path| SimpleFeatureSelector::take_vec_opt(path, source, "features", support)) {
            None => vec![],
            Some(Ok(vec)) => vec,
            Some(Err(err)) => return Err(err)
        };

        Ok(ServiceSelector {
            id: id,
            tags: tags,
            features: features,
            private: ()
        })
    }
}

impl ServiceSelector {
    /// Create a new selector that accepts all services.
    pub fn new() -> Self {
        Self::default()
    }

    /// Selector for a service with a specific id.
    pub fn with_id(self, id: Id<ServiceId>) -> Self {
        ServiceSelector {
            id: self.id.and(Exactly::Exactly(id)),
            .. self
        }
    }

    ///  Restrict results to services that have all the tags in `tags`.
    pub fn with_tags(self, tags: &[Id<TagId>]) -> Self {
        ServiceSelector {
            tags: merge(self.tags, tags),
            .. self
        }
    }

    /// Restrict results to services that have all the getters in `getters`.
    pub fn with_features(mut self, features: &[SimpleFeatureSelector]) -> Self {
        ServiceSelector {
            features: {self.features.extend_from_slice(features); self.features},
            .. self
        }
    }

    /// Restrict results to services that are accepted by two selector.
    pub fn and(mut self, mut other: ServiceSelector) -> Self {
        ServiceSelector {
            id: self.id.and(other.id),
            tags: self.tags.union(&other.tags).cloned().collect(),
            features: {self.features.append(&mut other.features); self.features},
            private: (),
        }
    }
}



/// A selector for one or more getter channels.
///
///
/// # Example
///
/// ```
/// use foxbox_taxonomy::selector::*;
/// use foxbox_taxonomy::services::*;
///
/// let selector = GetterSelector::new()
///   .with_parent(Id::new("foxbox"))
///   .with_kind(ChannelKind::CurrentTimeOfDay);
/// ```
///
/// # JSON
///
/// A selector is an object with the following fields:
///
/// - (optional) string `id`: accept only a channel with a given id;
/// - (optional) string `service`: accept only channels of a service with a given id;
/// - (optional) array of string `tags`:  accept only channels with all the tags in the array;
/// - (optional) array of string `service_tags`:  accept only channels of a service with all the
///        tags in the array;
/// - (optional) string|object `kind` (see `ChannelKind`): accept only channels of a given kind.
///
/// While each field is optional, at least one field must be provided.
///
/// ```
/// use foxbox_taxonomy::selector::*;
///
/// // A selector with all fields defined.
/// let json_selector = "{                         \
///   \"id\": \"setter 1\",                        \
///   \"service\": \"service 1\",                  \
///   \"tags\": [\"tag 1\", \"tag 2\"],            \
///   \"service_tags\": [\"tag 3\", \"tag 4\"],    \
///   \"kind\": \"Ready\"                          \
/// }";
///
/// FeatureSelector::from_str(json_selector).unwrap();
///
/// // The following will be rejected because no field is provided:
/// let json_empty = "{}";
/// match GetterSelector::from_str(json_empty) {
///   Err(ParseError::EmptyObject {..}) => { /* as expected */ },
///   other => panic!("Unexpected result {:?}", other)
/// }
/// ```
#[derive(Clone, Debug, Deserialize, Default)]
pub struct BaseFeatureSelector<T> where T: Clone + Debug + Deserialize + Default {
    /// If `Exactly(id)`, return only the channel with the corresponding id.
    pub id: Exactly<Id<FeatureId>>,

    /// Restrict results to features that appear in `services`.
    pub services: T,

    ///  Restrict results to channels that have all the tags in `tags`.
    pub tags: HashSet<Id<TagId>>,

    /// If `Exatly(k)`, restrict results to channels that produce values
    /// of kind `k`.
    pub implements: Exactly<Id<ImplementId>>,

    private: (),
}

pub type SimpleFeatureSelector = BaseFeatureSelector<()>;
pub type FeatureSelector = BaseFeatureSelector<Vec<ServiceSelector>>;

impl Parser<FeatureSelector> for FeatureSelector {
    fn description() -> String {
        "FeatureSelector".to_owned()
    }
    fn parse(path: Path, source: &JSON, support: &DeserializeSupport) -> Result<Self, ParseError> {
        let services = try!(match path.push("services", |path| ServiceSelector::take_vec_opt(path, source, "services", support)) {
            None => Ok(vec![]),
            Some(result) => {
                result
            }
        });
        let base = try!(SimpleFeatureSelector::parse(path, source, support));
        Ok(BaseFeatureSelector {
            services: services,
            id: base.id,
            tags: base.tags,
            implements: base.implements,
            private: ()
        })
    }
}

impl Parser<SimpleFeatureSelector> for SimpleFeatureSelector {
    fn description() -> String {
        "SimpleFeatureSelector".to_owned()
    }
    fn parse(path: Path, source: &JSON, support: &DeserializeSupport) -> Result<Self, ParseError> {
        let id = try!(match path.push("id", |path| Exactly::take_opt(path, source, "id", support)) {
            None => Ok(Exactly::Always),
            Some(result) => {
                result
            }
        });
        let tags : HashSet<_> = match path.push("tags", |path| Id::take_vec_opt(path, source, "tags", support)) {
            None => HashSet::new(),
            Some(Ok(mut vec)) => {
                vec.drain(..).collect()
            }
            Some(Err(err)) => return Err(err),
        };
        let implements = try!(match path.push("implements", |path| Exactly::take_opt(path, source, "implements", support)) {
            None => Ok(Exactly::Always),
            Some(result) => {
                result
            }
        });
        Ok(BaseFeatureSelector {
            id: id,
            services: (),
            tags: tags,
            implements: implements,
            private: ()
        })
    }
}

impl<T> BaseFeatureSelector<T> where T: Clone + Debug + Deserialize + Default {
    /// Create a new selector that accepts all getter channels.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restrict to a channel with a specific id.
    pub fn with_id(self, id: Id<FeatureId>) -> Self {
        BaseFeatureSelector {
            id: self.id.and(Exactly::Exactly(id)),
            .. self
        }
    }

    /// Restrict to a channel with a specific kind.
    pub fn with_implements(self, id: Id<ImplementId>) -> Self {
        BaseFeatureSelector {
            implements: self.implements.and(Exactly::Exactly(id)),
            .. self
        }
    }

    ///  Restrict to channels that have all the tags in `tags`.
    pub fn with_tags(self, tags: &[Id<TagId>]) -> Self {
        BaseFeatureSelector {
            tags: merge(self.tags, tags),
            .. self
        }
    }
}

impl BaseFeatureSelector<Vec<ServiceSelector>> {
    /// Restrict to a channel with a specific parent.
    pub fn with_service(self, services: &[ServiceSelector]) -> Self {
        let mut self_services = self.services;
        BaseFeatureSelector {
            services: {self_services.extend_from_slice(services); self_services},
            .. self
        }
    }
}



/*
/// A selector for one or more setter channels.
///
/// # JSON
///
/// A selector is an object with the following fields:
///
/// - (optional) string `id`: accept only a channel with a given id;
/// - (optional) string `service`: accept only channels of a service with a given id;
/// - (optional) array of string `tags`:  accept only channels with all the tags in the array;
/// - (optional) array of string `service_tags`:  accept only channels of a service with all the
///        tags in the array;
/// - (optional) string|object `kind` (see ChannelKind): accept only channels of a given kind.
///
/// While each field is optional, at least one field must be provided.
///
/// ```
/// use foxbox_taxonomy::selector::*;
///
/// // A selector with all fields defined.
/// let json_selector = "{                         \
///   \"id\": \"setter 1\",                        \
///   \"service\": \"service 1\",                  \
///   \"tags\": [\"tag 1\", \"tag 2\"],            \
///   \"service_tags\": [\"tag 3\", \"tag 4\"],    \
///   \"kind\": \"Ready\"                          \
/// }";
///
/// SetterSelector::from_str(json_selector).unwrap();
///
/// // The following will be rejected because no field is provided:
/// let json_empty = "{}";
/// match SetterSelector::from_str(json_empty) {
///   Err(ParseError::EmptyObject {..}) => { /* as expected */ },
///   other => panic!("Unexpected result {:?}", other)
/// }
/// ```
#[derive(Clone, Debug, Deserialize, Default)]
pub struct SetterSelector {
    /// If `Exactly(id)`, return only the channel with the corresponding id.
    pub id: Exactly<Id<Setter>>,

    /// If `Exactly(id)`, return only channels that are immediate children
    /// of service `id`.
    pub parent: Exactly<Id<ServiceId>>,

    ///  Restrict results to channels that have all the tags in `tags`.
    pub tags: HashSet<Id<TagId>>,

    ///  Restrict results to channels offered by a service that has all the tags in `tags`.
    pub service_tags: HashSet<Id<TagId>>,

    /// If `Exactly(k)`, restrict results to channels that accept values
    /// of kind `k`.
    pub kind: Exactly<ChannelKind>,

    /// Make sure that we can't instantiate from another crate.
    private: (),
}

impl Parser<SetterSelector> for SetterSelector {
    fn description() -> String {
        "SetterSelector".to_owned()
    }
    fn parse(path: Path, source: &JSON) -> Result<Self, ParseError> {
        let mut is_empty = true;
        let id = try!(match path.push("id", |path| Exactly::take_opt(path, source, "id")) {
            None => Ok(Exactly::Always),
            Some(result) => {
                is_empty = false;
                result
            }
        });
        let service_id = try!(match path.push("service", |path| Exactly::take_opt(path, source, "service")) {
            None => Ok(Exactly::Always),
            Some(result) => {
                is_empty = false;
                result
            }
        });
        let tags : HashSet<_> = match path.push("tags", |path| Id::take_vec_opt(path, source, "tags")) {
            None => HashSet::new(),
            Some(Ok(mut vec)) => {
                is_empty = false;
                vec.drain(..).collect()
            }
            Some(Err(err)) => return Err(err),
        };
        let service_tags : HashSet<_> = match path.push("service_tags", |path| Id::take_vec_opt(path, source, "service_tags")) {
            None => HashSet::new(),
            Some(Ok(mut vec)) => {
                is_empty = false;
                vec.drain(..).collect()
            }
            Some(Err(err)) => return Err(err),
        };
        let kind = try!(match path.push("kind", |path| Exactly::take_opt(path, source, "kind")) {
            None => Ok(Exactly::Always),
            Some(result) => {
                is_empty = false;
                result
            }
        });
        if is_empty {
            Err(ParseError::empty_object(&path))
        } else {
            Ok(SetterSelector {
                id: id,
                parent: service_id,
                tags: tags,
                service_tags: service_tags,
                kind: kind,
                private: ()
            })
        }
    }
}

impl SetterSelector {
    /// Create a new selector that accepts all getter channels.
    pub fn new() -> Self {
        SetterSelector::default()
    }

    /// Selector to a channel with a specific id.
    pub fn with_id(self, id: Id<Setter>) -> Self {
        SetterSelector {
            id: self.id.and(Exactly::Exactly(id)),
            .. self
        }
    }

    /// Selector to channels with a specific parent.
    pub fn with_parent(self, id: Id<ServiceId>) -> Self {
        SetterSelector {
            parent: self.parent.and(Exactly::Exactly(id)),
            .. self
        }
    }

    /// Selector to channels with a specific kind.
    pub fn with_kind(self, kind: ChannelKind) -> Self {
        SetterSelector {
            kind: self.kind.and(Exactly::Exactly(kind)),
            .. self
        }
    }

    ///  Restrict to channels that have all the tags in `tags`.
    pub fn with_tags(self, tags: Vec<Id<TagId>>) -> Self {
        SetterSelector {
            tags: merge(self.tags, tags),
            .. self
        }
    }

    ///  Restrict to channels offered by a service that has all the tags in `tags`.
    pub fn with_service_tags(self, tags: Vec<Id<TagId>>) -> Self {
        SetterSelector {
            service_tags: merge(self.service_tags, tags),
            .. self
        }
    }

    /// Restrict results to channels that are accepted by two selector.
    pub fn and(self, other: Self) -> Self {
        SetterSelector {
            id: self.id.and(other.id),
            parent: self.parent.and(other.parent),
            tags: self.tags.union(&other.tags).cloned().collect(),
            service_tags: self.service_tags.union(&other.service_tags).cloned().collect(),
            kind: self.kind.and(other.kind),
            private: (),
        }
    }

    /// Determine if a channel is matched by this selector.
    pub fn matches(&self, service_tags: &HashSet<Id<TagId>>, channel: &Channel<Setter>) -> bool {
        if !self.id.matches(&channel.id) {
            return false;
        }
        if !self.parent.matches(&channel.service) {
            return false;
        }
        if !self.kind.matches(&channel.mechanism.kind) {
            return false;
        }
        if !has_selected_tags(&self.tags, &channel.tags) {
            return false;
        }
        if !has_selected_tags(&self.service_tags, service_tags) {
            return false;
        }
        true
    }
}

/// An acceptable interval of time.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Period {
    #[serde(default)]
    pub min: Option<Duration>,
    #[serde(default)]
    pub max: Option<Duration>,
}
impl Period {
    pub fn and(self, other: Self) -> Self {
        let min = match (self.min, other.min) {
            (None, x) |
            (x, None) => x,
            (Some(min1), Some(min2)) => Some(cmp::max(min1, min2))
        };
        let max = match (self.max, other.max) {
            (None, x) |
            (x, None) => x,
            (Some(max1), Some(max2)) => Some(cmp::min(max1, max2))
        };
        Period {
            min: min,
            max: max
        }
    }

    pub fn and_option(a: Option<Self>, b: Option<Self>) -> Option<Self> {
        match (a, b) {
            (None, x) |
            (x, None) => x,
            (Some(a), Some(b)) => Some(a.and(b))
        }
    }

    pub fn matches(&self, duration: &Duration) -> bool {
        if let Some(ref min) = self.min {
            if min > duration {
                return false;
            }
        }
        if let Some(ref max) = self.max {
            if max < duration {
                return false;
            }
        }
        true
    }

    pub fn matches_option(period: &Option<Self>, duration: &Option<Duration>) -> bool {
        match (period, duration) {
            (&Some(ref period), &Some(ref duration))
                if !period.matches(duration) => false,
            (&Some(_), &None) => false,
            _ => true,
        }
    }
}

*/
