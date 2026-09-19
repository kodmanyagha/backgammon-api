macro_rules! serde_json_map {
    () => {
        &mut serde_json::Map::new()
    };
}

macro_rules! insert_json_field {
    ($json_val:expr, $field:expr, $val:expr) => {
        $json_val
            .as_object_mut()
            .unwrap_or(serde_json_map!())
            .insert($field.into(), json!($val))
    };
}

pub(crate) use insert_json_field;
pub(crate) use serde_json_map;
