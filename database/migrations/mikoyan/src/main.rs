use std::time::Instant;

use clap::Parser;
use futures_util::{StreamExt, TryStreamExt};
use mongodb::{
    bson::doc,
    options::{FindOptions, FullDocumentType, UpdateOneModel},
    Namespace,
};
use tokio::{self, io::AsyncWriteExt};

mod clapper;
mod convert;
mod db;
mod error;
mod normalize;
mod record;

// use error::Error;
use normalize::{normalize_user, NormalizeError};

use clapper::Args;

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     let args = Args::parse();

//     // Start change stream before processing records to ensure no skips
//     let change_stream_handle = tokio::spawn(change_stream(args.clone()));

//     if let Err(e) = connect_and_process(&args).await {
//         let mut file = tokio::fs::OpenOptions::new()
//             .append(true)
//             .create(true)
//             .open(&args.logs)
//             .await?;
//         // Write errors to logs file
//         file.write_all(format!("{}\n", e).as_bytes()).await?;
//     };

//     // Change stream continues to run until Ctrl+C is received
//     if let Err(e) = change_stream_handle.await {
//         let mut file = tokio::fs::OpenOptions::new()
//             .append(true)
//             .create(true)
//             .open(&args.logs)
//             .await?;
//         // Write errors to logs file
//         file.write_all(format!("{}\n", e).as_bytes()).await?;
//     };

//     Ok(())
// }

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Send 'c' to toggle change stream, 'n' to toggle normalization.");

    // Create channels to control the tasks
    let (change_stream_tx, change_stream_rx) = watch::channel(true);
    let (connect_and_process_tx, connect_and_process_rx) = watch::channel(false);

    // Start user input listener
    let input_handle =
        spawn_input_listener(change_stream_tx.clone(), connect_and_process_tx.clone());

    // Start tasks with initial states
    let mut change_stream_handle = spawn_change_stream(args.clone(), change_stream_rx);
    let mut connect_and_process_handle =
        spawn_connect_and_process(args.clone(), connect_and_process_rx);

    // Gracefully shut down tasks
    connect_and_process_tx.send(false).unwrap();
    change_stream_tx.send(false).unwrap();

    if let Some(handle) = connect_and_process_handle.take() {
        handle.await?;
    }
    // Wait for all tasks to finish
    if let Some(handle) = change_stream_handle.take() {
        handle.await?;
    }

    input_handle.await?;

    Ok(())
}

fn spawn_input_listener(
    change_stream_tx: watch::Sender<bool>,
    connect_and_process_tx: watch::Sender<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            match line.trim() {
                "c" => {
                    let new_state = !*change_stream_tx.borrow();
                    change_stream_tx.send(new_state).unwrap();
                    println!("Change stream toggled to: {}", new_state);
                }
                "n" => {
                    let new_state = !*connect_and_process_tx.borrow();
                    connect_and_process_tx.send(new_state).unwrap();
                    println!("Normalization toggled to: {}", new_state);
                }
                _ => {
                    println!("Press 'c' to toggle change stream, 'n' to toggle normalization.");
                }
            }
        }
    })
}

fn spawn_change_stream(
    args: Args,
    mut change_stream_rx: watch::Receiver<bool>,
) -> Option<JoinHandle<()>> {
    Some(tokio::spawn(async move {
        loop {
            if *change_stream_rx.borrow() {
                println!("Change stream running...");
                tokio::select! {
                    _ = change_stream_logic(args.clone()) => {},
                    _ = change_stream_rx.changed() => {
                        if !*change_stream_rx.borrow() {
                            println!("Change stream stopped.");
                            break;
                        }
                    }
                }
            } else {
                change_stream_rx.changed().await.unwrap();
            }
        }
    }))
}

async fn change_stream_logic(args: Args) {
    change_stream(args).await.unwrap()
}

fn spawn_connect_and_process(
    args: Args,
    mut connect_and_process_rx: watch::Receiver<bool>,
) -> Option<JoinHandle<()>> {
    Some(tokio::spawn(async move {
        loop {
            if *connect_and_process_rx.borrow() {
                println!("Normalization running...");
                tokio::select! {
                    _ = connect_and_process_logic(args.clone()) => {},
                    _ = connect_and_process_rx.changed() => {
                        if !*connect_and_process_rx.borrow() {
                            println!("Normalization stopped.");
                            break;
                        }
                    }
                }
            } else {
                connect_and_process_rx.changed().await.unwrap();
            }
        }
    }))
}

async fn connect_and_process_logic(args: Args) {
    connect_and_process(&args).await.unwrap();
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

    let collection_name = "bulk_users1";

    while let Ok(Some(change)) = change_stream.next().await.transpose() {
        if let Some(full_document) = change.full_document {
            match normalize_user(&full_document) {
                Ok(normalized_user) => {
                    let alternate_users_collection =
                        db::get_collection(&client, collection_name).await?;
                    let query = doc! {"_id": normalized_user.get_object_id("_id").expect("all records to have _id, because normalize handles otherwise")};

                    // Update new collection with latest change, or insert if not found
                    alternate_users_collection
                        .update_one(query, doc! { "$set": normalized_user })
                        .upsert(true)
                        .await?;
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

    Ok::<(), mongodb::error::Error>(())
}

/// Queries for all records in batches, normalizes them, and inserts normalized records in new collection.
/// If record exists in new collection, update is ignored.
async fn connect_and_process(args: &Args) -> Result<(), mongodb::error::Error> {
    let client = db::client(&args.uri).await?;
    let user_collection = db::get_collection(&client, "user2").await?;

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

    let instant = Instant::now();

    while let Some(user) = cursor.try_next().await? {
        match normalize_user(&user) {
            Ok(normalized_user) => {
                let namespace = Namespace::new("freecodecamp", "bulk_users1");

                let filter = doc! {
                    "_id": normalized_user.get_object_id("_id").expect("all records to have _id, because normalize handles otherwise"),
                };

                // If records does not exist, insert it
                // TODO: If the script is stopped, restarting will result in skipping updated records.
                let update_one_model = UpdateOneModel::builder()
                    .upsert(true)
                    .namespace(namespace)
                    .filter(filter)
                    .update(doc! { "$setOnInsert": normalized_user })
                    .build();

                let write_op = mongodb::options::WriteModel::UpdateOne(update_one_model);
                write_pool.push(write_op);

                if write_pool.len() >= args.batch_size as usize {
                    let _ = client.bulk_write(write_pool.clone()).await?;
                    write_pool.clear();
                    count += args.batch_size;
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

        // Calculate the number of records processed per second
        // Determine the current milestone and log only if it's a new one
        let milestone = count / (user_collection_size.div_ceil(100) as u32);
        if milestone > last_logged_milestone {
            let records_per_second = count / instant.elapsed().as_secs() as u32;
            println!(
                "Users Processed: {} / {} ({} records/s)",
                count, user_collection_size, records_per_second
            );
            last_logged_milestone = milestone;
        }
    }

    Ok(())
}
