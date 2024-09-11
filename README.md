# File Upload Server

A Rust-based file upload server using Warp and PostgreSQL, providing APIs for uploading files, retrieving file metadata, and downloading files.

## Features

1. Upload files (separates files into parts, stores metadata in PostgreSQL)
2. Retrieve metadata for all uploaded files
3. Download files by ID (reconstructs the original file from its parts)

## Prerequisites

- Docker
- Docker Compose

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/file-upload-server.git
cd file-upload-server
```

### 2. Build and run with Docker

Build the Docker image and start the services:

```bash
docker-compose up --build
```

The server will be accessible at `http://localhost:3030`.

To run in detached mode:

```bash
docker-compose up -d --build
```

### 3. Stopping the application

To stop the application:

```bash
docker-compose down
```

To stop and remove all data (including the database volume):

```bash
docker-compose down -v
```

## API Documentation

1. **Upload File**
   - URL: `/upload`
   - Method: POST
   - Content-Type: multipart/form-data
   - Response: File ID (JSON)

2. **Get Uploaded Files Data**
   - URL: `/files`
   - Method: GET
   - Response: Array of file metadata (JSON)

3. **Download File**
   - URL: `/download/{file_id}`
   - Method: GET
   - Response: File content (binary)

## Docker Instructions

### Viewing Logs

View logs (if running in detached mode):

```bash
docker-compose logs
```

Follow logs in real-time:

```bash
docker-compose logs -f
```

### Rebuilding After Changes

If you modify the Rust code or Dockerfile:

1. Rebuild the image:
   ```bash
   docker-compose build
   ```

2. Restart the containers:
   ```bash
   docker-compose up
   ```

### Accessing the PostgreSQL Database

1. Find the PostgreSQL container ID:
   ```bash
   docker ps
   ```

2. Open a psql session:
   ```bash
   docker exec -it <container_id> psql -U user -d filedb
   ```

### Troubleshooting

1. Permission issues: Try running Docker commands with `sudo`.
2. Database connection issues: Check the `DATABASE_URL` in `docker-compose.yml`.
3. Other issues: Check logs with `docker-compose logs`.

## Development

1. Ensure Rust and Cargo are installed on your local machine.
2. Modify `src/main.rs` as needed.
3. Update `Cargo.toml` for dependency changes.
4. Rebuild and run using the Docker instructions above.

## License

[MIT License](https://opensource.org/licenses/MIT)