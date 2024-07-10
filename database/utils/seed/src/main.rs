use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Collection,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::task::JoinHandle;

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
        .run_command(doc! {"ping": 1}, None)
        .await?;
    let db = client.database("freecodecamp");

    let collection = db.collection::<Document>(collection_name);
    Ok(collection)
}

#[tokio::main]
async fn main() -> Result<(), mongodb::error::Error> {
    let mut handles = Vec::new();

    for _ in 0..7 {
        let handle: JoinHandle<Result<(), mongodb::error::Error>> = tokio::spawn(async move {
            let user_collection = get_collection(
                "mongodb://127.0.0.1:27017/freecodecamp?directConnection=true",
                "user",
            )
            .await?;
            seed_users(&user_collection).await
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Error: {:?}", e);
        }
    }

    Ok(())
}

async fn seed_users(user_collection: &Collection<Document>) -> Result<(), mongodb::error::Error> {
    let mut cursor = user_collection.find(doc! {}, None).await?;

    let mut rng: StdRng = StdRng::from_entropy();

    let mut docs = Vec::new();

    while let Some(user) = cursor.try_next().await? {
        let mut new_user = user.clone();

        new_user.remove("_id");

        // Duplicate user, creating a unique email, username, and unsubscribeId
        let c = format!("{:X}", rng.gen::<u64>());
        let email = format!("fcc_{c}@gmail.com");
        new_user.insert("email", email);
        let username = format!("fcc_{c}");
        new_user.insert("username", username);
        let unsubscribe_id = format!("fcc_{c}");
        new_user.insert("unsubscribeId", unsubscribe_id);

        docs.push(new_user);

        if docs.len() == 1000 {
            user_collection.insert_many(&docs, None).await?;
            docs.clear();
            cursor = user_collection.find(doc! {}, None).await?;
        }
        // user_collection.insert_one(new_user, None).await?;
    }
    // user_collection.insert_many(docs, None).await?;
    Ok(())
    // Ok(docs)
}
