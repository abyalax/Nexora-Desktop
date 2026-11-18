use nacre_core::dock::items::{name_from_path, Pin};
use nacre_core::dock::pins;
use std::io::Write;
use std::path::Path;

pub fn name_for(path: &str) -> String {
    let n = nacre_platform::wins::names::friendly(path);
    if n.is_empty() {
        name_from_path(path)
    } else {
        n
    }
}

fn default_pins() -> Vec<Pin> {
    let win = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    let explorer = format!("{win}\\explorer.exe");
    vec![Pin {
        name: name_for(&explorer),
        path: explorer,
    }]
}

pub fn load(path: &Path) -> Vec<Pin> {
    match std::fs::read_to_string(path) {
        Ok(text) => pins::parse(&text, name_for),
        Err(_) => default_pins(),
    }
}

pub fn save(path: &Path, list: &[Pin]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(pins::serialize(list).as_bytes())?;
    f.sync_all()?;
    drop(f);
    std::fs::rename(&tmp, path)
}
