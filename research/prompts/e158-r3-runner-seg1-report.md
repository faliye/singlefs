# E158 第 3 次跑第一段 执行员报告（交接版，2026-09-27）

主 agent 来消息要求交接（上下文到线、跑满一小时），停在下面「做到一半」那一处。

## 一、结论

- 第一段的产物全部出来、落盘：九条臂各一份 `r3-seg1` 产物 + 一份跨臂 `r3-compare` 产物 + `r2-all` 在快照上重出一份（路径与完成标记见第三节）。
- 丢写（岔路单第 1 行翻面观测第一样，H1d 与 H1e，G0）：**-配置 六臂（甲-配置、甲-配置续、乙-配置、乙-配置续、丙-配置、丙-配置续）在 H1d、H1e 上 Q1∪Q3 与 Q2 都是 0**；**今天、甲-槽、乙-槽 三臂丢写**（今天 H1d 304 / 1680、H1e 64 / 64；甲-槽 H1d 152 / 864、H1e 32 / 64；乙-槽 H1d 288 / 2560、H1e 64 / 128）。-槽 两臂丢的全在「读回全零」那一种造法上（N-槽 看不见全零，登记第四节推的 ④ 成立）。命令与原样输出在第五节。
  - ⚠️ 登记第六节写明 H1d 对 -配置 各臂不算判据（N-配置 按构造恒真）；H1e 的被测挂载与 H1d 不断那一格是同一个构造，今天那一臂上装置算的 N-配置 在 H1e 16 次被测挂载上也全真——H1e 对 -配置 各臂同样是臂定义推得出的 0，-配置 各臂的丢写判据实际要落到实七、实八、H1g、H1f（第二、三段）。这一句是我读产物之后的观察，交主 agent。
- 多读（第三样，Q5，臂减今天、只算不断那一格）：丙 两臂每格 +1032 到 +1576 次读调用（重扫整个 journal 环）；甲 三臂与在 W 下拒的 乙 臂是负数（拒得早、读得少）；乙 在第 1 轮之后照常挂载的那 15 格 +84 到 +96。第五节有原样行。
- **两处阳性对照不过，按 V1 会作废量，交主 agent 定**：
  1. PC-N 今天那一臂 `verdict=fail`（W、T1 两格）：只挂在「撤故障之后按新实例表判被藏那条根为被抛弃」那一句——b = records 时新实例第一次发布的 txg 恰好等于被藏那条根的 txg，根槽位置只由 txg 定，写行那次发布把被藏根的槽盖掉了（`hidden_in_ring_after=false`、被抛弃集合为空）；其余三句（`clause_ok=true`、`clause_effective_before_hidden=true`、`clause_rollback_one=true`）都成立。登记 V1「今天那一臂的 ⇒ 全部臂的同一批量」按字面会作废全部臂的 Q1–Q3。这一格在跑产物之前的试跑里就看见了，写进了登记修订第 13 条，PC-N 没改。
  2. PC-多读 乙 三臂 `verdict=fail`：乙 在 W 下拒（不走取号之后那一段），读调用比照常挂载的今天少 416–424 次；登记 5.4 对 乙 期望「≥ R × 失败落点数且 > 0」没有算到今天那一臂走完挂载还有几百次读。按 V1 作废 乙 三臂的 Q5。
- 其余阳性对照每臂都过：PC-O、PC-N0（8 格）、PC-N0 与今天逐项相同（F12 不触发）、PC-丢写、PC-只读（拒了的八臂）、乙 的 PC-N T1。7.3 第二、三、四行、F11 推断、F3、F4、F13、F14、S4、V2、7.2 各锚点都过或为 0（第四节）。
- 推翻条件：第二段的实七、实八、H1g 上 -配置 某臂 Q1∪Q2∪Q3 > 0；或 v3 重验（第八节）时产物与已落盘的不逐字节相同。

## 二、快照、S0、S1–S3、S8

- 快照：2026-09-27 对 `crates/` 做了一次 `cp -a`（`/tmp/claude-1000/e158-r3-runner/snapshot-crates`）；`git rev-parse HEAD` = `73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321`；`crates/` 145 个文件的 sha256 清单 `snapshot-manifest.sha256`，清单本身 sha256 = `74912842b9a57eeb09336fbe3207866c9204fd565bbc1930c994b9bede44a41a`（写进每份产物第一行 `E7INPUT name=crates_snapshot …`）；`git status --porcelain crates/` 104 行原样在 `snapshot-git-status.txt`（再核一次清单仍相同时取的）。
- S0（快照上逐条重核，行号现查）：`recovery.rs:716` 择根那一遍 `Unreadable => continue` 仍丢读错的槽；`mount.rs:2398` 只判「落后」；`transaction.rs:696` 取号写 tail 0；`transaction.rs:1025` 根槽之后轮换系统配置、`:1613`/`:1658` 写末条记录计数器；`mount.rs:888` `rebuilt_allocator` 仍再择根再读实例表；`recovery.rs:2035` 验点名单元 `.all` 第一份读不出就停；`mount.rs:750` 影子账只 `unreadable += 1`。快照里没有 A1 那种拒（`grep -rn AbandonedRootLedgerUnreadable` 零命中），S0 例外不适用。**S0 不停。**
- S1：九份副本 `arms/<臂>/`（臂名 today jia-cfg jia-cfg-carry jia-slot yi-cfg yi-cfg-carry yi-slot bing-cfg bing-cfg-carry）都从 `snapshot-crates` 派生，按仓里 `research/mutations/e158_arms.tsv` 新加的 15 行 r3 行（第 36–51 行，含一行注释）套用（`apply_arm.py`，旧串恰好命中一次）：甲-配置、甲-槽、乙-配置、乙-槽、丙-配置 各 11 行，三条「续」各 12 行，今天 0 行。
- S2：九份都编得过（0 warning）。S3：九份两两比，差别只落在臂表点名的五个文件里（`mount.rs`、`transaction.rs`、`history.rs`、`model_comparison.rs`、`singlefs-harness/Cargo.toml`），今天那一份与快照只差装置 bin 一个文件。
- S8 / `r2-all`：前后两次核工作树 `crates/` 清单与快照逐字节相同，其间照 `driver_e158_r2_all` 同一条命令跑 `r2-all`，产物加 `E7INPUT` 头落成 `research/results/e158-root-choice-repair-2026-09-27-r2-all-today.out`（1055 行，末行 `E7RESULT name=done emitted=1054`）；这一次没有任何 `verdict=fail`，判定词那一处改动（两处 `H1c 里找不到` 改成 `not_constructible`）不改它的任何一行。与旧的 `…-2026-09-26-r2-all-today.out` 对不上：`r2_h1d_cell` 800 → 832 行、`r2_seventh_batch_crash_finding` 1 → 0 行（H1d 摘要 `cases=832 … last_confirmed 696`；实七-乙 那一段 `findings_reading_back_mapping_still_unreadable=0`，第二段的 S5 可能复现不出，先提醒）。`replay.sh` 那一行已改指新文件，旧文件留着。之后工作树的 `crates/` 已被别人改过（`mutations.tsv`、`checker/src/image.rs`、`walk.rs` 等），这一行照旧会漂，不归这一次。

## 三、做完的：装置、臂表、产物

- 入库装置 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`：在 `fn main` 之前加「十一、第 3 次跑」一节（模式 `r3-seg1`、`r3-constants`、`r3-pc`、`r3-h1d`、`r3-h1e`、`r3-compare`），`main` 先派 `run_second_run_mode` 再派 `run_third_run_mode`；第 2 次跑的函数只调用不改，只改了登记点名的两处判定词。装置源码各臂同一份；臂的等待钩子与观测经特性 `e158-r3-arm`（臂表收尾那一行在副本的 harness `Cargo.toml` 里声明且缺省开，主工作区不声明）接进来。**这是一个新建的入库装置 bin 改动，要进代码轮判决的点名清单（门禁 56 号）——那条义务在跑整轮门禁的一方。**
- 臂表：`research/mutations/e158_arms.tsv` 末尾经 `insert-row.py` 追加 16 行（1 行注释 + 15 行 r3 行）；臂的源码片段草稿在 `arm-src/`。
- 产物（`research/results/`，第一行都是 `E7INPUT name=crates_snapshot key=E158 sha256=74912842…a41a`）：

| 文件 | 行数（`wc -l`） | 末行 |
|---|---|---|
| `e158-root-choice-repair-2026-09-27-r3-seg1-today.out` | 1867 | `E7RESULT name=done emitted=1866` |
| `…-r3-seg1-jia-cfg.out` / `…-jia-cfg-carry.out` | 235 / 235 | `emitted=234` / `emitted=234` |
| `…-r3-seg1-jia-slot.out` | 1051 | `emitted=1050` |
| `…-r3-seg1-yi-cfg.out` / `…-yi-cfg-carry.out` | 1196 / 1196 | `emitted=1195` / `emitted=1195` |
| `…-r3-seg1-yi-slot.out` | 2828 | `emitted=2827` |
| `…-r3-seg1-bing-cfg.out` / `…-bing-cfg-carry.out` | 1099 / 1099 | `emitted=1098` / `emitted=1098` |
| `…-r3-seg1-compare.out`（读上面九份） | 61 | `E7RESULT name=done emitted=60` |
| `e158-root-choice-repair-2026-09-27-r2-all-today.out` | 1055 | `E7RESULT name=done emitted=1054` |

  产物由 bin sha256 `66fce613…3e1e73c` 那一版编出；之后改名（naming-lint）那一版 `ed256105…c73300`（`final-bin-v2.rs`）在九份副本上重编重跑，九份产物与 compare 与已落盘的**逐字节相同**（`rerun-after-rename.log` 里 `identical_to_results` 十行）。
- 4b 齐不齐（`grep -c` 数的，原样）：

```text
today           h1d_cells=1680 h1d_groups=48 h1d_groups_pass=48 h1e_cells=64 pc_lines=13
jia-cfg         h1d_cells=48 h1d_groups=48 h1d_groups_pass=48 h1e_cells=64 pc_lines=13
jia-cfg-carry   h1d_cells=48 h1d_groups=48 h1d_groups_pass=48 h1e_cells=64 pc_lines=13
jia-slot        h1d_cells=864 h1d_groups=48 h1d_groups_pass=48 h1e_cells=64 pc_lines=13
yi-cfg          h1d_cells=928 h1d_groups=64 h1d_groups_pass=64 h1e_cells=128 pc_lines=13
yi-cfg-carry    h1d_cells=928 h1d_groups=64 h1d_groups_pass=64 h1e_cells=128 pc_lines=13
yi-slot         h1d_cells=2560 h1d_groups=64 h1d_groups_pass=64 h1e_cells=128 pc_lines=13
bing-cfg        h1d_cells=912 h1d_groups=48 h1d_groups_pass=48 h1e_cells=64 pc_lines=13
bing-cfg-carry  h1d_cells=912 h1d_groups=48 h1d_groups_pass=48 h1e_cells=64 pc_lines=13
```

  （命令：对每份 `grep -c 'name=r3_h1d_cell '`、`grep -c 'name=r3_h1d_group '`、`grep 'name=r3_h1d_group ' | grep -c 'verdict=pass'`、`grep -c 'name=r3_h1e_cell '`、`grep -c 'name=r3_positive_control '`；原输出多一栏 `h1d_not_applicable`，今天那一份的 1 是 PC-只读那一行的 `not_applicable`，不是 H1d 的组。）每组格数 = 1 + 2(J + 1) + 屏障数（拒的臂 = 1）逐组对上（`r3_h1d_group … verdict=pass` 全部）；H1e 每种时长 64 格（7.2 锚点行 `verdict=pass`）。三条「续」臂的族摘要与各自的非续臂除臂名外逐字相同（`diff` 过）。

## 四、判决行的点名（第 4c 步）

`grep -n 'name=verdict'` 在这十一份产物（九份 `r3-seg1`、`r3-seg1-compare`、`2026-09-27-r2-all-today`）里都是 0 行——这个装置的判定写在各行的 `verdict=` 字段里。按字段数（命令：`grep -H -o 'name=[a-z0-9_]* [^ ]*.* verdict=[a-z_0-9]*' <那十一份> | …| sort | uniq -c`，只列不是 pass 的，原样）：

```text
     24 r2-all-today.out name=r2_q1_0 verdict=not_applicable
      1 r3-seg1-compare.out name=r3_f11 verdict=inference_holds
      3 r3-seg1-compare.out name=r3_positive_control verdict=fail
      2 r3-seg1-today.out name=r3_positive_control verdict=fail
      1 r3-seg1-today.out name=r3_positive_control verdict=not_applicable
```

| 产物（行号） | 字段 | 取值 | 为什么 | 登记预期内？ |
|---|---|---|---|---|
| `…-r3-seg1-today.out` PC-N 两行（`grep -n 'control=PC-N '` 现取） | `verdict` | `fail` | 「撤故障之后被藏根被抛弃」那一句不成立：被藏根的槽被新实例同 txg 的写行根盖掉（`hidden_in_ring_after=false abandoned_after=`） | 否（登记预期它 pass）；修订第 13 条产物前已记 |
| `…-r3-seg1-today.out` PC-只读 | `verdict` | `not_applicable` | 今天那一臂在 PC-N 上不拒，PC-只读只对拒了的臂 | 是 |
| `…-r3-seg1-compare.out` 第 30、35、40 行 | `verdict`（PC-多读，乙-配置、乙-配置续、乙-槽） | `fail`（`extra_read_calls=-416/-416/-424 faults=3`） | 乙 在 W 下拒、不走取号之后那一段，比照常挂载的今天少读 | 否（登记预期 ≥ R × 3 且 > 0） |
| `…-r3-seg1-compare.out` | `r3_f11 verdict` | `inference_holds` | 丙-配置 与 甲-配置 在 O(1 只) 之外 58 格逐项相同 | 是（第四节推的 ③ 成立） |
| `…-2026-09-27-r2-all-today.out` | `r2_q1_0 verdict` | `not_applicable` ×24 | 第 2 次跑那一族里没有记录可断的组合，与 09-26 那份同 | 是 |

计数类字段（命令：`grep -H -o -E '(void_cells_v2|fault_not_reached_cells|s4_mismatches|f4_failures|f13_failures|formula_mismatches|tail_check_failures|differences|mismatches|cells_differing_outside_o1_and_cell_22|nonzero_cells)=[0-9]+' …r3-seg1-*.out | awk …`，原样）：

```text
cells_differing_outside_o1_and_cell_22 lines=1 sum=0 nonzero_lines=0
differences lines=14 sum=0 nonzero_lines=0
f13_failures lines=117 sum=0 nonzero_lines=0
f4_failures lines=18 sum=0 nonzero_lines=0
fault_not_reached_cells lines=18 sum=0 nonzero_lines=0
formula_mismatches lines=18 sum=0 nonzero_lines=0
mismatches lines=2 sum=0 nonzero_lines=0
nonzero_cells lines=1 sum=0 nonzero_lines=0
s4_mismatches lines=189 sum=0 nonzero_lines=0
tail_check_failures lines=36 sum=0 nonzero_lines=0
void_cells_v2 lines=18 sum=0 nonzero_lines=0
```

F14 那 36 行（四个 n1 的关闭 U × 九臂）、7.2 锚点与 5.6 常量回比与臂代码核对那 354 行，全部 `verdict=pass`。

## 五、三样的读数（命令与原样输出）

丢写表（命令：`grep -h 'name=r3_h1[de]_cell ' research/results/e158-root-choice-repair-2026-09-27-r3-seg1-*.out | grep -v 'verdict=void' | awk -f /tmp/claude-1000/e158-r3-runner/loss_table.awk | sort`；Q1 = 覆盖格（最后确认那一版的单元在历史末尾被改的格）；Q3(+3c) = 读回更旧或读失败，或 ③c 尾巴读失败；Q1uQ3 = 登记第六节「丢写格数」；Q2 = 回滚格；Q3=LC = 读回最后确认那一版的格数）：

```text
arm family              cells     Q1  Q3(+3c)     3c    Q1uQ3     Q2   Q1uQ2uQ3  Q3=LC
bing-cfg-carry h1d        912      0        0      0        0      0          0    912
bing-cfg-carry h1e         64      0        0      0        0      0          0     64
bing-cfg h1d              912      0        0      0        0      0          0    912
bing-cfg h1e               64      0        0      0        0      0          0     64
jia-cfg-carry h1d          48      0        0      0        0      0          0     48
jia-cfg-carry h1e          64      0        0      0        0      0          0     64
jia-cfg h1d                48      0        0      0        0      0          0     48
jia-cfg h1e                64      0        0      0        0      0          0     64
jia-slot h1d              864    134      144      4      152     24        152    720
jia-slot h1e               64     32       16     16       32     32         32     64
today h1d                1680    268      288      8      304     48        304   1392
today h1e                  64     64       32     32       64     64         64     64
yi-cfg-carry h1d          928      0        0      0        0      0          0    928
yi-cfg-carry h1e          128      0        0      0        0      0          0    128
yi-cfg h1d                928      0        0      0        0      0          0    928
yi-cfg h1e                128      0        0      0        0      0          0    128
yi-slot h1d              2560    264      272      5      288     32        288   2288
yi-slot h1e               128     64       32     32       64     64         64    128
```

按造法分（同一批行，`form` 与 `duration` 两栏；丢写 = Q1∪Q2∪Q3）：今天 H1d 在 read_fails / reads_zeros 两种造法、W / O(1 只) 两种时长上都是 72 / 416，T1（只跑不断那一格）8 / 8；甲-槽 read_fails 全 0、reads_zeros W 72 / 416、O(1 只) 72 / 416、T1 8 / 8；乙-槽 read_fails 全 0、reads_zeros 四种时长各 72 / 416；H1e 上今天两种造法各 32 / 32，甲-槽、乙-槽 reads_zeros 各 32 / 32。-配置 六臂各格全 0。

Q0（今天那一臂，被测挂载 `Ok` 之后撤故障被藏根被抛弃的组数；命令 `grep 'name=r3_h1d_cell ' …-today.out | grep 'cut=none' | awk …` 取 b、form、duration、`hidden_in_ring_after_a`、`q0_hidden_abandoned_after_a`）：H1d 32 / 48、H1e 8 / 16；b = records 且时长 W、T1 的组里被藏根不在根环里（被新实例同 txg 的根盖掉）、Q0 = false，其余（b = records 的 O(1 只)、b = unit_first_copy 全部）被藏根在环里且被抛弃。全族不为 0，F8 不触发。

F6（第四节推的 ①）：今天那一臂 H1d 48 次被测挂载 `device_n_cfg=true` 48 次；H1e 16 次也是 16 次。F6 不触发。

多读 Q5（`…-r3-seg1-compare.out` 原样行，每臂三行：PC-N、H1d 不断那一格、H1e 每组 m = 1 那一行）：

```text
E7RESULT name=r3_q5_extra_reads arm=jia-cfg family=h1d cells=48 positive_cells=0 total_extra_read_calls=-21076 total_extra_read_bytes=-78020608 minimum_extra_read_calls=-504 maximum_extra_read_calls=-383
E7RESULT name=r3_q5_extra_reads arm=jia-slot family=h1d cells=48 positive_cells=0 total_extra_read_calls=-10634 total_extra_read_bytes=-39403520 minimum_extra_read_calls=-508 maximum_extra_read_calls=0
E7RESULT name=r3_q5_extra_reads arm=yi-cfg family=h1d cells=48 positive_cells=15 total_extra_read_calls=-13536 total_extra_read_bytes=-47271936 minimum_extra_read_calls=-500 maximum_extra_read_calls=92
E7RESULT name=r3_q5_extra_reads arm=yi-slot family=h1d cells=48 positive_cells=15 total_extra_read_calls=-3286 total_extra_read_bytes=-9441280 minimum_extra_read_calls=-505 maximum_extra_read_calls=84
E7RESULT name=r3_q5_extra_reads arm=bing-cfg family=h1d cells=48 positive_cells=48 total_extra_read_calls=59948 total_extra_read_bytes=260833280 minimum_extra_read_calls=1032 maximum_extra_read_calls=1572
E7RESULT name=r3_q5_extra_reads arm=bing-cfg family=h1e cells=16 positive_cells=16 total_extra_read_calls=17596 total_extra_read_bytes=75530240 minimum_extra_read_calls=1032 maximum_extra_read_calls=1154
```

（续臂的 Q5 行在 compare 产物第 17–19、32–34、47–49 行，与非续臂差几十次读——续臂取号那一刻另读 4 个系统配置槽。）乙 在 read_fails 的 O(1 只) 与 T1 上第 1 轮之后照常挂载（15 格为正），读回全零那一种造法因为「读成就进缓存」（修订第 3 条）重读不出新东西、R 轮之后照拒。

7.3 各行（compare 产物原样）：`row=2 witness=device_n_cfg jia_arm=jia-cfg cells=75 mismatches=0 verdict=pass`、`row=2 witness=device_n_slot jia_arm=jia-slot cells=75 mismatches=0 verdict=pass`；`row=3` 今天、甲 三臂、丙 两臂各 `cells=17 differences=0 verdict=pass`；`row=4 … cells=75 nonzero_cells=0 verdict=pass`。PC-N0 与今天逐项相同：八臂各 `cells=8 differences=0 verdict=pass`。

## 六、岔路表（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行，逐样）

| 那一样 | 状态 | 还差什么 | 剩下的量能不能让它翻面 | 算它的命令 |
|---|---|---|---|---|
| 丢写（Q1∪Q3，另报 Q2），每个候选 | 未够判。G0 的 H1d、H1e 上：甲-配置 / 甲-配置续 / 乙-配置 / 乙-配置续 / 丙-配置 / 丙-配置续 全 0；今天、甲-槽、乙-槽 > 0（-槽 的丢写全在读回全零那一种造法） | 实七-甲、实七-乙、实八三段（岔路单点名的另两类历史）；-配置 各臂在 H1d、H1e 上的 0 按登记第六节不算判据（H1e 同一构造、N-配置 恒真，见第一节） | 能：第二段实七、实八与第三段 H1g、H1f 上 -配置 某臂 > 0 就让它出局；-槽 两臂已 > 0，后面的量翻不回来 | 第五节丢写表那条 awk |
| 多拒（Q4） | 没量（第二段） | L0–L6、PC-多拒、实七 / 实八逐次挂载；5.5 第 8 条那一类「可免的拒」按时长分栏 | 能 | — |
| 多读（Q5） | H1d 不断那一格、H1e、PC-N 上有数（第五节）；乙 三臂的 Q5 按 V1 作废（PC-多读 fail） | L 族、实七 / 实八；PC-多读 那一句怎么处置交主 agent | 能（两候选都不丢写时才比它） | `…-r3-seg1-compare.out` 的 `r3_q5_extra_reads` 行 |
| 代码审阅第 22 条那一格（Q6） | 没量（第二段：L5、PC-22） | L5 全部 | 能 | — |
| 够判条件第二半（判据那一样今天有没有） | 已够判（登记 3.2 读代码的答案，S0 在快照上逐条核过，见第二节） | — | 不能 | 第二节 S0 行号 |
| 阳性对照 V1 | PC-N 今天那一臂 fail、PC-多读 乙 三臂 fail | 主 agent 定这两句的处置（按字面：前者作废全部臂 Q1–Q3，后者作废 乙 三臂 Q5） | — | 第四节 |

## 七、变异

- 装置变异 9 条（登记第九节 M1、M3 装置那一半、M5、M6、M8、M10、M12、M13、M14）：各有一条单测，行追加进 `crates/mutations.tsv` 第 945–953 行（`insert-row.py` 追加；之后改名与防撞锚又用 Edit 改了自己这 9 行里的 3 个测试名、M13 的原文与替换文，别人的行没动）。在草稿副本 `mut/` 上逐条改坏、跑点名的单测：**9 条全抓**（`rerun-after-rename.log` 末尾 `caught` 9 行，收尾 `restored baseline rc=0`）——这是 v2 那一版装置上的结果，v3 改了 M13 的锚（第八节），v3 上没重跑。
- 臂表变异 4 条（M2、M4、M11、M15）：落在 `research/mutations/e158_r3_arm_mutations.tsv`（新文件，没有同名 bin，门禁 33 号不管它），在臂副本的一次性拷贝上改坏、编译、跑 `r3-pc`：M2、M4、M15 点名的阳性对照 `fail`；M11 让八格 PC-N0 与 PC-N、PC-O 都成 `not_constructible`（mkfs 之后第一次挂载就被这一臂拒，起始镜像造不出），算抓到、形态不同。4 条全抓（`arm-mutations.log`）。
- 没跑的：M3 臂那一半（取样点 H1g，第三段）、M7（PC-22，第二段）、M9（L1(a)，第二段）。
- 三个数：装置 9 条 抓到 9 / 无效 0 / 没红 0；臂表 4 条 抓到 4 / 无效 0 / 没红 0。
- 单测 68 条（`grep -c '#\[test\]'` = 68，其中第 3 次跑新加 11 条），v2 装置在今天那一份副本上 `test result: ok. 68 passed`。

## 八、做到一半的（接手从这里起）

1. **装置 v3 没重验**。产物之后为过门禁 33 号（我新加的代码与第 2 次跑 6 行变异原文撞锚：`crates/mutations.tsv` 第 837、838、839、844、846、851 行各在 bin 里命中 2 次）与 clippy（我自己新代码的 `geometry` 遮蔽两处、一个可折叠的 `if`）在仓里又改了 bin（只改写法：`fails = true` 改成块、`*written_over = … || …`、`this_write_is_the_injected_failure`、`copies` 改 match、`1 + highest_root.max(highest_record)`、`count` 改名 `intercepted`、两处 `base_pool_geometry`、F4 那个 if 折叠），并同步改了 `crates/mutations.tsv` 第 952 行（M13）的原文与替换文。现在仓里 bin sha256 = `4d515fdee15c76893446cae3509f1e56baab4c22c55d78fc625e5f080ddaaf07`，**与出产物的那一版不同**。要做：`cp` 仓里 bin 到 `arms/<九臂>/crates/singlefs-harness/src/bin/`、重编、`bash run_products.sh`（它会把现有 `products/` 覆盖——先 `mv products products-v2`）、逐份 `cmp` 与 `research/results/` 下九份 + compare（期望逐字节相同）；今天那一份副本上重跑单测（68 条）与 clippy（登记里 7 个编码纪律 lint，现存只该剩第 2 次跑那两处 `ranges`/`run` 遮蔽：`e158_root_choice_repair.rs:7244`、`:7245`）；`trial/today` 换成仓里 bin 后跑 `check_mutations.sh`（它读 `r3-mutation-rows.tsv`，先 `grep '第 3 次跑 M' crates/mutations.tsv | cut -f1-6 > r3-mutation-rows.tsv` 刷新）；重跑门禁 33 号（修完 M13 那一行之后重跑：rc=1，E158 零命中；红的全是别人的行，`crates/mutations.tsv` 第 308、309、310、319、494、759、865、870、874、877、878 行，工作树别处正在被改，不归这一段）。
2. **`bash research/scripts/replay.sh E158` 没跑**（会把 E158 下 15+2 行全跑一遍；登记行已加：`E158|@driver_e158_r3_seg1_today||e158-root-choice-repair-2026-09-27-r3-seg1-today.out|exact`、`…r3_seg1_compare…`，驱动在 `driver_e158_r2_all` 之后）。预期：compare 那一行逐字节相同；r3-seg1 今天与 r2-all 两行读工作树的 `crates/`，工作树已经不是快照，可能对不上，照实报。
3. **实验页、索引行、`experiments-history.md` 条目都没写**；doc-lint 没跑；门禁 27、34、40、69、75、84、85、86、88、99 号（读实验页的那几道）没跑。门禁 40 号会因为十份新产物没被实验页点名而红。已跑的：33 号 rc=1（见上，E158 已清零）、52 号 rc=1（E142 段序列表，与这一段无关）、80 号 rc=0 末行 `✓ 152 个实验二进制各自至少有一条绝对值断言…`、96 号 rc=0 末行 `✓ 实验源码纪律：扫了 152 个文件…`。
   实验页要写的（给接手的）：页首状态括注接「2026-09-27 第 3 次跑第一段（…）」；在「2026-09-26：第 2 次跑第一段」一节之前加「2026-09-27：第 3 次跑第一段（重跑登记 `research/prompts/e158-r3-prereg.md`）」一节，点名第三节表里十份新产物与 `…-2026-09-27-r2-all-today.out`、`research/mutations/e158_r3_arm_mutations.tsv`；新代号（Q0–Q8、H1e、L0–L6、PC-N、PC-O、PC-N0、N_真、V5、F13、F14、M1–M15 等）加进 `doc-lint:not-numbers`；「影响的决策」表五行都要写 2026-09-27 的回看（`grep -c 'E158（'` 在 D16、D23、D22、D8、D28 五个决策文件里都是 0，全是备料），另加这一段正文会提到的 D23 已定项 15、18，D16 已定项 1、7 四行（备料）；页末「历史版本」加 2026-09-27 条。
4. 登记「十二、修订」：已写 14 条（落盘，在产物起跑之前）；之后只把那一段的时刻标签改成实际的时刻（产物之后，内容没动）。产物之后的装置改动（改名、防撞锚、clippy）都不改输出，还没写进修订——按定义「产物跑过之后不改登记」，交主 agent 决定记在哪。

## 九、草稿目录里留着的副本（都没删，理由）

- `/tmp/claude-1000/e158-r3-runner/snapshot-crates`：开工快照（第二段要从同一次快照派生，登记写死）。
- `/tmp/claude-1000/e158-r3-runner/arms/{today,jia-cfg,jia-cfg-carry,jia-slot,yi-cfg,yi-cfg-carry,yi-slot,bing-cfg,bing-cfg-carry}`（各带 `target/`，每份约 330–440 MB）：第八节第 1 条要在上面重编重验，第二段也用。
- `/tmp/claude-1000/e158-r3-runner/trial/today`、`/tmp/claude-1000/e158-r3-runner/mut`：装置变异核对（`check_mutations.sh`）用，第八节第 1 条还要跑。
- 删了：`trial/` 下另三份试编副本、`mutarm/`（臂表变异的一次性拷贝，`check_arm_mutations.sh` 会从 `arms/` 重建）、`products-first-try/`。
- 脚本：`apply_arm.py`、`make_arms.sh`、`run_products.sh`、`check_mutations.sh`、`check_arm_mutations.sh`、`rerun_after_rename.sh`、`loss_table.awk`、`assemble_bin.py`（最后这个已经不用：仓里 bin 是最新的，别再从 `r3_section.rs` 拼）。
- 我自己起过一个等待循环写错了（`pgrep -f` 匹配到了自己，等自己），发现后用 `proc.py stop 2492552` 停了，子进程随之退出，没有残留。

## 十、没做什么

- 没做第二段、第三段的任何量（实七、实八、L0–L6、PC-多拒、PC-22、PC-554、Q4、Q6、敏感性点）。
- 没判任何候选出局或胜出（那是主 agent 对着岔路单判）；没判 PC-N / PC-多读 两处 fail 怎么处置。
- 没跑重型测试、没跑门禁全量、没提交。
- 没改 `crates/singlefs-core`、`crates/singlefs-checker`（臂改动只在草稿副本）。
