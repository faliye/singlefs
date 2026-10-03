# m3-formal-r2 攻方腿（Opus）的模型：复跑说明

全部在冻结副本的拷贝上做，不读、不改主仓 `crates/`。数都是副本上量的，不是入库装置上的数。

## 复跑

```
S=/tmp/claude-1000/<草稿目录>
M=research/prompts/m3-formal-r2-opus-model
rsync -a --exclude target --exclude .git /tmp/claude-1000/m3-formal-r2/tree/ $S/tree/
cp Cargo.toml Cargo.lock $S/tree/
python3 $M/patch_recovery.py $S/tree/crates/singlefs-core/src/recovery.rs
python3 $M/patch_e165.py $S/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
python3 $M/patch_e165_k10.py $S/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
(cd $S/tree && nice -n 19 bash <仓>/research/scripts/run-with-memory-cap.sh auto bash <仓>/research/scripts/capped.sh 10 \
   cargo build --release --offline -p singlefs-checker-tier --bin e165_replay_bounded_enumeration --target-dir $S/target)
E165_ATTACK_BIN=$S/target/release/e165_replay_bounded_enumeration nice -n 19 bash <仓>/research/scripts/run-with-memory-cap.sh 2G \
   bash <仓>/research/scripts/capped.sh 10 bash $M/run-all.sh $S/runs
python3 $M/summarize.py $S/runs > $S/runs/summary.txt
```

`runs/*.out` 只留 `E7RESULT` 行；与这里入库的逐字节比（`sha256sum`）。十二个开关组合 + 对抗一族一遍，本机 10 线程约 45 秒。

## 开关

`SINGLEFS_ATTACK_MUTANT`（`patch_recovery.py` 打进副本 `recovery.rs`，没设取 0 = 原文）：

| 值 | 改了哪一判 |
|---|---|
| 0 | 原文 |
| 1 | 锚点读得出那一支，jsn 对不上不断（`recovery.rs` 第 3111 行那一判；七·2 写的必红世界「跳过 I-8.3 那一判」） |
| 2 | 同 `crates/mutations.tsv` 第 36 行：锚点读不出时不看 txg |
| 3 | 同表第 406 行：每条记录都当末条 |
| 4 | 同 E165 自造 Z1：一条都不施加 |
| 5 | 施加后分配记录树根清成全零 |
| 6 | 施加后实例表指针取记录里的映射根指针 |
| 9 | 同 Z2：不看在飞上限 |
| 10 | 水位之上两条以上记录就一条都不施加 |

`ATTACK_ROW_ROOT=1`（`patch_e165.py` 打进装置）：第 0 版的根带一条非零的分配记录树根（照写行发布），之后各版全零（照带文件的发布）。

装置里加的量（原有判据一条不改，`E7RESULT name=attack` / `name=attack_k10` 两种行）：`vm2_*` 是 Verus 核 `m2_holds` 的可执行替身；`full_root_mismatch` 是返回的根与第 j 版整条根不等；`legal_*` 是 C、C6 域里只数合法崩溃状态（甲、乙两种切法任一种下停在某一段）；`attack_k10` 是对抗一族里「同实例 txg 随计数器不减」成立的态上的 AS 与 A-互。
