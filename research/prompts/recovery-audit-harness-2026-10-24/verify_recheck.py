#!/usr/bin/env python3
"""Original/control/original causal recheck, patching only a temporary harness copy."""
import hashlib
import difflib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SOURCE = ROOT / 'crates/singlefs-harness'

HOOKS = '''
    fn reread_at_after_a_bad_read(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.reread_at_after_a_bad_read(offset, buffer)
    }
    fn read_at_straight_from_the_disk(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at_straight_from_the_disk(offset, buffer)
    }
    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        self.inner.keep_the_self_verified_reread(offset, bytes)
    }
    fn note_the_read_failing_self_verification_and_forget_its_bytes(&self, offset: DeviceOffsetInBytes, length: u64) {
        self.inner.note_the_read_failing_self_verification_and_forget_its_bytes(offset, length)
    }
'''

def main():
    product_paths = sorted(p for name in ('singlefs-harness', 'singlefs-core', 'singlefs-format', 'singlefs-replay-prefix', 'singlefs-reference-filesystem', 'singlefs-checker') for p in (ROOT / 'crates' / name).rglob('*.rs') if 'audits' not in p.parts)
    hashes = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in product_paths}
    (HERE / 'evidence/recheck-source-sha256.txt').write_text(''.join(f'{digest}  {p.relative_to(ROOT)}\n' for p, digest in hashes.items()))
    with tempfile.TemporaryDirectory(prefix='singlefs-harness-causal-recheck-') as directory:
        work = Path(directory)
        harness = work / 'crates/harness'
        (work / '.claude').mkdir()
        shutil.copy2(ROOT / '.claude/heavy-thread-default', work / '.claude/heavy-thread-default')
        shutil.copytree(SOURCE / 'src', harness / 'src')
        original = {p.name: p.read_text() for p in (harness / 'src').glob('*.rs')}
        manifest = (SOURCE / 'Cargo.toml').read_text()
        for name in ('singlefs-core', 'singlefs-checker', 'singlefs-format', 'singlefs-reference-filesystem'):
            manifest = manifest.replace(f'path = "../{name}"', f'path = "{ROOT / "crates" / name}"')
        (harness / 'Cargo.toml').write_text(manifest)
        (work / 'src').mkdir()
        (work / 'src/lib.rs').write_text((HERE / 'probes.rs').read_text())
        (work / 'Cargo.toml').write_text(f'''[package]
name="singlefs-harness-causal-recheck"
version="0.0.0"
edition="2021"
[dependencies]
singlefs-harness={{path="{harness}"}}
singlefs-core={{path="{ROOT / 'crates/singlefs-core'}"}}
[profile.release]
overflow-checks=true
debug-assertions=true
''')
        patched = original.copy()
        for name, typename in [('lib.rs', 'RecordingBlockDevice'), ('fault_injection.rs', 'FaultInjectingBlockDevice'), ('read_tally.rs', 'ReadLoggingBlockDevice')]:
            needle = f'impl<Inner: BlockDevice> BlockDevice for {typename}<Inner> {{'
            assert patched[name].count(needle) == 1
            extra = HOOKS
            if name == 'read_tally.rs':
                extra += '''
    fn discard_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<singlefs_core::block_device::DiscardOutcome, BlockDeviceError> {
        self.inner.discard_at(offset, length)
    }
'''
            patched[name] = patched[name].replace(needle, needle + extra)
        needle = '''                self.judge_reported_ceiling(answer, *reported_ceiling, counts)?;'''
        assert patched['model.rs'].count(needle) == 1
        patched['model.rs'] = patched['model.rs'].replace(needle, '''                if answer.expected_read_back.is_some() {
                    return Err(ModelDisagreement::new(ModelDisagreementAspect::NotModeled, "audit: cold-start requires readback".into(), "audit: got publishes".into()));
                }
''' + needle)
        (HERE / 'evidence/temporary-control.patch').write_text(''.join(
            ''.join(difflib.unified_diff(original[name].splitlines(keepends=True), patched[name].splitlines(keepends=True), fromfile='original/src/' + name, tofile='temporary-control/src/' + name))
            for name in original if original[name] != patched[name]
        ))
        env = os.environ.copy()
        env['CARGO_TARGET_DIR'] = str(ROOT / 'target/harness-causal-recheck')
        env.pop('SINGLEFS_HARNESS_AUDIT_CONTROL', None)
        for label, sources, control in [('original-first', original, False), ('temporary-controls', patched, True), ('original-again', original, False)]:
            for name, source in sources.items():
                (harness / 'src' / name).write_text(source)
            current = env.copy()
            if control:
                current['SINGLEFS_HARNESS_AUDIT_CONTROL'] = '1'
            print('ROUND ' + label, flush=True)
            subprocess.run(['cargo', 'test', '--release', '--offline', '-j', '2', '--manifest-path', str(work / 'Cargo.toml'), '--lib', '--', '--nocapture', '--test-threads=1'], cwd=ROOT, env=current, check=True)
    for path, digest in hashes.items():
        assert hashlib.sha256(path.read_bytes()).hexdigest() == digest, f'Product changed: {path}'
    print('HARNESS_CAUSAL_RECHECK_COMPLETE original/control/original passed; product sources unchanged.', flush=True)

if __name__ == '__main__':
    main()
