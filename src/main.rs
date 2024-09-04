use async_compression::futures::bufread::GzipDecoder;
use async_compression::tokio::write::GzipEncoder;
use futures::io::{self, BufReader, ErrorKind};
// trait `StreamExt` which provides `next` is implemented
use futures::stream::StreamExt;
// trait `TryStreamExt` which provides `map_err` is implemented
use futures::{AsyncBufReadExt, TryStreamExt};
use tokio::io::{stdout, AsyncWriteExt};

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

    let mut encoder = GzipEncoder::new(stdout());

    while let Some(line_res) = lines.next().await {
        if let Err(e) = line_res {
            eprintln!("Error reading line: {}", e);
            break;
        }
        replace(line_res.unwrap(), &mut encoder).await?;
    }
    encoder.shutdown().await?;

    let mut out = encoder.into_inner();
    out.flush().await?;

    Ok(())
}

async fn replace<W: AsyncWriteExt + Unpin>(
    line: String,
    encoder: &mut W,
) -> Result<(), std::io::Error> {
    // e.g. Replace all lines starting with "#CHROM" to lowercase
    if line.starts_with("#CHROM") {
        encoder.write_all(line.to_lowercase().as_bytes()).await?;
    } else {
        encoder.write_all(line.as_bytes()).await?;
    }
    encoder.write_all(b"\n").await?;
    Ok(())
}
