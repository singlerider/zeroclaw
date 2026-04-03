/// Describes a single secret field discovered via `#[derive(Configurable)]`.
#[derive(Debug, Clone)]
pub struct SecretFieldInfo {
    /// Full dotted name (e.g. `channels.matrix.access-token`)
    pub name: &'static str,
    /// Category for grouping in `zeroclaw props list`
    pub category: &'static str,
    /// Whether this field currently has a non-empty value
    pub is_set: bool,
}

/// Runtime type classification for config property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropKind {
    String,
    Bool,
    Integer,
    Float,
    /// An enum or other serde-serializable type (parsed as TOML string).
    Enum,
}

impl PropKind {
    /// Display-friendly type name for CLI output.
    pub fn display_name(self, raw_type: &str) -> &str {
        match self {
            Self::String => "String",
            Self::Bool => "bool",
            Self::Integer | Self::Float | Self::Enum => raw_type,
        }
    }
}

/// Describes a single property field discovered via `#[derive(Configurable)]`.
#[derive(Clone)]
pub struct PropFieldInfo {
    /// Full dotted name (e.g. `channels.telegram.draft-update-interval-ms`)
    pub name: &'static str,
    /// Category for grouping in property listings
    pub category: &'static str,
    /// Current value formatted for display (secrets show `"****"`)
    pub display_value: String,
    /// Raw Rust type string for display (e.g. `"bool"`, `"u64"`, `"Option<StreamMode>"`)
    pub type_hint: &'static str,
    /// Runtime type classification
    pub kind: PropKind,
    /// Whether this field is marked `#[secret]`
    pub is_secret: bool,
    /// Returns valid variant names for enum fields (None for non-enum fields)
    pub enum_variants: Option<fn() -> Vec<String>>,
}

impl PropFieldInfo {
    pub fn is_enum(&self) -> bool {
        self.enum_variants.is_some()
    }
}

impl std::fmt::Debug for PropFieldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropFieldInfo")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("is_secret", &self.is_secret)
            .finish_non_exhaustive()
    }
}

/// The trait for describing a channel
pub trait ChannelConfig {
    /// human-readable name
    fn name() -> &'static str;
    /// short description
    fn desc() -> &'static str;
}

// Maybe there should be a `&self` as parameter for custom channel/info or what...

pub trait ConfigHandle {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
}
