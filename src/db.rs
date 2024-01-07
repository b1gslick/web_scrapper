pub mod databse_mod {
    use crate::Options;
    use dotenv::dotenv;
    use mongodb::bson::{doc, to_document};
    use mongodb::{
        options::{ClientOptions, ResolverConfig},
        Client,
    };
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::error::Error;

    #[derive(Clone, Default, Serialize, Deserialize, Debug)]
    pub struct DbOptions {
        pub db_name: String,
        pub collection_name: String,
    }

    async fn get_connect() -> Result<mongodb::Client, Box<dyn Error>> {
        dotenv().ok();

        // let DB_SRV = env::var("DB_SRV").unwrap_or("web-scraper".to_string());
        let db_pass = env::var("DB_PASS").unwrap_or("password".to_string());
        let db_host = env::var("DB_HOST").unwrap_or("localhost".to_string());
        let db_port = env::var("DB_PORT").unwrap_or("27017".to_string());
        let db_user = env::var("DB_USER").unwrap_or("web-finder".to_string());
        let db_name = env::var("DB_NAME").unwrap_or("options".to_string());

        let client_uri = format!(
            "mongodb://{}:{}@{}:{}/{}",
            db_user, db_pass, db_host, db_port, db_name
        );

        let options =
            ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
                .await?;
        let client = Client::with_options(options)?;

        Ok(client)
    }

    async fn get_collection<T>(
        db_option: DbOptions,
    ) -> Result<mongodb::Collection<T>, Box<dyn Error>> {
        let client = get_connect().await?;
        let collection = client
            .database(&db_option.db_name)
            .collection(&db_option.collection_name);
        Ok(collection)
    }

    pub async fn write_data(options: Options, db_option: DbOptions) -> Result<(), Box<dyn Error>> {
        let bson_document = to_document(&options).unwrap();
        let collection = get_collection(db_option).await?;
        // collection.insert_one(bson_document, None).await?;
        let filter = doc! {"name": "eco"};
        let result = collection
            .find_one_and_replace(filter, bson_document, None)
            .await?;
        log::info!("Update reulst is {:?}", result);
        Ok(())
    }
    pub async fn get_old_data(db_option: DbOptions) -> Result<Options, Box<dyn Error>> {
        let collection = get_collection(db_option).await?;
        let options: Options = collection
            .find_one(None, None)
            .await?
            .expect("No find document");
        Ok(options)
    }
}
