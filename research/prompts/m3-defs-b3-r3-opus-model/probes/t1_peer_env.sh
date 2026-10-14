#!/usr/bin/env bash
# T1-1：capped.sh N 设的线程变量到不了第二台。multi-host-run.sh --peer（host-dispatch.py 挑中第二台时走它）经 peer-host-lib.sh 的 peer_host_run 起命令，
# 环境只给最少几样（peer-host-lib.sh 文件头「只供测试的开关」一段、multi-host-run.sh 文件头「带到第二台的环境」一句）。
# 这里用 peer-host-lib.sh 自己的「第二台是本机」模式（peer_host_is_this_machine=1，与 ssh 同一种最小环境）在本机演示，不连第二台、不读多机配置。
# 用法：bash t1_peer_env.sh <树> <标签>
set -uo pipefail
tree="$1" label="$2"
cd "$tree" || exit 2
bash scripts/hosts/capped.sh 31 bash -c '
  source scripts/hosts/peer-host-lib.sh
  peer_host_is_this_machine=1 PEER_CARGO_BIN_DIRECTORY=/nonexistent
  echo "name=peer-env tree='"$label"' side=local RUST_TEST_THREADS=${RUST_TEST_THREADS:-unset} CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-unset} SINGLEFS_THREAD_CAP=${SINGLEFS_THREAD_CAP:-unset}"
  peer_host_run / "echo name=peer-env tree='"$label"' side=peer RUST_TEST_THREADS=\${RUST_TEST_THREADS:-unset} CARGO_BUILD_JOBS=\${CARGO_BUILD_JOBS:-unset} SINGLEFS_THREAD_CAP=\${SINGLEFS_THREAD_CAP:-unset}"
'
grep -n "带到第二台的环境" scripts/hosts/multi-host-run.sh | sed "s/^/name=peer-env-doc tree=$label multi-host-run.sh line=/" | cut -d: -f1
