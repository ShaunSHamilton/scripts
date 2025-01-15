use clap::Parser;
use futures_util::{StreamExt, TryStreamExt};
use mongodb::{
    bson::doc,
    options::{FindOptions, FullDocumentType, InsertOneModel},
    Namespace,
};
use tokio::{self, io::AsyncWriteExt, select};

mod clapper;
mod convert;
mod db;
mod error;
mod normalize;
mod record;

use error::Error;
use normalize::{normalize_user, NormalizeError};

use clapper::Args;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    // Start change stream before processing records to ensure no skips
    let change_stream_handle = tokio::spawn(change_stream(args.clone()));

    if let Err(e) = connect_and_process(&args).await {
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&args.logs)
            .await?;
        // Write errors to logs file
        file.write_all(format!("{}\n", e).as_bytes()).await?;
    };

    // Change stream continues to run until Ctrl+C is received
    if let Err(e) = change_stream_handle.await {
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&args.logs)
            .await?;
        // Write errors to logs file
        file.write_all(format!("{}\n", e).as_bytes()).await?;
    };

    Ok(())
}

async fn change_stream(args: Args) -> Result<(), mongodb::error::Error> {
    let client = db::client(&args.uri).await?;
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&args.logs)
        .await?;
    let user_collection = db::get_collection(&client, "user2").await?;
    let change_stream = user_collection
        .watch()
        .full_document(FullDocumentType::UpdateLookup)
        .await?;

    let mut change_stream = change_stream.fuse();

    let collection_name = if args.bulk_write {
        "bulk_users1"
    } else {
        "insert_users1"
    };

    loop {
        select! {
            _ = tokio::signal::ctrl_c() => {
                println!("Received Ctrl+C, shutting down...");
                break;
            }
            change = change_stream.select_next_some() => {
                match change {
                    Err(e) => {
                        file.write_all(format!(": {}\n", e).as_bytes()).await?;
                    }
                    Ok(change) => {
                        if let Some(full_document) = change.full_document {
                            match normalize_user(&full_document) {
                                Ok(normalized_user) => {
                                    let alternate_users_collection = db::get_collection(&client, collection_name).await?;
                                    let query = doc! {"_id": normalized_user.get_object_id("_id").expect("all records to have _id, because normalize handles otherwise")};

                                    // Update new collection with latest change, or insert if not found
                                    alternate_users_collection.update_one(query, doc! { "$set": normalized_user }).upsert(true).await?;
                                }
                                Err(normalize_error) => {
                                    match normalize_error {
                                        NormalizeError::UnhandledType { id, error } => {
                                            // Write errors to logs file
                                            file.write_all(format!("{}: {}\n", id, error).as_bytes())
                                            .await?;
                                        }
                                        NormalizeError::ConfusedId { doc } => {
                                            // Write errors to logs file
                                            file.write_all(format!("{}: {}\n", "Confused ID", doc).as_bytes())
                                            .await?;
                                        }
                                        NormalizeError::NullEmail { doc } => {
                                            // Add user record to own collection
                                            let recovered_users_collection =
                                            db::get_collection(&client, "recovered_users1").await?;
                                            recovered_users_collection.insert_one(doc).await?;
                                        }
                                    }
                                }
                            }
                        } else {
                            file.write_all(
                                format!("{}: {}\n", "Change Stream Error", "No full document").as_bytes(),
                            )
                            .await?;
                        }
                    }
                }
            }
        };
    }

    Ok::<(), mongodb::error::Error>(())
}

/// Queries for all records in batches, normalizes them, and inserts normalized records in new collection.
/// If record exists in new collection, update is ignored.
async fn connect_and_process(args: &Args) -> Result<(), mongodb::error::Error> {
    let client = db::client(&args.uri).await?;
    let user_collection = db::get_collection(&client, "user2").await?;
    let insert_users_collection = db::get_collection(&client, "insert_users1").await?;

    let user_collection_size = user_collection.estimated_document_count().await?;

    let find_ops = FindOptions::builder().batch_size(args.batch_size).build();
    let mut cursor = user_collection.find(doc! {}).with_options(find_ops).await?;

    let mut logs_file = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&args.logs)
        .await?;

    let mut count: u32 = 0;
    let mut last_logged_milestone = 0;
    let mut write_pool = Vec::with_capacity(args.batch_size as usize);

    while let Some(user) = cursor.try_next().await? {
        match normalize_user(&user) {
            Ok(normalized_user) => {
                if args.bulk_write {
                    let namespace = Namespace::new("freecodecamp", "bulk_users1");

                    let insert_one_model = InsertOneModel::builder()
                        .document(normalized_user)
                        .namespace(namespace)
                        .build();

                    let write_op = mongodb::options::WriteModel::InsertOne(insert_one_model);
                    write_pool.push(write_op);

                    if write_pool.len() >= args.batch_size as usize {
                        let _ = client.bulk_write(write_pool.clone()).await?;
                        write_pool.clear();
                        count += args.batch_size;
                    }
                } else {
                    let _ = insert_users_collection.insert_one(normalized_user).await;
                    count += 1;
                }
            }
            Err(normalize_error) => {
                match normalize_error {
                    NormalizeError::UnhandledType { id, error } => {
                        logs_file
                            .write_all(format!("{}: {}\n", id, error).as_bytes())
                            .await?;
                    }
                    NormalizeError::ConfusedId { doc } => {
                        logs_file
                            .write_all(format!("{}: {}\n", "Confused ID", doc).as_bytes())
                            .await?;
                    }
                    NormalizeError::NullEmail { doc } => {
                        // Add user record to own collection
                        let recovered_users_collection =
                            db::get_collection(&client, "recovered_users").await?;
                        recovered_users_collection.insert_one(doc).await?;
                    }
                }
            }
        }

        // if count % (user_collection_size.div_ceil(100) as u32) == 0 {
        //     println!("Users Processed: {} / {}", count, user_collection_size);
        // }

        // Determine the current milestone and log only if it's a new one
        let milestone = count / (user_collection_size.div_ceil(100) as u32);
        if milestone > last_logged_milestone {
            println!("Users Processed: {} / {}", count, user_collection_size);
            last_logged_milestone = milestone;
        }
    }

    Ok(())
}
