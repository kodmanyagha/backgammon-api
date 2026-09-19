macro_rules! set_opt_str_field {
    ($model:expr, $map:expr, $field:ident) => {
        if let Some(v) = $map.get(stringify!($field)) {
            if v.is_null() {
                $model.$field = sea_orm::ActiveValue::Set(None);
            } else if let Some(s) = v.as_str() {
                $model.$field = sea_orm::ActiveValue::Set(Some(s.to_string()));
            }
        }
    };
}
pub(crate) use set_opt_str_field;

macro_rules! set_str_field {
    ($model:expr, $map:expr, $field:ident) => {
        if let Some(v) = $map.get(stringify!($field)) {
            if let Some(s) = v.as_str() {
                $model.$field = sea_orm::ActiveValue::Set(s.into());
            }
        }
    };
}
pub(crate) use set_str_field;

macro_rules! set_opt_i32_field {
    ($model:expr, $map:expr, $field:ident) => {
        if let Some(v) = $map.get(stringify!($field)) {
            if v.is_null() {
                $model.$field = sea_orm::ActiveValue::Set(None);
            } else if let Some(s) = v.as_i32() {
                $model.$field = sea_orm::ActiveValue::Set(Some(s.into()));
            }
        }
    };
}
pub(crate) use set_opt_i32_field;

macro_rules! set_opt_u64_field {
    ($model:expr, $map:expr, $field:ident) => {
        if let Some(v) = $map.get(stringify!($field)) {
            if v.is_null() {
                $model.$field = sea_orm::ActiveValue::Set(None);
            } else if let Some(s) = v.as_u64() {
                $model.$field = sea_orm::ActiveValue::Set(Some(s.into()));
            }
        }
    };
}
pub(crate) use set_opt_u64_field;
