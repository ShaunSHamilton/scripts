use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Collection,
};

pub async fn get_collection(
    client: &Client,
    collection_name: &str,
) -> mongodb::error::Result<Collection<Document>> {
    let db = client.database("freecodecamp");

    let collection = db.collection::<Document>(collection_name);
    Ok(collection)
}

pub async fn client(uri: &str) -> mongodb::error::Result<Client> {
    let mut client_options = ClientOptions::parse(uri).await?;

    client_options.app_name = Some("Rust Mongeese".to_string());

    // Get a handle to the cluster
    let client = Client::with_options(client_options)?;

    // Ping the server to see if you can connect to the cluster
    client
        .database("freecodecamp")
        .run_command(doc! {"ping": 1})
        .await?;

    Ok(client)
}
