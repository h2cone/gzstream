use async_compression::futures::bufread::GzipDecoder;
use async_compression::tokio::write::GzipEncoder;
use futures::io::{self, BufReader, ErrorKind};
use futures::stream::StreamExt;
use futures::{AsyncBufReadExt, TryStreamExt};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

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
    let buf_cap = 2 << 12;
    let mut encoder = GzipEncoder::new(Vec::with_capacity(buf_cap));
    // Channel to send chunks to the main thread
    let (tx, mut rx) = mpsc::channel(buf_cap);

    tokio::spawn(async move {
        // Read line by line
        while let Some(line_res) = lines.next().await {
            if let Err(e) = line_res {
                eprintln!("Error reading line: {}", e);
                break;
            }
            let line = line_res.unwrap();
            replace_line(line, &mut encoder, buf_cap, &tx)
                .await
                .unwrap();
        }
        encoder.shutdown().await.unwrap();
        let buffer = encoder.get_ref();
        tx.send(buffer.clone()).await.unwrap();
        drop(tx);
    });

    while let Some(buffer) = rx.recv().await {
        process_chunk(&buffer).await?;
    }
    Ok(())
}

async fn replace_line(
    line: String,
    encoder: &mut GzipEncoder<Vec<u8>>,
    limit: usize,
    sender: &mpsc::Sender<Vec<u8>>,
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
        sender.send(buffer.clone()).await?;
        buffer.clear();
    }
    Ok(())
}

async fn process_chunk(chunk: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // e.g. Write to stdout
    tokio::io::stdout().write_all(chunk).await?;
    Ok(())
}
