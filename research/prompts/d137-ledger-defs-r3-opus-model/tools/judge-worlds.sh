#!/usr/bin/env bash
# 在 make-worlds.sh 造出的世界根上单跑门禁 doc-experiments 的两格（judge-entries-in-sync、preregistration-judge-features），逐个打退出码；
# 门禁输出写进 <世界目录>/<世界名>.<格名>.out。期望与「照 Rust、bash 的语义该判成什么」在 <模型目录>/out/expected.txt 第二节。
# 用法（在仓根起）：bash <模型目录>/tools/judge-worlds.sh <make-worlds.sh 造出的目录>
# 退出码：0 都跑了（不判对错）；2 不在仓根或缺参数。
set -uo pipefail
worlds="${1:?要给 make-worlds.sh 造出的目录}"
[[ -f .claude/gate.d/doc-experiments.sh ]] || { echo "  ✗ 要在仓根起（找不到 .claude/gate.d/doc-experiments.sh）" >&2; exit 2; }
echo "name=judge_inputs gate_sha256=$(sha256sum .claude/gate.d/doc-experiments.sh | cut -c1-8)"
report() {   # report <世界名> <格名> <退出码>
  echo "name=judge_world world=$1 cell=$2 rc=$3 last=$(grep -E '✓|✗|!' "$worlds/$1.$2.out" | grep -v '汇总' | tail -n 1 | cut -c1-260)"
}
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=攻方世界：新公开入口签名里带数组参数，核判段入口表同步格认不认得出' bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync "$worlds/sync-array-parameter-new-entry" > "$worlds/sync-array-parameter-new-entry.judge-entries-in-sync.out" 2>&1
report sync-array-parameter-new-entry judge-entries-in-sync $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=攻方世界的对照：同一个新入口参数写成切片，核判段入口表同步格判红' bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync "$worlds/sync-slice-parameter-new-entry-control" > "$worlds/sync-slice-parameter-new-entry-control.judge-entries-in-sync.out" 2>&1
report sync-slice-parameter-new-entry-control judge-entries-in-sync $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=攻方世界：判段路径上的私有函数签名里带数组参数，核同步格会不会把七个入口判成认不出' bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync "$worlds/sync-array-parameter-on-the-path" > "$worlds/sync-array-parameter-on-the-path.judge-entries-in-sync.out" 2>&1
report sync-array-parameter-on-the-path judge-entries-in-sync $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=攻方世界的对照：路径上的私有函数参数写成切片，核同步格判绿' bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync "$worlds/sync-slice-parameter-on-the-path-control" > "$worlds/sync-slice-parameter-on-the-path-control.judge-entries-in-sync.out" 2>&1
report sync-slice-parameter-on-the-path-control judge-entries-in-sync $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=攻方世界：新公开入口与表里的入口互相调、写在它后面，核同步格的记忆化会不会漏报' bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync "$worlds/sync-cycle-new-entry-after" > "$worlds/sync-cycle-new-entry-after.judge-entries-in-sync.out" 2>&1
report sync-cycle-new-entry-after judge-entries-in-sync $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=攻方世界的对照：同样互相调、新入口写在前面，核同步格判红' bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync "$worlds/sync-cycle-new-entry-before-control" > "$worlds/sync-cycle-new-entry-before-control.judge-entries-in-sync.out" 2>&1
report sync-cycle-new-entry-before-control judge-entries-in-sync $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 judge-entries-in-sync 格; 范围=1 格; 为什么=攻方世界：装置调签名带数组的新入口，核同步格这一半' bash .claude/gate.d/doc-experiments.sh --check judge-entries-in-sync "$worlds/chain-array-parameter-unjudged" > "$worlds/chain-array-parameter-unjudged.judge-entries-in-sync.out" 2>&1
report chain-array-parameter-unjudged judge-entries-in-sync $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 preregistration-judge-features 格; 范围=1 格; 为什么=攻方世界：装置调签名带数组的新入口、登记没带 gpu，核跑前登记那一格这一半' bash .claude/gate.d/doc-experiments.sh --check preregistration-judge-features "$worlds/chain-array-parameter-unjudged" > "$worlds/chain-array-parameter-unjudged.preregistration-judge-features.out" 2>&1
report chain-array-parameter-unjudged preregistration-judge-features $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 preregistration-judge-features 格; 范围=1 格; 为什么=攻方世界：列表项里行首是命令的行，行尾 shell 注释写着 --features gpu，核注释截不截' bash .claude/gate.d/doc-experiments.sh --check preregistration-judge-features "$worlds/prereg-list-item-trailing-comment" > "$worlds/prereg-list-item-trailing-comment.preregistration-judge-features.out" 2>&1
report prereg-list-item-trailing-comment preregistration-judge-features $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 preregistration-judge-features 格; 范围=1 格; 为什么=攻方世界：表格整格是命令，格里行尾 shell 注释写着 --features gpu，核注释截不截' bash .claude/gate.d/doc-experiments.sh --check preregistration-judge-features "$worlds/prereg-table-cell-trailing-comment" > "$worlds/prereg-table-cell-trailing-comment.preregistration-judge-features.out" 2>&1
report prereg-table-cell-trailing-comment preregistration-judge-features $?
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 的 preregistration-judge-features 格; 范围=1 格; 为什么=攻方世界的对照：同一条命令写在围栏里，核注释截掉之后判红' bash .claude/gate.d/doc-experiments.sh --check preregistration-judge-features "$worlds/prereg-fence-trailing-comment-control" > "$worlds/prereg-fence-trailing-comment-control.preregistration-judge-features.out" 2>&1
report prereg-fence-trailing-comment-control preregistration-judge-features $?
