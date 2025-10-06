//! # V0 -> V1
//!
//! ## Changes
//!
//! ### `EnvExamTemp` -> `ExamCreatorExam`
//!
//! #### Add
//!
//! - `version` as Int
//!   - `1`

use mongodb::bson;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::migrations::v0;

prisma_rust_schema::import_types!(
    schema_path = "https://raw.githubusercontent.com/freeCodeCamp/freeCodeCamp/2366e1ab6ba929c441640dde07045f2a266cc56e/api/prisma/schema.prisma",
    derive = [Clone, Debug, Serialize, Deserialize, PartialEq],
    patch = [
      struct ExamCreatorExam {
        #[serde(default = "version")]
        pub version: i32
      },
    ],
    include = [
        "ExamCreatorExam",
        "ExamCreatorUser",
        "ExamEnvironmentExam",
        "ExamEnvironmentQuestionSet",
        "ExamEnvironmentMultipleChoiceQuestion",
        "ExamEnvironmentConfig",
        "ExamEnvironmentQuestionType",
        "ExamEnvironmentAudio",
        "ExamEnvironmentAnswer",
        "ExamEnvironmentTagConfig",
        "ExamEnvironmentQuestionSetConfig",
        "ExamEnvironmentExamAttempt",
        "ExamEnvironmentQuestionSetAttempt",
        "ExamEnvironmentMultipleChoiceQuestionAttempt"
    ]
);

fn version() -> i32 {
    1
}

impl From<v0::EnvExamTemp> for ExamCreatorExam {
    fn from(v0_env_exam: v0::EnvExamTemp) -> Self {
        let json: Value = serde_json::to_value(&v0_env_exam).unwrap();
        let v1: Self = serde_json::from_value(json).unwrap();
        v1
    }
}

#[cfg(test)]
mod v1_to_v2 {
    use cmp::compare_structs;
    use mongodb::bson::oid::ObjectId;

    use crate::migrations::{
        v0,
        v1::{ExamCreatorExam, ExamEnvironmentConfig},
    };

    #[test]
    fn env_exam_temp_to_exam_creator_exam() {
        let v0 = v0::EnvExamTemp {
            id: ObjectId::new(),
            question_sets: vec![],
            config: v0::EnvConfig {
                name: String::from("Test"),
                note: String::new(),
                tags: vec![],
                total_time_in_m_s: 100,
                question_sets: vec![],
                retake_time_in_m_s: 100,
                passing_percent: 80.0,
            },
            prerequisites: vec![],
            deprecated: false,
        };

        let v0_cop = v0.clone();
        let v1 = ExamCreatorExam {
            id: v0_cop.id,
            question_sets: vec![],
            config: ExamEnvironmentConfig {
                name: v0_cop.config.name,
                note: v0_cop.config.note,
                tags: vec![],
                total_time_in_m_s: v0_cop.config.total_time_in_m_s,
                question_sets: vec![],
                retake_time_in_m_s: v0_cop.config.retake_time_in_m_s,
                passing_percent: v0_cop.config.passing_percent,
            },
            prerequisites: vec![],
            deprecated: v0_cop.deprecated,
            version: 1,
        };

        let new: ExamCreatorExam = v0.into();

        compare_structs!(
            v1,
            new,
            id,
            question_sets,
            config,
            prerequisites,
            deprecated,
            version
        );
    }
}
