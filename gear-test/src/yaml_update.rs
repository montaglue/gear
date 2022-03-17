use std::{path::PathBuf, fs, io::BufWriter};

use serde::{de::DeserializeOwned, Serialize};

fn read_old_from_file<P, Old>(path: P) -> anyhow::Result<Old>
where
    P: AsRef<std::path::Path>,
    Old: DeserializeOwned, {
    let file = fs::File::open(path.as_ref())
        .map_err(|e| anyhow::anyhow!("Error loading '{}': {}", path.as_ref().display(), e))?;
    let u = serde_yaml::from_reader::<_, Old>(file)
        .map_err(|e| anyhow::anyhow!("Error decoding '{}': {}", path.as_ref().display(), e))?;

    Ok(u)
}

fn write_new_to_file<P, New>(path: P, new: New) -> anyhow::Result<()>
where
    P: AsRef<std::path::Path>,
    New: Serialize, {
    let file = fs::File::open(path.as_ref())
        .map_err(|e| anyhow::anyhow!("Error loading '{}': {}", path.as_ref().display(), e))?;
    
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &new)
        .map_err(|e| anyhow::anyhow!("Error decoding '{}': {}", path.as_ref().display(), e))?;

    Ok(())
}

pub fn yaml_update<Old, New, F>(files: Vec<PathBuf>, mut f: F) -> anyhow::Result<()> 
where
    Old: DeserializeOwned,
    New: Serialize,
    F: FnMut(Old) -> anyhow::Result<New>, {
    for path in files {
        if path.is_dir() {
            for entry in path.read_dir().expect("read_dir call failed").flatten() {
                let old = read_old_from_file::<_, Old>(&entry.path())?;
                let new = f(old)?;
                write_new_to_file(&entry.path(), new)?;
            }
        } else {
            let old = read_old_from_file::<_, Old>(&path)?;
            let new = f(old)?;
            write_new_to_file(&path, new)?;
        }
    }
    Ok(())
}
