use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::dir::Dir;

const MAX_TOTAL_COUNT: f64 = 10000.0;
const AGING_FACTOR: f64 = 0.9;

pub struct DirStore {
    file: PathBuf,
    pub dirs: Vec<Dir>,
}

impl DirStore {
    pub fn load(file: PathBuf) -> io::Result<DirStore> {
        let content = match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };
        let dirs = content.lines().filter_map(parse_line).collect();
        Ok(DirStore { file, dirs })
    }

    pub fn add(&mut self, path: &Path, now: u64) {
        match self.dirs.iter_mut().find(|d| d.path == path) {
            Some(dir) => {
                dir.count += 1.0;
                dir.last_access = now;
            }
            None => self.dirs.push(Dir { path: path.to_path_buf(), count: 1.0, last_access: now }),
        }
        self.age();
    }

    pub fn remove(&mut self, path: &Path) {
        self.dirs.retain(|d| d.path != path);
    }

    pub fn save(&self) -> io::Result<()> {
        let content: String = self
            .dirs
            .iter()
            .map(|d| format!("{};{};{}\n", d.path.display(), d.count, d.last_access))
            .collect();
        let tmp = self.file.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&tmp, content)?;
        fs::rename(&tmp, &self.file)
    }

    fn age(&mut self) {
        let total: f64 = self.dirs.iter().map(|d| d.count).sum();
        if total > MAX_TOTAL_COUNT {
            for dir in &mut self.dirs {
                dir.count *= AGING_FACTOR;
            }
            self.dirs.retain(|d| d.count >= 1.0);
        }
    }
}

/// `path;count;last_access` - split from the right so paths may contain ';'.
fn parse_line(line: &str) -> Option<Dir> {
    let mut fields = line.rsplitn(3, ';');
    let last_access = fields.next()?.parse().ok()?;
    let count = fields.next()?.parse().ok()?;
    let path = fields.next().filter(|p| !p.is_empty())?;
    Some(Dir { path: PathBuf::from(path), count, last_access })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn temp_file() -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "zust-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir.join("data")
    }

    #[test]
    fn load_missing_file_gives_empty_store() {
        let store = DirStore::load(temp_file()).unwrap();
        assert!(store.dirs.is_empty());
    }

    #[test]
    fn load_parses_lines_and_skips_invalid_ones() {
        let file = temp_file();
        fs::write(&file, "/home/hane;12;1778353961\ngarbage\n/a;b;c;2;5\n\n").unwrap();
        let store = DirStore::load(file).unwrap();
        assert_eq!(
            store.dirs,
            vec![
                Dir { path: "/home/hane".into(), count: 12.0, last_access: 1778353961 },
                Dir { path: "/a;b;c".into(), count: 2.0, last_access: 5 },
            ]
        );
    }

    #[test]
    fn add_creates_new_entry() {
        let mut store = DirStore::load(temp_file()).unwrap();
        store.add(Path::new("/x"), 100);
        assert_eq!(store.dirs, vec![Dir { path: "/x".into(), count: 1.0, last_access: 100 }]);
    }

    #[test]
    fn add_existing_entry_increments_count_and_updates_time() {
        let mut store = DirStore::load(temp_file()).unwrap();
        store.add(Path::new("/x"), 100);
        store.add(Path::new("/x"), 200);
        assert_eq!(store.dirs, vec![Dir { path: "/x".into(), count: 2.0, last_access: 200 }]);
    }

    #[test]
    fn add_ages_counts_when_total_exceeds_limit() {
        let file = temp_file();
        fs::write(&file, "/big;10000;1\n/small;1;1\n").unwrap();
        let mut store = DirStore::load(file).unwrap();
        store.add(Path::new("/big"), 2);
        assert_eq!(store.dirs.len(), 1, "count < 1 after aging is dropped");
        assert!((store.dirs[0].count - 10001.0 * 0.9).abs() < 1e-6);
    }

    #[test]
    fn remove_deletes_entry() {
        let mut store = DirStore::load(temp_file()).unwrap();
        store.add(Path::new("/x"), 1);
        store.add(Path::new("/y"), 1);
        store.remove(Path::new("/x"));
        assert_eq!(store.dirs.len(), 1);
        assert_eq!(store.dirs[0].path, PathBuf::from("/y"));
    }

    #[test]
    fn save_then_load_roundtrips() {
        let file = temp_file();
        let mut store = DirStore::load(file.clone()).unwrap();
        store.add(Path::new("/home/hane"), 1778353961);
        store.add(Path::new("/with;semicolon"), 5);
        store.save().unwrap();
        assert_eq!(DirStore::load(file).unwrap().dirs, store.dirs);
    }
}
