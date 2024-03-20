/*!
A library for parsing CSL styles.

Citationberg deserializes CSL styles from XML into Rust structs. It supports
[CSL 1.0.2](https://docs.citationstyles.org/en/stable/specification.html).

This crate is not a CSL processor, so you are free to choose whatever data
model and data types you need for your bibliographic needs. If you need to
render citations, you can use
[Hayagriva](https://github.com/typst/hayagriva) which uses this crate under
the hood.

Parse your style like this:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::fs;
use citationberg::Style;

let string = fs::read_to_string("tests/independent/ieee.csl")?;
let style = citationberg::Style::from_xml(&string)?;

let Style::Independent(independent) = style else {
    panic!("IEEE is an independent style");
};

assert_eq!(independent.info.title.value, "IEEE");
# Ok(())
# }
```

You can also parse a [`DependentStyle`] or a [`IndependentStyle`] directly.
*/

#![deny(missing_docs)]
#![deny(unsafe_code)]

#[cfg(feature = "json")]
pub mod json;
pub mod taxonomy;

mod util;

use std::fmt::{self, Debug};
use std::num::{NonZeroI16, NonZeroUsize};

use quick_xml::de::{Deserializer, SliceReader};
use serde::{Deserialize, Serialize};
use taxonomy::{
    DateVariable, Kind, Locator, NameVariable, NumberVariable, OtherTerm, Term, Variable,
};

use self::util::*;

/// Result type for functions that serialize and deserialize XML.
pub type XmlResult<T> = Result<T, XmlError>;

/// Error type for functions that serialize and deserialize XML.
pub type XmlError = quick_xml::de::DeError;

const EVENT_BUFFER_SIZE: Option<NonZeroUsize> = NonZeroUsize::new(4096);

/// Allow every struct with formatting properties to convert to a `Formatting`.
pub trait ToFormatting {
    /// Obtain a `Formatting`.
    fn to_formatting(&self) -> Formatting;
}

macro_rules! to_formatting {
    ($name:ty, self) => {
        impl ToFormatting for $name {
            fn to_formatting(&self) -> Formatting {
                Formatting {
                    font_style: self.font_style,
                    font_variant: self.font_variant,
                    font_weight: self.font_weight,
                    text_decoration: self.text_decoration,
                    vertical_align: self.vertical_align,
                }
            }
        }
    };
    ($name:ty) => {
        impl ToFormatting for $name {
            fn to_formatting(&self) -> Formatting {
                self.formatting.clone()
            }
        }
    };
}

/// Allow every struct with affix properties to convert to a `Affixes`.
pub trait ToAffixes {
    /// Obtain the `Affixes`.
    fn to_affixes(&self) -> Affixes;
}

macro_rules! to_affixes {
    ($name:ty, self) => {
        impl ToAffixes for $name {
            fn to_affixes(&self) -> Affixes {
                Affixes {
                    prefix: self.prefix.clone(),
                    suffix: self.suffix.clone(),
                }
            }
        }
    };
    ($name:ty) => {
        impl ToAffixes for $name {
            fn to_affixes(&self) -> Affixes {
                self.affixes.clone()
            }
        }
    };
}

/// A CSL style.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
struct RawStyle {
    /// The style's metadata.
    pub info: StyleInfo,
    /// The locale used if the user didn't specify one.
    /// Overrides the default locale of the parent style.
    #[serde(rename = "@default-locale")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_locale: Option<LocaleCode>,
    /// The CSL version the style is compatible with.
    #[serde(rename = "@version")]
    pub version: String,
    /// How notes or in-text citations are displayed. Must be present in
    /// independent styles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<Citation>,
    /// How bibliographies are displayed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibliography: Option<Bibliography>,
    /// The style's settings. Must be present in dependent styles.
    #[serde(flatten)]
    pub independent_settings: Option<IndependentStyleSettings>,
    /// Reusable formatting rules.
    #[serde(rename = "macro", default)]
    pub macros: Vec<CslMacro>,
    /// Override localized strings.
    #[serde(default)]
    pub locale: Vec<Locale>,
}

impl RawStyle {
    /// Retrieve the link to the parent style for dependent styles.
    pub fn parent_link(&self) -> Option<&InfoLink> {
        self.info
            .link
            .iter()
            .find(|link| link.rel == InfoLinkRel::IndependentParent)
    }
}

impl From<IndependentStyle> for RawStyle {
    fn from(value: IndependentStyle) -> Self {
        Self {
            info: value.info,
            default_locale: value.default_locale,
            version: value.version,
            citation: Some(value.citation),
            bibliography: value.bibliography,
            independent_settings: Some(value.settings),
            macros: value.macros,
            locale: value.locale,
        }
    }
}

impl From<DependentStyle> for RawStyle {
    fn from(value: DependentStyle) -> Self {
        Self {
            info: value.info,
            default_locale: value.default_locale,
            version: value.version,
            citation: None,
            bibliography: None,
            independent_settings: None,
            macros: Vec::new(),
            locale: Vec::new(),
        }
    }
}

impl From<Style> for RawStyle {
    fn from(value: Style) -> Self {
        match value {
            Style::Independent(i) => i.into(),
            Style::Dependent(d) => d.into(),
        }
    }
}

/// An independent CSL style.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IndependentStyle {
    /// The style's metadata.
    pub info: StyleInfo,
    /// The locale used if the user didn't specify one.
    pub default_locale: Option<LocaleCode>,
    /// The CSL version the style is compatible with.
    pub version: String,
    /// How notes or in-text citations are displayed.
    pub citation: Citation,
    /// How bibliographies are displayed.
    pub bibliography: Option<Bibliography>,
    /// The style's settings. Must be present in dependent styles.
    pub settings: IndependentStyleSettings,
    /// Reusable formatting rules.
    pub macros: Vec<CslMacro>,
    /// Override localized strings.
    pub locale: Vec<Locale>,
}

impl IndependentStyle {
    /// Create a style from an XML string.
    pub fn from_xml(xml: &str) -> XmlResult<Self> {
        let de = &mut deserializer(xml);
        IndependentStyle::deserialize(de)
    }

    /// Remove all non-required data that does not influence the style's
    /// formatting.
    pub fn purge(&mut self, level: PurgeLevel) {
        self.info.purge(level);
    }
}

/// How much metadata to remove from the style.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PurgeLevel {
    /// Retain some basic metadata.
    Basic,
    /// Purge all metadata.
    Full,
}

impl<'de> Deserialize<'de> for IndependentStyle {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let raw_style = RawStyle::deserialize(deserializer)?;
        let style: Style = raw_style.try_into().map_err(serde::de::Error::custom)?;

        match style {
            Style::Independent(i) => Ok(i),
            Style::Dependent(_) => Err(serde::de::Error::custom(
                "expected an independent style but got a dependent style",
            )),
        }
    }
}

/// A dependent CSL style.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DependentStyle {
    /// The style's metadata.
    pub info: StyleInfo,
    /// The locale used if the user didn't specify one.
    /// Overrides the default locale of the parent style.
    pub default_locale: Option<LocaleCode>,
    /// The CSL version the style is compatible with.
    pub version: String,
    /// The link to the parent style.
    pub parent_link: InfoLink,
}

impl DependentStyle {
    /// Create a style from an XML string.
    pub fn from_xml(xml: &str) -> XmlResult<Self> {
        let de = &mut deserializer(xml);
        DependentStyle::deserialize(de)
    }

    /// Remove all non-required data that does not influence the style's
    /// formatting.
    pub fn purge(&mut self, level: PurgeLevel) {
        self.info.purge(level);
    }
}

impl<'de> Deserialize<'de> for DependentStyle {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let raw_style = RawStyle::deserialize(deserializer)?;
        let style: Style = raw_style.try_into().map_err(serde::de::Error::custom)?;

        match style {
            Style::Dependent(d) => Ok(d),
            Style::Independent(_) => Err(serde::de::Error::custom(
                "expected a dependent style but got an independent style",
            )),
        }
    }
}

/// A CSL style.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[allow(clippy::large_enum_variant)]
pub enum Style {
    /// An independent style.
    Independent(IndependentStyle),
    /// A dependent style.
    Dependent(DependentStyle),
}

impl Style {
    /// Create a style from an XML string.
    pub fn from_xml(xml: &str) -> XmlResult<Self> {
        let de = &mut deserializer(xml);
        Style::deserialize(de)
    }

    /// Write the style to an XML string.
    pub fn to_xml(&self) -> XmlResult<String> {
        let mut buf = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buf, Some("style"))?;
        self.serialize(ser)?;
        Ok(buf)
    }

    /// Remove all non-required data that does not influence the style's
    /// formatting.
    pub fn purge(&mut self, level: PurgeLevel) {
        match self {
            Self::Independent(i) => i.purge(level),
            Self::Dependent(d) => d.purge(level),
        }
    }

    /// Get the style's metadata.
    pub fn info(&self) -> &StyleInfo {
        match self {
            Self::Independent(i) => &i.info,
            Self::Dependent(d) => &d.info,
        }
    }
}

impl<'de> Deserialize<'de> for Style {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let raw_style = RawStyle::deserialize(deserializer)?;
        raw_style.try_into().map_err(serde::de::Error::custom)
    }
}

impl Serialize for Style {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        RawStyle::from(self.clone()).serialize(serializer)
    }
}

impl TryFrom<RawStyle> for Style {
    type Error = StyleValidationError;

    fn try_from(value: RawStyle) -> Result<Self, Self::Error> {
        let has_bibliography = value.bibliography.is_some();
        if let Some(citation) = value.citation {
            if let Some(settings) = value.independent_settings {
                Ok(Self::Independent(IndependentStyle {
                    info: value.info,
                    default_locale: value.default_locale,
                    version: value.version,
                    citation,
                    bibliography: value.bibliography,
                    settings,
                    macros: value.macros,
                    locale: value.locale,
                }))
            } else {
                Err(StyleValidationError::MissingClassAttr)
            }
        } else if has_bibliography {
            Err(StyleValidationError::MissingCitation)
        } else if let Some(parent_link) = value.parent_link().cloned() {
            Ok(Self::Dependent(DependentStyle {
                info: value.info,
                default_locale: value.default_locale,
                version: value.version,
                parent_link,
            }))
        } else {
            Err(StyleValidationError::MissingParent)
        }
    }
}

/// An error that occurred while validating a style.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StyleValidationError {
    /// The CSL style did have a `cs:bibliography` child but not a
    /// `cs:citation`.
    MissingCitation,
    /// A dependent style was missing the `independent-parent` link.
    MissingParent,
    /// An independent style was missing the `class` attribute on `cs:style`
    MissingClassAttr,
}

impl fmt::Display for StyleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingCitation => "root element is missing `cs:citation` child despite having a `cs:bibliography`",
            Self::MissingParent => "`cs:link` tag with `independent-parent` as a `rel` attribute is missing but no `cs:citation` was defined",
            Self::MissingClassAttr => "`cs:style` tag is missing the `class` attribute",
        })
    }
}

fn deserializer(xml: &str) -> Deserializer<SliceReader<'_>> {
    let mut style_deserializer = Deserializer::from_str(xml);
    style_deserializer.event_buffer_size(EVENT_BUFFER_SIZE);
    style_deserializer
}

/// A style with its own formatting rules.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct IndependentStyleSettings {
    /// How the citations are displayed.
    #[serde(rename = "@class")]
    pub class: StyleClass,
    /// Whether to use a hyphen when initializing a name.
    ///
    /// Defaults to `true`.
    #[serde(
        rename = "@initialize-with-hyphen",
        default = "IndependentStyleSettings::default_initialize_with_hyphen",
        deserialize_with = "deserialize_bool"
    )]
    pub initialize_with_hyphen: bool,
    /// Specifies how to reformat page ranges.
    #[serde(rename = "@page-range-format")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_range_format: Option<PageRangeFormat>,
    /// How to treat the non-dropping name particle when printing names.
    #[serde(rename = "@demote-non-dropping-particle", default)]
    pub demote_non_dropping_particle: DemoteNonDroppingParticle,
    /// Options for the names within. Only defined for dependent styles.
    #[serde(flatten)]
    pub options: InheritableNameOptions,
}

impl IndependentStyleSettings {
    /// Return the default value for `initialize_with_hyphen`.
    pub const fn default_initialize_with_hyphen() -> bool {
        true
    }
}

/// An RFC 1766 language code.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocaleCode(pub String);

impl<'a> LocaleCode {
    /// Get the US English locale.
    pub fn en_us() -> Self {
        Self("en-US".to_string())
    }

    /// Get the base language code.
    pub fn parse_base(&self) -> Option<BaseLanguage> {
        let mut parts = self.0.split('-').take(2);
        let first = parts.next()?;

        match first {
            "i" | "I" => {
                let second = parts.next()?;
                if second.is_empty() {
                    return None;
                }

                Some(BaseLanguage::Iana(second.to_string()))
            }
            "x" | "X" => {
                let second = parts.next()?;
                if second.len() > 8 || second.is_empty() {
                    return None;
                }

                let mut code = [0; 8];
                code[..second.len()].copy_from_slice(second.as_bytes());
                Some(BaseLanguage::Unregistered(code))
            }
            _ if first.len() == 2 => {
                let mut code = [0; 2];
                code.copy_from_slice(first.as_bytes());
                Some(BaseLanguage::Iso639_1(code))
            }
            _ => None,
        }
    }

    /// Get the language's extensions.
    pub fn extensions(&'a self) -> impl Iterator<Item = &'a str> + 'a {
        self.0
            .split('-')
            .enumerate()
            .filter_map(|(i, e)| {
                if i == 0 && ["x", "X", "i", "I"].contains(&e) {
                    None
                } else {
                    Some(e)
                }
            })
            .skip(1)
    }

    /// Check whether the language is English.
    pub fn is_english(&self) -> bool {
        let en = "en";
        let hyphen = "-";
        self.0.starts_with(en)
            && (self.0.len() == 2
                || self.0.get(en.len()..en.len() + hyphen.len()) == Some(hyphen))
    }

    /// Get the fallback locale for a locale.
    pub fn fallback(&self) -> Option<LocaleCode> {
        match self.parse_base()? {
            BaseLanguage::Iso639_1(code) => match &code {
                b"af" => Some("af-ZA"),
                b"bg" => Some("bg-BG"),
                b"ca" => Some("ca-AD"),
                b"cs" => Some("cs-CZ"),
                b"da" => Some("da-DK"),
                b"de" => Some("de-DE"),
                b"el" => Some("el-GR"),
                b"en" => Some("en-US"),
                b"es" => Some("es-ES"),
                b"et" => Some("et-EE"),
                b"fa" => Some("fa-IR"),
                b"fi" => Some("fi-FI"),
                b"fr" => Some("fr-FR"),
                b"he" => Some("he-IL"),
                b"hr" => Some("hr-HR"),
                b"hu" => Some("hu-HU"),
                b"is" => Some("is-IS"),
                b"it" => Some("it-IT"),
                b"ja" => Some("ja-JP"),
                b"km" => Some("km-KH"),
                b"ko" => Some("ko-KR"),
                b"lt" => Some("lt-LT"),
                b"lv" => Some("lv-LV"),
                b"mn" => Some("mn-MN"),
                b"nb" => Some("nb-NO"),
                b"nl" => Some("nl-NL"),
                b"nn" => Some("nn-NO"),
                b"pl" => Some("pl-PL"),
                b"pt" => Some("pt-PT"),
                b"ro" => Some("ro-RO"),
                b"ru" => Some("ru-RU"),
                b"sk" => Some("sk-SK"),
                b"sl" => Some("sl-SI"),
                b"sr" => Some("sr-RS"),
                b"sv" => Some("sv-SE"),
                b"th" => Some("th-TH"),
                b"tr" => Some("tr-TR"),
                b"uk" => Some("uk-UA"),
                b"vi" => Some("vi-VN"),
                b"zh" => Some("zh-CN"),
                _ => None,
            }
            .map(ToString::to_string)
            .map(LocaleCode)
            .filter(|f| f != self),
            _ => None,
        }
    }
}

impl fmt::Display for LocaleCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// The base language in a [`LocaleCode`].
pub enum BaseLanguage {
    /// A language code.
    Iso639_1([u8; 2]),
    /// An IANA language code.
    Iana(String),
    /// An unregistered / experimental language code.
    Unregistered([u8; 8]),
}

impl BaseLanguage {
    /// Get the language code.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Iso639_1(code) => std::str::from_utf8(code).unwrap(),
            Self::Iana(code) => code,
            Self::Unregistered(code) => std::str::from_utf8(code).unwrap(),
        }
    }
}

/// How the citations are displayed.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StyleClass {
    /// Citations are inlined in the text.
    InText,
    /// Citations are displayed in foot- or endnotes.
    Note,
}

/// How to reformat page ranges.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PageRangeFormat {
    /// “321–28”
    /// Aliases: `chicago` until CSL 1.1
    // Rename needed because the number is not used as word boundary by heck.
    #[serde(alias = "chicago")]
    #[serde(rename = "chicago-15")]
    Chicago15,
    /// “321–28”
    #[serde(rename = "chicago-16")]
    Chicago16,
    /// “321–328”
    #[default]
    Expanded,
    /// “321–8”
    Minimal,
    /// “321–28”
    MinimalTwo,
}

impl PageRangeFormat {
    /// Use a page range format to format a range of pages.
    pub fn format(
        self,
        range: std::ops::Range<i32>,
        buf: &mut impl fmt::Write,
        separator: Option<&str>,
    ) -> Result<(), fmt::Error> {
        let separator = separator.unwrap_or("–");

        write!(buf, "{}{}", range.start, separator)?;
        let end = if range.end >= range.start {
            range.end
        } else {
            expand(range.start, range.end)
        };

        match self {
            _ if range.start < 0 || range.end < 0 => write!(buf, "{}", end),
            PageRangeFormat::Expanded => write!(buf, "{}", end),

            PageRangeFormat::Chicago15 | PageRangeFormat::Chicago16
                if range.start < 100 || range.start % 100 == 0 =>
            {
                write!(buf, "{}", end)
            }
            PageRangeFormat::Minimal => {
                write!(buf, "{}", changed_part(range.start, end, 0))
            }
            PageRangeFormat::MinimalTwo if end < 10 => {
                write!(buf, "{}", changed_part(range.start, end, 1))
            }
            PageRangeFormat::Chicago15
                if range.start > 100 && (1..10).contains(&(range.start % 100)) =>
            {
                write!(buf, "{}", changed_part(range.start, end, 0))
            }
            PageRangeFormat::Chicago15
                if closest_smaller_power_of_10(range.start) == 1000 =>
            {
                let changed = changed_part(range.start, end, 1);
                if closest_smaller_power_of_10(changed) == 100 {
                    write!(buf, "{end}")
                } else {
                    write!(buf, "{changed}")
                }
            }
            PageRangeFormat::Chicago15
            | PageRangeFormat::Chicago16
            | PageRangeFormat::MinimalTwo => {
                write!(buf, "{}", changed_part(range.start, end, 1))
            }
        }
    }
}

// Taken from https://github.com/citation-style-language/citeproc-rs/blob/master/crates/proc/src/page_range.rs
fn closest_smaller_power_of_10(num: i32) -> i32 {
    let answer = 10_f64.powf((num as f64).log10().floor()) as i32;

    // these properties need to hold. I think they do, but the float conversions
    // might mess things up...
    debug_assert!(answer <= num);
    debug_assert!(answer > num / 10);
    answer
}

// Taken from https://github.com/citation-style-language/citeproc-rs/blob/master/crates/proc/src/page_range.rs
fn expand(a: i32, b: i32) -> i32 {
    let mask = closest_smaller_power_of_10(b) * 10;
    (a - (a % mask)) + (b % mask)
}

fn changed_part(a: i32, b: i32, min: u32) -> i32 {
    let mut base = (a.max(b) as f64).log10().floor() as u32;

    // Check whether the digit at the given base is the same
    while {
        let a_digit = a as i64 / 10_i64.saturating_pow(base);
        let b_digit = b as i64 / 10_i64.saturating_pow(base);
        a_digit == b_digit && base > min
    } {
        base -= 1;
    }

    (b as i64 % 10_i64.saturating_pow(base + 1)) as i32
}

/// How to treat the non-dropping name particle when printing names.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DemoteNonDroppingParticle {
    /// Treat as part of the first name.
    Never,
    /// Treat as part of the first name except when sorting.
    SortOnly,
    /// Treat as part of the family name.
    #[default]
    DisplayAndSort,
}

/// Citation style metadata
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct StyleInfo {
    /// The authors of the style
    #[serde(rename = "author")]
    #[serde(default)]
    pub authors: Vec<StyleAttribution>,
    /// Contributors to the style
    #[serde(rename = "contributor")]
    #[serde(default)]
    pub contibutors: Vec<StyleAttribution>,
    /// Which format the citations are in.
    #[serde(default)]
    pub category: Vec<StyleCategory>,
    /// Which academic field the style is used in.
    #[serde(default)]
    pub field: Vec<Field>,
    /// A unique identifier for the style. May be a URL or an UUID.
    pub id: String,
    /// The ISSN for the source of the style's publication.
    #[serde(default)]
    pub issn: Vec<String>,
    /// The eISSN for the source of the style's publication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eissn: Option<String>,
    /// The ISSN-L for the source of the style's publication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issnl: Option<String>,
    /// Links with more information about the style.
    #[serde(default)]
    pub link: Vec<InfoLink>,
    /// When the style was initially published.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<Timestamp>,
    /// Under which license the style is published.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rights: Option<License>,
    /// A short description of the style.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<LocalString>,
    /// The title of the style.
    pub title: LocalString,
    /// A shortened version of the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_short: Option<LocalString>,
    /// When the style was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<Timestamp>,
}

impl StyleInfo {
    /// Remove all non-required fields.
    pub fn purge(&mut self, level: PurgeLevel) {
        self.field.clear();
        self.issn.clear();
        self.eissn = None;
        self.issnl = None;
        self.published = None;
        self.summary = None;
        self.updated = None;

        match level {
            PurgeLevel::Basic => {
                for person in self.authors.iter_mut().chain(self.contibutors.iter_mut()) {
                    person.email = None;
                    person.uri = None;
                }
                self.link.retain(|i| {
                    matches!(i.rel, InfoLinkRel::IndependentParent | InfoLinkRel::Zelf)
                });
            }
            PurgeLevel::Full => {
                self.authors.clear();
                self.contibutors.clear();
                self.link.retain(|i| i.rel == InfoLinkRel::IndependentParent);
                self.rights = None;
            }
        }
    }
}

/// A string annotated with a locale.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocalString {
    /// The string's locale.
    #[serde(rename = "@lang")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<LocaleCode>,
    /// The string's value.
    #[serde(rename = "$value", default)]
    pub value: String,
}

/// A person affiliated with the style.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct StyleAttribution {
    /// The person's name.
    pub name: String,
    /// The person's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// A URI for the person.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// Which category this style belongs in.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StyleCategory {
    /// Which format the citations are in. May only appear once as a child of `category`.
    CitationFormat {
        /// Which format the citations are in.
        #[serde(rename = "@citation-format")]
        format: CitationFormat,
    },
    /// Which academic field the style is used in. May appear multiple times as a child of `category`.
    Field {
        /// Which academic field the style is used in.
        #[serde(rename = "@field")]
        field: Field,
    },
}

/// What type of in-text citation is used.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CitationFormat {
    /// “… (Doe, 1999)”
    AuthorDate,
    /// “… (Doe)”
    Author,
    /// “… \[1\]”
    Numeric,
    /// “… \[doe99\]”
    Label,
    /// The citation appears as a foot- or endnote.
    Note,
}

/// In which academic field the style is used.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Anthropology,
    Astronomy,
    Biology,
    Botany,
    Chemistry,
    Communications,
    Engineering,
    /// Used for generic styles like Harvard and APA.
    #[serde(rename = "generic-base")]
    GenericBase,
    Geography,
    Geology,
    History,
    Humanities,
    Law,
    Linguistics,
    Literature,
    Math,
    Medicine,
    Philosophy,
    Physics,
    PoliticalScience,
    Psychology,
    Science,
    SocialScience,
    Sociology,
    Theology,
    Zoology,
}

/// A link with more information about the style.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct InfoLink {
    /// The link's URL.
    #[serde(rename = "@href")]
    pub href: String,
    /// How the link relates to the style.
    #[serde(rename = "@rel")]
    pub rel: InfoLinkRel,
    /// A human-readable description of the link.
    #[serde(rename = "$value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The link's locale.
    #[serde(rename = "@xml:lang")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<LocaleCode>,
}

/// How a link relates to the style.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InfoLinkRel {
    /// Website of the style.
    #[serde(rename = "self")]
    Zelf,
    /// URL from which the style is derived. Must not appear in dependent styles.
    Template,
    /// URL of the style's documentation.
    Documentation,
    /// Parent of a dependent style. Must appear in dependent styles.
    IndependentParent,
}

/// An ISO 8601 chapter 5.4 timestamp.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Timestamp {
    /// The timestamp's value.
    #[serde(rename = "$text")]
    pub raw: String,
}

/// A license description.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct License {
    /// The license's name.
    #[serde(rename = "$text")]
    pub name: String,
    /// The license's URL.
    #[serde(rename = "@license")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// The license string's locale.
    #[serde(rename = "@xml:lang")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<LocaleCode>,
}

/// Formatting instructions for in-text or note citations.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Citation {
    /// How items are sorted within the citation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// The citation's formatting rules.
    pub layout: Layout,
    /// Expand names that are ambiguous in short form.
    ///
    /// Default: `false`
    #[serde(
        rename = "@disambiguate-add-givenname",
        default,
        deserialize_with = "deserialize_bool"
    )]
    pub disambiguate_add_givenname: bool,
    /// When to expand names that are ambiguous in short form.
    #[serde(rename = "@givenname-disambiguation-rule", default)]
    pub givenname_disambiguation_rule: DisambiguationRule,
    /// Disambiguate by adding more names that would otherwise be hidden by et al.
    ///
    /// Default: `false`
    #[serde(
        rename = "@disambiguate-add-names",
        default,
        deserialize_with = "deserialize_bool"
    )]
    pub disambiguate_add_names: bool,
    /// Disambiguate by adding an alphabetical suffix to the year.
    ///
    /// Default: `false`
    #[serde(
        rename = "@disambiguate-add-year-suffix",
        default,
        deserialize_with = "deserialize_bool"
    )]
    pub disambiguate_add_year_suffix: bool,
    /// Group items in cite by name.
    #[serde(rename = "@cite-group-delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cite_group_delimiter: Option<String>,
    /// How to collapse cites with similar items.
    #[serde(rename = "@collapse")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapse: Option<Collapse>,
    /// Delimiter between year suffixes.
    #[serde(rename = "@year-suffix-delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year_suffix_delimiter: Option<String>,
    /// Delimiter after a collapsed cite group.
    #[serde(rename = "@after-collapse-delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_collapse_delimiter: Option<String>,
    /// When near-note-distance is true.
    ///
    /// Default: `5`
    #[serde(
        rename = "@near-note-distance",
        default = "Citation::default_near_note_distance",
        deserialize_with = "deserialize_u32"
    )]
    pub near_note_distance: u32,
    /// Options for the names within.
    #[serde(flatten)]
    pub name_options: InheritableNameOptions,
}

impl Citation {
    /// Return the default value for `cite_group_delimiter` if implicitly needed
    /// due to presence of a `collapse` attribute.
    pub const DEFAULT_CITE_GROUP_DELIMITER: &'static str = ", ";

    /// Return a citation with default settings and the given layout.
    pub fn with_layout(layout: Layout) -> Self {
        Self {
            sort: None,
            layout,
            disambiguate_add_givenname: false,
            givenname_disambiguation_rule: DisambiguationRule::default(),
            disambiguate_add_names: false,
            disambiguate_add_year_suffix: false,
            cite_group_delimiter: None,
            collapse: None,
            year_suffix_delimiter: None,
            after_collapse_delimiter: None,
            near_note_distance: Self::default_near_note_distance(),
            name_options: Default::default(),
        }
    }

    /// Return the `year_suffix_delimiter`.
    pub fn get_year_suffix_delimiter(&self) -> &str {
        self.year_suffix_delimiter
            .as_deref()
            .or(self.layout.delimiter.as_deref())
            .unwrap_or_default()
    }

    /// Return the `after_collapse_delimiter`.
    pub fn get_after_collapse_delimiter(&self) -> &str {
        self.after_collapse_delimiter
            .as_deref()
            .or(self.layout.delimiter.as_deref())
            .unwrap_or_default()
    }

    /// Return the default `near_note_distance`.
    pub const fn default_near_note_distance() -> u32 {
        5
    }
}

/// When to expand names that are ambiguous in short form.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DisambiguationRule {
    /// Expand to disambiguate both cites and names.
    AllNames,
    /// Expand to disambiguate cites and names but only use initials.
    AllNamesWithInitials,
    /// Same as `AllNames` but only disambiguate the first person in a citation.
    PrimaryName,
    /// Same as `AllNamesWithInitials` but only disambiguate the first person in a citation.
    PrimaryNameWithInitials,
    /// Expand to disambiguate cites but not names.
    #[default]
    ByCite,
}

impl DisambiguationRule {
    /// Whether this rule allows full first names or only initials.
    pub fn allows_full_first_names(self) -> bool {
        match self {
            Self::AllNames | Self::PrimaryName | Self::ByCite => true,
            Self::AllNamesWithInitials | Self::PrimaryNameWithInitials => false,
        }
    }

    /// Whether this rule allows looking beyond the first name.
    pub fn allows_multiple_names(self) -> bool {
        match self {
            Self::AllNames | Self::AllNamesWithInitials | Self::ByCite => true,
            Self::PrimaryName | Self::PrimaryNameWithInitials => false,
        }
    }
}

/// How to collapse cites with similar items.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Collapse {
    /// Collapse items with increasing ranges for numeric styles.
    CitationNumber,
    /// Collapse items with the same authors and different years by omitting the author.
    Year,
    /// Same as `Year`, but equal years are omitted as well.
    YearSuffix,
    /// Same as `YearSuffix`, but also collapse the suffixes into a range.
    YearSuffixRanged,
}

/// Formatting instructions for the bibliography.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Bibliography {
    /// How items are sorted within the citation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// The citation's formatting rules.
    pub layout: Layout,
    /// Render the bibliography in a hanging indent.
    ///
    /// Default: `false`
    #[serde(rename = "@hanging-indent", default, deserialize_with = "deserialize_bool")]
    pub hanging_indent: bool,
    /// When set, the second field is aligned.
    #[serde(rename = "@second-field-align")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_field_align: Option<SecondFieldAlign>,
    /// The line spacing within the bibliography as a multiple of regular line spacing.
    #[serde(rename = "@line-spacing", default = "Bibliography::default_line_spacing")]
    pub line_spacing: NonZeroI16,
    /// Extra space between entries as a multiple of line height.
    #[serde(rename = "@entry-spacing", default = "Bibliography::default_entry_spacing")]
    pub entry_spacing: i16,
    /// When set, subsequent identical names are replaced with this.
    #[serde(rename = "@subsequent-author-substitute")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsequent_author_substitute: Option<String>,
    /// How to replace subsequent identical names.
    #[serde(rename = "@subsequent-author-substitute-rule", default)]
    pub subsequent_author_substitute_rule: SubsequentAuthorSubstituteRule,
    /// Options for the names within.
    #[serde(flatten)]
    pub name_options: InheritableNameOptions,
}

impl Bibliography {
    /// Return a bibliography with default settings and the given layout.
    pub fn with_layout(layout: Layout) -> Self {
        Self {
            sort: None,
            layout,
            hanging_indent: false,
            second_field_align: None,
            line_spacing: Self::default_line_spacing(),
            entry_spacing: Self::default_entry_spacing(),
            subsequent_author_substitute: None,
            subsequent_author_substitute_rule: Default::default(),
            name_options: Default::default(),
        }
    }

    /// Return the default `line_spacing`.
    fn default_line_spacing() -> NonZeroI16 {
        NonZeroI16::new(1).unwrap()
    }

    /// Return the default `entry_spacing`.
    const fn default_entry_spacing() -> i16 {
        1
    }
}

/// How to position the first field if the second field is aligned in a bibliography.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecondFieldAlign {
    /// Put the first field in the margin and align with the margin.
    Margin,
    /// Flush the first field with the margin.
    Flush,
}

/// How to replace subsequent identical names in a bibliography.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubsequentAuthorSubstituteRule {
    /// When all names match, replace.
    #[default]
    CompleteAll,
    /// When all names match, replace each name.
    CompleteEach,
    /// Each matching name is replaced.
    PartialEach,
    /// Only the first matching name is replaced.
    PartialFirst,
}

/// How to sort elements in a bibliography or citation.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Sort {
    /// The ordered list of sorting keys.
    #[serde(rename = "key")]
    pub keys: Vec<SortKey>,
}

/// A sorting key.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SortKey {
    /// Sort by the value of a variable.
    Variable {
        /// The variable to sort by.
        #[serde(rename = "@variable")]
        variable: Variable,
        /// In which direction to sort.
        #[serde(rename = "@sort", default)]
        sort_direction: SortDirection,
    },
    /// Sort by the output of a macro.
    MacroName {
        /// The name of the macro.
        #[serde(rename = "@macro")]
        name: String,
        /// Override `[InheritedNameOptions::et_al_min]` and
        /// `[InheritedNameOptions::et_al_subsequent_min]` for macros.
        #[serde(
            rename = "@names-min",
            deserialize_with = "deserialize_u32_option",
            default
        )]
        #[serde(skip_serializing_if = "Option::is_none")]
        names_min: Option<u32>,
        /// Override `[InheritedNameOptions::et_al_use_first]` and
        /// `[InheritedNameOptions::et_al_subsequent_use_first]` for macros.
        #[serde(
            rename = "@names-use-first",
            deserialize_with = "deserialize_u32_option",
            default
        )]
        #[serde(skip_serializing_if = "Option::is_none")]
        names_use_first: Option<u32>,
        /// Override `[InheritedNameOptions::et_al_use_last]` for macros.
        #[serde(
            rename = "@names-use-last",
            deserialize_with = "deserialize_bool_option",
            default
        )]
        #[serde(skip_serializing_if = "Option::is_none")]
        names_use_last: Option<bool>,
        /// In which direction to sort.
        #[serde(rename = "@sort", default)]
        sort_direction: SortDirection,
    },
}

impl From<Variable> for SortKey {
    fn from(value: Variable) -> Self {
        Self::Variable {
            variable: value,
            sort_direction: SortDirection::default(),
        }
    }
}

impl SortKey {
    /// Retrieve the sort direction.
    pub const fn sort_direction(&self) -> SortDirection {
        match self {
            Self::Variable { sort_direction, .. } => *sort_direction,
            Self::MacroName { sort_direction, .. } => *sort_direction,
        }
    }
}

/// The direction to sort in.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortDirection {
    /// Sort in ascending order.
    #[default]
    Ascending,
    /// Sort in descending order.
    Descending,
}

/// A formatting rule.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Layout {
    /// Parts of the rule.
    #[serde(rename = "$value")]
    pub elements: Vec<LayoutRenderingElement>,
    // Formatting and affixes fields are rolled into this because
    // #[serde(flatten)] doesn't work with $value fields.
    /// Set the font style.
    #[serde(rename = "@font-style")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<FontStyle>,
    /// Choose normal or small caps.
    #[serde(rename = "@font-variant")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_variant: Option<FontVariant>,
    /// Set the font weight.
    #[serde(rename = "@font-weight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<FontWeight>,
    /// Choose underlining.
    #[serde(rename = "@text-decoration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<TextDecoration>,
    /// Choose vertical alignment.
    #[serde(rename = "@vertical-align")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<VerticalAlign>,
    /// The prefix.
    #[serde(rename = "@prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// The suffix.
    #[serde(rename = "@suffix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Delimit pieces of the output.
    #[serde(rename = "@delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
}

to_formatting!(Layout, self);
to_affixes!(Layout, self);

impl Layout {
    /// Return a layout.
    pub fn new(
        elements: Vec<LayoutRenderingElement>,
        formatting: Formatting,
        affixes: Option<Affixes>,
        delimiter: Option<String>,
    ) -> Self {
        let (prefix, suffix) = if let Some(affixes) = affixes {
            (affixes.prefix, affixes.suffix)
        } else {
            (None, None)
        };

        Self {
            elements,
            font_style: formatting.font_style,
            font_variant: formatting.font_variant,
            font_weight: formatting.font_weight,
            text_decoration: formatting.text_decoration,
            vertical_align: formatting.vertical_align,
            prefix,
            suffix,
            delimiter,
        }
    }

    /// Return a layout with default settings and the given elements.
    pub fn with_elements(elements: Vec<LayoutRenderingElement>) -> Self {
        Self::new(elements, Formatting::default(), None, None)
    }

    /// Find the child element that will render the given variable.
    pub fn find_variable_element(
        &self,
        variable: Variable,
        macros: &[CslMacro],
    ) -> Option<LayoutRenderingElement> {
        self.elements
            .iter()
            .find_map(|e| e.find_variable_element(variable, macros))
    }
}

/// Possible parts of a formatting rule.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutRenderingElement {
    /// Insert a term or variable.
    Text(Text),
    /// Format a date.
    Date(Date),
    /// Format a number.
    Number(Number),
    /// Format a list of names.
    Names(Names),
    /// Prints a label for a variable.
    Label(Label),
    /// Container for rendering elements.
    Group(Group),
    /// Conditional rendering.
    Choose(Choose),
}

impl LayoutRenderingElement {
    /// Find the child element that will render the given variable.
    pub fn find_variable_element(
        &self,
        variable: Variable,
        macros: &[CslMacro],
    ) -> Option<Self> {
        match self {
            Self::Text(t) => t.find_variable_element(variable, macros),
            Self::Choose(c) => c.find_variable_element(variable, macros),
            Self::Date(d) => {
                if d.variable.map(Variable::Date) == Some(variable) {
                    Some(self.clone())
                } else {
                    None
                }
            }
            Self::Number(n) => {
                if Variable::Number(n.variable) == variable {
                    Some(self.clone())
                } else {
                    None
                }
            }
            Self::Names(n) => {
                if n.variable.iter().any(|v| Variable::Name(*v) == variable) {
                    Some(self.clone())
                } else {
                    None
                }
            }
            Self::Group(g) => g
                .children
                .iter()
                .find_map(|e| e.find_variable_element(variable, macros)),
            Self::Label(_) => None,
        }
    }
}

/// Rendering elements.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RenderingElement {
    /// A layout element.
    Layout(Layout),
    /// Other rendering elements.
    Other(LayoutRenderingElement),
}

/// Print a term or variable.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Text {
    /// The term or variable to print.
    #[serde(flatten)]
    pub target: TextTarget,
    /// Override formatting style.
    #[serde(flatten)]
    pub formatting: Formatting,
    /// Add prefix and suffix.
    #[serde(flatten)]
    pub affixes: Affixes,
    /// Set layout level.
    #[serde(rename = "@display")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
    /// Whether to wrap this text in quotes.
    ///
    /// Default: `false`
    #[serde(rename = "@quotes", default, deserialize_with = "deserialize_bool")]
    pub quotes: bool,
    /// Remove periods from the output.
    ///
    /// Default: `false`
    #[serde(rename = "@strip-periods", default, deserialize_with = "deserialize_bool")]
    pub strip_periods: bool,
    /// Transform the text case.
    #[serde(rename = "@text-case")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_case: Option<TextCase>,
}

impl Text {
    /// Return a text with default settings and the given target.
    pub fn with_target(target: impl Into<TextTarget>) -> Self {
        Self {
            target: target.into(),
            formatting: Default::default(),
            affixes: Default::default(),
            display: None,
            quotes: false,
            strip_periods: false,
            text_case: None,
        }
    }

    /// Find the child element that will render the given variable.
    pub fn find_variable_element(
        &self,
        variable: Variable,
        macros: &[CslMacro],
    ) -> Option<LayoutRenderingElement> {
        match &self.target {
            TextTarget::Variable { var, .. } => {
                if *var == variable {
                    Some(LayoutRenderingElement::Text(self.clone()))
                } else {
                    None
                }
            }
            TextTarget::Macro { name } => {
                if let Some(m) = macros.iter().find(|m| m.name == *name) {
                    m.children
                        .iter()
                        .find_map(|e| e.find_variable_element(variable, macros))
                } else {
                    None
                }
            }
            TextTarget::Term { .. } => None,
            TextTarget::Value { .. } => None,
        }
    }
}

to_formatting!(Text);
to_affixes!(Text);

/// Various kinds of text targets.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TextTarget {
    /// Prints the value of a variable.
    Variable {
        #[serde(rename = "@variable")]
        /// The variable to print.
        var: Variable,
        #[serde(rename = "@form", default)]
        /// The form of the variable.
        form: LongShortForm,
    },
    /// Prints the text output of a macro.
    Macro {
        #[serde(rename = "@macro")]
        /// The name of the macro.
        name: String,
    },
    /// Prints a localized term.
    Term {
        /// The term to print.
        #[serde(rename = "@term")]
        term: Term,
        /// The form of the term.
        #[serde(rename = "@form", default)]
        form: TermForm,
        /// Whether the term is pluralized.
        #[serde(rename = "@plural", default, deserialize_with = "deserialize_bool")]
        plural: bool,
    },
    /// Prints a given string.
    Value {
        #[serde(rename = "@value")]
        /// The string to print.
        val: String,
    },
}

impl From<Variable> for TextTarget {
    fn from(value: Variable) -> Self {
        Self::Variable { var: value, form: LongShortForm::default() }
    }
}

impl From<Term> for TextTarget {
    fn from(value: Term) -> Self {
        Self::Term {
            term: value,
            form: TermForm::default(),
            plural: bool::default(),
        }
    }
}

/// Formats a date.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Date {
    /// The date to format.
    #[serde(rename = "@variable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable: Option<DateVariable>,
    /// How the localized date should be formatted.
    #[serde(rename = "@form")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<DateForm>,
    /// Which parts of the localized date should be included.
    #[serde(rename = "@date-parts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<DateParts>,
    /// Override the default date parts. Also specifies the order of the parts
    /// if `form` is `None`.
    #[serde(default)]
    pub date_part: Vec<DatePart>,
    /// Override formatting style.
    #[serde(flatten)]
    pub formatting: Formatting,
    /// Add prefix and suffix. Ignored when this defines a localized date format.
    #[serde(flatten)]
    pub affixes: Affixes,
    /// Delimit pieces of the output. Ignored when this defines a localized date format.
    #[serde(rename = "@delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
    /// Set layout level.
    #[serde(rename = "@display")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
    /// Transform the text case.
    #[serde(rename = "@text-case")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_case: Option<TextCase>,
}

to_formatting!(Date);
to_affixes!(Date);

impl Date {
    /// Whether this is a localized or a standalone date.
    pub const fn is_localized(&self) -> bool {
        self.form.is_some()
    }
}

/// Localized date formats.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DateForm {
    /// “12-15-2005”
    Numeric,
    /// “December 15, 2005”
    Text,
}

/// Which parts of a date should be included.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[allow(missing_docs)]
#[serde(rename_all = "kebab-case")]
pub enum DateParts {
    Year,
    YearMonth,
    #[default]
    YearMonthDay,
}

impl DateParts {
    /// Check if the date shall contain a month.
    pub const fn has_month(self) -> bool {
        matches!(self, Self::YearMonth | Self::YearMonthDay)
    }

    /// Check if the date shall contain a day.
    pub const fn has_day(self) -> bool {
        matches!(self, Self::YearMonthDay)
    }
}

/// Override the default date parts.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct DatePart {
    /// Kind of the date part.
    #[serde(rename = "@name")]
    pub name: DatePartName,
    /// Form of the date part.
    #[serde(rename = "@form")]
    #[serde(skip_serializing_if = "Option::is_none")]
    form: Option<DateAnyForm>,
    /// The string used to delimit two date parts.
    #[serde(rename = "@range-delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_delimiter: Option<String>,
    /// Override formatting style.
    #[serde(flatten)]
    pub formatting: Formatting,
    /// Add prefix and suffix. Ignored when this defines a localized date format.
    #[serde(flatten)]
    pub affixes: Affixes,
    /// Remove periods from the date part.
    ///
    /// Default: `false`
    #[serde(rename = "@strip-periods", default, deserialize_with = "deserialize_bool")]
    pub strip_periods: bool,
    /// Transform the text case.
    #[serde(rename = "@text-case")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_case: Option<TextCase>,
}

to_formatting!(DatePart);
to_affixes!(DatePart);

impl DatePart {
    /// Retrieve the default delimiter for the date part.
    pub const DEFAULT_DELIMITER: &'static str = "–";

    /// Retrieve the form.
    pub fn form(&self) -> DateStrongAnyForm {
        DateStrongAnyForm::for_name(self.name, self.form)
    }
}

/// The kind of a date part with its `form` attribute.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DatePartName {
    Day,
    Month,
    Year,
}

/// Any allowable date part format.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DateAnyForm {
    /// “1”
    Numeric,
    /// “01”
    NumericLeadingZeros,
    /// “1st”
    Ordinal,
    /// “January”
    Long,
    /// “Jan.”
    Short,
}

/// Strongly typed date part formats.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum DateStrongAnyForm {
    Day(DateDayForm),
    Month(DateMonthForm),
    Year(LongShortForm),
}

impl DateStrongAnyForm {
    /// Get a strongly typed date form for a name. Must return `Some` for valid
    /// CSL files.
    pub fn for_name(name: DatePartName, form: Option<DateAnyForm>) -> Self {
        match name {
            DatePartName::Day => {
                Self::Day(form.map(DateAnyForm::form_for_day).unwrap_or_default())
            }
            DatePartName::Month => {
                Self::Month(form.map(DateAnyForm::form_for_month).unwrap_or_default())
            }
            DatePartName::Year => {
                Self::Year(form.map(DateAnyForm::form_for_year).unwrap_or_default())
            }
        }
    }
}

impl DateAnyForm {
    /// Retrieve the form for a day.
    pub fn form_for_day(self) -> DateDayForm {
        match self {
            Self::NumericLeadingZeros => DateDayForm::NumericLeadingZeros,
            Self::Ordinal => DateDayForm::Ordinal,
            _ => DateDayForm::default(),
        }
    }

    /// Retrieve the form for a month.
    pub fn form_for_month(self) -> DateMonthForm {
        match self {
            Self::Short => DateMonthForm::Short,
            Self::Numeric => DateMonthForm::Numeric,
            Self::NumericLeadingZeros => DateMonthForm::NumericLeadingZeros,
            _ => DateMonthForm::default(),
        }
    }

    /// Retrieve the form for a year.
    pub fn form_for_year(self) -> LongShortForm {
        match self {
            Self::Short => LongShortForm::Short,
            _ => LongShortForm::default(),
        }
    }
}

/// How a day is formatted.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DateDayForm {
    /// “1”
    #[default]
    Numeric,
    /// “01”
    NumericLeadingZeros,
    /// “1st”
    Ordinal,
}

/// How a month is formatted.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DateMonthForm {
    /// “January”
    #[default]
    Long,
    /// “Jan.”
    Short,
    /// “1”
    Numeric,
    /// “01”
    NumericLeadingZeros,
}

/// Whether to format something in long or short form.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub enum LongShortForm {
    #[default]
    Long,
    Short,
}

/// Renders a number.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Number {
    /// The variable whose value is used.
    #[serde(rename = "@variable")]
    pub variable: NumberVariable,
    /// How the number is formatted.
    #[serde(rename = "@form", default)]
    pub form: NumberForm,
    /// Override formatting style.
    #[serde(flatten)]
    pub formatting: Formatting,
    /// Add prefix and suffix.
    #[serde(flatten)]
    pub affixes: Affixes,
    /// Set layout level.
    #[serde(rename = "@display")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
    /// Transform the text case.
    #[serde(rename = "@text-case")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_case: Option<TextCase>,
}

to_formatting!(Number);
to_affixes!(Number);

/// How a number is formatted.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NumberForm {
    /// “1”
    #[default]
    Numeric,
    /// “1st”
    Ordinal,
    /// “first”
    LongOrdinal,
    /// “I”
    Roman,
}

/// Renders a list of names.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Names {
    /// The variable whose value is used.
    #[serde(rename = "@variable", default)]
    pub variable: Vec<NameVariable>,
    /// Child elements.
    #[serde(rename = "$value", default)]
    pub children: Vec<NamesChild>,
    /// Delimiter between names.
    #[serde(rename = "@delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    delimiter: Option<String>,

    /// Delimiter between second-to-last and last name.
    #[serde(rename = "@and")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub and: Option<NameAnd>,
    /// Delimiter before et al.
    #[serde(rename = "@delimiter-precedes-et-al")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter_precedes_et_al: Option<DelimiterBehavior>,
    /// Whether to use the delimiter before the last name.
    #[serde(rename = "@delimiter-precedes-last")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter_precedes_last: Option<DelimiterBehavior>,
    /// Minimum number of names to use et al.
    #[serde(rename = "@et-al-min", deserialize_with = "deserialize_u32_option", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_min: Option<u32>,
    /// Maximum number of names to use before et al.
    #[serde(
        rename = "@et-al-use-first",
        deserialize_with = "deserialize_u32_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_use_first: Option<u32>,
    /// Minimum number of names to use et al. for repeated citations.
    #[serde(
        rename = "@et-al-subsequent-min",
        deserialize_with = "deserialize_u32_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_subsequent_min: Option<u32>,
    /// Maximum number of names to use before et al. for repeated citations.
    #[serde(
        rename = "@et-al-subsequent-use-first",
        deserialize_with = "deserialize_u32_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_subsequent_use_first: Option<u32>,
    /// Whether to use the last name in the author list when there are at least
    /// `et_al_min` names.
    #[serde(
        rename = "@et-al-use-last",
        deserialize_with = "deserialize_bool_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_use_last: Option<bool>,
    /// Which name parts to display for personal names.
    #[serde(rename = "@name-form")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_form: Option<NameForm>,
    /// Whether to initialize the first name if `initialize-with` is Some.
    #[serde(
        rename = "@initialize",
        deserialize_with = "deserialize_bool_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize: Option<bool>,
    /// String to initialize the first name with.
    #[serde(rename = "@initialize-with")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize_with: Option<String>,
    /// Whether to turn the name around.
    #[serde(rename = "@name-as-sort-order")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_as_sort_order: Option<NameAsSortOrder>,
    /// Delimiter between given name and first name. Only used if
    /// `name-as-sort-order` is Some.
    #[serde(rename = "@sort-separator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_separator: Option<String>,

    /// Set the font style.
    #[serde(rename = "@font-style")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<FontStyle>,
    /// Choose normal or small caps.
    #[serde(rename = "@font-variant")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_variant: Option<FontVariant>,
    /// Set the font weight.
    #[serde(rename = "@font-weight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<FontWeight>,
    /// Choose underlining.
    #[serde(rename = "@text-decoration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<TextDecoration>,
    /// Choose vertical alignment.
    #[serde(rename = "@vertical-align")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<VerticalAlign>,

    /// The prefix.
    #[serde(rename = "@prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// The suffix.
    #[serde(rename = "@suffix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// Set layout level.
    #[serde(rename = "@display")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
}

impl Names {
    /// Return names with default settings and the given variables.
    pub fn with_variables(variables: Vec<NameVariable>) -> Self {
        Self {
            variable: variables,
            children: Vec::default(),
            delimiter: None,

            and: None,
            delimiter_precedes_et_al: None,
            delimiter_precedes_last: None,
            et_al_min: None,
            et_al_use_first: None,
            et_al_subsequent_min: None,
            et_al_subsequent_use_first: None,
            et_al_use_last: None,
            name_form: None,
            initialize: None,
            initialize_with: None,
            name_as_sort_order: None,
            sort_separator: None,

            font_style: None,
            font_variant: None,
            font_weight: None,
            text_decoration: None,
            vertical_align: None,

            prefix: None,
            suffix: None,

            display: None,
        }
    }

    /// Return the delimiter given some name options.
    pub fn delimiter<'a>(&'a self, name_options: &'a InheritableNameOptions) -> &'a str {
        self.delimiter
            .as_deref()
            .or(name_options.name_delimiter.as_deref())
            .unwrap_or_default()
    }

    /// Return the name element.
    pub fn name(&self) -> Option<&Name> {
        self.children.iter().find_map(|c| match c {
            NamesChild::Name(n) => Some(n),
            _ => None,
        })
    }

    /// Return the et-al element.
    pub fn et_al(&self) -> Option<&EtAl> {
        self.children.iter().find_map(|c| match c {
            NamesChild::EtAl(e) => Some(e),
            _ => None,
        })
    }

    /// Return the label element.
    pub fn label(&self) -> Option<(&VariablelessLabel, NameLabelPosition)> {
        let mut pos = NameLabelPosition::BeforeName;
        self.children.iter().find_map(|c| match c {
            NamesChild::Label(l) => Some((l, pos)),
            NamesChild::Name(_) => {
                pos = NameLabelPosition::AfterName;
                None
            }
            _ => None,
        })
    }

    /// Return the substitute element.
    pub fn substitute(&self) -> Option<&Substitute> {
        self.children.iter().find_map(|c| match c {
            NamesChild::Substitute(s) => Some(s),
            _ => None,
        })
    }

    /// Return the inheritable name options.
    pub fn options(&self) -> InheritableNameOptions {
        InheritableNameOptions {
            and: self.and,
            delimiter_precedes_et_al: self.delimiter_precedes_et_al,
            delimiter_precedes_last: self.delimiter_precedes_last,
            et_al_min: self.et_al_min,
            et_al_use_first: self.et_al_use_first,
            et_al_subsequent_min: self.et_al_subsequent_min,
            et_al_subsequent_use_first: self.et_al_subsequent_use_first,
            et_al_use_last: self.et_al_use_last,
            name_form: self.name_form,
            initialize: self.initialize,
            initialize_with: self.initialize_with.clone(),
            name_as_sort_order: self.name_as_sort_order,
            sort_separator: self.sort_separator.clone(),
            name_delimiter: None,
            names_delimiter: self.delimiter.clone(),
        }
    }

    /// Convert a [`Names`] within a substitute to a name using the parent element.
    pub fn from_names_substitute(&self, child: &Self) -> Names {
        if child.name().is_some()
            || child.et_al().is_some()
            || child.substitute().is_some()
        {
            return child.clone();
        }

        let formatting = child.to_formatting().apply(self.to_formatting());
        let options = self.options().apply(&child.options());

        Names {
            variable: if child.variable.is_empty() {
                self.variable.clone()
            } else {
                child.variable.clone()
            },
            children: self
                .children
                .iter()
                .filter(|c| !matches!(c, NamesChild::Substitute(_)))
                .cloned()
                .collect(),
            delimiter: child.delimiter.clone().or_else(|| self.delimiter.clone()),

            and: options.and,
            delimiter_precedes_et_al: options.delimiter_precedes_et_al,
            delimiter_precedes_last: options.delimiter_precedes_last,
            et_al_min: options.et_al_min,
            et_al_use_first: options.et_al_use_first,
            et_al_subsequent_min: options.et_al_subsequent_min,
            et_al_subsequent_use_first: options.et_al_subsequent_use_first,
            et_al_use_last: options.et_al_use_last,
            name_form: options.name_form,
            initialize: options.initialize,
            initialize_with: options.initialize_with,
            name_as_sort_order: options.name_as_sort_order,
            sort_separator: options.sort_separator,

            font_style: formatting.font_style,
            font_variant: formatting.font_variant,
            font_weight: formatting.font_weight,
            text_decoration: formatting.text_decoration,
            vertical_align: formatting.vertical_align,

            prefix: child.prefix.clone().or_else(|| self.prefix.clone()),
            suffix: child.suffix.clone().or_else(|| self.suffix.clone()),
            display: child.display.or(self.display),
        }
    }
}

to_formatting!(Names, self);
to_affixes!(Names, self);

/// Where the `cs:label` element within a `cs:names` element appeared relative
/// to `cs:name`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NameLabelPosition {
    /// The label appeared after the name element.
    AfterName,
    /// The label appeared before the name element.
    BeforeName,
}

/// Possible children for a `cs:names` element.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NamesChild {
    /// A `cs:name` element.
    Name(Name),
    /// A `cs:et-al` element.
    EtAl(EtAl),
    /// A `cs:label` element.
    Label(VariablelessLabel),
    /// A `cs:substitute` element.
    Substitute(Substitute),
}

/// Configuration of how to print names.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Name {
    /// Delimiter between names.
    #[serde(rename = "@delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    delimiter: Option<String>,
    /// Which name parts to display for personal names.
    #[serde(rename = "@form")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<NameForm>,
    /// Name parts for formatting for the given and family name.
    #[serde(rename = "name-part")]
    parts: Vec<NamePart>,
    /// Options for this name.
    #[serde(flatten)]
    options: InheritableNameOptions,
    /// Override formatting style.
    #[serde(flatten)]
    pub formatting: Formatting,
    /// Add prefix and suffix.
    #[serde(flatten)]
    pub affixes: Affixes,
}

to_formatting!(Name);
to_affixes!(Name);

impl Name {
    /// Retrieve [`NamePart`] configuration for the given name.
    pub fn name_part_given(&self) -> Option<&NamePart> {
        self.parts.iter().find(|p| p.name == NamePartName::Given)
    }

    /// Retrieve [`NamePart`] configuration for the family name.
    pub fn name_part_family(&self) -> Option<&NamePart> {
        self.parts.iter().find(|p| p.name == NamePartName::Family)
    }

    /// Retrieve the [`NameOptions`] for this name.
    pub fn options<'s>(&'s self, inherited: &'s InheritableNameOptions) -> NameOptions {
        let applied = inherited.apply(&self.options);
        NameOptions {
            and: applied.and,
            delimiter: self
                .delimiter
                .as_deref()
                .or(inherited.name_delimiter.as_deref())
                .unwrap_or(", "),
            delimiter_precedes_et_al: applied
                .delimiter_precedes_et_al
                .unwrap_or_default(),
            delimiter_precedes_last: applied.delimiter_precedes_last.unwrap_or_default(),
            et_al_min: applied.et_al_min,
            et_al_use_first: applied.et_al_use_first,
            et_al_subsequent_min: applied.et_al_subsequent_min,
            et_al_subsequent_use_first: applied.et_al_subsequent_use_first,
            et_al_use_last: applied.et_al_use_last.unwrap_or_default(),
            form: self.form.or(inherited.name_form).unwrap_or_default(),
            initialize: applied.initialize.unwrap_or(true),
            initialize_with: self
                .options
                .initialize_with
                .as_deref()
                .or(inherited.initialize_with.as_deref()),
            name_as_sort_order: applied.name_as_sort_order,
            sort_separator: self
                .options
                .sort_separator
                .as_deref()
                .or(inherited.sort_separator.as_deref())
                .unwrap_or(", "),
        }
    }
}

/// Global configuration of how to print names.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(default)]
pub struct InheritableNameOptions {
    /// Delimiter between second-to-last and last name.
    #[serde(rename = "@and")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub and: Option<NameAnd>,
    /// Delimiter inherited to `cs:name` elements.
    #[serde(rename = "@name-delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_delimiter: Option<String>,
    /// Delimiter inherited to `cs:names` elements.
    #[serde(rename = "@names-delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names_delimiter: Option<String>,
    /// Delimiter before et al.
    #[serde(rename = "@delimiter-precedes-et-al")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter_precedes_et_al: Option<DelimiterBehavior>,
    /// Whether to use the delimiter before the last name.
    #[serde(rename = "@delimiter-precedes-last")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter_precedes_last: Option<DelimiterBehavior>,
    /// Minimum number of names to use et al.
    #[serde(rename = "@et-al-min", deserialize_with = "deserialize_u32_option", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_min: Option<u32>,
    /// Maximum number of names to use before et al.
    #[serde(
        rename = "@et-al-use-first",
        deserialize_with = "deserialize_u32_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_use_first: Option<u32>,
    /// Minimum number of names to use et al. for repeated citations.
    #[serde(
        rename = "@et-al-subsequent-min",
        deserialize_with = "deserialize_u32_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_subsequent_min: Option<u32>,
    /// Maximum number of names to use before et al. for repeated citations.
    #[serde(
        rename = "@et-al-subsequent-use-first",
        deserialize_with = "deserialize_u32_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_subsequent_use_first: Option<u32>,
    /// Whether to use the last name in the author list when there are at least
    /// `et_al_min` names.
    #[serde(
        rename = "@et-al-use-last",
        deserialize_with = "deserialize_bool_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub et_al_use_last: Option<bool>,
    /// Which name parts to display for personal names.
    #[serde(rename = "@name-form")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_form: Option<NameForm>,
    /// Whether to initialize the first name if `initialize-with` is Some.
    #[serde(
        rename = "@initialize",
        deserialize_with = "deserialize_bool_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize: Option<bool>,
    /// String to initialize the first name with.
    #[serde(rename = "@initialize-with")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize_with: Option<String>,
    /// Whether to turn the name around.
    #[serde(rename = "@name-as-sort-order")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_as_sort_order: Option<NameAsSortOrder>,
    /// Delimiter between given name and first name. Only used if
    /// `name-as-sort-order` is Some.
    #[serde(rename = "@sort-separator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_separator: Option<String>,
}

/// Definite name options. Obtain from [`Name::options`] using
/// [`InheritableNameOptions`].
pub struct NameOptions<'s> {
    /// Delimiter between second-to-last and last name.
    pub and: Option<NameAnd>,
    /// Delimiter to separate names.
    pub delimiter: &'s str,
    /// Delimiter before et al.
    pub delimiter_precedes_et_al: DelimiterBehavior,
    /// Whether to use the delimiter before the last name.
    pub delimiter_precedes_last: DelimiterBehavior,
    /// Minimum number of names to use et al.
    pub et_al_min: Option<u32>,
    /// Maximum number of names to use before et al.
    pub et_al_use_first: Option<u32>,
    /// Minimum number of names to use et al. for repeated citations.
    pub et_al_subsequent_min: Option<u32>,
    /// Maximum number of names to use before et al. for repeated citations.
    pub et_al_subsequent_use_first: Option<u32>,
    /// Whether to use the last name in the author list when there are at least
    /// `et_al_min` names.
    pub et_al_use_last: bool,
    /// Which name parts to display for personal names.
    pub form: NameForm,
    /// Whether to initialize the first name if `initialize-with` is Some.
    pub initialize: bool,
    /// String to initialize the first name with.
    pub initialize_with: Option<&'s str>,
    /// Whether to turn the name around.
    pub name_as_sort_order: Option<NameAsSortOrder>,
    /// Delimiter between given name and first name. Only used if
    /// `name-as-sort-order` is Some.
    pub sort_separator: &'s str,
}

impl InheritableNameOptions {
    /// Apply the child options to the parent options.
    pub fn apply(&self, child: &Self) -> Self {
        Self {
            and: child.and.or(self.and),
            name_delimiter: child
                .name_delimiter
                .clone()
                .or_else(|| self.name_delimiter.clone()),
            names_delimiter: child
                .names_delimiter
                .clone()
                .or_else(|| self.names_delimiter.clone()),
            delimiter_precedes_et_al: child
                .delimiter_precedes_et_al
                .or(self.delimiter_precedes_et_al),
            delimiter_precedes_last: child
                .delimiter_precedes_last
                .or(self.delimiter_precedes_last),
            et_al_min: child.et_al_min.or(self.et_al_min),
            et_al_use_first: child.et_al_use_first.or(self.et_al_use_first),
            et_al_subsequent_min: child
                .et_al_subsequent_min
                .or(self.et_al_subsequent_min),
            et_al_subsequent_use_first: child
                .et_al_subsequent_use_first
                .or(self.et_al_subsequent_use_first),
            et_al_use_last: child.et_al_use_last.or(self.et_al_use_last),
            name_form: child.name_form.or(self.name_form),
            initialize: child.initialize.or(self.initialize),
            initialize_with: child
                .initialize_with
                .clone()
                .or_else(|| self.initialize_with.clone()),
            name_as_sort_order: child.name_as_sort_order.or(self.name_as_sort_order),
            sort_separator: child
                .sort_separator
                .clone()
                .or_else(|| self.sort_separator.clone()),
        }
    }
}

impl NameOptions<'_> {
    /// Whether the nth name is suppressed given the number of names and this
    /// configuration.
    pub fn is_suppressed(&self, idx: usize, length: usize, is_subsequent: bool) -> bool {
        // This is not suppressed if we print the last element and this is it.
        if self.et_al_use_last && idx + 1 >= length {
            return false;
        }

        // If this is a subsequent citation of the same item, use other CSL options, if they exist
        let (et_al_min, et_al_use_first) = if is_subsequent {
            (
                self.et_al_subsequent_min.or(self.et_al_min),
                self.et_al_subsequent_use_first.or(self.et_al_use_first),
            )
        } else {
            (self.et_al_min, self.et_al_use_first)
        };

        let et_al_min = et_al_min.map_or(usize::MAX, |u| u as usize);
        let et_al_use_first = et_al_use_first.map_or(usize::MAX, |u| u as usize);

        length >= et_al_min && idx + 1 > et_al_use_first
    }
}

/// How to render the delimiter before the last name.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NameAnd {
    /// Use the string "and".
    Text,
    /// Use the ampersand character.
    Symbol,
}

/// When delimiters shall be inserted.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DelimiterBehavior {
    /// Only used for lists with more than one (`-precedes-et-al`) or two
    /// (`-precedes-last`) names.
    #[default]
    Contextual,
    /// Only use if the preceding name is inverted (per `name-as-sort-order`).
    AfterInvertedName,
    /// Always use the delimiter for this condition.
    Always,
    /// Never use the delimiter for this condition.
    Never,
}

/// How many name parts to print.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NameForm {
    /// Print all name parts
    #[default]
    Long,
    /// Print only the family name part and non-dropping-particle.
    Short,
    /// Count the total number of names.
    Count,
}

/// In which order to print the names.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NameAsSortOrder {
    /// Only the first name is turned around.
    First,
    /// All names are turned around.
    All,
}

/// How to format a given name part.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct NamePart {
    /// Which name part this applies to.
    #[serde(rename = "@name")]
    pub name: NamePartName,
    /// Override formatting style.
    #[serde(flatten)]
    pub formatting: Formatting,
    /// Add prefix and suffix.
    #[serde(flatten)]
    pub affixes: Affixes,
    /// Transform the text case.
    #[serde(rename = "@text-case")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_case: Option<TextCase>,
}

to_formatting!(NamePart);
to_affixes!(NamePart);

/// Which part of the name a [`NamePart`] applies to.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NamePartName {
    /// The given name.
    Given,
    /// The family name.
    Family,
}

/// Configure the et al. abbreviation.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct EtAl {
    /// Which term to use.
    #[serde(rename = "@term", default)]
    pub term: EtAlTerm,
    /// Override formatting style.
    #[serde(flatten)]
    pub formatting: Formatting,
}

to_formatting!(EtAl);

/// Which term to use for et al.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum EtAlTerm {
    /// “et al.”
    #[default]
    #[serde(rename = "et al", alias = "et-al")]
    EtAl,
    /// “and others”
    #[serde(rename = "and others", alias = "and-others")]
    AndOthers,
}

impl From<EtAlTerm> for Term {
    fn from(term: EtAlTerm) -> Self {
        match term {
            EtAlTerm::EtAl => Term::Other(OtherTerm::EtAl),
            EtAlTerm::AndOthers => Term::Other(OtherTerm::AndOthers),
        }
    }
}

/// What to do if the name variable is empty.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Substitute {
    /// The layout to use instead.
    #[serde(rename = "$value")]
    pub children: Vec<LayoutRenderingElement>,
}

/// Print a label for a number variable.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Label {
    /// The variable for which to print the label.
    #[serde(rename = "@variable")]
    pub variable: NumberVariable,
    /// The form of the label.
    #[serde(flatten)]
    pub label: VariablelessLabel,
}

/// A label without its variable.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct VariablelessLabel {
    /// What variant of label is chosen.
    #[serde(rename = "@form", default)]
    pub form: TermForm,
    /// How to pluiralize the label.
    #[serde(rename = "@plural", default)]
    pub plural: LabelPluralize,
    /// Override formatting style.
    #[serde(flatten)]
    pub formatting: Formatting,
    /// Add prefix and suffix.
    #[serde(flatten)]
    pub affixes: Affixes,
    /// Transform the text case.
    #[serde(rename = "@text-case")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_case: Option<TextCase>,
    /// Remove periods from the output.
    ///
    /// Default: `false`
    #[serde(rename = "@strip-periods", default, deserialize_with = "deserialize_bool")]
    pub strip_periods: bool,
}

to_formatting!(VariablelessLabel);
to_affixes!(VariablelessLabel);

/// How to pluralize a label.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LabelPluralize {
    /// Match plurality of the variable.
    #[default]
    Contextual,
    /// Always use the plural form.
    Always,
    /// Always use the singular form.
    Never,
}

/// A group of formatting instructions that is only shown if no variable is
/// referenced or at least one referenced variable is populated.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Group {
    /// The formatting instructions.
    #[serde(rename = "$value")]
    pub children: Vec<LayoutRenderingElement>,
    // Formatting and affixes fields are rolled into this because
    // #[serde(flatten)] doesn't work with $value fields.
    /// Set the font style.
    #[serde(rename = "@font-style")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<FontStyle>,
    /// Choose normal or small caps.
    #[serde(rename = "@font-variant")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_variant: Option<FontVariant>,
    /// Set the font weight.
    #[serde(rename = "@font-weight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<FontWeight>,
    /// Choose underlining.
    #[serde(rename = "@text-decoration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<TextDecoration>,
    /// Choose vertical alignment.
    #[serde(rename = "@vertical-align")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<VerticalAlign>,
    /// The prefix.
    #[serde(rename = "@prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// The suffix.
    #[serde(rename = "@suffix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    /// Delimit pieces of the output.
    #[serde(rename = "@delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
    /// Set layout level.
    #[serde(rename = "@display")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
}

to_formatting!(Group, self);
to_affixes!(Group, self);

/// A conditional group of formatting instructions.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Choose {
    /// If branch of the conditional group.
    #[serde(rename = "if")]
    pub if_: ChooseBranch,
    /// Other branches of the conditional group. The first matching branch is used.
    #[serde(rename = "else-if")]
    #[serde(default)]
    pub else_if: Vec<ChooseBranch>,
    /// The formatting instructions to use if no branch matches.
    #[serde(rename = "else")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otherwise: Option<ElseBranch>,
    /// The delimiter between rendering elements in the chosen branch.
    #[serde(rename = "@delimiter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
}

impl Choose {
    /// Return an iterator over all branches with a condition.
    pub fn branches(&self) -> impl Iterator<Item = &ChooseBranch> {
        std::iter::once(&self.if_).chain(self.else_if.iter())
    }

    /// Find the child element that renders the given variable.
    pub fn find_variable_element(
        &self,
        variable: Variable,
        macros: &[CslMacro],
    ) -> Option<LayoutRenderingElement> {
        self.branches()
            .find_map(|b| {
                b.children
                    .iter()
                    .find_map(|c| c.find_variable_element(variable, macros))
            })
            .clone()
    }
}

/// A single branch of a conditional group.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ChooseBranch {
    /// Other than this choose, two elements would result in the same
    /// rendering.
    #[serde(
        rename = "@disambiguate",
        deserialize_with = "deserialize_bool_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disambiguate: Option<bool>,
    /// The variable contains numeric data.
    #[serde(rename = "@is-numeric")]
    /// The variable contains an approximate date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_numeric: Option<Vec<Variable>>,
    /// The variable contains an approximate date.
    #[serde(rename = "@is-uncertain-date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_uncertain_date: Option<Vec<DateVariable>>,
    /// The locator matches the given type.
    #[serde(rename = "@locator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<Vec<Locator>>,
    /// Tests the position of this citation in the citations to the same item.
    /// Only ever true for citations.
    #[serde(rename = "@position")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Vec<TestPosition>>,
    /// Tests whether the item is of a certain type.
    #[serde(rename = "@type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<Vec<Kind>>,
    /// Tests whether the default form of this variable is non-empty.
    #[serde(rename = "@variable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable: Option<Vec<Variable>>,
    /// How to handle the set of tests.
    #[serde(rename = "@match")]
    #[serde(default)]
    pub match_: ChooseMatch,
    #[serde(rename = "$value", default)]
    /// The formatting instructions to use if the condition matches.
    pub children: Vec<LayoutRenderingElement>,
}

impl ChooseBranch {
    /// Retrieve the test of this branch. Valid CSL files must return `Some`
    /// here.
    pub fn test(&self) -> Option<ChooseTest> {
        if let Some(disambiguate) = self.disambiguate {
            if !disambiguate {
                None
            } else {
                Some(ChooseTest::Disambiguate)
            }
        } else if let Some(is_numeric) = &self.is_numeric {
            Some(ChooseTest::IsNumeric(is_numeric))
        } else if let Some(is_uncertain_date) = &self.is_uncertain_date {
            Some(ChooseTest::IsUncertainDate(is_uncertain_date))
        } else if let Some(locator) = &self.locator {
            Some(ChooseTest::Locator(locator))
        } else if let Some(position) = &self.position {
            Some(ChooseTest::Position(position))
        } else if let Some(type_) = &self.type_ {
            Some(ChooseTest::Type(type_))
        } else {
            self.variable.as_ref().map(|variable| ChooseTest::Variable(variable))
        }
    }
}

/// The formatting instructions to use if no branch matches.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct ElseBranch {
    /// The formatting instructions.
    #[serde(rename = "$value")]
    pub children: Vec<LayoutRenderingElement>,
}

/// A single test in a conditional group.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ChooseTest<'a> {
    /// Other than this choose, two elements would result in the same
    /// rendering.
    Disambiguate,
    /// The variable contains numeric data.
    IsNumeric(&'a [Variable]),
    /// The variable contains an approximate date.
    IsUncertainDate(&'a [DateVariable]),
    /// The locator matches the given type.
    Locator(&'a [Locator]),
    /// Tests the position of this citation in the citations to the same item.
    /// Only ever true for citations.
    Position(&'a [TestPosition]),
    /// Tests whether the item is of a certain type.
    Type(&'a [Kind]),
    /// Tests whether the default form of this variable is non-empty.
    Variable(&'a [Variable]),
}

/// Possible positions of a citation in the citations to the same item.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestPosition {
    /// The first citation to the item.
    First,
    /// Previously cited.
    Subsequent,
    /// Directly following a citation to the same item but the locators don't necessarily match.
    IbidWithLocator,
    /// Directly following a citation to the same item with the same locators.
    Ibid,
    /// Other citation within `near-note-distance` of the same item.
    NearNote,
}

/// How to handle the set of tests in a conditional group.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChooseMatch {
    /// All tests must match.
    #[default]
    All,
    /// At least one test must match.
    Any,
    /// No test must match.
    None,
}

impl ChooseMatch {
    /// Check whether the iterator of tests is true for this match type.
    pub fn test(self, mut tests: impl Iterator<Item = bool>) -> bool {
        match self {
            Self::All => tests.all(|t| t),
            Self::Any => tests.any(|t| t),
            Self::None => tests.all(|t| !t),
        }
    }
}

/// A reusable set of formatting instructions.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct CslMacro {
    /// The name of the macro.
    #[serde(rename = "@name")]
    pub name: String,
    /// The formatting instructions.
    #[serde(rename = "$value")]
    #[serde(default)]
    pub children: Vec<LayoutRenderingElement>,
}

/// Root element of a locale file.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct LocaleFile {
    /// The version of the locale file.
    #[serde(rename = "@version")]
    pub version: String,
    /// Which languages or dialects this data applies to.
    #[serde(rename = "@lang")]
    pub lang: LocaleCode,
    /// Metadata of the locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<LocaleInfo>,
    /// The terms used in the locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<Terms>,
    /// How to format dates in the locale file.
    #[serde(default)]
    pub date: Vec<Date>,
    /// Style options for the locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_options: Option<LocaleOptions>,
}

impl LocaleFile {
    /// Create a locale from an XML string.
    pub fn from_xml(xml: &str) -> XmlResult<Self> {
        let locale: Self = quick_xml::de::from_str(xml)?;
        Ok(locale)
    }

    /// Write the locale to an XML string.
    pub fn to_xml(&self) -> XmlResult<String> {
        let mut buf = String::new();
        let ser = quick_xml::se::Serializer::with_root(&mut buf, Some("style"))?;
        self.serialize(ser)?;
        Ok(buf)
    }
}

/// Supplemental localization data in a citation style.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Locale {
    /// Which languages or dialects this data applies to. Must be `Some` if this
    /// appears in a locale file.
    #[serde(rename = "@lang")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<LocaleCode>,
    /// Metadata of the locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<LocaleInfo>,
    /// The terms used in the locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<Terms>,
    /// How to format dates in the locale file.
    #[serde(default)]
    pub date: Vec<Date>,
    /// Style options for the locale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_options: Option<LocaleOptions>,
}

impl Locale {
    /// Get a term translation.
    pub fn term(&self, term: Term, form: TermForm) -> Option<&LocalizedTerm> {
        self.terms.as_ref().and_then(|terms| {
            terms
                .terms
                .iter()
                .find(|t| t.name.is_lexically_same(term) && t.form == form)
        })
    }

    /// Retrieve a struct for ordinal term lookups if this locale contains any
    /// ordinal terms.
    pub fn ordinals(&self) -> Option<OrdinalLookup<'_>> {
        self.terms.as_ref().and_then(|terms| {
            terms.terms.iter().any(|t| t.name.is_ordinal()).then(|| {
                OrdinalLookup::new(terms.terms.iter().filter(|t| t.name.is_ordinal()))
            })
        })
    }

    /// Create a locale from a CBOR file.
    #[cfg(feature = "ciborium")]
    pub fn from_cbor(reader: &[u8]) -> Result<Self, CborDeserializeError> {
        ciborium::de::from_reader(reader)
    }

    /// Write the locale to a CBOR file.
    #[cfg(feature = "ciborium")]
    pub fn to_cbor(&self) -> Result<Vec<u8>, CborSerializeError> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(self, &mut buf)?;
        Ok(buf)
    }
}

/// Get the right forms of ordinal terms for numbers.
pub struct OrdinalLookup<'a> {
    terms: Vec<&'a LocalizedTerm>,
    legacy_behavior: bool,
}

impl<'a> OrdinalLookup<'a> {
    fn new(ordinal_terms: impl Iterator<Item = &'a LocalizedTerm>) -> Self {
        let terms = ordinal_terms.collect::<Vec<_>>();
        let mut legacy_behavior = false;
        // Must not define "OtherTerm::Ordinal"
        let defines_ordinal =
            terms.iter().any(|t| t.name == Term::Other(OtherTerm::Ordinal));

        if !defines_ordinal {
            // Contains OtherTerm::OrdinalN(1) - OtherTerm::OrdinalN(4)
            legacy_behavior = (1..=4).all(|n| {
                terms.iter().any(|t| t.name == Term::Other(OtherTerm::OrdinalN(n)))
            })
        }

        Self { terms, legacy_behavior }
    }

    /// Create an empty lookup that will never return matches.
    pub const fn empty() -> Self {
        Self { terms: Vec::new(), legacy_behavior: false }
    }

    /// Look up a short ordinal for a number.
    pub fn lookup(&self, n: i32, gender: Option<GrammarGender>) -> Option<&'a str> {
        let mut best_match: Option<&'a LocalizedTerm> = None;

        // Prefer match with o > 9 and the smallest difference to n
        let mut change_match = |other_match: &'a LocalizedTerm| {
            let Some(current) = best_match else {
                best_match = Some(other_match);
                return;
            };

            // Extract the number from the term name.
            let Term::Other(OtherTerm::OrdinalN(other_n)) = other_match.name else {
                return;
            };

            let Term::Other(OtherTerm::OrdinalN(curr_n)) = current.name else {
                best_match = Some(other_match);
                return;
            };

            best_match = Some(if other_n >= 10 && curr_n < 10 {
                other_match
            } else if other_n < 10 && curr_n >= 10 {
                current
            } else {
                // Both matches are either < 10 or >= 10.
                // Check the gender form.
                if gender == current.gender && gender != other_match.gender {
                    current
                } else if gender != current.gender && gender == other_match.gender {
                    other_match
                } else {
                    // Choose the smallest difference.
                    let diff_other = (n - other_n as i32).abs();
                    let diff_curr = (n - curr_n as i32).abs();

                    if diff_other <= diff_curr {
                        other_match
                    } else {
                        current
                    }
                }
            })
        };

        for term in self.terms.iter().copied() {
            let Term::Other(term_name) = term.name else { continue };

            let hit = match term_name {
                OtherTerm::Ordinal => true,
                OtherTerm::OrdinalN(o) if self.legacy_behavior => {
                    let class = match (n, n % 10) {
                        (11..=13, _) => 4,
                        (_, v @ 1..=3) => v as u8,
                        _ => 4,
                    };
                    o == class
                }
                OtherTerm::OrdinalN(o @ 0..=9) => match term.match_ {
                    Some(OrdinalMatch::LastDigit) | None => n % 10 == o as i32,
                    Some(OrdinalMatch::LastTwoDigits) => n % 100 == o as i32,
                    Some(OrdinalMatch::WholeNumber) => n == o as i32,
                },
                OtherTerm::OrdinalN(o @ 10..=99) => match term.match_ {
                    Some(OrdinalMatch::LastTwoDigits) | None => n % 100 == o as i32,
                    Some(OrdinalMatch::WholeNumber) => n == o as i32,
                    _ => false,
                },
                _ => false,
            };

            if hit {
                change_match(term);
            }
        }

        best_match.and_then(|t| t.single().or_else(|| t.multiple()))
    }

    /// Look up a long ordinal for a number. Does not include fallback to
    /// regular ordinals.
    pub fn lookup_long(&self, n: i32) -> Option<&'a str> {
        self.terms
            .iter()
            .find(|t| {
                let Term::Other(OtherTerm::LongOrdinal(o)) = t.name else { return false };
                if n > 0 && n <= 10 {
                    n == o as i32
                } else {
                    match t.match_ {
                        Some(OrdinalMatch::LastTwoDigits) | None => n % 100 == o as i32,
                        Some(OrdinalMatch::WholeNumber) => n == o as i32,
                        _ => false,
                    }
                }
            })
            .and_then(|t| t.single().or_else(|| t.multiple()))
    }
}

impl From<LocaleFile> for Locale {
    fn from(file: LocaleFile) -> Self {
        Self {
            lang: Some(file.lang),
            info: file.info,
            terms: file.terms,
            date: file.date,
            style_options: file.style_options,
        }
    }
}

impl TryFrom<Locale> for LocaleFile {
    type Error = ();

    fn try_from(value: Locale) -> Result<Self, Self::Error> {
        if value.lang.is_some() {
            Ok(Self {
                version: "1.0".to_string(),
                lang: value.lang.unwrap(),
                info: value.info,
                terms: value.terms,
                date: value.date,
                style_options: value.style_options,
            })
        } else {
            Err(())
        }
    }
}

/// Metadata of a locale.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocaleInfo {
    /// The translators of the locale.
    #[serde(rename = "translator")]
    #[serde(default)]
    pub translators: Vec<StyleAttribution>,
    /// The license under which the locale is published.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rights: Option<License>,
    /// When the locale was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<Timestamp>,
}

/// Term localization container.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Terms {
    /// The terms.
    #[serde(rename = "term")]
    pub terms: Vec<LocalizedTerm>,
}

/// A localized term.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocalizedTerm {
    /// The term key.
    #[serde(rename = "@name")]
    pub name: Term,
    /// The localization.
    #[serde(rename = "$text")]
    #[serde(skip_serializing_if = "Option::is_none")]
    localization: Option<String>,
    /// The singular variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    single: Option<String>,
    /// The plural variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    multiple: Option<String>,
    /// The variant of this term translation.
    #[serde(rename = "@form", default)]
    pub form: TermForm,
    /// Specify the when this ordinal term is used.
    #[serde(rename = "@match")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_: Option<OrdinalMatch>,
    /// Specify for which grammatical gender this term has to get corresponding ordinals
    #[serde(rename = "@gender")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<GrammarGender>,
    /// Specify which grammatical gender this ordinal term matches
    #[serde(rename = "@gender-form")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_form: Option<GrammarGender>,
}

impl LocalizedTerm {
    /// Get the singular variant of this term translation. Shall be defined for
    /// valid CSL files.
    pub fn single(&self) -> Option<&str> {
        self.single.as_deref().or(self.localization.as_deref())
    }

    /// Get the plural variant of this term translation. Shall be defined for
    /// valid CSL files.
    pub fn multiple(&self) -> Option<&str> {
        self.multiple.as_deref().or(self.localization.as_deref())
    }
}

/// The variant of a term translation.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TermForm {
    /// The default variant.
    #[default]
    Long,
    /// The short noun variant.
    Short,
    /// The related verb.
    Verb,
    /// The related verb (short form).
    VerbShort,
    /// The symbol variant.
    Symbol,
}

impl TermForm {
    /// Which form is the next fallback if this form is not available.
    pub const fn fallback(self) -> Option<Self> {
        match self {
            Self::Long => None,
            Self::Short => Some(Self::Long),
            Self::Verb => Some(Self::Long),
            Self::VerbShort => Some(Self::Verb),
            Self::Symbol => Some(Self::Short),
        }
    }
}

/// Specify when which ordinal term is used.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrdinalMatch {
    /// Match the last digit for ordinal terms between zero and nine and the
    /// last two otherwise.
    #[default]
    LastDigit,
    /// Always match on the last two non-zero digits.
    LastTwoDigits,
    /// Match on the exact number.
    WholeNumber,
}

/// A grammatical gender. Use `None` for neutral.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrammarGender {
    Feminine,
    Masculine,
}

/// Options for the locale.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocaleOptions {
    /// Only use ordinals for the first day in a month.
    ///
    /// Default: `false`
    #[serde(
        rename = "@limit-day-ordinals-to-day-1",
        deserialize_with = "deserialize_bool_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_day_ordinals_to_day_1: Option<bool>,
    /// Whether to place punctuation inside of quotation marks.
    ///
    /// Default: `false`
    #[serde(
        rename = "@punctuation-in-quote",
        deserialize_with = "deserialize_bool_option",
        default
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub punctuation_in_quote: Option<bool>,
}

/// Formatting properties.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Formatting {
    /// Set the font style.
    #[serde(rename = "@font-style")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<FontStyle>,
    /// Choose normal or small caps.
    #[serde(rename = "@font-variant")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_variant: Option<FontVariant>,
    /// Set the font weight.
    #[serde(rename = "@font-weight")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<FontWeight>,
    /// Choose underlining.
    #[serde(rename = "@text-decoration")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<TextDecoration>,
    /// Choose vertical alignment.
    #[serde(rename = "@vertical-align")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<VerticalAlign>,
}

impl Formatting {
    /// Check if this formatting is empty.
    pub fn is_empty(&self) -> bool {
        self.font_style.is_none()
            && self.font_variant.is_none()
            && self.font_weight.is_none()
            && self.text_decoration.is_none()
            && self.vertical_align.is_none()
    }

    /// Merge with a base formatting.
    pub fn apply(self, base: Self) -> Self {
        Self {
            font_style: self.font_style.or(base.font_style),
            font_variant: self.font_variant.or(base.font_variant),
            font_weight: self.font_weight.or(base.font_weight),
            text_decoration: self.text_decoration.or(base.text_decoration),
            vertical_align: self.vertical_align.or(base.vertical_align),
        }
    }
}

/// Font style.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    /// Normal font style.
    #[default]
    Normal,
    /// Italic font style.
    Italic,
}

/// Font variant.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FontVariant {
    /// Normal font variant.
    #[default]
    Normal,
    /// Small caps font variant.
    SmallCaps,
}

/// Font weight.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FontWeight {
    /// Normal font weight.
    #[default]
    Normal,
    /// Bold font weight.
    Bold,
    /// Light font weight.
    Light,
}

/// Text decoration.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextDecoration {
    /// No text decoration.
    #[default]
    None,
    /// Underline text decoration.
    Underline,
}

/// Vertical alignment.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VerticalAlign {
    /// No vertical alignment.
    #[default]
    #[serde(rename = "")]
    None,
    /// Align on the baseline.
    Baseline,
    /// Superscript vertical alignment.
    Sup,
    /// Subscript vertical alignment.
    Sub,
}

/// Prefixes and suffixes.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Affixes {
    /// The prefix.
    #[serde(rename = "@prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// The suffix.
    #[serde(rename = "@suffix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

/// On which layout level to display the citation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Display {
    /// Block stretching from margin to margin.
    Block,
    /// Put in the left margin.
    LeftMargin,
    /// Align on page after `LeftMargin`.
    RightInline,
    /// `Block` and indented.
    Indent,
}

/// How to format text.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextCase {
    /// lowecase.
    Lowercase,
    /// UPPERCASE.
    Uppercase,
    /// Capitalize the first word.
    CapitalizeFirst,
    /// Capitalize All Words.
    CapitalizeAll,
    /// Sentence case. *Deprecated*.
    #[serde(rename = "sentence")]
    SentenceCase,
    /// Title case. Only applies to English.
    #[serde(rename = "title")]
    TitleCase,
}

impl TextCase {
    /// Check whether this case can be applied to languages other than English.
    pub fn is_language_independent(self) -> bool {
        match self {
            Self::Lowercase
            | Self::Uppercase
            | Self::CapitalizeFirst
            | Self::CapitalizeAll => true,
            Self::SentenceCase | Self::TitleCase => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::de::DeserializeOwned;
    use std::{error::Error, fs};

    fn folder<F>(
        files: &'static str,
        extension: &'static str,
        kind: &'static str,
        mut check: F,
    ) where
        F: FnMut(&str) -> Option<Box<dyn Error>>,
    {
        let mut failures = 0;
        let mut tests = 0;

        // Read each `.csl` file in the `tests` directory.
        for entry in fs::read_dir(files).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map(|os| os.to_str().unwrap()) != Some(extension)
                || !entry.file_type().unwrap().is_file()
            {
                continue;
            }

            tests += 1;

            let source = fs::read_to_string(&path).unwrap();
            let result = check(&source);
            if let Some(err) = result {
                failures += 1;
                println!("❌ {:?} failed: \n\n{:#?}", &path, &err);
            }
        }

        if failures == 0 {
            print!("\n🎉")
        } else {
            print!("\n😢")
        }

        println!(
            " {} out of {} {} files parsed successfully",
            tests - failures,
            tests,
            kind
        );

        if failures > 0 {
            panic!("{} tests failed", failures);
        }
    }

    fn check_style(csl_files: &'static str, kind: &'static str) {
        folder(csl_files, "csl", kind, |source| {
            let de = &mut deserializer(source);
            let result: Result<RawStyle, _> = serde_path_to_error::deserialize(de);
            match result {
                Ok(_) => None,
                Err(err) => Some(Box::new(err)),
            }
        })
    }

    fn check_locale(locale_files: &'static str) {
        folder(locale_files, "xml", "Locale", |source| {
            let de = &mut deserializer(source);
            let result: Result<LocaleFile, _> = serde_path_to_error::deserialize(de);
            match result {
                Ok(_) => None,
                Err(err) => Some(Box::new(err)),
            }
        })
    }

    #[track_caller]
    fn to_cbor<T: Serialize>(style: &T) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(style, &mut buf).unwrap();
        buf
    }

    #[track_caller]
    fn from_cbor<T: DeserializeOwned>(reader: &[u8]) -> T {
        ciborium::de::from_reader(reader).unwrap()
    }

    #[test]
    fn test_independent() {
        check_style("tests/independent", "independent CSL style");
    }

    #[test]
    fn test_dependent() {
        check_style("tests/dependent", "dependent CSL style");
    }

    #[test]
    fn test_locale() {
        check_locale("tests/locales");
    }

    /// Be sure to check out the CSL
    /// [styles](https://github.com/citation-style-language/styles) repository
    /// into a sibling folder to run this test.
    #[test]
    fn roundtrip_cbor_all() {
        fs::create_dir_all("tests/artifacts/styles").unwrap();
        for style_thing in
            fs::read_dir("../styles/").expect("please check out the CSL styles repo")
        {
            let thing = style_thing.unwrap();
            if thing.file_type().unwrap().is_dir() {
                continue;
            }

            let path = thing.path();
            let extension = path.extension();
            if let Some(extension) = extension {
                if extension.to_str() != Some("csl") {
                    continue;
                }
            } else {
                continue;
            }

            eprintln!("Testing {}", path.display());
            let source = fs::read_to_string(&path).unwrap();
            let style = Style::from_xml(&source).unwrap();
            let cbor = to_cbor(&style);
            fs::write(
                format!(
                    "tests/artifacts/styles/{}.cbor",
                    path.file_stem().unwrap().to_str().unwrap()
                ),
                &cbor,
            )
            .unwrap();
            let style2 = from_cbor(&cbor);
            assert_eq!(style, style2);
        }
    }

    /// Be sure to check out the CSL
    /// [locales](https://github.com/citation-style-language/locales) repository
    /// into a sibling folder to run this test.
    #[test]
    fn roundtrip_cbor_all_locales() {
        fs::create_dir_all("tests/artifacts/locales").unwrap();
        for style_thing in
            fs::read_dir("../locales/").expect("please check out the CSL locales repo")
        {
            let thing = style_thing.unwrap();
            if thing.file_type().unwrap().is_dir() {
                continue;
            }

            let path = thing.path();
            let extension = path.extension();
            if let Some(extension) = extension {
                if extension.to_str() != Some("xml")
                    || !path
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("locales-")
                {
                    continue;
                }
            } else {
                continue;
            }

            eprintln!("Testing {}", path.display());
            let source = fs::read_to_string(&path).unwrap();
            let locale = LocaleFile::from_xml(&source).unwrap();
            let cbor = to_cbor(&locale);
            fs::write(
                format!(
                    "tests/artifacts/locales/{}.cbor",
                    path.file_stem().unwrap().to_str().unwrap()
                ),
                &cbor,
            )
            .unwrap();
            let locale2 = from_cbor(&cbor);
            assert_eq!(locale, locale2);
        }
    }

    #[test]
    fn test_expand() {
        assert_eq!(expand(103, 4), 104);
        assert_eq!(expand(133, 4), 134);
        assert_eq!(expand(133, 54), 154);
        assert_eq!(expand(100, 4), 104);
    }

    #[test]
    fn page_range() {
        fn run(format: PageRangeFormat, start: i32, end: i32) -> String {
            let mut buf = String::new();
            format.format(start..end, &mut buf, None).unwrap();
            buf
        }

        let c15 = PageRangeFormat::Chicago15;
        let c16 = PageRangeFormat::Chicago16;
        let exp = PageRangeFormat::Expanded;
        let min = PageRangeFormat::Minimal;
        let mi2 = PageRangeFormat::MinimalTwo;

        // https://docs.citationstyles.org/en/stable/specification.html#appendix-v-page-range-formats

        assert_eq!("1–8", run(c16, 1, 8));
        assert_eq!("3–10", run(c15, 3, 10));
        assert_eq!("71–72", run(c15, 71, 72));
        assert_eq!("100–104", run(c15, 100, 4));
        assert_eq!("600–613", run(c15, 600, 613));
        assert_eq!("1100–1123", run(c15, 1100, 1123));
        assert_eq!("107–8", run(c15, 107, 108));
        assert_eq!("505–17", run(c15, 505, 517));
        assert_eq!("1002–6", run(c15, 1002, 1006));
        assert_eq!("321–25", run(c15, 321, 325));
        assert_eq!("415–532", run(c15, 415, 532));
        assert_eq!("11564–68", run(c15, 11564, 11568));
        assert_eq!("13792–803", run(c15, 13792, 13803));
        assert_eq!("1496–1504", run(c15, 1496, 1504));
        assert_eq!("2787–2816", run(c15, 2787, 2816));

        assert_eq!("3–10", run(c16, 3, 10));
        assert_eq!("71–72", run(c16, 71, 72));
        assert_eq!("92–113", run(c16, 92, 113));
        assert_eq!("100–104", run(c16, 100, 4));
        assert_eq!("600–613", run(c16, 600, 613));
        assert_eq!("1100–1123", run(c16, 1100, 1123));
        assert_eq!("107–8", run(c16, 107, 108));
        assert_eq!("505–17", run(c16, 505, 517));
        assert_eq!("1002–6", run(c16, 1002, 1006));
        assert_eq!("321–25", run(c16, 321, 325));
        assert_eq!("415–532", run(c16, 415, 532));
        assert_eq!("1087–89", run(c16, 1087, 1089));
        assert_eq!("1496–500", run(c16, 1496, 1500));
        assert_eq!("11564–68", run(c16, 11564, 11568));
        assert_eq!("13792–803", run(c16, 13792, 13803));
        assert_eq!("12991–3001", run(c16, 12991, 13001));

        assert_eq!("42–45", run(exp, 42, 45));
        assert_eq!("321–328", run(exp, 321, 328));
        assert_eq!("2787–2816", run(exp, 2787, 2816));

        assert_eq!("42–5", run(min, 42, 45));
        assert_eq!("321–8", run(min, 321, 328));
        assert_eq!("2787–816", run(min, 2787, 2816));

        assert_eq!("7–8", run(mi2, 7, 8));
        assert_eq!("42–45", run(mi2, 42, 45));
        assert_eq!("321–28", run(mi2, 321, 328));
        assert_eq!("2787–816", run(mi2, 2787, 2816));
    }
}
