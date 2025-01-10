use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, to_document, Bson, Document},
    options::{ClientOptions, FindOptions, InsertOneModel},
    Client, Collection, Namespace,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio;

mod utils;
use utils::no_option::NOption;

const BULK_WRITE: bool = true;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let collection_name = if BULK_WRITE {
        "user_bulk"
    } else {
        "user_replace"
    };

    let uri = "mongodb://127.0.0.1:27017/freecodecamp?directConnection=true";
    let mut client_options = ClientOptions::parse(uri).await?;

    client_options.app_name = Some("Rust Mongeese".to_string());

    // Get a handle to the cluster
    let client = Client::with_options(client_options)?;

    // Ping the server to see if you can connect to the cluster
    client
        .database("freecodecamp")
        .run_command(doc! {"ping": 1})
        .await?;
    let db = client.database("freecodecamp");

    let collection = db.collection::<Document>(collection_name);

    let find_ops = FindOptions::builder().batch_size(2000).build();
    let mut cursor = collection.find(doc! {}).with_options(find_ops).await?;

    let mut write_pool = vec![];
    let mut count = 0;
    while let Some(user) = cursor.try_next().await? {
        let normalized_user = normalize_user(user)?;

        if BULK_WRITE {
            let namespace = Namespace::new("freecodecamp", collection_name);

            let replace_one_model = InsertOneModel::builder()
                .document(normalized_user)
                .namespace(namespace)
                .build();

            let write_op = mongodb::options::WriteModel::InsertOne(replace_one_model);
            write_pool.push(write_op);

            if write_pool.len() >= 4000 {
                let _ = client.bulk_write(write_pool.clone()).await?;
                write_pool.clear();
                count += 4000;
            }
        } else {
            let _ = collection.insert_one(normalized_user).await;
            count += 1;
        }

        // Print count every 20_000
        if count % 20_000 == 0 {
            println!("Processed {} users", count);
        }
    }

    Ok(())
}

fn normalize_user(user: Document) -> Result<Document, Error> {
    let mut normalized_user =
        User::new(user.get_object_id("_id").expect("all records to have _id"));

    let completed_challenges = match user.get("completedChallenges") {
        Some(Bson::Array(completed_challenges)) => {
            let mut new_completed_challenges = vec![];
            for completed_challenge in completed_challenges {
                match completed_challenge {
                    Bson::Document(completed_challenge) => {
                        let challenge_type = match completed_challenge.get("challengeType") {
                            Some(Bson::Int32(v)) => NOption::Some(*v),
                            Some(Bson::Int64(v)) => NOption::Some(*v as i32),
                            Some(Bson::Double(v)) => NOption::Some(*v as i32),
                            _ => NOption::Null,
                        };

                        let completed_date = match completed_challenge.get("completedDate") {
                            Some(Bson::Double(v)) => *v as i64,
                            Some(Bson::DateTime(v)) => (*v).timestamp_millis(),
                            Some(Bson::Int32(v)) => *v as i64,
                            Some(Bson::Int64(v)) => *v,
                            Some(Bson::Timestamp(v)) => (v.time * 1000) as i64,
                            _ => continue,
                        };

                        let files = match completed_challenge.get("files") {
                            Some(Bson::Array(array)) => {
                                let mut files = vec![];
                                for file in array {
                                    if let Ok(file) = from_bson::<File>(file.clone()) {
                                        files.push(file);
                                    }
                                }
                                files
                            }
                            _ => vec![],
                        };

                        let github_link = match completed_challenge.get("githubLink") {
                            Some(Bson::String(v)) => NOption::Some(v.clone()),
                            _ => NOption::Null,
                        };

                        let id = match completed_challenge.get("id") {
                            Some(Bson::String(v)) => v.clone(),
                            _ => continue,
                        };

                        let is_manually_approved =
                            match completed_challenge.get("isManuallyApproved") {
                                Some(Bson::Boolean(v)) => NOption::Some(*v),
                                _ => NOption::Null,
                            };

                        let solution = match completed_challenge.get("solution") {
                            Some(Bson::String(v)) => NOption::Some(v.clone()),
                            _ => NOption::Null,
                        };

                        let new_completed_challenge = CompletedChallenge {
                            challenge_type,
                            completed_date,
                            files,
                            github_link,
                            id,
                            is_manually_approved,
                            solution,
                        };

                        new_completed_challenges.push(new_completed_challenge);
                    }
                    _ => (),
                };
            }
            new_completed_challenges
        }
        _ => vec![],
    };

    normalized_user.completed_challenges = completed_challenges;

    let email = user
        .get_str("email")
        .expect("all users to have email field");
    let email = email.to_lowercase();
    normalized_user.email = email;

    let document = to_document(&normalized_user).expect("to_document to succeed");

    Ok(document)
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct User {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub email: String,
    pub completed_challenges: Vec<CompletedChallenge>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletedChallenge {
    pub challenge_type: NOption<i32>,
    pub completed_date: i64,
    pub files: Vec<File>,
    pub github_link: NOption<String>,
    pub id: String,
    pub is_manually_approved: NOption<bool>,
    pub solution: NOption<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct File {
    pub contents: String,
    pub ext: String,
    pub key: String,
    pub name: String,
    pub path: String,
}

impl User {
    fn new(_id: ObjectId) -> Self {
        Self {
            _id,
            email: Default::default(),
            completed_challenges: Default::default(),
        }
    }
}

pub async fn get_collection(
    uri: &str,
    collection_name: &str,
) -> mongodb::error::Result<Collection<Document>> {
    let mut client_options = ClientOptions::parse(uri).await?;

    client_options.app_name = Some("Rust Mongeese".to_string());

    // Get a handle to the cluster
    let client = Client::with_options(client_options)?;

    // Ping the server to see if you can connect to the cluster
    client
        .database("freecodecamp")
        .run_command(doc! {"ping": 1})
        .await?;
    let db = client.database("freecodecamp");

    let collection = db.collection::<Document>(collection_name);
    Ok(collection)
}

pub enum Error {
    MongoDB(mongodb::error::Error),
    FileSystem(std::io::Error),
}

impl From<mongodb::error::Error> for Error {
    fn from(e: mongodb::error::Error) -> Self {
        Error::MongoDB(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::FileSystem(e)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::MongoDB(e) => write!(f, "MongoDB Error: {}", e),
            Error::FileSystem(e) => write!(f, "FileSystem Error: {}", e),
        }
    }
}
