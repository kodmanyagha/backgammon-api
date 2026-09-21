use serde_json::Error;
use serde_json::Value;

pub fn merge_json(obj1: &mut Value, obj2: &Value) {
    if let (Some(map1), Some(map2)) = (obj1.as_object_mut(), obj2.as_object()) {
        for (key, value2) in map2 {
            if let Some(value1) = map1.get_mut(key) {
                if value1.is_object() && value2.is_object() {
                    merge_json(value1, value2);
                } else {
                    *value1 = value2.clone();
                }
            } else {
                map1.insert(key.clone(), value2.clone());
            }
        }
    }
}

fn to_value<T: serde::ser::Serialize>(value: &T) -> Result<serde_json::Value, Error> {
    serde_json::to_value(value)
}

fn from_value<T: serde::ser::Serialize + serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, Error> {
    serde_json::from_value(value)
}

fn merge_value(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_value(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (Value::Array(ref mut a), &Value::Array(ref b)) => {
            a.extend(b.clone());
        }
        (Value::Array(ref mut a), &Value::Object(ref b)) => {
            a.extend([Value::Object(b.clone())]);
        }
        (_, Value::Null) => {}
        (a, b) => {
            *a = b.clone();
        }
    }
}

pub fn merge_struct<T: serde::ser::Serialize + serde::de::DeserializeOwned>(
    base: &T,
    overrides: &T,
) -> Result<Value, Error> {
    let mut left = to_value(base)?;
    let right = to_value(overrides)?;
    merge_value(&mut left, &right);
    from_value(left)
}
