//! Parser for CSL-JSON.
//!
//! This is only available when the `json` feature is enabled.

use std::borrow::Cow;
use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};
use unscanny::Scanner;

/// A CSL-JSON item.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct Item(pub BTreeMap<String, Value>);

impl Item {
    /// The item's ID.
    pub fn id(&self) -> Option<Cow<str>> {
        self.0.get("id")?.to_str()
    }

    /// The item type.
    pub fn type_(&self) -> Option<Cow<str>> {
        self.0.get("type")?.to_str()
    }

    /// Whether any of the fields values contains any HTML.
    pub fn has_html(&self) -> bool {
        self.0.values().any(|v| v.has_html())
    }

    /// Whether this entry may contain "cheater syntax" for odd fields.
    pub fn may_have_hack(&self) -> bool {
        self.0.contains_key("note")
    }
}

/// A field in an CSL-JSON item.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum Value {
    /// A string value.
    String(String),
    /// A number value.
    Number(i64),
    /// A list of names.
    Names(Vec<NameValue>),
    /// A date value.
    Date(DateValue),
}

impl Value {
    /// Convert to a string if this is a string or number.
    pub fn to_str(&self) -> Option<Cow<str>> {
        match self {
            Value::String(s) => Some(s.as_str().into()),
            Value::Number(n) => Some(n.to_string().into()),
            Value::Date(_) => None,
            Value::Names(_) => None,
        }
    }

    /// Whether the value contains any HTML.
    pub fn has_html(&self) -> bool {
        match self {
            Value::String(s) => s.contains('<'),
            Value::Number(_) => false,
            Value::Date(_) => false,
            Value::Names(_) => false,
        }
    }
}

/// The representation of a name.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum NameValue {
    /// A name that doesn't necessarily follow the schema of a `NameItem`.
    Literal(LiteralName),
    /// A name that is defined by a collection of parts.
    Item(NameItem),
}

/// A name that is defined by a collection of parts.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct NameItem {
    /// The family name.
    #[serde(default)]
    pub family: String,
    /// The given name.
    pub given: Option<String>,
    /// A name particle like `"de las"`.
    pub non_dropping_particle: Option<String>,
    /// A name particle like `"Rev."`.
    pub dropping_particle: Option<String>,
    /// A name suffix like `"Jr., Ph.D."`.
    pub suffix: Option<String>,
}

/// A name that doesn't necessarily follow the schema of a `NameItem`. May be
/// useful for institutional names.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct LiteralName {
    /// The literal name.
    pub literal: String,
}

/// The representation of a date.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum DateValue {
    Raw {
        raw: FixedDateRange,
        literal: Option<String>,
        season: Option<String>,
    },
    DateParts {
        date_parts: VecDateRange,
        literal: Option<String>,
        season: Option<String>,
    },
}

impl TryFrom<DateValue> for FixedDateRange {
    type Error = ();

    fn try_from(value: DateValue) -> Result<Self, Self::Error> {
        match value {
            DateValue::Raw { raw, .. } => Ok(raw),
            DateValue::DateParts { date_parts, .. } => date_parts.try_into(),
        }
    }
}

impl From<DateValue> for VecDateRange {
    fn from(value: DateValue) -> Self {
        match value {
            DateValue::Raw { raw, .. } => raw.into(),
            DateValue::DateParts { date_parts, .. } => date_parts,
        }
    }
}

impl<'de> Deserialize<'de> for DateValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case", untagged)]
        enum DateReprRaw {
            Raw {
                raw: FixedDateRange,
                literal: Option<String>,
                season: Option<NumberOrString>,
            },
            DateParts {
                #[serde(rename = "date-parts")]
                date_parts: VecDateRange,
                literal: Option<String>,
                season: Option<NumberOrString>,
            },
        }

        let raw = DateReprRaw::deserialize(deserializer)?;
        Ok(match raw {
            DateReprRaw::Raw { raw, literal, season } => DateValue::Raw {
                raw,
                literal,
                season: season.map(NumberOrString::into_string),
            },
            DateReprRaw::DateParts { date_parts, literal, season } => {
                DateValue::DateParts {
                    date_parts,
                    literal,
                    season: season.map(NumberOrString::into_string),
                }
            }
        })
    }
}

impl Serialize for DateValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DateValue::Raw { raw, .. } => VecDateRange::from(*raw).serialize(serializer),
            DateValue::DateParts { date_parts, .. } => date_parts.serialize(serializer),
        }
    }
}

/// A range of dates defined by arbitrary sequences of integer components.
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct VecDateRange(pub Vec<VecDate>);

impl From<FixedDateRange> for VecDateRange {
    fn from(value: FixedDateRange) -> Self {
        let mut v = Vec::new();
        v.push(value.start.into());
        if let Some(end) = value.end {
            v.push(end.into());
        }
        VecDateRange(v)
    }
}

/// A date defined by an arbitrary sequence integer components.
#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct VecDate(pub Vec<i16>);

impl From<FixedDate> for VecDate {
    fn from(value: FixedDate) -> Self {
        let mut v = Vec::new();
        v.push(value.year);
        if let Some(month) = value.month {
            v.push(month as i16);
            if let Some(day) = value.day {
                v.push(day as i16);
            }
        }
        VecDate(v)
    }
}

impl<'de> Deserialize<'de> for VecDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = Vec::<NumberOrString>::deserialize(deserializer)?;
        Ok(VecDate(
            v.into_iter()
                .filter_map(|v| match v {
                    NumberOrString::Number(n) => Some(Ok(n)),
                    NumberOrString::String(s) if s.is_empty() => None,
                    NumberOrString::String(s) => Some(s.parse().map_err(|_| {
                        serde::de::Error::custom(format!("invalid number: {}", s))
                    })),
                })
                .collect::<Result<_, _>>()?,
        ))
    }
}

/// A range of dates defined by fixed components.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FixedDateRange {
    /// The start of the range.
    pub start: FixedDate,
    /// The optional end of the range.
    pub end: Option<FixedDate>,
}

impl TryFrom<VecDateRange> for FixedDateRange {
    type Error = ();

    fn try_from(value: VecDateRange) -> Result<Self, Self::Error> {
        let mut v = value.0.into_iter();
        let start = v.next().ok_or(())?.into();
        let end = v.next().map(|v| v.into());
        if v.next().is_some() {
            return Err(());
        }
        Ok(FixedDateRange { start, end })
    }
}

impl FromStr for FixedDateRange {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = Scanner::new(s);
        let start = parse_date(&mut s).ok_or(())?;
        let end =
            if s.eat() == Some('/') { Some(parse_date(&mut s).ok_or(())?) } else { None };

        Ok(FixedDateRange { start, end })
    }
}

impl<'de> Deserialize<'de> for FixedDateRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| serde::de::Error::custom("invalid date"))
    }
}

/// A date defined by fixed components.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct FixedDate {
    pub year: i16,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl From<VecDate> for FixedDate {
    fn from(value: VecDate) -> Self {
        let mut v = value.0.into_iter();
        let year = v.next().unwrap();
        let month = v.next().map(|v| (v - 1) as u8);
        let day = v.next().map(|v| (v - 1) as u8);
        FixedDate { year, month, day }
    }
}

impl FromStr for FixedDate {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = Scanner::new(s);
        parse_date(&mut s).ok_or(())
    }
}

impl<'de> Deserialize<'de> for FixedDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| serde::de::Error::custom("invalid date"))
    }
}

fn parse_date(s: &mut Scanner<'_>) -> Option<FixedDate> {
    let year = s.eat_while(char::is_ascii_digit);
    let year = year.parse().ok()?;
    if s.peek() != Some('-') {
        return Some(FixedDate { year, month: None, day: None });
    }
    s.eat();

    let month = s.eat_while(char::is_ascii_digit);
    let month = month.parse::<u8>().ok()? - 1;
    if month > 11 {
        return None;
    }

    if s.peek() != Some('-') {
        return Some(FixedDate { year, month: Some(month), day: None });
    }
    s.eat();

    let day = s.eat_while(char::is_ascii_digit);
    let day = day.parse::<u8>().ok()? - 1;
    if day > 31 {
        return None;
    }

    Some(FixedDate { year, month: Some(month), day: Some(day) })
}

/// A CSL-JSON citation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    /// A unique ID for the citation.
    pub citation_id: String,
    /// The individual parts of the citation.
    pub citation_items: Vec<CitationItem>,
    /// The citation's properties.
    pub properties: CitationProperties,
}

/// An individual part of a citation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CitationItem {
    /// A unique ID for the citation item.
    pub id: String,
    /// A locator value (e.g. a page number).
    pub locator: Option<String>,
    /// What kind of locator to use (e.g. `"page"`).
    pub label: Option<String>,
    /// Whether to suppress the author for this item.
    #[serde(default)]
    pub suppress_author: bool,
    /// Something to print before this item.
    pub prefix: Option<String>,
    /// Something to print after this item.
    pub suffix: Option<String>,
    /// Defines the relationship of this item to other cited items with the same
    /// key.
    pub position: Option<u8>,
    /// Whether this key was already cited in close range before.
    pub near_note: Option<bool>,
}

impl<'de> Deserialize<'de> for CitationItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct CitationItemRaw {
            id: NumberOrString,
            locator: Option<NumberOrString>,
            label: Option<String>,
            #[serde(default)]
            suppress_author: bool,
            prefix: Option<String>,
            suffix: Option<String>,
            position: Option<u8>,
            near_note: Option<bool>,
        }

        let raw = CitationItemRaw::deserialize(deserializer)?;
        Ok(CitationItem {
            id: raw.id.into_string(),
            locator: raw.locator.map(NumberOrString::into_string),
            label: raw.label,
            suppress_author: raw.suppress_author,
            prefix: raw.prefix,
            suffix: raw.suffix,
            position: raw.position,
            near_note: raw.near_note,
        })
    }
}

/// Properties of a citation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationProperties {
    /// The footnote number in which the citation is located in the document.
    note_index: Option<u32>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberOrString {
    Number(i16),
    String(String),
}

impl NumberOrString {
    fn into_string(self) -> String {
        match self {
            NumberOrString::Number(n) => n.to_string(),
            NumberOrString::String(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let mut map = BTreeMap::new();
        map.insert("title".to_string(), Value::String("The Title".to_string()));
        map.insert(
            "author".to_string(),
            Value::Names(vec![NameValue::Item(NameItem {
                family: "Doe".to_string(),
                given: Some("John".to_string()),
                non_dropping_particle: None,
                dropping_particle: None,
                suffix: None,
            })]),
        );
        map.insert(
            "date".to_string(),
            Value::Date(DateValue::Raw {
                raw: FixedDateRange::from_str("2021-09-10/2022-01-01").unwrap(),
                literal: None,
                season: None,
            }),
        );

        let item = Item(map);
        println!("{}", serde_json::to_string_pretty(&item).unwrap());
    }
}
