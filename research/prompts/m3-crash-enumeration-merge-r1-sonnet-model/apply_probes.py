import re, sys, pathlib
root = pathlib.Path(sys.argv[1]) / "crates/singlefs-checker-tier/tests"
support = root / "zz_probe_support"
support.mkdir(exist_ok=True)
(support / "mod.rs").write_text('''#![allow(dead_code)]
use singlefs_checker_tier::crash::{layer0_plan_state_count, quick_tier_expansion, Layer0SegmentExpansion};
use singlefs_checker_tier::crash_identity::{step_write_table_with_barriers, SectorImage};
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn probe(label: &str, base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>]) {
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    let table = step_write_table_with_barriers(writes, segments, 0..writes.len());
    let base_digest = SectorImage::of_base(base).digest();
    let at = |limit: usize| {
        layer0_plan_state_count(base, writes, segments, &move |_i, s: &[usize]| {
            if s.len() <= limit {
                Layer0SegmentExpansion::EveryProperSubset
            } else {
                Layer0SegmentExpansion::NotExpanded
            }
        })
    };
    let quick = layer0_plan_state_count(base, writes, segments, &quick_tier_expansion);
    println!(
        "ZZPROBE label={label} writes={} segments={} sizes={:?} table_digest={} base_digest={} states6={} states28={} states30={} states_full={} states_quick_tier_jia2={}",
        writes.len(), segments.len(), sizes, hex(&table), hex(&base_digest), at(6), at(28), at(30), at(1000), quick
    );
}
''')

def edit(name, fn):
    p = root / name
    s = p.read_text()
    s2 = fn(s)
    assert s2 != s, name
    p.write_text(s2)

PATHMOD = '#[path = "zz_probe_support/mod.rs"]\nmod zz_probe_support;\n'

# amplification common
def common(s):
    s = s.replace("    let limit = ExpandUpToWritesPerSegment::from_environment_or(default_expand_up_to);\n    let expansion = move",
      "    zz_probe(flow_name, base, writes, segments);\n    let limit = ExpandUpToWritesPerSegment::from_environment_or(default_expand_up_to);\n    let expansion = move", 1)
    s += '''
#[path = "../zz_probe_support/mod.rs"]
mod zz_probe_support;
fn zz_probe(label: &str, base: &singlefs_harness::memory_pool::MemoryPool, writes: &[singlefs_harness::memory_pool::RetainedWrite], segments: &[Vec<usize>]) {
    zz_probe_support::probe(&format!("AMP {label}"), base, writes, segments);
}
'''
    return s
edit("common_crash_points/mod.rs", common)

for name, label in [("crash_amplification_fixed_script_through_unmount.rs","AMP through-unmount"),("crash_amplification_fixed_script_through_rollback.rs","AMP through-rollback")]:
    def f(s, label=label):
        s = s.replace("    let plan = plan_crash_points(&flow);\n", "    zz_probe_support::probe(\"%s\", &base, &writes, &segments);\n    let plan = plan_crash_points(&flow);\n" % label, 1)
        s = s.replace("mod common_crash_points;\n", "mod common_crash_points;\n" + PATHMOD, 1)
        return s
    edit(name, f)

# layer0 side
def layer0(name, test_src, need_path=True):
    def f(s):
        if need_path:
            # insert after first 'mod common;' line
            s = re.sub(r'(mod common;\n)', r'\1' + PATHMOD, s, count=1)
        return s + "\n" + test_src
    edit(name, f)

layer0("crash_enumeration_multi_record_publish_stream.rs",
 '#[test]\nfn zz_probe_layer0_multi_record() { let p = prepare("zz"); zz_probe_support::probe("L0 multi-record", &p.base, &p.writes, &p.segments); }\n')
layer0("crash_enumeration_unlink_after_the_multi_record_publish_stream.rs",
 '#[test]\nfn zz_probe_layer0_unlink() { let p = prepare("zz"); zz_probe_support::probe("L0 unlink", &p.base, &p.writes, &p.segments); }\n')
layer0("crash_enumeration_fixed_script_stream.rs",
 '#[test]\nfn zz_probe_layer0_second_stream() { let p = prepare("zz", Script::ReuseAfterRaisingFloorThenNormalUnmount); zz_probe_support::probe("L0 second-stream", &p.base, &p.writes, &p.segments);\n let q = prepare("zz2", Script::RollbackToFirstVersion); zz_probe_support::probe("L0 through-rollback", &q.base, &q.writes, &q.segments); }\n')
layer0("crash_enumeration_new_pool_file_creation_stream.rs",
 '#[test]\nfn zz_probe_layer0_first() { let p = prepare("zz"); zz_probe_support::probe("L0 first-stream", &p.base, &p.writes, &p.segments); }\n')
layer0("crash_enumeration_position_addressed_trees.rs",
 '#[test]\nfn zz_probe_layer0_pat() { for stream in PositionAddressedStream::ALL { let p = prepare(stream); zz_probe_support::probe(&format!("L0 pat {stream:?}"), &p.base, &p.writes, &p.segments); } }\n')
print("ok")
