use anyhow::{Context, Result, anyhow};
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use clap::Parser;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

#[derive(Parser, Debug)]
#[command(
    name = "r2-uploader",
    author,
    version,
    about = "Uploads large files (e.g. 2GB videos) to Cloudflare R2 using S3 Multipart Upload."
)]
struct Args {
    /// Cloudflare R2 Account ID. Can also be set via R2_ACCOUNT_ID environment variable.
    #[arg(short, long, env = "R2_ACCOUNT_ID")]
    account_id: String,

    /// Cloudflare R2 Access Key ID. Can also be set via R2_ACCESS_KEY_ID environment variable.
    #[arg(short = 'i', long, env = "R2_ACCESS_KEY_ID")]
    access_key_id: String,

    /// Cloudflare R2 Secret Access Key. Can also be set via R2_SECRET_ACCESS_KEY environment variable.
    #[arg(short = 's', long, env = "R2_SECRET_ACCESS_KEY")]
    secret_access_key: String,

    /// Cloudflare R2 Bucket name.
    #[arg(short, long)]
    bucket: String,

    /// Local file path of the video/file to upload.
    #[arg(short, long)]
    file_path: PathBuf,

    /// Key (destination path) in the R2 bucket.
    #[arg(short, long)]
    key: String,

    /// Chunk size in megabytes (MB). Must be at least 5MB. Default is 15MB.
    #[arg(short = 'c', long, default_value_t = 15)]
    chunk_size_mb: u64,

    /// Number of concurrent chunk uploads. Default is 4.
    #[arg(short = 'p', long, default_value_t = 4)]
    concurrency: usize,
}

/// Detect basic mime types from the file extension
fn detect_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => match ext.to_ascii_lowercase().as_str() {
            "mp4" => "video/mp4",
            "mkv" => "video/x-matroska",
            "mov" => "video/quicktime",
            "avi" => "video/x-msvideo",
            "webm" => "video/webm",
            "m4v" => "video/x-m4v",
            _ => "application/octet-stream",
        },
        None => "application/octet-stream",
    }
}

/// Helper task to upload a single chunk of the file
async fn upload_part(
    client: &Client,
    bucket: &str,
    key: &str,
    upload_id: &str,
    part_number: i32,
    file_path: &Path,
    offset: u64,
    size: u64,
    pb: &ProgressBar,
) -> Result<CompletedPart> {
    // Open the file and seek to the requested chunk offset
    let mut file = File::open(file_path)
        .await
        .with_context(|| format!("Failed to open file at {:?}", file_path))?;
    file.seek(SeekFrom::Start(offset)).await?;

    // Read the chunk into memory
    let mut buffer = vec![0u8; size as usize];
    file.read_exact(&mut buffer).await?;

    // Prepare ByteStream
    let body = aws_sdk_s3::primitives::ByteStream::from(buffer);

    // Call S3 UploadPart API
    let upload_part_res = client
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .part_number(part_number)
        .body(body)
        .send()
        .await
        .with_context(|| format!("Failed to upload part {}", part_number))?;

    let etag = upload_part_res
        .e_tag()
        .ok_or_else(|| anyhow!("No ETag returned for part {}", part_number))?
        .to_string();

    // Notify progress
    pb.inc(size);

    Ok(CompletedPart::builder()
        .part_number(part_number)
        .e_tag(etag)
        .build())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Verify local file exists
    if !args.file_path.exists() {
        return Err(anyhow!("Target file does not exist: {:?}", args.file_path));
    }

    let file_size = tokio::fs::metadata(&args.file_path)
        .await
        .context("Failed to query target file metadata")?
        .len();

    let chunk_size = args.chunk_size_mb * 1024 * 1024;
    if chunk_size < 5 * 1024 * 1024 {
        return Err(anyhow!(
            "Chunk size must be at least 5MB (R2/S3 requirement). Provided: {}MB",
            args.chunk_size_mb
        ));
    }

    // Set up AWS S3 Configuration targeting Cloudflare R2
    let credentials = Credentials::new(
        args.access_key_id,
        args.secret_access_key,
        None,
        None,
        "Static",
    );

    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(aws_config::Region::new("auto"))
        .endpoint_url(format!(
            "https://{}.r2.cloudflarestorage.com",
            args.account_id
        ))
        .load()
        .await;

    let client = Client::new(&config);

    // Prepare upload part metadata offsets
    let mut offset = 0;
    let mut part_number = 1;
    let mut parts_info = Vec::new();

    while offset < file_size {
        let current_chunk_size = std::cmp::min(chunk_size, file_size - offset);
        parts_info.push((part_number, offset, current_chunk_size));
        offset += current_chunk_size;
        part_number += 1;
    }

    let total_parts = parts_info.len();
    println!("File Path: {:?}", args.file_path);
    println!(
        "File Size: {} bytes ({:.2} GB)",
        file_size,
        file_size as f64 / 1_073_741_824.0
    );
    println!(
        "Chunk Size: {} bytes ({} MB)",
        chunk_size, args.chunk_size_mb
    );
    println!("Total chunks to upload: {}", total_parts);
    println!("Concurrency limit: {}", args.concurrency);

    let mime_type = detect_mime_type(&args.file_path);

    // Step 1: Initiate Multipart Upload
    println!("Initiating Multipart Upload on Cloudflare R2...");
    let initiate_res = client
        .create_multipart_upload()
        .bucket(&args.bucket)
        .key(&args.key)
        .content_type(mime_type)
        .send()
        .await
        .context("Failed to initiate multipart upload on Cloudflare R2")?;

    let upload_id = initiate_res
        .upload_id()
        .ok_or_else(|| anyhow!("Could not retrieve upload_id from R2 response"))?
        .to_string();

    println!("Multipart upload initiated. Upload ID: {}", upload_id);

    // Initialize Progress Bar
    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) - Uploading parts...")?
            .progress_chars("#>-"),
    );

    // Setup concurrent stream parameters
    let client_arc = Arc::new(client);
    let bucket_arc = Arc::new(args.bucket);
    let key_arc = Arc::new(args.key);
    let upload_id_arc = Arc::new(upload_id);
    let file_path_arc = Arc::new(args.file_path);

    // Step 2: Upload parts concurrently
    let upload_results = futures::stream::iter(parts_info)
        .map(|(p_num, off, sz)| {
            let client = Arc::clone(&client_arc);
            let bucket = Arc::clone(&bucket_arc);
            let key = Arc::clone(&key_arc);
            let upload_id = Arc::clone(&upload_id_arc);
            let file_path = Arc::clone(&file_path_arc);
            let pb = pb.clone();

            tokio::spawn(async move {
                upload_part(
                    &client, &bucket, &key, &upload_id, p_num, &file_path, off, sz, &pb,
                )
                .await
            })
        })
        .buffer_unordered(args.concurrency)
        .collect::<Vec<_>>()
        .await;

    let mut completed_parts = Vec::new();
    let mut has_error = false;

    for res in upload_results {
        match res {
            Ok(Ok(part)) => {
                completed_parts.push(part);
            }
            Ok(Err(err)) => {
                eprintln!("\nError in chunk upload: {:?}", err);
                has_error = true;
            }
            Err(join_err) => {
                eprintln!("\nTokio task join error: {:?}", join_err);
                has_error = true;
            }
        }
    }

    if has_error {
        pb.finish_and_clear();
        eprintln!("Error occurred during upload. Aborting multipart upload on R2...");
        if let Err(e) = client_arc
            .abort_multipart_upload()
            .bucket(&*bucket_arc)
            .key(&*key_arc)
            .upload_id(&*upload_id_arc)
            .send()
            .await
        {
            eprintln!("Failed to abort multipart upload: {:?}", e);
        }
        return Err(anyhow!("Upload aborted due to previous chunk errors."));
    }

    pb.finish_with_message("All chunks uploaded successfully.");

    // Sort completed parts by part number (S3 requires parts list sorted in ascending order)
    completed_parts.sort_by_key(|p| p.part_number());

    // Step 3: Complete Multipart Upload
    println!("Completing multipart upload...");
    let mut completed_upload = CompletedMultipartUpload::builder();
    for part in completed_parts {
        completed_upload = completed_upload.parts(part);
    }

    client_arc
        .complete_multipart_upload()
        .bucket(&*bucket_arc)
        .key(&*key_arc)
        .upload_id(&*upload_id_arc)
        .multipart_upload(completed_upload.build())
        .send()
        .await
        .context("Failed to complete multipart upload on Cloudflare R2")?;

    println!("Upload completed successfully to Cloudflare R2!");
    Ok(())
}
