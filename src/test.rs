// use mockall::predicate::*;
// use mockall::mock;
// use warp::http::StatusCode;
// use warp::test::request;
// use std::sync::Arc;
// use bytes::Bytes;

// // Mock the database pool
// mock! {
//     PgPool {}
//     impl Clone for PgPool {
//         fn clone(&self) -> Self;
//     }
// }

// // Helper function to create a mock db pool
// fn mock_db_pool() -> Arc<MockPgPool> {
//     Arc::new(MockPgPool::new())
// }

// #[tokio::test]
// async fn test_upload_file() {
//     let db = mock_db_pool();
    
//     // Set up expectations
//     db.expect_execute()
//         .times(2)  // Once for file parts, once for file metadata
//         .returning(|_| Ok(sqlx::postgres::PgQueryResult::default()));

//     // Create a mock file
//     let file_content = Bytes::from_static(b"test file content");
//     let form = warp::multipart::mock()
//         .file("file", "test.txt", file_content.clone())
//         .build();

//     // Create the filter
//     let filter = warp::path("upload")
//         .and(warp::multipart::form())
//         .and(with_db(db.clone()))
//         .and_then(upload_file);

//     // Perform the request
//     let response = request()
//         .method("POST")
//         .path("/upload")
//         .body(form)
//         .reply(&filter)
//         .await;

//     assert_eq!(response.status(), StatusCode::OK);
    
//     // You may want to add more assertions here based on the expected response body
// }

// #[tokio::test]
// async fn test_get_uploaded_files() {
//     let db = mock_db_pool();
    
//     // Set up expectations
//     db.expect_fetch_all()
//         .times(1)
//         .returning(|_| Ok(vec![FileMetadata {
//             id: "test_id".to_string(),
//             original_name: "test.txt".to_string(),
//             parts: vec!["part1".to_string(), "part2".to_string()],
//         }]));

//     // Create the filter
//     let filter = warp::path("files")
//         .and(with_db(db.clone()))
//         .and_then(get_uploaded_files);

//     // Perform the request
//     let response = request()
//         .method("GET")
//         .path("/files")
//         .reply(&filter)
//         .await;

//     assert_eq!(response.status(), StatusCode::OK);
    
//     // Parse the response body and add more specific assertions
//     let files: Vec<FileMetadata> = serde_json::from_slice(response.body()).unwrap();
//     assert_eq!(files.len(), 1);
//     assert_eq!(files[0].id, "test_id");
//     assert_eq!(files[0].original_name, "test.txt");
//     assert_eq!(files[0].parts, vec!["part1", "part2"]);
// }

// #[tokio::test]
// async fn test_download_file() {
//     let db = mock_db_pool();
    
//     // Set up expectations for metadata fetch
//     db.expect_fetch_one()
//         .with(eq("SELECT * FROM files WHERE id = $1"))
//         .times(1)
//         .returning(|_| Ok(FileMetadata {
//             id: "test_id".to_string(),
//             original_name: "test.txt".to_string(),
//             parts: vec!["part1".to_string(), "part2".to_string()],
//         }));

//     // Set up expectations for part fetches
//     db.expect_fetch_one()
//         .with(eq("SELECT * FROM files WHERE id = $1"))
//         .times(2)
//         .returning(|_| Ok(DocumentPart {
//             id: "part_id".to_string(),
//             name: "part_name".to_string(),
//             content: vec![1, 2, 3, 4, 5],
//         }));

//     // Create the filter
//     let filter = warp::path!("download" / String)
//         .and(with_db(db.clone()))
//         .and_then(download_file);

//     // Perform the request
//     let response = request()
//         .method("GET")
//         .path("/download/test_id")
//         .reply(&filter)
//         .await;

//     assert_eq!(response.status(), StatusCode::OK);
//     assert_eq!(response.headers()["Content-Disposition"], "attachment; filename=\"test.txt\"");
//     assert_eq!(response.body(), &[1, 2, 3, 4, 5, 1, 2, 3, 4, 5]);
// }

// #[tokio::test]
// async fn test_upload_file_error() {
//     let db = mock_db_pool();
    
//     // Set up expectations to simulate an error
//     db.expect_execute()
//         .times(1)
//         .returning(|_| Err(sqlx::Error::RowNotFound));

//     // Create a mock file
//     let file_content = Bytes::from_static(b"test file content");
//     let form = warp::multipart::mock()
//         .file("file", "test.txt", file_content.clone())
//         .build();

//     // Create the filter
//     let filter = warp::path("upload")
//         .and(warp::multipart::form())
//         .and(with_db(db.clone()))
//         .and_then(upload_file);

//     // Perform the request
//     let response = request()
//         .method("POST")
//         .path("/upload")
//         .body(form)
//         .reply(&filter)
//         .await;

//     assert_eq!(response.status(), StatusCode::BAD_REQUEST);
//     // You may want to add more assertions here based on the expected error response
// }