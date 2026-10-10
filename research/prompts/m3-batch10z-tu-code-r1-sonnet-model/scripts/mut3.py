#!/usr/bin/env python3
"""mut.py <名字...>  在 tree-m 上套一个变异、编、跑入库用例文件、还原。名字见 MUTANTS。"""
import os, subprocess, sys, shutil, re, time
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet')
TREE = os.environ.get('TREE','tree-m')
T = W + '/' + TREE
S = T + '/crates/singlefs-core/src/'
R = '/home/fy5090/code/singlefs/research/scripts'
MS, AL, MT = 'mounted_session.rs', 'allocator.rs', 'mount.rs'
M = {}
def mut(name, f, old, new): M[name] = (f, old, new)
cnt1 = '                    self.empty_publishes_until_the_ring_slot_of_r_is_written,\n                    self.empty_publishes_until_the_ring_slot_of_r_is_written,\n'
mut('textCount1Zero', MS, cnt1, '                    0,\n                    self.empty_publishes_until_the_ring_slot_of_r_is_written,\n')
mut('textCount2Zero', MS, cnt1, '                    self.empty_publishes_until_the_ring_slot_of_r_is_written,\n                    0,\n')
mut('textReleasedMin', MS, '                        .map(|(_, released)| *released)\n                        .sum::<u64>(),', '                        .map(|(_, released)| *released)\n                        .min().unwrap_or(0),')
mut('textDeviceZero', MS, '.map(|(device, released)| format!("盘 {} 上 {released} 个", device.0))', '.map(|(device, released)| format!("盘 {} 上 {released} 个", device.0 * 0))')
mut('textRPlusOne', MS, '                txg_of_r.0\n            ),', '                txg_of_r.0 + 1\n            ),')
mut('textNoneSlotSwap', MS, '                self.ring_slot_of_r.region, self.ring_slot_of_r.slot\n            ),', '                self.ring_slot_of_r.slot, self.ring_slot_of_r.region\n            ),')
mut('releasedReversed', MS, '(*device, before.saturating_sub(after))', '(*device, after.saturating_sub(*before))')
mut('releasedZero', MS, '(*device, before.saturating_sub(after))', '(*device, 0)')
mut('releasedFirstDeviceOnly', MS, '(*device, before.saturating_sub(after))', '(*device, if device.0 == 0 { before.saturating_sub(after) } else { 0 })')
mut('walkSkipped', MS, '        RemainderOfTheUserChange::WalkTheQueue => {\n            let mut version_to_walk', '        RemainderOfTheUserChange::WalkTheQueue if true => Ok(()),\n        RemainderOfTheUserChange::WalkTheQueue => {\n            let mut version_to_walk')
mut('overlayIgnored', MS, '                Some(written) => {\n                    if let Some(run_start) = unwritten_run_start.take() {', '                Some(_written) if true => {\n                    unwritten_run_start.get_or_insert(block_index);\n                }\n                Some(written) => {\n                    if let Some(run_start) = unwritten_run_start.take() {')
mut('keepOffByOne', MS, '.insert(first_block + block_offset, block.to_vec());', '.insert(first_block + block_offset + 1, block.to_vec());')
mut('zeroesNotKept', MS, '        self.keep_in_memory(offset, &zeros);', '        let _ = &zeros;')
mut('writeNotKept', MS, '        self.keep_in_memory(offset, bytes);\n        Ok(())', '        let _ = bytes;\n        Ok(())')
mut('lengthUnaligned', MS, 'if !offset.0.is_multiple_of(block_bytes) || !length.is_multiple_of(block_bytes) {', 'if !offset.0.is_multiple_of(block_bytes) {')
mut('rangeEndLt', MS, 'Some(end) if end <= device_size => Ok(()),', 'Some(end) if end < device_size => Ok(()),')
mut('barrierErr', MS, '    fn barrier(&mut self) -> Result<(), BlockDeviceError> {\n        Ok(())\n    }\n\n    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {\n        self.physical_block_size', '    fn barrier(&mut self) -> Result<(), BlockDeviceError> {\n        Err(BlockDeviceError::ProbeFailed { attribute: "x" })\n    }\n\n    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {\n        self.physical_block_size')
mut('rehearseSkipsChange', MS, '            MountedSession::publish_the_change_raising_the_floor_for_space(\n                parameters,\n                &mut devices_keeping_writes_in_memory,\n                &mut rehearsed_allocator,\n                &mut rehearsed_version,\n                instance,\n                shadow_ledger,\n                change,\n            )\n            .map(|_published| ())', '            { let _ = (&mut rehearsed_allocator, &mut rehearsed_version, change, shadow_ledger); Ok::<(), UserChangeRefused>(()) }')
mut('frozenCheckDropped', MS, '        if let Some(frozen) = allocator.frozen_publish() {\n            return Err(\n                RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {\n                    checkpoint_txg: frozen.checkpoint_txg(),\n                    instance: frozen.instance(),\n                },\n            );\n        }\n        let start = match &*current {', '        let start = match &*current {')
mut('noIsoReport', MS, '    if isolated_slots_on_each_device\n        .iter()\n        .all(|(_, isolated)| *isolated == 0)', '    if isolated_slots_on_each_device\n        .iter()\n        .all(|(_, isolated)| *isolated == u64::MAX)')
mut('noIsoPush', MS, '        if slots_the_push_would_release_isolation_of\n            .iter()\n            .all(|(_, isolated)| *isolated == 0)', '        if slots_the_push_would_release_isolation_of\n            .iter()\n            .all(|(_, isolated)| *isolated == u64::MAX)')

def run(name):
    f, old, new = M[name]
    path = S + f
    src = open(path).read()
    if src.count(old) != 1:
        return f'{name}: 旧串命中 {src.count(old)} 次，跳过'
    shutil.copy(path, path + '.orig')
    try:
        open(path, 'w').write(src.replace(old, new, 1))
        env = dict(os.environ, CARGO_TARGET_DIR=W + '/target-' + TREE.split('-')[1])
        b = subprocess.run(['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '6G', 'bash', R + '/capped.sh', '8', 'cargo', 'test', '--release', '--offline', '-p', 'singlefs-harness', '--test', 'raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table', '--no-run'], cwd=T, env=env, capture_output=True, text=True)
        if b.returncode != 0:
            err = [l for l in b.stderr.splitlines() if l.startswith('error')][:2]
            return f'{name}: 编不过 {err}'
        exe = [l.split('(')[1].rstrip(')') for l in (b.stderr + b.stdout).splitlines() if 'Executable' in l and 'raise_floor' in l][0]
        r = subprocess.run(['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '6G', 'bash', R + '/capped.sh', '6', os.path.join(T, exe) if not exe.startswith('/') else exe], cwd=T, capture_output=True, text=True)
        out = r.stdout + r.stderr
        failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
        tail = [l for l in out.splitlines() if l.startswith('test result')][-1:] 
        return f'{name}: rc={r.returncode} 红 {len(failed)} 条 {" ".join(x[:70] for x in failed[:3])} | {tail}'
    finally:
        shutil.move(path + '.orig', path)

if __name__ == '__main__':
    names = sys.argv[1:] or list(M)
    for n in names:
        t = time.time()
        print(run(n), f'({time.time()-t:.0f}s)', flush=True)
