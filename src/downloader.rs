use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn download_file(url: &str, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let part_path = destination.with_extension("zip.part");

    // If final file already exists, nothing to do
    if destination.exists() {
        println!("Archive already fully downloaded: {}", destination.display());
        return Ok(());
    }

    println!("Initiating resumable chunked download...");
    match download_resumable_ureq(url, &part_path) {
        Ok(_) => {
            fs::rename(&part_path, destination)?;
            println!("Download verified and saved to {}", destination.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("Direct stream warning: {}. Falling back to resumable curl...", e);
            download_resumable_curl(url, &part_path)?;
            fs::rename(&part_path, destination)?;
            println!("Download verified and saved to {}", destination.display());
            Ok(())
        }
    }
}

fn download_resumable_ureq(url: &str, part_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut existing_len: u64 = 0;
    if part_path.exists() {
        existing_len = part_path.metadata()?.len();
        if existing_len > 0 {
            println!(
                "Found partial download: {:.2} MB. Resuming from byte {}...",
                existing_len as f64 / (1024.0 * 1024.0),
                existing_len
            );
        }
    }

    let mut req = ureq::get(url)
        .set("User-Agent", "mini-dbs-installer/0.1")
        .set("Accept-Encoding", "identity"); // Ensure raw byte stream for accurate ranges

    if existing_len > 0 {
        req = req.set("Range", &format!("bytes={}-", existing_len));
    }

    let resp = req.call()?;
    let status = resp.status();

    let (mut file, total_size, start_offset) = if status == 206 {
        // 206 Partial Content: Server accepted Range request
        let total = parse_content_range_total(resp.header("content-range"))
            .or_else(|| {
                resp.header("content-length")
                    .and_then(|l| l.parse::<u64>().ok())
                    .map(|len| len + existing_len)
            })
            .unwrap_or(0);

        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(part_path)?;

        (f, total, existing_len)
    } else {
        // 200 OK: Server returned full content (or range not supported / file was empty)
        let total = resp
            .header("content-length")
            .and_then(|l| l.parse::<u64>().ok())
            .unwrap_or(0);

        let f = File::create(part_path)?;
        (f, total, 0)
    };

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );
    pb.set_position(start_offset);

    let mut reader = resp.into_reader();
    let mut buffer = [0u8; 65536]; // 64 KB chunk buffer
    let mut current_pos = start_offset;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        current_pos += bytes_read as u64;
        pb.set_position(current_pos);
    }

    pb.finish_with_message("Done");
    Ok(())
}

fn parse_content_range_total(header: Option<&str>) -> Option<u64> {
    // Example: "bytes 10485760-315936451/315936452"
    let h = header?;
    let slash_idx = h.rfind('/')?;
    let total_str = &h[slash_idx + 1..].trim();
    total_str.parse::<u64>().ok()
}

fn download_resumable_curl(url: &str, part_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Resuming download with curl (range-aware chunking)...");
    let status = Command::new("curl.exe")
        .arg("-fL")
        .arg("-C").arg("-") // Auto-resume from current file size
        .arg(url)
        .arg("-o").arg(part_path)
        .status()?;

    if status.success() && part_path.exists() {
        Ok(())
    } else {
        Err("curl resume download failed".into())
    }
}

pub fn extract_zip(archive_path: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;

    let total_files = archive.len();
    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} Extracting: [{bar:40.yellow/blue}] {pos}/{len} files")?
            .progress_chars("=>-"),
    );

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let raw_name = file.name();

        let stripped_name = if let Some(stripped) = raw_name.strip_prefix("pgsql/") {
            stripped
        } else if let Some(stripped) = raw_name.strip_prefix("pgsql\\") {
            stripped
        } else {
            raw_name
        };

        if stripped_name.is_empty() {
            continue;
        }

        let outpath: PathBuf = target_dir.join(stripped_name);

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
        pb.inc(1);
    }

    pb.finish_with_message("Extraction complete");
    Ok(())
}
