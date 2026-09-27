# E158 第 4 次跑第二段重跑 seg2b：装置记账撞键修掉、12 臂重跑、主工作区单测改绿（执行员，2026-09-27 JST 12:35–14:2x）

## 一、结论

- **装置修了一处**（臂副本 bin 第 20458–20463 行那次读回）：崩溃分支挂载之后的读回改用分支自己的账。新函数 `ledger_after_the_mount_of_a_crash_branch`：拷一份主路径的账，把挂载之后读得出、而崩溃镜像里没有同一条（键与根记录逐字节都相同）的根，改记成这次挂载承接的那一版。承接规则照主路径 `observed_mount`：交回 `Ok` 取 E 的内容号，查不到就取字面最后确认那一版；拒了取装置判的 E。先写单测 `crash_branch_read_back_after_its_mount_goes_by_the_roots_that_branch_wrote`：在今天的装置上红（`left: "failed:content_differs_from_the_ledger" right: "last_confirmed"`，草稿 `red-first.log`），修完转绿。两条变异证红：M39 把读回改回拿 `&main.ledger`，M40 把判别取反。
- **另两处照认定第 1 条查了，没有撞键，没修**：12 臂整臂加诊断跑（stdout 与 seg2 产物逐字节相同，12/12），计数原样如下。
  - 同一条时间线内复用键：主路径与注入分支每一步做完，镜像里每条自证根按 (实例, txg) 比根记录字节，`key_reuse place=executor_image` 12 臂全是 0。账里同一个键被改记成另一个内容号的（`carry_overwrite`）也是 12 臂全 0。
  - 第 20447 行传给 `evaluate_a_random_mount` 的主路径账：崩溃镜像上的根与主路径同键不同字节的（`crash_image_vs_main`）12 臂全 0。所以按主路径账查 E 的内容号不会撞，撞键只出在分支挂载新写出的根上（`after_mount_vs_main`，today / -槽 各 5 次，-配置 各 4 次）。另有 E 不在崩溃镜像里的 16 格（实七-乙 crash2 × 12 臂、实八 crash3 × 4 臂）：E 是分支挂载按记录重建出来的，键也不撞。
  - 草稿里的计数表：`/tmp/claude-1000/e158-r4-seg2b/diag/diag-*.err`（命令见第五节）。**推翻条件**：把第 20447 行也换成分支自己的账之后，任何一臂的挂载行字段变了。这一样没量，量的是撞键次数 = 0。
- **seg2b 12 臂与 compare 都跑完**，每份以 `name=done` 收尾，`name=stop` 0 条。H-随、H-随全、实七-甲三族逐族与 seg2 逐字节相同，**认定第 2 条的停机没触发**（第三节）。变的只有两处：-配置 8 臂的实七-乙 3 格 → 0 格；today 与 -槽 3 臂的实八有 8 行变了，但丢写数不变。
- **乙-配置、乙-配置续 在 seg2b 上**：H-随 6/37、实七-甲 0/10、实七-乙 0/6、实八 0/2（compare-r2 第 266–273 行，原样见第三节）。H-随 那 6 格仍全是 ③c（`q3c_lost_cells=6`），不算 ③c 是 2 格，Q1 只算环内且不算 ③c 是 1 格。
- **compare 有两份**：`…-r4-seg2b-compare.out` 我漏设了 `SINGLEFS_E158_SNAPSHOT_SHA256`，首行是 `sha256=unset`。不许覆盖，所以另存 `…-r4-seg2b-compare-r2.out`，两份只差第 1 行（`diff` 原样见第三节）。**以 compare-r2 为准**，第一份留不留交主 agent 定。
- **主工作区补丁**在 `patch/`：`crates.patch`（sha256 `9a58a6cb…`）。对主工作区现状（A3a 之后，bin 仍是 `06d75b96…`，`crates/mutations.tsv` 1242 行）`git apply --check` 过了。补丁打在主工作区现状的副本上：bin 单测 95 passed / 0 failed；变异 12 行全部锚点恰好 1 次、基线 ok、施加后 FAILED、已还原。
- **要主 agent 知道的一件事**：第一版 `crates.patch` 是我在变异 M40 正施加在副本里的时候生成的，补丁里带着 M40。对主工作区现状重做时单测红，我才发现，已作废，另存为 `/tmp/claude-1000/e158-r4-seg2b/crates.patch.bad-contains-M40`，现在的 `crates.patch` 是重生成的。臂副本里的 bin 不受影响：它 13:14 从 dev 副本拷进来，那时 dev 副本上的变异早已还原；核过它含 `if !roots_in_the_crash_image`，函数与调用两段和主补丁逐字相同。

## 二、装置与臂副本

- 12 份臂副本：`cp -a /tmp/claude-1000/e158-r4-seg2/arms/<臂>` 到 `/tmp/claude-1000/e158-r4-seg2b/arms/<臂>`，原臂目录没动。`diff -rq -x target` 抽查 3 臂，只差 bin 一个文件。bin 在 12 份里逐字相同：改前 `12b82f49…`（12/12），改后 **`e71224fc02fde638e92f2548c19c71c2f7032b65ec7056a1bc014bebbac8dba8`**（12/12）。diff 107 行，1 行删、101 行加（草稿 `arm-bin-fix.diff`）。
- 臂副本上（dev 副本）：`cargo test --bin e158_root_choice_repair` 95 passed（`grep 'test result' green-all.log`：`test result: ok. 95 passed; 0 failed; …`）；fmt 过；命名纪律过（`✓ 命名纪律通过：查了 1 个 .rs 文件、6914 个声明的名字`）。clippy（`check.sh` 的 lint 集）在这个 bin 上 0 条，红在别的文件上：`e156_allocation_basis_counts.rs` 6 处、`tests/checker_narrow_invariants_and_abandoned_roots.rs` 2 处，都是快照里原有的，没修。
- 12 份 release 编译 rc=0、0 warning。臂二进制 sha256（`build-arms.txt` 原样）：

```
today ac25b3fe0c58c26c52100d8e5b1c6851fafc8e3b47a53914df7ad204235dd5d2
jia-cfg 7d40eaee3ddbbfb2a4a67321727596d1e89b1c14bb1841a896d5153c76c8f544
jia-cfg-carry ca8fe53e39b3cf83f957e8e64f7b881cc0ca043c142f6a2aa0e96dad86759908
jia-slot bc953bb75d52aff7df0b8afdd4d915fbf20e25ce93b51f140c6c35459d02aa24
yi-cfg c276e48658833be42ed80aa85dcc19343763a4d420f4b36681175c05661c908e
yi-cfg-carry e69ee61fc79a3d6c6cf7314a2008e02afeff4fc5e481ca2d7f12c0d28d035657
yi-slot 6c1d7b0086daaf239c0f2ad68adfe1ca67ff593bd616e7c1a283971dec5746ce
yi-narrow-cfg bfcbd8e4d5e69a7ee75f264669f1c9b0b8b8de21f2afd66025cde569c91165eb
yi-narrow-cfg-carry 21ff7425f6cc3a3cd6d93086c0a247f4efc371bc5c41f1f26c4fcd46f2a92e5d
yi-narrow-slot fc61e8e384852f97cbe8aa429c9c6e8b3ba3f35524f0164513cda675eec8794f
bing-cfg 9b73913a6ecf70e2dd2fc8f9480eb400f555ba4053befbafeef8d4418df6005f
bing-cfg-carry b6702d7b513320a9e2a7b8f785174e803cf278a7efbe00d7b37c112546efb320
```

- 产物首行的快照指纹照派发取 `ad9f5442…`。这个指纹是拿 seg2 那一版装置（`12b82f49…`）算出来的，**没把 seg2b 的装置 `e71224fc…` 算进去**。要不要为 seg2b 另算一个指纹，交主 agent 定。
- 跑法照 seg2 的 `run_arm.sh`，环境变量相同。非今天臂读的 `SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT` 改指 seg2b 那份今天臂产物。脚本在 `/tmp/claude-1000/e158-r4-seg2b/run_arm.sh`。today 用 `capped.sh 4`；其余 11 臂先 8 条并行、再 3 条，都是 `capped.sh 1`，同时在跑的线程不超过 8。负载：别的会话有几条 cargo test，没有 qemu、vm-bench、e152、fio，没等锁。
- 登记（`e158-r4-prereg.md`）一个字没改：这次改的是装置，判据没动；产物跑过之后也不许再改登记。装置修改与 compare 双份要不要写进修订段，交主 agent。

## 三、产物、齐不齐（4b）、与 seg2 比

产物 14 份，都在 `research/results/` 下：`e158-root-choice-repair-2026-09-27-r4-seg2b-{today,jia-cfg,jia-cfg-carry,jia-slot,yi-cfg,yi-cfg-carry,yi-slot,yi-narrow-cfg,yi-narrow-cfg-carry,yi-narrow-slot,bing-cfg,bing-cfg-carry}.out`，外加 `…-seg2b-compare.out` 与 `…-seg2b-compare-r2.out`。sha256 在草稿 `products.sha256`（14 行）。seg2 那 13 份没动。

4b（命令同 seg2 报告第三节，前缀换成 seg2b）的输出原样：

```
bing-cfg-carry summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520 stop=0
bing-cfg summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520 stop=0
jia-cfg-carry summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520 stop=0
jia-cfg summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520 stop=0
jia-slot summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=64 mounts=537 stop=0
today summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=64 mounts=537 stop=0
yi-cfg-carry summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520 stop=0
yi-cfg summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520 stop=0
yi-narrow-cfg-carry summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520 stop=0
yi-narrow-cfg summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520 stop=0
yi-narrow-slot summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=64 mounts=537 stop=0
yi-slot summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=64 mounts=537 stop=0
```

格的个数与 seg2 相同，没立的格也相同：-配置 8 臂的实七-甲 `branches_not_constructible=3`、实八 `=4`，8 臂都是这样。

**与 seg2 逐族比（认定第 2 条的停机判据）**。命令对每臂执行 `cmp <(grep "family=$f " seg2) <(grep "family=$f " seg2b)`，$f 取 h-random、h-random-all、seventh-fault；再用 `diff` 数变了的行、按族归类。今天那一臂的输出原样：

```
today lines_diff=8 … h-random:same:1182 h-random-all:same:397 seventh-fault:same:87 changed_families=      8 family=eighth-crash  nofamily_changed=0
```

- -配置 8 臂：三族都是 `same`，`lines_diff=7`，全在 `family=seventh-crash`。
- -槽 3 臂：三族都是 `same`，`lines_diff=8`，全在 `family=eighth-crash`。
- 12 臂不带 family 的行都没变（`nofamily_changed=0`）。

完整输出在本报告的命令记录里，草稿没另存。三族全都逐字节相同，**不停**。

compare-r2 与 seg2 compare 比：除 `r4_compare_input` 的路径与首行外，只有 8 行 `r4_loss` 变了，都是 -配置 8 臂的实七-乙，`lost_write_cells=3 … q3_lost_cells=3 … q3_last_confirmed_cells=3` → `lost_write_cells=0 … q3_lost_cells=0 … q3_last_confirmed_cells=6`。两份 seg2b compare 之间（`diff …-seg2b-compare.out …-seg2b-compare-r2.out`）原样：

```
1c1
< E7INPUT name=crates_snapshot key=E158 sha256=unset
---
> E7INPUT name=crates_snapshot key=E158 sha256=ad9f544263a5a61bf0bed305ea6e98701042e63ff2ad5e96f2b66552f360f9dc
```

丢写，每臂 × 族（从 compare-r2 的 `r4_loss` 行抽 `lost_write_cells`）：-配置 8 臂都是 `h-random 6、seventh-fault 0、seventh-crash 0、eighth-crash 0`；today、jia-slot、yi-slot、yi-narrow-slot 都是 `h-random 43、seventh-fault 10、seventh-crash 1、eighth-crash 3`（与 seg2 相同）。乙两臂的原样行（compare-r2:266–273）：

```
E7RESULT name=r4_loss arm=yi-cfg family=h-random geometry=harness-4gib cells=37 lost_write_cells=6 q1_overwrite_cells=2 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=6 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-cfg family=seventh-fault geometry=harness-4gib cells=10 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=10
E7RESULT name=r4_loss arm=yi-cfg family=seventh-crash geometry=harness-4gib cells=6 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=6
E7RESULT name=r4_loss arm=yi-cfg family=eighth-crash geometry=harness-4gib cells=2 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=2
E7RESULT name=r4_loss arm=yi-cfg-carry family=h-random geometry=harness-4gib cells=37 lost_write_cells=6 q1_overwrite_cells=2 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=6 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-cfg-carry family=seventh-fault geometry=harness-4gib cells=10 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=10
E7RESULT name=r4_loss arm=yi-cfg-carry family=seventh-crash geometry=harness-4gib cells=6 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=6
E7RESULT name=r4_loss arm=yi-cfg-carry family=eighth-crash geometry=harness-4gib cells=2 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=2
```

## 四、判决行（4c）

`grep -c 'name=verdict'` 在 14 份上都是 0。判决写在各行的 `verdict=` 字段里，取值分布如下：11 份臂产物全是 `59 verdict=pass`；today 是 `64 pass`、`2 reproduced`、`1 not_triggered`、`1 S5b_not_reproduced_on_the_base`；两份 compare 各是 `42 pass`、`1 inference_holds`、`12 not_constructible`。`=fail` 一个都没有。表示违例或失败的计数（`mismatches|s4_mismatches|f6_failures|segments_panicked|extra_refusals|refusals_with_truth_newer_false`）取正值的 0 处。`r4_void` 是 `84 voided=false`。点名如下，与 seg2 同位同值，都是登记预期内的：

| 产物:行 | 字段 = 取值 | 为什么 / 预期内吗 |
|---|---|---|
| `…-seg2b-today.out:1778` | `verdict=S5b_not_reproduced_on_the_base` | 实七-乙 两边都复现不出。11.2 S5b 规定不停，预期内 |
| `…-seg2b-compare-r2.out:209–220`（`…-compare.out` 同行号） | `verdict=not_constructible` 12 句 [今] | 前提在第四段，这次只喂了第二段的前缀。按 V1 ⑤ 不作废（第五段认定第 3 项），预期内 |
| -配置 8 份的实七-甲 / 实八 摘要行 | `branches_not_constructible=3` / `=4` | 修订第 19 条规定不立格；拿不准算不算「失败类计数」，所以点名 |

## 五、主工作区那份 bin 的 9 条单测（第 4 条）与补丁

- **挑的构造**：选「系统配置没见证到被藏的根」这一形，没按快照指纹分流。理由：分流等于在主工作区上把这几条断言关掉或换成另一套，而改构造能让同一批断言在今天的 crates 上照样有判别力。做法抄 `tests/common/mod.rs:433`：被测那次调用里，每块盘世代号最大的那一槽系统配置读回全 0（`ReadsZeros`，只接在落点表末尾，前面各项的下标不变），挂载的取号写落回这一槽之后照常读得出。各条改法：
  - `first_txg_…`、`replay_stops_…`、`the_row_publish_before_…`：落点表套上 `with_the_witness_unreadable`。
  - `abandonment_step_produces_…from_two_hidden_roots…` 改名为 `abandonment_step_abandons_the_hidden_newest_root_unless_the_row_publish_lands_on_its_slot`：藏两条根时，较旧那一槽仍见证第二新那条根，这一形在今天的 crates 上造不出来；改成「藏一条、只坏第一个点名单元的第一份 ⇒ 最新那条被抛弃」与「断全部记录 ⇒ 抛弃集合为空」两格。
  - `mount_writable_then_raise_floor_…`、`mount_writable_trajectory_…`：节点改从测试模块里的 `abandonment_step_nodes` 取，k 只取 1、另加见证不可读、n1 取 1–6。原先靠 k = 4 那几格才有抬 F 的目标，k = 1 时 n1 ≤ 3 上限等于现行 F，所以要把 n1 放到 6（调试读数：n1 = 1–3 的节点上 `floor=ceiling`）。
  - `an_overwritten_…`（第 3 次跑模块）：故障表加两块系统配置槽读回全 0。
  - `fault_point_fires_…`：步号改取主路径上第一个至少有两次屏障的步。
  - `shortest_reproduction_on_today_is_a_rollback_cell` 改名为 `shortest_reproduction_with_the_witness_unreadable_returns_the_older_version`：前两步照跑，崩溃恢复那一步之前把系统配置最新槽清零，再断言 `Ok`、`literal_last=1`、`effective_content=Some(0)`、`older_version_returned`。
  - 装置本体（`abandonment_step_node_list` 等产物用的代码）没改，只改测试模块。
- 补丁（`patch/crates.patch`，只改这个 bin 一个文件，13 个 hunk）打在主工作区现状的副本上：`test result: ok. 95 passed; 0 failed; …`（草稿 `main-all-4.log`）；fmt 过；命名纪律过（6943 个名字）；这个 bin 上 clippy 0 条（红的是别人的 `e161_crash_state_dedup_and_time_split.rs` 3 处）。
- 变异（`prove_mutations.py`，每行都做：锚点子串计数 → 基线跑点名用例 → 施加 → 再跑 → 还原并比 sha256），在主工作区现状加补丁的副本上跑，12 行都是 `anchor_count=1 baseline=ok mutated=FAILED restored=True`（草稿 `prove-main2.out`）。**抓到 12 / 无效 0 / 没红 0**；内存撞顶、超时各 0。12 行分别是：
  - 第 659 行替换行（主表里的原锚点在今天的主工作区上命中 2 次，门禁 33 号红的正是它）；
  - 第 844、846 行（点名的用例改名）；
  - M36 与 M29（第二段草稿，M36 点名的用例改名）；
  - M39、M40（新加的两条）；
  - 第 551、845、849、850、947 行（点名的用例我改过，主表不用动）。
  另外，主表 68 行加第二段草稿 11 行，在补丁打上之后锚点全部恰好 1 次（第 659 行按替换行算）。
- 交付（`/tmp/claude-1000/e158-r4-seg2b/patch/`）：
  - `crates.patch`：sha256 `9a58a6cbec482530362ad5520a1937c03381adbca85fc9c11406ccde68e56e3b`，14:1x JST 对主工作区现状 `git apply --check` 过。
  - `mutations-replacements.tsv`：`5dff68a7…`。第 659 行用第二段执行员那一版，原样没改；第 844、846 行只改第六段。
  - `mutations-append.tsv`：`fa25b0a6…`。第 1 行替换第二段草稿 `/tmp/claude-1000/e158-r4-seg2/mutations-append.tsv` 的第 9 行（M36）；第 2、3 行是 M39、M40。
  - 行号对的是主工作区现在的 1242 行表，现查过 551–947 这几行仍是原来那几条。

## 六、岔路表（`research/prompts/c554-fix-forks.md`）

| 行 | 这一段的读数（算法：compare-r2 的 `grep 'name=r4_loss arm=<臂> '`） | 状态 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1（乙，用户已定） | 乙-配置：H-随 6/37、实七-甲 0/10、实七-乙 **0/6**（seg2 是 3/6，那 3 格是装置撞键）、实八 0/2。H-随 那 6 格全是 ③c（`q3c_lost_cells=6`）；分解：不算 ③c 2 格，Q1 只算环内且不算 ③c 1 格 | 第二段的数已够写页。按字面读，乙在 H-随 上仍丢写 > 0；怎么读 ③c 按认定第 3 条并列两栏，交写页 | 不能：第二段四族都跑完了，这次只改了实七-乙那 3 格 |
| 2（乙-配置续，用户已定） | 与乙-配置逐族相同：6/37、0/10、0/6、0/2 | 同上 | 同上 |
| 3（乙 / 乙-窄读，记欠） | 这一段没量读缓存峰值（照派发）。乙-窄读 两条 -配置臂的丢写与乙逐族相同 | 记欠，不变 | — |

多拒、多读（Q4、Q5）：seg2b 的 compare-r2 与 seg2 只差上面那 8 行 `r4_loss`，所以多拒、多读的行逐字节没变。

## 七、门禁（登记给 experiment-runner 的 14 道；这次不写实验页）

命令是逐个 `nice -n 19 bash .claude/gate.d/<文件>`，日志在草稿 `gate-<阶段>.log`。

| 阶段 | 退出码 | 首行，或点名的是谁 |
|---|---|---|
| 33 | 1 | `✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次…`：只点名第 659 行（`grep -c '^ *crates/mutations.tsv:'` = 1），替换行在 `patch/` |
| 52 | 1 | `✗ 段序列登记表与 E142 产物…对不上，共 2 处`：E142 / layout kb，不是我动的 |
| 80 | 0 | `✓ 154 个实验二进制各自至少有一条绝对值断言…` |
| 96 | 0 | `✓ 实验源码纪律：扫了 154 个文件（读不动的 0 个）…` |
| 27 | 1 | `✗ 格式常量在 kb 与实验源码之间对不上`：e142、e157，不是我动的 |
| 34 | 1 | `✗ 实验索引与正文对不上 1 处`：E161，不是我动的 |
| 40 | 1 | `✗ 有实验产物没被 experiments.md 点名…`：里面有这次 14 份 seg2b（`grep -c r4-seg2b` = 14）。这次不写页，是预期内的 |
| 69 | 1 | `✗`：E162 装置，以及 m2-kb-writeback 报告里引了 `/tmp`；e158 命中 0 |
| 75 | 0 | `✓ 决策与实验双向登记对得上…（查了实验页 160 个、决策 28 条…` |
| 84 | 0 | `✓ …点了名（点名 15 个）` |
| 85 | 0 | `✓ …（点了产物的 16 页里判了 16 页）` |
| 86 | 1 | `✗`：E162、E163，不是我动的；「删 research 下那些文件」那一句没照做 |
| 88 | 0 | `✓ …（判了 15 行，对照 155 份登记着的产物）` |
| 99 | 0 | `✓ 登记了路径与共用项的实验页判过了（4 份、19 条路径）…` |

## 八、没做什么

- 没写实验页、索引行、`experiments-history.md`（派发要求）。**`replay.sh` 没登记 seg2b**：臂副本在 `/tmp`，今天那一臂要先跑，其余臂要 `SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT` 与 `SINGLEFS_E158_INVESTIGATOR_REPORT` 两个环境变量（同 seg2）。这 14 份新产物还没有实验页点名，门禁 40 号红在它们身上，交写页那一段。
- 没改登记。seg2b 的装置修改、compare 两份、快照指纹没算进装置这三样，要不要写进修订段，交主 agent。
- 没在主工作区就地改 bin、没追加 `crates/mutations.tsv`、没跑 59 号与任何重型测试、没提交。没判结论能不能推翻或确立决策。
- 第 20447 行只查了撞键次数，没把它换成分支的账重跑去比挂载行。登记没有读缓存峰值（岔路第 3 行），这次也没量。
- 臂副本上 clippy 红在快照原有的 e156 与 tests 两个文件上；主工作区副本上红在 e161。都不是我的改动，没修。

## 九、草稿与清理

- 删了：12 份诊断副本 `diag/<臂>`（today 491M，其余 11 份各 367–370M）；`dev-today` 1.7G、`main` 1.7G、`dev-debug` 1.8G、`main-debug` 966M；`lint-before`、`lint-after`、`lint-main` 各 1.1M。清单在 `deleted.txt` 与 `progress.md`。
- 没删 `/tmp/claude-1000/e158-r4-seg2b/arms/`（12 份臂副本，连 target，4.5G 左右）：写页那一段跑五段合并的 `r4-compare`、以后复跑 seg2b，都要这一套修过的二进制。
- 留着的文本：`report.md`、`progress.md`、`patch/`、`products.sha256`、`build-arms.txt`、`arm-bin-fix.diff`、`arm-bin-fixed.rs`、`bin-before-fix.rs`、`main-bin-original.rs`、`crates.patch.bad-contains-M40`（作废，留作对照）、`diag/diag-*.{out,err}`、`diag/patch_diag.py`、`mutlogs/`、`prove_mutations.py`、`prove-main*.{tsv,out}`、各门禁与测试日志。
