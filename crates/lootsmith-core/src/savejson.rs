//! Wire-faithful JSON conversion for protobuf `Message`s.
//!
//! Every field's order, wire type and duplicate fields are preserved exactly,
//! so `save → JSON → save` round-trips byte-identically:
//!
//! ```json
//! {"lootsmith-protobuf": 1, "fields": [
//!   {"n": 1,  "s": "GD_Soldier.Character.CharClass_Soldier"},
//!   {"n": 2,  "i": 72},
//!   {"n": 18, "m": {"fields": [...] }},
//!   {"n": 34, "b": "<base64>"},
//!   {"n": 21, "f64": "0"} ]}
//! ```
//!
//! A length-delimited value is tagged `"s"` (UTF-8 string), `"b"` (base64
//! bytes) or `"m"` (nested message). The tag is chosen with a readability
//! heuristic on export; rebuilding always reproduces the exact payload bytes
//! regardless of which tag was picked.

use crate::error::{Error, Result};
use crate::proto::{Field, Message, WireType};
use base64::Engine as _;
use serde_json::{json, Value};

const MAGIC: &str = "lootsmith-protobuf";
const BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

/// Serialize a whole save to JSON text (see the module docs for the format).
pub fn save_to_json(save: &crate::savegame::SaveGame) -> String {
    message_to_json(&save.message).to_string()
}

/// Parse JSON text produced by [`save_to_json`] back into a [`SaveGame`].
pub fn save_from_json(text: &str) -> Result<crate::savegame::SaveGame> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| Error::Proto(format!("invalid save JSON: {e}")))?;
    if value.get(MAGIC).and_then(Value::as_i64) != Some(1) {
        return Err(Error::Proto(
            "not a lootsmith save JSON (missing \"lootsmith-protobuf\": 1)".into(),
        ));
    }
    Ok(crate::savegame::SaveGame {
        message: message_from_json(&value)?,
    })
}

/// Convert a parsed message to JSON (see the module docs for the format).
pub fn message_to_json(message: &Message) -> Value {
    let fields: Vec<Value> = message.fields.iter().map(field_to_json).collect();
    json!({ MAGIC: 1, "fields": fields })
}

fn field_to_json(field: &Field) -> Value {
    match field.wire {
        WireType::Varint => json!({ "n": field.number, "i": field.as_varint() }),
        WireType::Fixed64 => json!({ "n": field.number, "f64": field.as_fixed64() }),
        WireType::Fixed32 => json!({ "n": field.number, "f32": field.as_fixed32() }),
        WireType::LengthDelimited => {
            let payload = field.as_bytes().unwrap_or(&[]);
            let mut as_message = field.as_message().ok().flatten();
            if as_message.as_ref().is_some_and(|m| m.fields.is_empty()) {
                as_message = None;
            }
            if let Ok(text) = std::str::from_utf8(payload) {
                if as_message.is_none() {
                    return json!({ "n": field.number, "s": text });
                }
            }
            if let Some(sub) = as_message {
                return json!({ "n": field.number, "m": message_to_json(&sub) });
            }
            let text = BASE64.encode(payload);
            json!({ "n": field.number, "b": text })
        }
        // Start/end group fields are rejected at parse time; unreachable.
        _ => Value::Null,
    }
}

/// Rebuild a message from [`message_to_json`] output.
pub fn message_from_json(value: &Value) -> Result<Message> {
    let mut message = Message::default();
    let Some(fields) = value.get("fields").and_then(Value::as_array) else {
        return Err(Error::Proto("missing \"fields\" array".into()));
    };
    for field in fields {
        let number = field
            .get("n")
            .and_then(Value::as_u64)
            .ok_or_else(|| Error::Proto("field object without \"n\"".into()))?
            as u32;
        if let Some(varint) = field.get("i").and_then(Value::as_u64) {
            message.push(Field::varint(number, varint));
        } else if let Some(fixed) = field.get("f64").and_then(Value::as_u64) {
            message.push(Field::fixed64(number, fixed));
        } else if let Some(fixed) = field.get("f32").and_then(Value::as_u64) {
            message.push(Field::fixed32(number, fixed as u32));
        } else if let Some(text) = field.get("s").and_then(Value::as_str) {
            message.push(Field::bytes(number, text.as_bytes()));
        } else if let Some(b64) = field.get("b").and_then(Value::as_str) {
            let payload = BASE64
                .decode(b64)
                .map_err(|e| Error::Proto(format!("bad base64 field {number}: {e}")))?;
            message.push(Field::bytes(number, &payload));
        } else if let Some(sub) = field.get("m") {
            let sub = message_from_json(sub)?;
            message.push(Field::bytes(number, &sub.to_bytes()));
        } else {
            return Err(Error::Proto(format!(
                "field {number} has no recognised value tag"
            )));
        }
    }
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A message exercising every wire type, order and nesting.
    fn sample() -> Message {
        let mut nested = Message::default();
        nested.push(Field::string(1, "inner"));
        nested.push(Field::varint(2, 5));
        let mut message = Message::default();
        message.push(Field::varint(3, 0)); // defaults must survive
        message.push(Field::string(1, "GD_Soldier.Character.CharClass_Soldier"));
        message.push(Field::string(1, "duplicate-ok"));
        message.push(Field::varint(2, 72));
        message.push(Field::fixed64(21, 1234));
        message.push(Field::fixed32(22, 99));
        message.push(Field::bytes(34, &[0, 159, 255, 1, 2]));
        message.push(Field::string(26, ""));
        message.push(Field::bytes(18, &nested.to_bytes()));
        message
    }

    #[test]
    fn json_round_trip_is_byte_identical() {
        let message = sample();
        let text = message_to_json(&message).to_string();
        let rebuilt =
            message_from_json(&serde_json::from_str(&text).expect("json")).expect("parse back");
        assert_eq!(message.to_bytes(), rebuilt.to_bytes());
    }

    #[test]
    fn json_round_trip_of_save_message_is_byte_identical() {
        let save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        let text = save_to_json(&save);
        let rebuilt = save_from_json(&text).expect("parse back");
        assert_eq!(save.to_bytes(), rebuilt.to_bytes());
    }

    use super::{save_from_json, save_to_json};
    use crate::savegame::SaveGame;
}
