#!/usr/bin/env python3
"""复核完整版本身份、结论契约与撕裂抽象；在新临时工程运行公开 API 探针。"""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p/'crates/singlefs-checker-tier/Cargo.toml').exists())

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verus',action='store_true')
    args=parser.parse_args()
    work=Path(tempfile.mkdtemp(prefix='singlefs-tier-recheck-'))
    (work/'src').mkdir()
    packages=['singlefs-checker-tier','singlefs-checker','singlefs-core','singlefs-harness','singlefs-format']
    text='[package]\nname="checker-public-audit"\nversion="0.0.0"\nedition="2021"\n[dependencies]\n'
    text+=''.join(name+'={path="'+str(ROOT/'crates'/name)+'"}\n' for name in packages)
    text+='[profile.release]\nopt-level=3\ndebug=true\noverflow-checks=true\ndebug-assertions=true\n'
    (work/'Cargo.toml').write_text(text)
    shutil.copy(HERE/'evidence/probe.Cargo.lock',work/'Cargo.lock')
    source=(HERE/'verdict_encoding_and_torn_state_probes.rs').read_text()
    test_source=(ROOT/'crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs').read_text()
    start=test_source.index('    fn note_the_invariants_held(')
    end=test_source.index('\n    }\n',start)+7
    coverage_method=test_source[start:end]
    source=source.replace('__COMMON_FIXTURE_PATH__',str(ROOT/'crates/singlefs-harness/tests/common/mod.rs')).replace('__COVERAGE_METHOD__',coverage_method)
    (work/'src/main.rs').write_text(source)
    (HERE/'evidence/coverage-method-extracted.rs').write_text(coverage_method)

    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(ROOT/'target')
    if not env.get('ROCKSDB_LIB_DIR'):
        libs=sorted((Path.home()/'.local/share/singlefs/rocksdb-prebuilt').glob('*/*/lib/librocksdb.a'))
        if len(libs)==1:
            lib=str(libs[0].parent)
            env.update(ROCKSDB_LIB_DIR=lib,ROCKSDB_STATIC='1',SNAPPY_LIB_DIR=lib,SNAPPY_STATIC='1')
    print('临时目录：',work,flush=True)
    subprocess.run(['cargo','build','--release','--offline','--manifest-path',str(work/'Cargo.toml')],cwd=ROOT,env=env,check=True)
    executable=ROOT/'target/release/checker-public-audit'
    for index in range(3):
        print(f'ROUND fresh-process-{index+1}',flush=True)
        subprocess.run(['cargo','run','--release','--offline','--manifest-path',str(work/'Cargo.toml')],cwd=ROOT,env=env,check=True)
    if args.verus:
        verus=Path(env.get('SINGLEFS_VERUS_ROOT',str(Path.home()/'.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux')))/'verus'
        with (HERE/'evidence/formal-kernel-recheck.json').open('w') as stdout, (HERE/'evidence/formal-kernel-recheck.stderr').open('w') as stderr:
            subprocess.run([str(verus),str(ROOT/'crates/singlefs-replay-prefix/src/lib.rs'),'--crate-type=lib','--no-cheating','--output-json','--rlimit','10','--num-threads','2'],cwd=work,env=env,stdout=stdout,stderr=stderr,check=True)
    print('TIER_RECHECK_COMPLETE 三个独立进程及正反对照通过。',flush=True)

if __name__=='__main__': main()
