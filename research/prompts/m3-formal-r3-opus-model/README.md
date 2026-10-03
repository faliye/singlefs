# m3-formal-r3 攻方腿（Opus）的模型

全部在冻结副本 `/tmp/claude-1000/m3-formal-r3/tree/` 的拷贝上跑；数是副本上的，不是入库装置上的。

## 复跑

```
S=<草稿目录>
rsync -a --exclude target --exclude .git /tmp/claude-1000/m3-formal-r3/tree/ $S/tree/        # 打补丁的那份
rsync -a --exclude target --exclude .git /tmp/claude-1000/m3-formal-r3/tree/ $S/tree-base/   # 原样的那份
M=research/prompts/m3-formal-r3-opus-model
python3 $M/patch_attack.py $S/tree            # 读同目录的 e165_attack_functions.rs；每处原文恰好命中一次
# 编（两份各一个 target 目录）：
(cd $S/tree-base && bash research/scripts/run-with-memory-cap.sh 3G bash research/scripts/capped.sh 8 cargo build --release --offline -p singlefs-checker-tier --bin e165_replay_bounded_enumeration --target-dir $S/target-base)
(cd $S/tree && bash research/scripts/run-with-memory-cap.sh 3G bash research/scripts/capped.sh 8 cargo build --release --offline -p singlefs-checker-tier --bin e165_replay_bounded_enumeration --target-dir $S/target)
# 原样装置：light 两族、segment1 对抗一族 A 域（与 research/results/ 的产物逐字节比，见 runs/base-equivalence.txt）
$S/target-base/release/e165_replay_bounded_enumeration --bounds light --family legal
$S/target-base/release/e165_replay_bounded_enumeration --bounds segment1 --family adversarial --only A
# 打补丁的装置：各开关组合
bash research/scripts/run-with-memory-cap.sh 3G bash $M/run-all.sh $S/target/release/e165_replay_bounded_enumeration $S/runs-attack
python3 $M/summarize.py $S/runs-attack > summary-all.txt        # 与 runs/summary-all.txt 比
# Verus（0.2026.09.20.aef82ed，--no-cheating）
V=~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus
python3 $M/verus_variants.py $S/tree-base/crates/singlefs-replay-prefix/src/lib.rs $S/verus
bash research/scripts/run-with-memory-cap.sh 600M bash $M/run-verus.sh $V $S/verus                       # → runs/verus-today-spec-verdicts.txt
python3 $M/s2_strengthen.py $S/tree-base/crates/singlefs-replay-prefix/src/lib.rs $S/s00/src/lib.rs
python3 $M/verus_variants.py $S/s00/src/lib.rs $S/verus-s2v
bash research/scripts/run-with-memory-cap.sh 600M bash $M/run-verus.sh $V $S/verus-s2v                   # → runs/verus-s2-spec-verdicts.txt
python3 $M/s2_plus.py $S/s00/src/lib.rs $S/s01/src/lib.rs
python3 $M/verus_variants.py $S/s01/src/lib.rs $S/verus-s2p
bash research/scripts/run-with-memory-cap.sh 1G bash $M/run-verus.sh $V $S/verus-s2p --rlimit 200       # → runs/verus-s2-plus-spec-verdicts.txt
(cd $S/verus-s2p/v08-back-chain-off && $V --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating --rlimit 1000 src/lib.rs)
# 卸载记号的池上原型（内存稀疏盘、原样的核）
cp $M/m3_formal_r3_attack_unmount_marker.rs $S/tree-base/crates/singlefs-harness/tests/
(cd $S/tree-base && bash research/scripts/run-with-memory-cap.sh 2000M bash research/scripts/capped.sh 8 cargo test --offline -p singlefs-harness --test m3_formal_r3_attack_unmount_marker --target-dir $S/target-base -- --nocapture)
```
（`research/scripts/…` 指主仓的那几份，在副本里跑时写主仓的绝对路径。）

## 开关

见 `patch_attack.py` 文件头：`ATTACK_CORE` 1–9（核里的九种改法）、`ATTACK_GLUE` 1–3（拼根）、`ATTACK_GEN` 位 1 / 2（生成器：写行发布的分配记录树根、卸载记号）、
`ATTACK_SKIP_PRELUDE=1`。开关全 0 时打补丁的装置除多出的 `name=attack` 行与末行计数之外，E7RESULT 行与入库产物逐字节相同（`runs/base-equivalence.txt`）。

## runs/

- `summary-all.txt`：`summarize.py` 对 22 次跑的 (运行, 域) 合计；`pairs-summary.txt`：C、C6 九个对集合的 J5a、J5b。
- `k10-summary.txt`：对抗一族 K10 态上的 AS 三条与「两种读法期望不同时实现等于哪一种」，逐 n 也列；`txg-cap-summary.txt`：E1、E2 上 txg 顶格两种写法的计数，末两行是表第 1003 行在 E2 上的判决行与 E1 的见证。
- `exits.txt`：退出码；`core3`、`core9`、`core3-adv` 退 1 是装置自己的作废条款（V4 域非空、F11 报错态数）触发，产物行齐全。
- `verus-*-verdicts.txt`：13 个核变体各一行判定行；`verus-s2-plus-v08-*`：同一个变体 rlimit 200 与 1000 的结局。
- `unmount-marker.txt`：池上原型打出的 `ATTACK` 行与测试结局。
