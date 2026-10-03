#!/usr/bin/env bash
# 喂重型测试闸（仓拷贝里那一份）的几组 JSON，打退出码：54 号样本标记、仓内新样本直接起、自证登记按路径尾部认、样本的 setup.sh。
# 用法：hook-probes.sh <仓拷贝根> <草稿目录>；只在 <草稿目录> 下建目录与文件，不起被判的命令。复跑挂钟：约 20 秒。
set -uo pipefail
R="$1"; S="$2"; here="$(cd "$(dirname "$0")" && pwd)"; D="$S/detections.jsonl"
probe() { python3 "$here/hook-probe.py" "$R" "$D" "$@"; }
echo "## A 54 号：整份仓拷贝，标记目录在不在"
rm -rf "$R/.layer0-sample-tools"
probe implementation-writer "$R" "bash .claude/gate.d/checker-tier-layer0-replay.sh $R"
mkdir -p "$R/.layer0-sample-tools" && : > "$R/.layer0-sample-tools/quick-tier"
probe implementation-writer "$R" "bash .claude/gate.d/checker-tier-layer0-replay.sh $R"
probe implementation-writer "$R" "bash .claude/gate.d/checker-tier-layer0-replay.sh --full $R"
probe three-way-attack "$R" "bash .claude/gate.d/checker-tier-layer0-replay.sh $R"
( cd "$R" && PATH="$PWD/.layer0-sample-tools:$PATH" && echo "标记目录插到 PATH 最前面之后 cargo 解到：$(type -P cargo)" )
rm -rf "$R/.layer0-sample-tools"
echo "## B 仓内样本 quick-features-green 照拒绝说明直接起"
F="$R/.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-green"
ls -l "$F/.layer0-sample-tools/cargo" "$F/.layer0-sample-tools/rustc"
probe tooling-writer "$F" "bash $R/.claude/gate.d/checker-tier-layer0-replay.sh ."
( cd "$F" && PATH="$PWD/.layer0-sample-tools:$PATH" && echo "样本的假工具插到 PATH 最前面之后 cargo 解到：$(type -P cargo)" )
echo "## C 自证登记按路径尾部认、不读正文"
T="$S/evil-tree/research/scripts"; mkdir -p "$T"
printf '#!/usr/bin/env bash\n# 改过的自证\ncargo test --release -p singlefs-checker-tier --lib --tests\n' > "$T/claim-experiment.sh.new" && mv "$T/claim-experiment.sh.new" "$T/claim-experiment.sh"
cp "$T/claim-experiment.sh" "$T/my-new-check.sh.new" && mv "$T/my-new-check.sh.new" "$T/my-new-check.sh"
probe implementation-writer "$R" "bash $T/claim-experiment.sh"
probe implementation-writer "$R" "bash $T/claim-experiment.sh --selftest"
probe implementation-writer "$R" "bash $T/my-new-check.sh --selftest"
echo "## D 样本档：阶段正文逐字节核，样本里的 setup.sh 不读"
E="$S/evil-gate/.claude/gate.d"; mkdir -p "$E/fixtures/doc-text.sh/red"
cp "$R/.claude/gate.d/doc-text.sh" "$E/doc-text.sh"
printf '#!/usr/bin/env bash\ncargo test --release -p singlefs-checker-tier --lib --tests\n' > "$E/fixtures/doc-text.sh/red/setup.sh.new" && mv "$E/fixtures/doc-text.sh/red/setup.sh.new" "$E/fixtures/doc-text.sh/red/setup.sh"
printf 'exit=1\nwant=x\n' > "$E/fixtures/doc-text.sh/red/expect"
probe implementation-writer "$R" "bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $E"
probe implementation-writer "$R" "bash $E/fixtures/doc-text.sh/red/setup.sh"
echo "## F 真仓的阶段目录喂 stage-selftest.sh（有 samples-heavy 的三道）"
probe tooling-writer "$R" "bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d"
