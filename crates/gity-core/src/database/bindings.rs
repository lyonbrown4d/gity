use sea_orm::Value;
use std::sync::LazyLock;

/// Preserve legacy SQL schema types when parameters are supplied through JSON.
/// Only named timestamp columns are converted; arbitrary text is never inspected
/// as a date merely because its contents resemble one.
pub(super) fn postgres_values(sql: &str, values: Vec<Value>) -> Vec<Value> {
    static INSERT: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"(?is)^\s*INSERT\s+INTO\s+[\w\"`]+\s*\(([^)]+)\)\s*VALUES\s*\(([^)]+)\)"#,
        )
        .expect("constant regex")
    });
    static FIELD: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r#"([a-zA-Z_][a-zA-Z0-9_]*)[\"`]?\s*(?:=|<=|>=|<>|<|>)\s*$"#)
            .expect("constant regex")
    });
    let inserted = INSERT.captures(sql).map(|c| {
        c[1].split(',')
            .zip(c[2].split(','))
            .filter(|(_, expression)| expression.trim() == "?")
            .map(|(v, _)| v.trim().trim_matches(['\"', '`']).to_owned())
            .collect::<Vec<_>>()
    });
    let positions = sql.match_indices('?').map(|(i, _)| i).collect::<Vec<_>>();
    values
        .into_iter()
        .enumerate()
        .map(|(i, value)| {
            if let Value::Bool(Some(value)) = value {
                return Value::BigInt(Some(i64::from(value)));
            }
            let column = inserted
                .as_ref()
                .and_then(|v| v.get(i))
                .cloned()
                .or_else(|| {
                    positions
                        .get(i)
                        .and_then(|index| FIELD.captures(&sql[..*index]).map(|c| c[1].to_owned()))
                });
            if column.as_deref().is_some_and(timestamp_column) {
                if matches!(&value, Value::String(None)) {
                    return Value::ChronoDateTime(None);
                }
                if let Value::String(Some(text)) = &value {
                    let time = chrono::DateTime::parse_from_rfc3339(text)
                        .map(|v| v.naive_utc())
                        .ok()
                        .or_else(|| {
                            ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"]
                                .into_iter()
                                .find_map(|format| {
                                    chrono::NaiveDateTime::parse_from_str(text, format).ok()
                                })
                        });
                    if let Some(time) = time {
                        return Value::ChronoDateTime(Some(time));
                    }
                }
            }
            value
        })
        .collect()
}
fn timestamp_column(name: &str) -> bool {
    [
        "created_at",
        "updated_at",
        "deleted_at",
        "expires_at",
        "revoked_at",
        "last_used_at",
        "last_contact_at",
        "run_after",
        "locked_until",
        "started_at",
        "finished_at",
        "done_at",
        "released_at",
    ]
    .contains(&name)
}

#[cfg(test)]
mod tests;
