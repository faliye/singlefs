#!/usr/bin/env bash
# 攻方腿草稿：编差分探针跑一次，再搭三个证红变异体各编各跑一次。整条经内存包装起。
set -u
export ROCKSDB_LIB_DIR=/home/fy5090/.local/share/singlefs/rocksdb-prebuilt/librocksdb-sys-0.19.0+11.8.1/67873f27d6329162/lib
export ROCKSDB_STATIC=1
export SNAPPY_LIB_DIR=/home/fy5090/.local/share/singlefs/rocksdb-prebuilt/librocksdb-sys-0.19.0+11.8.1/67873f27d6329162/lib
export SNAPPY_STATIC=1
cd /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack
cargo build --offline --manifest-path /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-ws/Cargo.toml --target-dir /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/target-probe > /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-build.txt 2>&1
echo "probe build rc=$?"
/home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/target-probe/debug/attack-codec-differential 3000 > /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-out.txt 2>&1
echo "probe run rc=$?"
tail -n 3 /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-out.txt
python3 /home/fy5090/code/singlefs/research/prompts/m4-close-code-r1-opus-model/tools/make_mutant.py /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack every-byte-is-zero-skips-the-first-byte /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-ws/src /home/fy5090/code/singlefs/research/prompts/m4-close-code-r1-opus-model/probe/Cargo.toml
python3 /home/fy5090/code/singlefs/research/prompts/m4-close-code-r1-opus-model/tools/make_mutant.py /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack container-tree-grows-only-past-the-fanout /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-ws/src /home/fy5090/code/singlefs/research/prompts/m4-close-code-r1-opus-model/probe/Cargo.toml
python3 /home/fy5090/code/singlefs/research/prompts/m4-close-code-r1-opus-model/tools/make_mutant.py /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack inode-leaf-borrow-rounds-up /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-ws/src /home/fy5090/code/singlefs/research/prompts/m4-close-code-r1-opus-model/probe/Cargo.toml
cargo build --offline --manifest-path /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-mut-every-byte-is-zero-skips-the-first-byte/Cargo.toml --target-dir /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/target-mut > /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/mut-build-every-byte.txt 2>&1
echo "mutant every-byte-is-zero-skips-the-first-byte build rc=$?"
/home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/target-mut/debug/attack-codec-differential 1000 > /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/mut-out-every-byte-is-zero-skips-the-first-byte.txt 2>&1
echo "mutant every-byte-is-zero-skips-the-first-byte run rc=$?"
cargo build --offline --manifest-path /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-mut-container-tree-grows-only-past-the-fanout/Cargo.toml --target-dir /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/target-mut > /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/mut-build-container.txt 2>&1
echo "mutant container-tree-grows-only-past-the-fanout build rc=$?"
/home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/target-mut/debug/attack-codec-differential 1000 > /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/mut-out-container-tree-grows-only-past-the-fanout.txt 2>&1
echo "mutant container-tree-grows-only-past-the-fanout run rc=$?"
cargo build --offline --manifest-path /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/probe-mut-inode-leaf-borrow-rounds-up/Cargo.toml --target-dir /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/target-mut > /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/mut-build-inode.txt 2>&1
echo "mutant inode-leaf-borrow-rounds-up build rc=$?"
/home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/target-mut/debug/attack-codec-differential 1000 > /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/mut-out-inode-leaf-borrow-rounds-up.txt 2>&1
echo "mutant inode-leaf-borrow-rounds-up run rc=$?"
grep -h '^name=' /home/fy5090/.local/share/singlefs/work/m4-close-code-r1-attack/mut-out-*.txt
