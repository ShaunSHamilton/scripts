use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOptions, InsertOneModel, WriteModel},
    Client, Collection, Namespace,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

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

#[tokio::main]
async fn main() -> Result<(), mongodb::error::Error> {
    seed_users().await?;

    Ok(())
}

async fn seed_users() -> Result<(), mongodb::error::Error> {
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

    let user_collection = db.collection::<Document>("user");

    for _ in 0..500 {
        let find_options = FindOptions::builder().batch_size(100).build();
        let mut cursor = user_collection
            .find(doc! {})
            .with_options(find_options)
            .await?;

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

            let insert_model = InsertOneModel::builder()
                .namespace(Namespace::new("freecodecamp", "user2"))
                .document(new_user)
                .build();

            let write_model = WriteModel::InsertOne(insert_model);

            docs.push(write_model);

            if docs.len() == 500 {
                client.bulk_write(docs.clone()).await?;

                docs.clear();
            }
        }
    }
    Ok(())
}
