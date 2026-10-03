#!/usr/bin/env bash
# 第二轮攻方（Opus）U1/U2 探针：只喂本仓 heavy-test-guard.sh JSON、看退出码；被判命令一条都不起。
# 造的「坏树」全在草稿目录 <S>/ 下，主工作区只读。用法：probes.sh <草稿目录>
set -uo pipefail
S="$1"; here="$(cd "$(dirname "$0")" && pwd)"; R="$(cd "$here/../../.." && pwd)"
D="$S/detections.jsonl"; mkdir -p "$S"; : > "$D"
p(){ python3 "$here/hook-probe.py" "$D" "$@"; }

echo "## U1-a 不带 --item 的 stage-selftest.sh 只按名字认：改过的非本仓拷贝喂字节相同的轻样本目录，放行、正文不读"
E="$S/evil-stage-selftest"; rm -rf "$E"
mkdir -p "$E/.claude/singlefs-ai-sop/scripts" "$E/gate.d/fixtures"
cp "$R/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$E/.claude/singlefs-ai-sop/scripts/"
printf '\n# EVIL: SINGLEFS_HEAVY_TESTS=commit cargo test --release -p singlefs-checker-tier --lib --tests\n' >> "$E/.claude/singlefs-ai-sop/scripts/stage-selftest.sh"
cp "$R/.claude/gate.d/doc-text.sh" "$E/gate.d/"
cp -a "$R/.claude/gate.d/fixtures/doc-text.sh" "$E/gate.d/fixtures/"
echo "改过的拷贝与本仓那一份是否逐字节不同："; diff -q "$R/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$E/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" || true
echo "拷贝末尾那一行（本该起真 checker 档）："; tail -1 "$E/.claude/singlefs-ai-sop/scripts/stage-selftest.sh"
p implementation-writer "$R" "bash $E/.claude/singlefs-ai-sop/scripts/stage-selftest.sh $E/gate.d"
echo "对照 1：同一个改过的拷贝带 --item（本仓那一份的 own-copy 检查应拒）"
p implementation-writer "$R" "bash $E/.claude/singlefs-ai-sop/scripts/stage-selftest.sh $E/gate.d --item doc-text.sh"
echo "对照 2：改过的拷贝喂与本仓暂存区不同的样本（样本不符应拒）"
E2="$S/evil-stage-badsample"; rm -rf "$E2"
mkdir -p "$E2/.claude/singlefs-ai-sop/scripts" "$E2/gate.d/fixtures/doc-text.sh/red"
cp "$R/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$E2/.claude/singlefs-ai-sop/scripts/"
cp "$R/.claude/gate.d/doc-text.sh" "$E2/gate.d/"
printf 'exit=1\nwant=x\n' > "$E2/gate.d/fixtures/doc-text.sh/red/expect"
p implementation-writer "$R" "bash $E2/.claude/singlefs-ai-sop/scripts/stage-selftest.sh $E2/gate.d"

echo "## U1-b 通配命令词的 --selftest 不当自证：登记成轻的也被当没登记而拒（方向安全，但看有没有误拒）"
p implementation-writer "$R" "bash research/scripts/admission*.py --selftest"
p implementation-writer "$R" "python3 research/scripts/crates-mutation-rows.p? --selftest"

echo "## U2-a 54 号样本档：假 rustc 不核内容（字节相同的假 cargo + 任意可执行 rustc），看放行与否"
G="$S/evil-54"; rm -rf "$G"; mkdir -p "$G/.layer0-sample-tools/cases"
cp "$R/.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/green/.layer0-sample-tools/cargo" "$G/.layer0-sample-tools/cargo"
printf '#!/bin/sh\necho EVIL-RUSTC\n' > "$G/.layer0-sample-tools/rustc"; chmod 755 "$G/.layer0-sample-tools/rustc" "$G/.layer0-sample-tools/cargo"
( cd "$G" && p three-way-attack "$G" "bash $R/.claude/gate.d/checker-tier-layer0-replay.sh $G" )
