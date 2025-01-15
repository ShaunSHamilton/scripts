use std::time::Duration;

use mongodb::bson::{self, de::Error, oid::ObjectId, DateTime, Document, Timestamp};

use crate::record::User;

#[derive(Debug)]
pub enum NormalizeError {
    UnhandledType { id: ObjectId, error: Error },
    NullEmail { doc: Document },
    ConfusedId { doc: Document },
}

pub fn normalize_user(user: &Document) -> Result<Document, NormalizeError> {
    if let Ok(id) = user.get_object_id("_id") {
        let mut normal_user: User = bson::from_document(user.clone()).map_err(|e| match e {
            Error::DeserializationError { message, .. } => {
                if message.contains("expected a non-empty string email") {
                    return NormalizeError::NullEmail { doc: user.clone() };
                }
                NormalizeError::UnhandledType {
                    id,
                    error: Error::EndOfStream,
                }
            }
            _ => NormalizeError::UnhandledType { id, error: e },
        })?;

        // Set the last updated time field to now
        normal_user.last_updated_at_in_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::new(0, 0))
            .as_millis() as u64;
        let new_user_document: Document = bson::to_document(&normal_user).unwrap();
        Ok(new_user_document)
    } else {
        Err(NormalizeError::ConfusedId { doc: user.clone() })
    }
}

pub fn num_to_datetime<N>(num: N) -> DateTime
where
    N: ToString,
{
    let s = num.to_string();

    // If float, remove the decimal part
    let s = if let Some(pos) = s.find('.') {
        &s[..pos]
    } else {
        &s
    };

    // Handle seconds, but assume milliseconds
    let num = if s.len() == 10 {
        let num = s.parse::<u64>().unwrap();
        num * 1000
    } else {
        s.parse::<u64>().unwrap()
    };

    DateTime::from_millis(num as i64)
}

pub trait ToMillis: ToString {
    fn to_millis(&self) -> u64 {
        let s = self.to_string();

        // If float, remove the decimal part
        let s = if let Some(pos) = s.find('.') {
            &s[..pos]
        } else {
            &s
        };

        // If the string contains any non-numeric characters, return 0
        if s.chars().any(|c| !c.is_numeric()) || s.is_empty() {
            return 0;
        }

        // Handle seconds, but assume milliseconds
        if s.len() == 10 {
            let num = s.parse::<u64>().unwrap();
            num * 1000
        } else {
            s.parse::<u64>().unwrap()
        }
    }
}

impl ToMillis for i64 {}
impl ToMillis for u64 {}
impl ToMillis for i32 {}
impl ToMillis for f64 {}
impl ToMillis for f32 {}
impl ToMillis for DateTime {
    fn to_millis(&self) -> u64 {
        let millis = self.timestamp_millis() as u64;
        if millis < 10_000_000_000 {
            return millis * 1000;
        }
        millis
    }
}
impl ToMillis for Timestamp {
    fn to_millis(&self) -> u64 {
        let millis = (self.time as u64) * 1000;
        if millis < 10_000_000_000 {
            return millis * 1000;
        }
        millis
    }
}

#[cfg(test)]
mod tests {
    use bson::doc;

    use super::*;

    #[test]
    fn normalize_bad_email_user() {
        let doc_1 = doc! {
            "_id": ObjectId::new(),
            "email": null,
            "username": "username",
            "unsubscribeId": "some-uuid".to_string()
        };

        let result = normalize_user(&doc_1);
        assert!(matches!(result, Err(NormalizeError::NullEmail { .. })));

        let doc_1 = doc! {
            "_id": ObjectId::new(),
            "email": "",
            "username": "username",
            "unsubscribeId": "some-uuid".to_string()
        };

        let result = normalize_user(&doc_1);
        assert!(matches!(result, Err(NormalizeError::NullEmail { .. })));
    }

    #[test]
    fn test_num_to_datetime() {
        let num = 1614556800;
        let dt = num_to_datetime(num);
        assert_eq!(dt.timestamp_millis(), 1614556800000);
    }

    #[test]
    fn test_to_millis() {
        let num = 1614556800;
        assert_eq!(num.to_millis(), 1614556800000);
    }

    #[test]
    fn test_to_millis_float() {
        let num_1 = 1614556800.0;
        let num_2 = 1614556800.123;
        assert_eq!(num_1.to_millis(), 1614556800000);
        assert_eq!(num_2.to_millis(), 1614556800000);
    }

    #[test]
    fn test_to_millis_datetime() {
        let num = 1614556800;
        let dt = DateTime::from_millis(num);
        assert_eq!(dt.to_millis(), 1614556800000);
    }

    #[test]
    fn test_to_millis_timestamp() {
        let num = 1614556800;
        let ts = Timestamp {
            time: num,
            increment: 0,
        };
        assert_eq!(ts.to_millis(), 1614556800000);
    }
}
