use clap::Parser;
use futures_util::TryStreamExt;
use mongodb::{
    bson::doc,
    options::{FindOptions, InsertOneModel},
    Namespace,
};
use tokio::{self, io::AsyncWriteExt};

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

    if let Err(e) = connect_and_process(&args).await {
        let mut file = tokio::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(args.logs)
            .await?;
        // Write errors to logs file
        file.write_all(format!("{}\n", e).as_bytes()).await?;
    };

    Ok(())
}

async fn connect_and_process(args: &Args) -> Result<(), mongodb::error::Error> {
    let client = db::client(&args.uri).await?;
    let user_collection = db::get_collection(&client, "user").await?;
    let insert_users_collection = db::get_collection(&client, "insert_users").await?;

    let user_collection_size = user_collection.estimated_document_count().await?;

    let find_ops = FindOptions::builder().batch_size(args.batch_size).build();
    let mut cursor = user_collection.find(doc! {}).with_options(find_ops).await?;

    let mut logs_file = tokio::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&args.logs)
        .await?;

    let mut count: u32 = 0;
    let mut write_pool = Vec::with_capacity(args.batch_size as usize);
    while let Some(user) = cursor.try_next().await? {
        match normalize_user(user) {
            Ok(normalized_user) => {
                if args.bulk_write {
                    let namespace = Namespace::new("freecodecamp", "bulk_users");

                    let replace_one_model = InsertOneModel::builder()
                        .document(normalized_user)
                        .namespace(namespace)
                        .build();

                    let write_op = mongodb::options::WriteModel::InsertOne(replace_one_model);
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

        if count % (user_collection_size.div_ceil(100) as u32) == 0 {
            println!("Users Processed: {} / {}", count, user_collection_size);
        }
    }

    Ok(())
}
