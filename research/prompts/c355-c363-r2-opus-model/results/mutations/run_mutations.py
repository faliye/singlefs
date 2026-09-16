import os, shutil, subprocess, sys
SRC='/home/fy5090/code/singlefs/research/prompts/c355-c363-r2-opus-model'
BASE='/tmp/claude-1000/c355-c363-r2-opus/mutants'
TARGET='/tmp/claude-1000/c355-c363-r2-opus/mutant-target'
mutations = [
 ('M1','v2_per_device','src/bin/v2_per_device.rs',
  '    free.iter().all(|slots| *slots >= DATA_SLOTS_PER_COPY * request_units + cost + TOMBSTONE_SLOTS_PER_COPY * containers)',
  '    free.iter().all(|slots| *slots >= DATA_SLOTS_PER_COPY * request_units + cost)'),
 ('M2','v2_per_device','src/bin/v2_per_device.rs',
  '    order.sort_by(|left, right| free[*right].cmp(&free[*left]).then(left.cmp(right)));\n    [order[0], order[1]]',
  '    order.sort_by(|left, right| free[*left].cmp(&free[*right]).then(left.cmp(right)));\n    [order[0], order[1]]'),
 ('M3','v3_empty_publish','src/bin/v3_empty_publish.rs',
  '            NodeId::InTree(TreeName::Extent | TreeName::Allocation | TreeName::Accounting, _) | NodeId::InodeLeafContainer | NodeId::InodeRoot => true,',
  '            NodeId::InTree(TreeName::Extent | TreeName::Allocation | TreeName::Accounting, _) | NodeId::InodeLeafContainer | NodeId::InodeRoot => false,'),
 ('M4','v3_empty_publish','src/bin/v3_empty_publish.rs',
  '        let index = index_of(slot);\n        for device in 0..DEVICES {',
  '        let index = index_of(slot);\n        for device in 0..1 {'),
 ('M4b','v3_empty_publish','src/bin/v3_empty_publish.rs',
  'ALLOCATION_RECORD_DEVICES_PLACEHOLDER', ''),
 ('M5','v3_empty_publish','src/bin/v3_empty_publish.rs',
  '                self.release_slots(old.slot, old.span);\n                self.record_release(&mut dirty, old.slot);',
  '                self.release_slots(old.slot, old.span);'),
 ('M6','v4_df','src/bin/v4_df.rs',
  '    (0..2).all(|device| state.unused[device] + state.pinned >= DATA_SLOTS_PER_COPY * state.request_units + state.cost)',
  '    true'),
 ('M7','v4_df','src/bin/v4_df.rs',
  'fn d16_residue(cost: i64) -> i64 { 5 + 6 * cost }',
  'fn d16_residue(cost: i64) -> i64 { let _ = cost; 0 }'),
]
only = sys.argv[1:] or [m[0] for m in mutations]
for name, binary, path, old, new in mutations:
    if name not in only: continue
    work = os.path.join(BASE, name)
    if os.path.isdir(work):
        shutil.rmtree(work)
    shutil.copytree(SRC, work, ignore=shutil.ignore_patterns('results'))
    file = os.path.join(work, path)
    text = open(file, encoding='utf-8').read()
    if name == 'M4b':
        sites = [('        let index = index_of(slot);\n        for device in 0..DEVICES {', '        let index = index_of(slot);\n        for device in 0..1 {'),
                 ('        for device in 0..DEVICES {\n            for node in self.allocation.touch(allocation_key(device, slot))', '        for device in 0..1 {\n            for node in self.allocation.touch(allocation_key(device, slot))')]
        for site_old, site_new in sites:
            assert text.count(site_old) == 1, site_old
            text = text.replace(site_old, site_new)
        old, new = text, text
    count = text.count(old)
    if count != 1:
        print(f'{name} ANCHOR_COUNT={count}'); continue
    open(file, 'w', encoding='utf-8').write(text.replace(old, new))
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET + '-' + name)
    build = subprocess.run(['cargo', 'build', '--release', '--bin', binary], cwd=work, env=env, capture_output=True, text=True)
    if build.returncode != 0:
        print(f'{name} BUILD_FAILED'); print(build.stderr[-800:]); continue
    run = subprocess.run([TARGET + '-' + name + '/release/' + binary], capture_output=True, text=True)
    out_path = os.path.join(BASE, name + '.out')
    open(out_path, 'w').write(run.stdout)
    open(os.path.join(BASE, name + '.err'), 'w').write(run.stderr)
    panic = [l for l in run.stderr.splitlines() if 'panicked' in l or 'assertion' in l or 'left:' in l or 'right:' in l]
    print(f'{name} exit={run.returncode} stderr_head={panic[:4]}')
