//! 审核复现：只在新建的临时库调用真实 dry-run 入口，比较调用前后文件字节。
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use prune_server::break_switches::BreakSwitches;
use prune_server::oversized_values::run_drop_oversized_values;
use prune_server::protocol::{EntryInTable, KeyInTable, Table};
use prune_server::storage::VerdictLibrary;

struct TemporaryDirectory(PathBuf);
impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("清理本次临时目录");
    }
}

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, here: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(here).expect("枚举临时库") {
            let path = entry.expect("目录项").path();
            if path.is_dir() {
                collect(root, &path, out);
            } else {
                out.insert(path.strip_prefix(root).unwrap().to_path_buf(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    collect(root, root, &mut out);
    out
}

fn main() {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let directory = std::env::temp_dir().join(format!("singlefs-document-audit-prune-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&directory).expect("只创建新的临时目录");
    let temporary = TemporaryDirectory(directory);
    let library_path = temporary.0.join("library");
    {
        let library = VerdictLibrary::open(&library_path, 1, BreakSwitches::none()).unwrap();
        let mut group = library.begin_group();
        library.add_write(&mut group, b"document-audit", 1, &[EntryInTable {
            table: Table::Conclusions, key: b"audit-small", value: b"unchanged-value",
        }]);
        library.commit_group(group).unwrap();
    }
    let before = files(&library_path);
    let exit_code = run_drop_oversized_values(&[
        "--state-directory".into(), temporary.0.to_str().unwrap().into(), "--dry-run".into(),
    ]);
    assert_eq!(exit_code, 0, "真实预演入口应成功");
    let after = files(&library_path);
    let names: BTreeSet<_> = before.keys().chain(after.keys()).collect();
    let changed: Vec<_> = names.into_iter().filter(|name| before.get(*name) != after.get(*name)).collect();
    for name in &changed {
        println!("AUDIT_DRY_RUN_FILE_CHANGED path={} before_bytes={:?} after_bytes={:?}",
            name.display(), before.get(*name).map(Vec::len), after.get(*name).map(Vec::len));
    }
    {
        let library = VerdictLibrary::open(&library_path, 1, BreakSwitches::none()).unwrap();
        let values = library.read_points(&[KeyInTable { table: Table::Conclusions, key: b"audit-small" }]).unwrap();
        assert_eq!(values[0].as_deref(), Some(b"unchanged-value".as_slice()));
    }
    println!("AUDIT_DRY_RUN_RESULT exit={exit_code} physical_files_changed={} fixture_value_unchanged=true", changed.len());
    assert!(!changed.is_empty(), "本次未复现预演改写内部文件，不能确认该差异");
}
