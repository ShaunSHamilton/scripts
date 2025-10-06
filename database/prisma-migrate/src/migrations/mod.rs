use mongodb::Client;
use tracing::{info, instrument};

use crate::db;

pub mod v0;
pub mod v1;
pub mod v2;

pub async fn migrate(client: &Client) -> Result<(), String> {
    // info!("Running v0 -> v1 migration...");
    // v0_to_v1(&client).await?;
    // info!("v0 -> v1 migration complete.");
    info!("Running v1 -> v2 migration...");
    v1_to_v2(&client).await?;
    info!("v1 -> v2 migration complete.");
    Ok(())
}

#[instrument(skip_all)]
pub async fn v1_to_v2(client: &Client) -> Result<(), String> {
    // `ExamEnvironmentExam`
    let exam_collection_v1 =
        db::get_collection::<v1::ExamEnvironmentExam>(&client, "ExamEnvironmentExam").await;
    let exam_collection_v2 =
        db::get_collection::<v2::ExamEnvironmentExam>(&client, "ExamEnvironmentExamV2").await;
    v2::migrate_exam_environment_exam(exam_collection_v1, exam_collection_v2).await?;

    // ExamCreatorUser
    let exam_creator_user_collection_v1 =
        db::get_collection::<v1::ExamCreatorUser>(&client, "ExamCreatorUser").await;
    let exam_creator_user_collection_v2 =
        db::get_collection::<v2::ExamCreatorUser>(&client, "ExamCreatorUserV2").await;
    v2::migrate_exam_creator_user(
        exam_creator_user_collection_v1,
        exam_creator_user_collection_v2,
    )
    .await?;

    // ExamEnvironmentExamAttempt
    let exam_environment_exam_attempt_collection_v1 =
        db::get_collection::<v1::ExamEnvironmentExamAttempt>(&client, "ExamEnvironmentExamAttempt")
            .await;
    let exam_environment_exam_attempt_collection_v2 = db::get_collection::<
        v2::ExamEnvironmentExamAttempt,
    >(&client, "ExamEnvironmentExamAttemptV2")
    .await;
    v2::migrate_exam_environment_exam_attempt(
        exam_environment_exam_attempt_collection_v1,
        exam_environment_exam_attempt_collection_v2,
    )
    .await?;

    // ExamCreatorExam
    let exam_creator_exam_collection_v1 =
        db::get_collection::<v1::ExamCreatorExam>(&client, "ExamCreatorExam").await;
    let exam_creator_exam_collection_v2 =
        db::get_collection::<v2::ExamCreatorExam>(&client, "ExamCreatorExamV2").await;
    v2::migrate_exam_creator_exam(
        exam_creator_exam_collection_v1,
        exam_creator_exam_collection_v2,
    )
    .await?;

    Ok(())
}

// #[instrument(skip_all)]
// pub async fn v0_to_v1(client: &Client) -> Result<(), String> {
//     let exam_collection_v0 = db::get_collection::<v0::EnvExamTemp>(&client, "EnvExamTemp").await;
//     let exam_collection_v1 =
//         db::get_collection::<v1::ExamCreatorExam>(&client, "ExamCreatorExam").await;

//     let mut exams_v0 = match exam_collection_v0.find(doc! {}).await {
//         Ok(e) => e,
//         Err(e) => {
//             error!("Unable to find EnvExamTemp collection");
//             return Err(e.to_string());
//         }
//     };

//     while let Some(exam_v0) = exams_v0.next().await {
//         let v0 = exam_v0.map_err(|e| {
//             error!("unable to get next exam from cursor.");
//             e.to_string()
//         })?;
//         info!("migrating {}", v0.id);
//         let v1: v1::ExamCreatorExam = v0.into();

//         exam_collection_v1.insert_one(v1).await.map_err(|e| {
//             error!("unable to insert exam");
//             e.to_string()
//         })?;
//     }

//     Ok(())
// }
