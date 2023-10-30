use serde::Deserialize;

pub fn deserialize_bool<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<bool, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        Bool(bool),
        String(String),
    }

    let deser = StringOrBool::deserialize(deserializer)?;
    Ok(match deser {
        StringOrBool::Bool(b) => b,
        StringOrBool::String(s) => s.to_ascii_lowercase() == "true",
    })
}

pub fn deserialize_bool_option<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<bool>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        Bool(bool),
        String(String),
    }

    let res = Option::<StringOrBool>::deserialize(deserializer)?;
    Ok(res.map(|s| match s {
        StringOrBool::Bool(b) => b,
        StringOrBool::String(s) => s.to_ascii_lowercase() == "true",
    }))
}

pub fn deserialize_u32<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<u32, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrUnsigned {
        Unsigned(u32),
        String(String),
    }

    let res = StringOrUnsigned::deserialize(deserializer)?;
    Ok(match res {
        StringOrUnsigned::Unsigned(u) => u,
        StringOrUnsigned::String(s) => {
            s.trim().parse().map_err(serde::de::Error::custom)?
        }
    })
}

pub fn deserialize_u32_option<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u32>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrUnsigned {
        Unsigned(u32),
        String(String),
    }

    let res = Option::<StringOrUnsigned>::deserialize(deserializer)?;
    res.map(|s| match s {
        StringOrUnsigned::Unsigned(u) => Ok(u),
        StringOrUnsigned::String(s) => s.trim().parse().map_err(serde::de::Error::custom),
    })
    .transpose()
}
