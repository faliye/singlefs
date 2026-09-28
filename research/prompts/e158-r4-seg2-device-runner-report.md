# E158 第 4 次跑第二段：装置写完、交接（执行员，2026-09-27；主 agent 要求在此交接）

## 一、结论

- 第二段装置写完、单测与变异证红、12 份第二段自己的臂副本编好；**第二段产物一份都没跑**（主 agent 要求交接，停在跑产物之前）。
- 主工作区 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 已是整份编得过的最终版：sha256 `12b82f491453e4f598f8262632dc949967508a8a1cae745a1b4db078499fb148`，24238 行；之后在主工作区跑 `cargo build -p singlefs-harness --bin e158_root_choice_repair`：`build_rc=0`，末行 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.00s`（之前那一次 `in 4.36s`、0 warning）。它与 12 份臂副本里的 bin `cmp` 相同。
- 主 agent 说的「unclosed delimiter」是我分段 Edit 插入模块的中间态已整份 `cp` 成编得过的版本（按主 agent 消息的做法）。
- 推翻条件：主工作区那份与 `/tmp/claude-1000/e158-r4-seg2/arms/<臂>/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 不 `cmp` 相同；在任一臂副本上 `cargo test --release --bin e158_root_choice_repair` 不是 94 passed。

## 二、做完的

| 样 | 在哪 | 读数 |
|---|---|---|
| 模式 `r4-seg2` | bin 第 16943 行（模式表）、第 17003 行（分派）；模块 `fourth_run_segment_two` 第 17443–22086 行（入口 `run_segment_two` 第 21732 行，单测模块第 21818 行起） | 五族：H-随（30 段）、H-随全（68 段）、实七-甲（6 个注入点分支）、实七-乙、实八（各 4 个崩溃状态分支）；PC-随；S5a/S5b/S7/S9；M22 |
| `r4-compare` 补两样 | bin 第 15829–15847 行（走岔之后）、第 15897–15902 行（`r4_q4_after_divergence`）、第 15963–16017 行（PC-随 -槽 与今天逐步相同） | 没有单测、变异不覆盖 |
| 单测 | `grep -c '#\[test\]'` 数出 94（第二段新加 12 条） | 最终源码的草稿副本上 `test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.43s`（`unit-all.log`） |
| fmt / clippy / naming-lint | 草稿副本 | `cargo fmt --all -- --check` rc=0；clippy（`-D warnings` 加编码纪律七条，dev 与 test 两档）errors=0 warnings=0；naming-lint 带 `.claude/abbreviations` 在这个文件上 0 处 |
| 装置变异 11 条 | 草稿 `/tmp/claude-1000/e158-r4-seg2/mutations-append.tsv`（11 行，六列同 `crates/mutations.tsv`，sha256 `17d03474…87ad`）；`crates/mutations.tsv` 没动，交主 agent 追加 | 抓到 11 / 无效 0 / 没红 0；收尾 `restored baseline rc=0 test result: ok. 94 passed`（`check_mutations.log`）。M22 在单测上走完的步数由 8 变 4、尾巴没跑（`mut-m22.log`：`left: 4 right: 8`） |
| 臂表 | 没改：第二段只用已有的 `r4` 行；12 份臂副本从 `/tmp/claude-1000/e158-r4-device/arms/<臂>` 各 `cp -a`，只换 bin | 每份与共用副本 `diff -rq --exclude=target` 只差 bin 一个文件；12 份 `build rc=0 warnings=0 errors=0 bin_cmp=same`（`build_arms.log`） |
| 登记修订 | `research/prompts/e158-r4-prereg.md` 第 714 行起「执行员修订·第二段」第 18–30 条（原判据没动） | 执行器与 harness 逐步比的停法、注入点与崩溃状态的映射、格与读回、内容号、回退读法（只报数）、丢写分解（只报数）、H-随全、compare 两样、PC-随句子、S5 两边、5.6 现查值、快照指纹、变异 |
| 新快照指纹 | 第 29 条 | **`ad9f544263a5a61bf0bed305ea6e98701042e63ff2ad5e96f2b66552f360f9dc`**（C0 `manifest-2.sha256` 里装置那一行换成新 bin 的 sha256 后整份的 sha256；`manifest-seg2.sha256`） |

## 三、今天那一臂的草稿试跑（不是产物，不入库；在旧的草稿副本上跑，最终源码与它只差注释、丢写分解两栏、panic 捕获与改名）

`trial2-today.out`（1834 行，2 分 28 秒，单线程）。要紧的几行（整行抄，截到 300 字）：

```
E7RESULT name=r4_segment_two_executor_agreement family=h-random compared_steps=881 mismatches=0 verdict=pass detail=[]
E7RESULT name=r4_segment_two_executor_agreement family=h-random-all compared_steps=2108 mismatches=0 verdict=pass detail=[]
E7RESULT name=r4_segment_two_executor_agreement family=seventh-fault compared_steps=31 mismatches=0 verdict=pass detail=[]
E7RESULT name=r4_stop_check clause=S5 family=seventh-fault harness_reproduces=true device_reproduces=true verdict=reproduced
E7RESULT name=r4_stop_check clause=S5 family=seventh-crash harness_reproduces=false device_reproduces=false verdict=S5b_not_reproduced_on_the_base
E7RESULT name=r4_stop_check clause=S5 family=eighth-crash harness_reproduces=true device_reproduces=true verdict=reproduced
E7RESULT name=r4_failure_clause clause=F19 family=h-random arm=today segments_without_an_abandoned_red=0 labels=[] verdict=not_triggered
```

今天那一臂 H-随 汇总（试跑）：`cells=43 lost_write_cells_q1_q2_q3=43 … q1_in_ring_overwrite_cells=17 q1_out_of_ring_overwrite_cells=15 … q2_rollback_cells=38 … q3_lost_cells=6 … q3c_lost_cells=40 …`。PC-随 三段在今天那一臂上全 `verdict=pass`。**要主 agent 先看的一处**：③c 在今天 43 格里 40 格读失败，Q1「已不在根环里」15 格——随机历史长、根环会转，正确的臂在这两栏上也可能为正（推的，没量过）；所以最终源码另报了「不算 ③c」「Q1 只算环内且不算 ③c」两种丢写格（修订第 23 条），丢写格本身仍按登记字面算。接手的执行员跑完各臂后先看这两栏在 -配置 各臂上是不是 0。

## 四、没开的（接手的执行员照做）

1. 今天那一臂先跑（别的臂要读它的产物）：
   `cd /tmp/claude-1000/e158-r4-seg2 && SEGMENT_TWO_FINGERPRINT=ad9f544263a5a61bf0bed305ea6e98701042e63ff2ad5e96f2b66552f360f9dc bash run_lane.sh T today`
   产物 `research/results/e158-root-choice-repair-2026-09-27-r4-seg2-today.out`。先看它的 `r4_segment_two_executor_agreement`、`r4_stop_check`、`name=stop` 行：有 S9/S7/S5a 停的族，别的臂自动跳过那一族。
2. 其余 11 条臂分 4 条 lane（每条进程单线程，合计 4 个线程），同一条命令换 lane 名与臂名，例：`bash run_lane.sh A jia-cfg yi-cfg bing-cfg`、`B jia-cfg-carry yi-cfg-carry bing-cfg-carry`、`C jia-slot yi-slot yi-narrow-slot`、`D yi-narrow-cfg yi-narrow-cfg-carry`（run_in_background 里每条 `{ …; echo $? > rc-<lane>; } &` 再 `wait`）。脚本写之前核同名文件不存在；每条臂约 3 分钟（今天那一臂试跑实测 2 分 28 秒，别的臂没量过）；每跑完一条往 `progress.md` 追加一行。
3. `r4-compare`：在 `arms/today` 里跑 `… ./target/release/e158_root_choice_repair r4-compare research/results/e158-root-choice-repair-2026-09-27-r4-seg2 > …-r4-seg2-compare.out`（同样经 `capped.sh 3` 与 `run-with-memory-cap.sh 10G`）。
4. 第 4c 步判决行逐个点名、岔路表、实验页不写、`replay.sh` 不改（派发）。`replay.sh` 以后要复跑这一段，今天那一臂要先跑、别的臂要环境变量 `SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT` 与 `SINGLEFS_E158_INVESTIGATOR_REPORT`，写复跑登记时照 `run_lane.sh` 里那一行。
5. 门禁：登记给 experiment-runner 的阶段一道都没跑。

## 五、没做什么

- 没跑任何第二段产物；没写实验页、索引行、`experiments-history.md`；没改 `replay.sh`；`crates/mutations.tsv` 没动（行在草稿 `mutations-append.tsv`）。
- compare 那两处改动没有单测、没有变异覆盖。
- 岔路表（岔路单第 1 行）：丢写 / 多拒 / 多读 / 第 22 条——这一段一样都没量到，全部「还差第二段产物」；剩下的量能不能让它翻面：能。
- 要进 `.claude/preflight-exclude` 的路径：`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（已在第 188 行，这一趟又改了）、`research/prompts/e158-r4-prereg.md`（加了修订）。
- 没跑重型测试，没提交。

## 六、草稿目录（`/tmp/claude-1000/e158-r4-seg2/`）

- 留着：`arms/` 下 12 份第二段臂副本（4.5G，接手的执行员跑产物要用）；`run_lane.sh`、`build_arms.sh`、`check_mutations.sh`、`mutations-append.tsv`、`manifest-seg2.sha256`、`src/`（分段源码，已并进主工作区）、`edits/`、试跑输出 `trial*-today.out`（不是产物）。
- 删了：`dev`（500M）、`dev2`（612M）、`mut`（612M）、`lintroot`（1.1M）。
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/bing-cfg：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（368M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/bing-cfg-carry：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（369M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/jia-cfg：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（367M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/jia-cfg-carry：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（368M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/jia-slot：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（367M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/today：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（490M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/yi-cfg：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（368M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/yi-cfg-carry：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（370M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/yi-narrow-cfg：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（368M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/yi-narrow-cfg-carry：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（370M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/yi-narrow-slot：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（368M）
- 没删 /tmp/claude-1000/e158-r4-seg2/arms/yi-slot：第二段这一臂的副本，接手的执行员跑 r4-seg2 产物要用（368M）
