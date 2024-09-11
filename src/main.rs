use bytes::BufMut;
use futures_util::{Stream, StreamExt, TryStreamExt};
use models::{DocumentPart, UploadResponse};
use rejections::BadRequest;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use warp::filters::multipart::{FormData, Part};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;
use warp::{Filter, Rejection, Reply};

mod models;
mod rejections;
#[cfg(test)]
mod test;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
struct FileMetadata {
    id: String,
    original_name: String,
    parts: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost/filedb".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Create the table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            original_name TEXT NOT NULL,
            parts TEXT[] NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS file_parts (
            id TEXT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            content BYTEA,
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let db = Arc::new(pool);

    let upload = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form())
        .and(with_db(db.clone()))
        .and_then(upload_file);

    let get_files = warp::path("files")
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(get_uploaded_files);

    let download = warp::path!("download" / String)
        .and(warp::get())
        .and(with_db(db.clone()))
        .and_then(download_file);

    let routes = upload.or(get_files).or(download);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    Ok(())
}

fn with_db(
    db: Arc<PgPool>,
) -> impl Filter<Extract = (Arc<PgPool>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}


async fn upload_file(
    form: FormData,
    dbx: Arc<PgPool>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let vec_file: Result<Vec<(String, String)>, warp::Error> = form
        .and_then(|field| async {
            let db = dbx.clone();
            process_field(field, db).await
        })
        .try_collect()
        .await;

    match vec_file {
        Ok(files) => Ok(warp::reply::json(&files)),
        Err(e) => Err(warp::reject::custom(
            BadRequest::new(&e.to_string())))
    }
}

async fn process_field(
    mut field: Part,
    db: Arc<PgPool>,
) -> Result<(String, String), warp::Error> {

    let mut bytes: Vec<u8> = Vec::new();
    while let Some(content) = field.data().await {
        let content = content?;
        bytes.put(content);
    }
    
    let file_name = field.filename().unwrap();
    let file_id = Uuid::new_v4().to_string();

    let mut file_parts = Vec::new();
    for (i, chunk) in bytes.chunks(5 * 1024 * 1024).enumerate() {
        let part_name = format!("{}_part_{}", file_id, i);
        let db = db.clone();
        let chunk = chunk.to_vec(); // Clone the chunk data
        let tokio_part_name = part_name.clone();
        tokio::spawn(async move {
            if let Err(e) = sqlx::query(
                "INSERT INTO file_parts (id, name, content) VALUES ($1, $2, $3)")
                .bind(&tokio_part_name)
                .bind(&tokio_part_name)
                .bind(&chunk)
                .execute(&*db)
                .await
            {
                eprintln!("Error saving file part {}: {}", tokio_part_name, e);
            }
        });

        file_parts.push(part_name);
    }

    sqlx::query("INSERT INTO files (id, original_name, parts) VALUES ($1, $2, $3)")
        .bind(&file_id)
        .bind(&file_name)
        .bind(&file_parts)
        .execute(&*db)
        .await.unwrap();

    Ok((field.name().to_string(), file_id))
}
async fn get_uploaded_files(db: Arc<PgPool>) -> Result<impl Reply, Rejection> {
    let files: Vec<FileMetadata> = sqlx::query_as("SELECT * FROM files")
        .fetch_all(&*db)
        .await
        .map_err(|e| {
            eprintln!("database error: {}", e);
            warp::reject::custom(BadRequest::new(&e.to_string()))
        })?;

    Ok(warp::reply::json(&files))
}

async fn download_file(id: String, db: Arc<PgPool>) -> Result<impl Reply, Rejection> {
    let metadata: FileMetadata = sqlx::query_as("SELECT * FROM files WHERE id = $1")
        .bind(&id)
        .fetch_one(&*db)
        .await
        .map_err(|e| {
            eprintln!("database error: {}", e);
            warp::reject::custom(BadRequest::new(&e.to_string()))
        })?;

    let mut content = Vec::new();
    for part in metadata.parts {
        let part:DocumentPart = sqlx::query_as("SELECT * FROM files WHERE id = $1")
        .bind(&part)
        .fetch_one(&*db)
        .await
        .map_err(|e| {
            eprintln!("database error: {}", e);
            warp::reject::custom(BadRequest::new(&e.to_string()))
        })?;

        content.extend(part.content)
    }

    Ok(warp::reply::with_header(
        content,
        "Content-Disposition",
        format!("attachment; filename=\"{}\"", metadata.original_name),
    ))
}
