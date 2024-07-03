use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use trieve_server::{
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        chunk_operator::get_pg_point_ids_from_qdrant_point_ids,
        qdrant_operator::{
            delete_points_from_qdrant, get_qdrant_collections, scroll_qdrant_collection_ids,
        },
    },
};

#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
        database_url,
        config,
    );

    let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
        .max_size(10)
        .build()
        .expect("Failed to create diesel_async pool");

    let web_pool = actix_web::web::Data::new(pool.clone());

    let collections = get_qdrant_collections().await?;

    for collection in collections {
        println!("starting on collection: {:?}", collection);

        let mut offset = Some(uuid::Uuid::nil().to_string());

        while let Some(cur_offset) = offset {
            let (qdrant_point_ids, new_offset) = scroll_qdrant_collection_ids(
                collection.clone(),
                Some(cur_offset.to_string()),
                Some(1000),
            )
            .await?;

            let pg_point_ids =
                get_pg_point_ids_from_qdrant_point_ids(qdrant_point_ids.clone(), web_pool.clone())
                    .await?;

            let qdrant_point_ids_not_in_pg = qdrant_point_ids
                .iter()
                .filter(|x| !pg_point_ids.contains(x))
                .cloned()
                .collect::<Vec<uuid::Uuid>>();

            if qdrant_point_ids_not_in_pg.len() > 0 {
                println!(
                    "len of qdrant_point_ids_not_in_pg: {:?}",
                    qdrant_point_ids_not_in_pg.len()
                );

                delete_points_from_qdrant(qdrant_point_ids_not_in_pg, collection.clone()).await?;
            }

            offset = new_offset;
        }
    }

    Ok(())
}
