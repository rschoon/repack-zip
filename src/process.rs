
use std::cmp::Ordering;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

use super::params::{ProcessParams, Sort};

pub fn process_file(filename: &Path, params: &ProcessParams) -> anyhow::Result<()> {
    let Some(mut output) = ({
        let input = File::open(filename)?;
        let mut zipin = zip::ZipArchive::new(input)?;

        let mut zipout = create_zip(filename, params)?;

        copy_zip(&mut zipin, zipout.as_mut(), params)?;

        match zipout {
            Some(z) => Some(z.finish()?),
            None => None,
        }
    }) else {
        return Ok(());
    };

    output.flush()?;
    output.persist(filename)?;

    Ok(())
}

fn create_zip(filename: &Path, params: &ProcessParams) -> anyhow::Result<Option<zip::ZipWriter<tempfile::NamedTempFile>>> {
    if params.dry_run {
        return Ok(None);
    }
    
    let output = if let Some(parent) = filename.parent() {
        tempfile::NamedTempFile::new_in(parent)
    } else {
        tempfile::NamedTempFile::new()
    }?;
    Ok(Some(zip::ZipWriter::new(output)))
}

fn copy_zip(
    src: &mut zip::ZipArchive<impl Read + Seek>,
    mut dest: Option<&mut zip::ZipWriter<impl Write + Seek>>,
    params: &ProcessParams,
) -> anyhow::Result<()> {
    let mut items = (0..src.len()).map(|idx|{
        Entry {
            idx,
            name: src.name_for_index(idx).map(Into::into)
        }
    }).collect::<Vec<_>>();

    sort_entries(&mut items, params.sort);

    for item in items {
        let mut entry = src.by_index(item.idx)?;
        println!("{}", entry.name());
        
        let Some(dest) = dest.as_deref_mut() else { continue };
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

fn sort_entries(items: &mut [Entry], sort: Option<Sort>) {
    let Some(sort) = sort else { return };

    let cmp: fn(&Entry, &Entry) -> Ordering = match sort {
        Sort::Normal => |a, b| a.name.cmp(&b.name),
        Sort::IgnoreCase => |a, b| unicase_cmp(a.name.as_deref(), b.name.as_deref())
    };

    items.sort_by(cmp);
}

fn unicase_cmp(a: Option<&str>, b: Option<&str>) -> Ordering {
    use unicase::UniCase;
    let key_a = a.map(UniCase::new);
    let key_b = b.map(UniCase::new);
    key_a.cmp(&key_b)
}

struct Entry {
    idx: usize,
    name: Option<Box<str>>,
}
