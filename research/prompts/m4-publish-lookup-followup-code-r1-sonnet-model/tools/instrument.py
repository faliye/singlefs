#!/usr/bin/env python3
"""在自己的源码副本里给发布路径上几处线性找加比较次数计数，并在测试文件末尾追加一条读数用例。只写 --tree。
用法：instrument.py --tree <源码副本>
每处替换的旧串必须恰好命中一次，否则退 2、一个字节不写。读数用例在 ../probe/probe_test.rs。"""
import argparse, pathlib, sys

ap = argparse.ArgumentParser()
ap.add_argument("--tree", required=True)
tree = pathlib.Path(ap.parse_args().tree)
model = pathlib.Path(__file__).resolve().parent.parent
core = tree / "crates/singlefs-core/src/transaction.rs"
text = core.read_text()

STATICS = """use lookup_work::LookupSite;
use sorted_identity_index::SortedIdentityIndex;

pub static PROBE_UNITS_LEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PROBE_REWRITTEN_LEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PROBE_WRITTEN_FIND: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PROBE_NAMED_FIND: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PROBE_ROLES_NAMED_CONTAINS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PROBE_KEPT_CONTAINS_RELEASE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PROBE_KEPT_CONTAINS_MAPPED_ROLES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
"""

subs = [
 ("statics",
  "use lookup_work::LookupSite;\nuse sorted_identity_index::SortedIdentityIndex;\n", STATICS),
 ("lens",
  "    let written_units: Vec<&PublishedUnit> = relocated_copies\n",
  "    PROBE_UNITS_LEN.store(units.len() as u64, std::sync::atomic::Ordering::Relaxed);\n"
  "    PROBE_REWRITTEN_LEN.store(rewritten.len() as u64, std::sync::atomic::Ordering::Relaxed);\n"
  "    let written_units: Vec<&PublishedUnit> = relocated_copies\n"),
 ("written_find",
  "                .find(|unit| unit.identity == *identity)\n                .expect(\"重写的角色都装了单元\")",
  "                .find(|unit| {\n                    PROBE_WRITTEN_FIND.fetch_add(1, std::sync::atomic::Ordering::Relaxed);\n"
  "                    unit.identity == *identity\n                })\n                .expect(\"重写的角色都装了单元\")"),
 ("named_find",
  "            .find(|unit| unit.identity == identity)\n            .expect(\"点名的角色都是这次重写、装过单元的\");",
  "            .find(|unit| {\n                PROBE_NAMED_FIND.fetch_add(1, std::sync::atomic::Ordering::Relaxed);\n"
  "                unit.identity == identity\n            })\n            .expect(\"点名的角色都是这次重写、装过单元的\");"),
 ("roles_named_contains",
  "            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n",
  "            .filter(|identity| {\n                !data_roles_named_before_the_last_record.iter().any(|named| {\n"
  "                    PROBE_ROLES_NAMED_CONTAINS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);\n"
  "                    named == identity\n                })\n            })\n"),
 ("kept_contains_release",
  "    for identity in roles_replaced_via_mapping(previous, roles, file_content)\n        .into_iter()\n"
  "        .filter(|identity| !kept_for_the_snapshots.contains(identity))\n    {\n",
  "    for identity in roles_replaced_via_mapping(previous, roles, file_content)\n        .into_iter()\n"
  "        .filter(|identity| {\n            !kept_for_the_snapshots.iter().any(|kept| {\n"
  "                PROBE_KEPT_CONTAINS_RELEASE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);\n"
  "                kept == identity\n            })\n        })\n    {\n"),
 ("kept_contains_mapped_roles",
  "        .into_iter()\n        .filter(|identity| !kept_for_the_snapshots.contains(identity))\n        .filter(|identity| match identity {\n",
  "        .into_iter()\n        .filter(|identity| {\n            !kept_for_the_snapshots.iter().any(|kept| {\n"
  "                PROBE_KEPT_CONTAINS_MAPPED_ROLES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);\n"
  "                kept == identity\n            })\n        })\n        .filter(|identity| match identity {\n"),
]
for name, old, new in subs:
    n = text.count(old)
    if n != 1:
        sys.exit(f"instrument: {name} 旧串命中 {n} 次（要恰好 1 次），一个字节没写")
    text = text.replace(old, new, 1)
core.write_text(text)
test_file = tree / "crates/singlefs-harness/tests/publish_lookup_by_identity.rs"
test_file.write_text(
    test_file.read_text() + "\n" + (model / "probe/probe_test.rs").read_text()
    + "\n" + (model / "probe/probe_test_relocation.rs").read_text()
)
print("instrumented", len(subs), "sites")
