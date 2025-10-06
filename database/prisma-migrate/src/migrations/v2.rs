//! # V1 -> V2
//!
//! ## Changes
//!
//! ### `ExamEnvironmentExam`
//!
//! #### Add
//!
//! - `config`
//!   - `totalTimeInS`
//!     - `Int`
//!   - `retakeTimeInS`
//!     - `Int`
//!
//! ### `ExamCreatorExam`
//!
//! Same as `ExamEnvironmentExam`
//!
//! ### `ExamEnvironmentExamAttempt`
//!
//! #### Add
//!
//! - `startTime`
//!   - `DateTime`
//! - `questionSets.questions`
//!   - `submissionTime`
//!     - `DateTime`
//!
//! ### `ExamCreatorUser`
//!
//! #### Add
//!
//! - `settings`
//! - `version`

use futures_util::StreamExt;
use mongodb::bson::{self, doc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

use crate::migrations::v1;

prisma_rust_schema::import_types!(
    schema_path = "https://raw.githubusercontent.com/ShaunSHamilton/freeCodeCamp/refs/heads/breaking_prisma-dates/api/prisma/schema.prisma",
    derive = [Clone, Debug, Serialize, Deserialize, PartialEq],
    patch = [
      struct ExamCreatorUser {
        #[serde(default = "version")]
        pub version: i32
      },
    ],
    include = [
        "ExamCreatorExam",
        "ExamEnvironmentExam",
        "ExamEnvironmentQuestionSet",
        "ExamEnvironmentMultipleChoiceQuestion",
        "ExamEnvironmentMultipleChoiceQuestionAttempt",
        "ExamEnvironmentConfig",
        "ExamEnvironmentQuestionType",
        "ExamEnvironmentAudio",
        "ExamEnvironmentAnswer",
        "ExamEnvironmentTagConfig",
        "ExamEnvironmentQuestionSetConfig",
        "ExamEnvironmentExamAttempt",
        "ExamEnvironmentQuestionSetAttempt",
        "ExamCreatorUser",
        "ExamCreatorUserSettings",
        "ExamCreatorDatabaseEnvironment"
    ]
);

pub fn version() -> i32 {
    1
}

impl From<v1::ExamEnvironmentExam> for ExamEnvironmentExam {
    fn from(f: v1::ExamEnvironmentExam) -> Self {
        let v1_config = f.config;

        let retake_time_in_s = v1_config.retake_time_in_m_s / 1000;
        let total_time_in_s = v1_config.total_time_in_m_s / 1000;

        let tags = v1_config
            .tags
            .into_iter()
            .map(|tag| ExamEnvironmentTagConfig::from(tag))
            .collect();
        let question_sets = v1_config
            .question_sets
            .into_iter()
            .map(|qs| ExamEnvironmentQuestionSetConfig::from(qs))
            .collect();

        let v2_config = ExamEnvironmentConfig {
            name: v1_config.name,
            note: v1_config.note,
            tags,
            total_time_in_s,
            question_sets,
            retake_time_in_s,
            passing_percent: v1_config.passing_percent,
        };

        let question_sets = f
            .question_sets
            .into_iter()
            .map(|qs| ExamEnvironmentQuestionSet::from(qs))
            .collect();

        ExamEnvironmentExam {
            id: f.id,
            question_sets,
            config: v2_config,
            prerequisites: f.prerequisites,
            deprecated: f.deprecated,
            version: 2,
        }
    }
}

impl From<v1::ExamEnvironmentTagConfig> for ExamEnvironmentTagConfig {
    fn from(f: v1::ExamEnvironmentTagConfig) -> Self {
        let json: Value = serde_json::to_value(&f).unwrap();
        let t: Self = serde_json::from_value(json).unwrap();
        t
    }
}

impl From<v1::ExamEnvironmentQuestionSetConfig> for ExamEnvironmentQuestionSetConfig {
    fn from(f: v1::ExamEnvironmentQuestionSetConfig) -> Self {
        let json: Value = serde_json::to_value(&f).unwrap();
        let t: Self = serde_json::from_value(json).unwrap();
        t
    }
}

impl From<v1::ExamEnvironmentQuestionSet> for ExamEnvironmentQuestionSet {
    fn from(f: v1::ExamEnvironmentQuestionSet) -> Self {
        let json: Value = serde_json::to_value(&f).unwrap();
        let t: Self = serde_json::from_value(json).unwrap();
        t
    }
}

impl From<v1::ExamCreatorExam> for ExamCreatorExam {
    fn from(f: v1::ExamCreatorExam) -> Self {
        let v1_config = f.config;

        let retake_time_in_s = v1_config.retake_time_in_m_s / 1000;
        let total_time_in_s = v1_config.total_time_in_m_s / 1000;

        let tags = v1_config
            .tags
            .into_iter()
            .map(|tag| ExamEnvironmentTagConfig::from(tag))
            .collect();
        let question_sets = v1_config
            .question_sets
            .into_iter()
            .map(|qs| ExamEnvironmentQuestionSetConfig::from(qs))
            .collect();

        let v2_config = ExamEnvironmentConfig {
            name: v1_config.name,
            note: v1_config.note,
            tags,
            total_time_in_s,
            question_sets,
            retake_time_in_s,
            passing_percent: v1_config.passing_percent,
        };

        let question_sets = f
            .question_sets
            .into_iter()
            .map(|qs| ExamEnvironmentQuestionSet::from(qs))
            .collect();

        ExamCreatorExam {
            id: f.id,
            question_sets,
            config: v2_config,
            prerequisites: f.prerequisites,
            deprecated: f.deprecated,
            version: 2,
        }
    }
}

impl From<v1::ExamCreatorUser> for ExamCreatorUser {
    fn from(v1_exam_creator_user: v1::ExamCreatorUser) -> Self {
        let settings = ExamCreatorUserSettings {
            database_environment: ExamCreatorDatabaseEnvironment::Production,
        };
        ExamCreatorUser {
            id: v1_exam_creator_user.id,
            email: v1_exam_creator_user.email,
            github_id: v1_exam_creator_user.github_id,
            name: v1_exam_creator_user.name,
            picture: v1_exam_creator_user.picture,
            settings,
            version: 1,
        }
    }
}

impl From<v1::ExamEnvironmentExamAttempt> for ExamEnvironmentExamAttempt {
    fn from(f: v1::ExamEnvironmentExamAttempt) -> Self {
        let start_time = bson::DateTime::from_millis(f.start_time_in_m_s as i64);

        let question_sets = f
            .question_sets
            .into_iter()
            .map(|qs| ExamEnvironmentQuestionSetAttempt::from(qs))
            .collect();

        ExamEnvironmentExamAttempt {
            id: f.id,
            exam_id: f.exam_id,
            user_id: f.user_id,
            generated_exam_id: f.generated_exam_id,
            start_time,
            question_sets,
            version: 2,
        }
    }
}

impl From<v1::ExamEnvironmentQuestionSetAttempt> for ExamEnvironmentQuestionSetAttempt {
    fn from(f: v1::ExamEnvironmentQuestionSetAttempt) -> Self {
        let questions = f
            .questions
            .into_iter()
            .map(|q| ExamEnvironmentMultipleChoiceQuestionAttempt::from(q))
            .collect();

        ExamEnvironmentQuestionSetAttempt {
            id: f.id,
            questions,
        }
    }
}

impl From<v1::ExamEnvironmentMultipleChoiceQuestionAttempt>
    for ExamEnvironmentMultipleChoiceQuestionAttempt
{
    fn from(f: v1::ExamEnvironmentMultipleChoiceQuestionAttempt) -> Self {
        let submission_time = bson::DateTime::from_millis(f.submission_time_in_m_s as i64);

        ExamEnvironmentMultipleChoiceQuestionAttempt {
            id: f.id,
            answers: f.answers,
            submission_time,
        }
    }
}

pub async fn migrate_exam_environment_exam(
    exam_collection_v1: mongodb::Collection<v1::ExamEnvironmentExam>,
    exam_collection_v2: mongodb::Collection<ExamEnvironmentExam>,
) -> Result<(), String> {
    let mut exams_v1 = match exam_collection_v1.find(doc! {"version": 1}).await {
        Ok(e) => e,
        Err(e) => {
            error!("Unable to find ExamEnvironmentExam collection");
            return Err(e.to_string());
        }
    };

    while let Some(exam_v1) = exams_v1.next().await {
        let exam_v1 = exam_v1.map_err(|e| {
            error!("unable to get next exam from cursor.");
            e.to_string()
        })?;
        info!("migrating {}", exam_v1.id);
        let exam_v2: ExamEnvironmentExam = exam_v1.into();

        exam_collection_v2.insert_one(exam_v2).await.map_err(|e| {
            error!("unable to insert exam");
            e.to_string()
        })?;
    }

    Ok(())
}

pub async fn migrate_exam_creator_user(
    exam_creator_user_collection_v1: mongodb::Collection<v1::ExamCreatorUser>,
    exam_creator_user_collection_v2: mongodb::Collection<ExamCreatorUser>,
) -> Result<(), String> {
    let mut users_v1 = match exam_creator_user_collection_v1
        .find(doc! {"version": 0})
        .await
    {
        Ok(e) => e,
        Err(e) => {
            error!("Unable to find ExamCreatorUser collection");
            return Err(e.to_string());
        }
    };

    while let Some(user_v1) = users_v1.next().await {
        let user_v1 = user_v1.map_err(|e| {
            error!("unable to get next user from cursor.");
            e.to_string()
        })?;
        info!("migrating {}", user_v1.id);
        let user_v2: ExamCreatorUser = user_v1.into();

        exam_creator_user_collection_v2
            .insert_one(user_v2)
            .await
            .map_err(|e| {
                error!("unable to insert user");
                e.to_string()
            })?;
    }

    Ok(())
}

pub async fn migrate_exam_environment_exam_attempt(
    exam_environment_exam_attempt_collection_v1: mongodb::Collection<
        v1::ExamEnvironmentExamAttempt,
    >,
    exam_environment_exam_attempt_collection_v2: mongodb::Collection<ExamEnvironmentExamAttempt>,
) -> Result<(), String> {
    let mut attempts_v1 = match exam_environment_exam_attempt_collection_v1
        .find(doc! {"version": 1})
        .await
    {
        Ok(e) => e,
        Err(e) => {
            error!("Unable to find ExamEnvironmentExamAttempt collection");
            return Err(e.to_string());
        }
    };

    while let Some(attempt_v1) = attempts_v1.next().await {
        let attempt_v1 = attempt_v1.map_err(|e| {
            error!("unable to get next attempt from cursor.");
            e.to_string()
        })?;
        info!("migrating {}", attempt_v1.id);
        let attempt_v2: ExamEnvironmentExamAttempt = attempt_v1.into();

        exam_environment_exam_attempt_collection_v2
            .insert_one(attempt_v2)
            .await
            .map_err(|e| {
                error!("unable to insert attempt");
                e.to_string()
            })?;
    }
    Ok(())
}

pub async fn migrate_exam_creator_exam(
    exam_creator_exam_collection_v1: mongodb::Collection<v1::ExamCreatorExam>,
    exam_creator_exam_collection_v2: mongodb::Collection<ExamCreatorExam>,
) -> Result<(), String> {
    let mut exams_v1 = match exam_creator_exam_collection_v1.find(doc! {}).await {
        Ok(e) => e,
        Err(e) => {
            error!("Unable to find ExamCreatorExam collection");
            return Err(e.to_string());
        }
    };

    while let Some(exam_v1) = exams_v1.next().await {
        let exam_v1 = exam_v1.map_err(|e| {
            error!("unable to get next exam from cursor.");
            e.to_string()
        })?;
        info!("migrating {}", exam_v1.id);
        let exam_v2: ExamCreatorExam = exam_v1.into();

        exam_creator_exam_collection_v2
            .insert_one(exam_v2)
            .await
            .map_err(|e| {
                error!("unable to insert exam");
                e.to_string()
            })?;
    }

    Ok(())
}
