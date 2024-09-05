use async_compression::futures::bufread::GzipDecoder;
use async_compression::tokio::write::GzipEncoder;
use futures::io::{self, BufReader, ErrorKind};
// trait `StreamExt` which provides `next` is implemented
use futures::stream::StreamExt;
// trait `TryStreamExt` which provides `map_err` is implemented
use futures::{AsyncBufReadExt, TryStreamExt};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let origin = "http://localhost:3000/download?key=sample.vcf.gz";

    let stream = reqwest::get(origin)
        .await?
        .bytes_stream()
        .map_err(|e| io::Error::new(ErrorKind::Other, e))
        .into_async_read();
    let stream_reader = BufReader::new(stream);
    let decoder = GzipDecoder::new(stream_reader);
    // Decoder also implements AsyncRead
    let reader = BufReader::new(decoder);
    let mut lines = reader.lines();
    // Output buffer capacity
    let buf_cap = 10 * 1024 * 1024;
    let mut encoder = GzipEncoder::new(Vec::with_capacity(buf_cap));

    while let Some(line_res) = lines.next().await {
        if let Err(e) = line_res {
            eprintln!("Error reading line: {}", e);
            break;
        }
        let line = line_res.unwrap();
        replace_line(line, &mut encoder, buf_cap).await?;
    }
    encoder.shutdown().await?;
    let buffer = encoder.get_ref();
    process_chunk(buffer).await?;

    Ok(())
}

async fn replace_line(
    line: String,
    encoder: &mut GzipEncoder<Vec<u8>>,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // e.g. Replace all lines starting with "#CHROM" to lowercase
    if line.starts_with("#CHROM") {
        encoder.write_all(line.to_lowercase().as_bytes()).await?;
    } else {
        encoder.write_all(line.as_bytes()).await?;
    }
    encoder.write_all(b"\n").await?;
    // Flush the buffer if exceeds the limit
    if encoder.get_ref().len() >= limit {
        let buffer = encoder.get_mut();
        process_chunk(buffer).await?;
        buffer.clear();
    }

    Ok(())
}

async fn process_chunk(chunk: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // e.g. Write to stdout
    tokio::io::stdout().write_all(chunk).await?;
    Ok(())
}
