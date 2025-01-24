use mongodb::bson::{self, Bson, DateTime};
use serde::Deserialize;

use crate::{
    normalize::ToMillis,
    record::{CompletedChallenge, File, NOption},
};

struct CompletedChallengeVisitor;

impl<'de> serde::de::Visitor<'de> for CompletedChallengeVisitor {
    type Value = CompletedChallenge;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct CompletedChallenge")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut challenge_type = None;
        let mut completed_date = None;
        let mut exam_results = None;
        let mut files = None;
        let mut github_link = None;
        let mut id = None;
        let mut is_manually_approved = None;
        let mut solution = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "challengeType" => {
                    if challenge_type.is_some() {
                        return Err(serde::de::Error::duplicate_field("challengeType"));
                    }

                    challenge_type = match map.next_value()? {
                        Bson::Int32(v) => Some(NOption::Some(v as u32)),
                        Bson::Int64(v) => Some(NOption::Some(v as u32)),
                        Bson::Double(v) => Some(NOption::Some(v as u32)),
                        _ => None,
                    };
                }
                "completedDate" => {
                    if completed_date.is_some() {
                        return Err(serde::de::Error::duplicate_field("completedDate"));
                    }

                    completed_date = match map.next_value()? {
                        Bson::Double(v) => Some(v.to_millis()),
                        Bson::DateTime(v) => Some(v.to_millis()),
                        Bson::Int32(v) => Some(v.to_millis()),
                        Bson::Int64(v) => Some(v.to_millis()),
                        Bson::Timestamp(v) => Some(v.to_millis()),
                        _ => None,
                    };
                }
                "examResults" => {
                    if exam_results.is_some() {
                        return Err(serde::de::Error::duplicate_field("examResults"));
                    }

                    exam_results = match map.next_value()? {
                        Bson::Document(doc) => {
                            let exam_results = bson::from_document(doc).map_err(|e| {
                                serde::de::Error::invalid_value(
                                    serde::de::Unexpected::Other(&e.to_string()),
                                    &"a valid ExamResults",
                                )
                            })?;
                            Some(exam_results)
                        }
                        _ => None,
                    };
                }
                "files" => {
                    if files.is_some() {
                        return Err(serde::de::Error::duplicate_field("files"));
                    }

                    files = match map.next_value()? {
                        Bson::Array(array) => {
                            let mut files = vec![];
                            for file in array {
                                let file: File = bson::from_bson(file).map_err(|e| {
                                    serde::de::Error::invalid_value(
                                        serde::de::Unexpected::Other(&e.to_string()),
                                        &"a valid File",
                                    )
                                })?;
                                files.push(file);
                            }
                            Some(files)
                        }
                        _ => None,
                    };
                }
                "githubLink" => {
                    if github_link.is_some() {
                        return Err(serde::de::Error::duplicate_field("githubLink"));
                    }

                    github_link = match map.next_value()? {
                        Bson::String(v) => Some(NOption::Some(v)),
                        _ => Some(NOption::Null),
                    };
                }
                "id" => {
                    if id.is_some() {
                        return Err(serde::de::Error::duplicate_field("id"));
                    }

                    id = match map.next_value()? {
                        Bson::String(v) => Some(v),
                        _ => None,
                    };
                }
                "isManuallyApproved" => {
                    if is_manually_approved.is_some() {
                        return Err(serde::de::Error::duplicate_field("isManuallyApproved"));
                    }

                    is_manually_approved = match map.next_value()? {
                        Bson::Boolean(v) => Some(NOption::Some(v)),
                        _ => Some(NOption::Null),
                    };
                }
                "solution" => {
                    if solution.is_some() {
                        return Err(serde::de::Error::duplicate_field("solution"));
                    }

                    solution = match map.next_value()? {
                        Bson::String(v) => Some(NOption::Some(v)),
                        _ => Some(NOption::Null),
                    };
                }
                _ => {
                    // println!("Skipping {key:?}");
                }
            }
        }

        let challenge_type = challenge_type.unwrap_or(NOption::Undefined);
        let completed_date =
            completed_date.unwrap_or(DateTime::now().timestamp_millis().to_millis());
        let exam_results = exam_results.unwrap_or_default();
        let files = files.unwrap_or_default();
        let github_link = github_link.unwrap_or_default();
        let id = id.unwrap_or_default();
        let is_manually_approved = is_manually_approved.unwrap_or_default();
        let solution = solution.unwrap_or_default();

        Ok(CompletedChallenge {
            challenge_type,
            completed_date,
            files,
            github_link,
            id,
            is_manually_approved,
            solution,
            exam_results,
        })
    }
}

impl<'de> Deserialize<'de> for CompletedChallenge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CompletedChallengeVisitor)
    }
}
