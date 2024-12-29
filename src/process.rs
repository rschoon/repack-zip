
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

pub struct ProcessParams {
    pub compress_threshold: u64
}

pub fn process_file(filename: &Path, params: &ProcessParams) -> anyhow::Result<()> {
    let mut output = {
        let output = if let Some(parent) = filename.parent() {
            tempfile::NamedTempFile::new_in(parent)
        } else {
            tempfile::NamedTempFile::new()
        }?;
        let mut zipout = zip::ZipWriter::new(output);

        let input = File::open(filename)?;
        let mut zipin = zip::ZipArchive::new(input)?;

        copy_zip(&mut zipin, &mut zipout, params)?;

        zipout.finish()?
    };

    output.flush()?;
    output.persist(filename)?;

    Ok(())
}

fn copy_zip(
    src: &mut zip::ZipArchive<impl Read + Seek>,
    dest: &mut zip::ZipWriter<impl Write + Seek>,
    params: &ProcessParams,
) -> anyhow::Result<()> {
    for i in 0..src.len() {
        let mut entry = src.by_index(i)?;
        if entry.is_dir() {
            dest.add_directory(entry.name(), SimpleFileOptions::default())?;
        } else if entry.is_symlink() {
            let mut target = String::new();
            entry.read_to_string(&mut target)?;
            dest.add_symlink(entry.name(), &target, SimpleFileOptions::default())?;
        } else {
            let mut options = SimpleFileOptions::default();
            if entry.size() >= params.compress_threshold {
                options = options.compression_method(zip::CompressionMethod::Deflated)
                    .compression_level(Some(264));
            } else {
                options = options.compression_method(zip::CompressionMethod::Stored);
            }

            dest.start_file(entry.name(), options)?;
            std::io::copy(&mut entry, dest)?;
        }
    }

    Ok(())
}
