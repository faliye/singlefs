# m4-publish-lookup-followup-code-r2：云端正推腿报告（Sonnet，2026-10-23）

被核对象：第 11 批对第一轮判决 `research/prompts/m4-publish-lookup-followup-code-r1-main-verification.md` 第三节第 1–4 条的实现。立场：正推，核「代码做的是不是第三节 1–4 条要的那件事」。代码引文写冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r2/repo/` 里的路径与行号（`cite-check.py --root` 指它）。方案按 `crates/` 今天的实现来谈。

## 零、复跑与产物

模型目录 `research/prompts/m4-publish-lookup-followup-code-r2-sonnet-model/`（未改的补丁原文、探针、变异表、第二台上的跑法、原样输出）。复跑（整条经内存包装，约一个多小时）：

`bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-followup-code-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r2/repo --work <新的草稿目录>`

它在 `--work` 里拼出与我跑过的同形的树（根 = 冻结副本 crates；`before` = 倒打 `patch/crates.patch`；`probe` = 加比较次数计数；`dropless` = 把 `impl Drop` 改回 cfg 之下），依次跑 `tools/peer_trip2.sh`、`peer_trip3.sh`、`peer_trip5.sh`、`peer_trip6.sh`、`peer_mutants_par.sh`，最后与 `out/rerun-expected.txt`（83 行）逐行比，打「N of 83 lines found」。挂钟、纳秒、微秒与 `t3_drop`（IR 字节数带路径，随机器变）那几行不比，`t3_drop` 的几个数贴在 T3.5。`rerun.sh --selftest` 在这份改过 `rerun_steps` 的脚本上会红（模板自证拿模板自带的步骤判），没当绿贴。

跑在哪：本机 `run-with-memory-cap.sh` 的内存额度这几小时被别的跑道占着，我排的一条按主 agent 的消息停了，之后编译与测试都经 `scripts/hosts/multi-host-run.sh --peer --memory-cap 16G` 走第二台（七趟：标签 `m4-r2-sonnet-mut1`、`-mut2`、`-trip2`、`-trip3`、`-trip4`、`-trip5`、`-trip6`；mut1 跑到第 5 行时驱动报第二台在用核太少，我自己停了它，改成 mut2 五个工人并行）。本机只跑了 harness 档快用例基线与变异前 4 行。日志里第二台上的目录路径已换成「〔第二台的目录〕」（模型目录 `out/` 下的日志同）。

文件 sha256（前 16 位）：

| 文件 | sha256 前缀 |
|---|---|
| `patch/crates.patch`（与 `_m4-publish-lookup-followup-code-r2-diff.md` 第 3 行登记的前缀相同） | 2ac508a255cc5f50 |
| `mutations/batch11-append.tsv`（同上，780c9a47f9eced7a） | 780c9a47f9eced7a |
| `mutations/batch11-replaced.tsv`（同上，31d6204e26df04ea） | 31d6204e26df04ea |
| `mutations/batch11-all20.tsv` | bff738a935579d85 |
| `mutations/alt-mutations.tsv` / `alt2` / `alt3` / `alt4` | 9e21fd940731a18a / df2f77765938e1f4 / da600db0ff3ecdbc / b32c96abfc0e6182 |
| `mutations/old-rows.tsv`（`git show HEAD:crates/mutations.tsv` 的第 374、386 行） | ba40db42595a0619 |
| `tests/diff_history_remount.rs` / `probe_scans.rs` / `bench_positions.rs` / `carried_counts.rs` | 8788500dff1150c8 / 750659a2e0d5f001 / bccc274c6b1927bf / ef48097609c73329 |
| `tools/mutate_run.py` / `peer_mutants.sh` / `peer_mutants_par.sh` | f610c43c8d20a085 / 2c540e13d9d461ef / 9bb3b18cf820393e |
| `tools/peer_trip2.sh` / `peer_trip3.sh` / `peer_trip4.sh` / `peer_trip5.sh` / `peer_trip6.sh` | 52f21eb9f4a3f4a0 / 4a73ebf3b1b651c4 / 42c97c022c2bf08f / 6403e08db249fc1d / b08cf6f8e92f060b |
| `tools/probe_patch.py` / `t3_drop.sh` / `t4_cells.sh` / `t4_cells2.sh` / `rerun.sh` | 50e39583c4aaae42 / 7cf6783ec3222aa1 / e84d9e6f14ac964e / 219bbfd45b4ad088 / a941a990ae3ce7d7 |
| `out/rerun-expected.txt` / `out/mutants-summary.tsv` / `out/diff-before-seeds1-32.lines` / `out/trip2-name-lines.txt` | eac7c810031b03fc / 3a7ed4b16a6ebfc5 / 87e7a01c214bf19c / 8c72e1ee70795929 |
| `mutations/gen_alt.py` / `gen_alt2.py` / `gen_alt3.py` / `gen_alt4.py` / `tools/summarize_mutant_logs.py` | f6326140d91571c6 / e30baedf68239c80 / 80ce2a9f040dfc0b / ddc8a57154644c50 / 625d96222e4532c4 |
| `out/trip2-raw.txt` / `trip3-raw.txt` / `trip4-raw.txt` / `trip5-raw.txt` / `trip6-raw.txt` / `mut2-raw.txt` | e100f559ca2a39e0 / 25eb9f0cd36e9f26 / 3e74f4b346be37f3 / 632104c71bdc4223 / 61b4af2237bd1fae / 25e266641b7dc9ea |
| `out/mutant-logs/*.log`（42 份，逐份 sha256 再合算） | b2cc7d37259e7855 |

基线（冻结副本拷到草稿目录，harness 档本机跑）：`test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 40.16s`（`publish_lookup_by_identity`，13 条用例）。

补丁原文的来历核过：从 `_m4-publish-lookup-followup-code-r2-diff.md` 第 8–1343 行抽出的 `crates.patch` 倒打到冻结副本上，全部 hunk 成功（前几块偏移 18 行：冻结副本含别的会话之后合入的改动）；`batch11-append.tsv`、`batch11-replaced.tsv` 的 20 行在冻结副本 `crates/mutations.tsv` 里逐行原样命中（`grep -cFx` 每行 1 次）。

## 一、T1　修补本身：新索引与改前的线性找取不取同一项、写不写同一份字节

判定：**成立**。

### T1.1　新索引同一角色取哪一项

`SortedIdentityIndex` 先按身份稳定排序，再 `dedup_by` 丢后一项：「pairs.sort_by(|(left, _), (right, _)| left.cmp(right));」（`crates/singlefs-core/src/transaction.rs:3513`）、「pairs.dedup_by(|(later, _), (earlier, _)| later == earlier);」（`crates/singlefs-core/src/transaction.rs:3515`）。稳定排序保留进来的次序、`dedup_by` 丢掉后到的，留下表里最先那一项，与 `iter().find` 取最先同一项。`FirstEntryOfEachIdentity::of_the_table` 按表的次序收，所以同一结论。比较用的 `Ord` 与改前用的 `==` 是同一个类型派生出来的：「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]」（`crates/singlefs-core/src/transaction.rs:2483`，`TransactionUnit`），不会出现 `cmp` 判相同而 `==` 判不同。

### T1.2　逐处列：建处、查处、改前改后各取哪一项

| 处 | 改前（补丁原文行） | 改后（冻结副本行） | 同一项？ |
|---|---|---|---|
| 映射位置换进 `units` | `research/prompts/_m4-publish-lookup-followup-code-r2-diff.md:105` 起 `for (identity, key) in &version.mapped_units` 里对有位置的 `units.iter_mut().find(\|unit\| unit.identity == *identity)` 取最先一项改槽 | 先问完位置收成 `Vec`（`crates/singlefs-core/src/transaction.rs:3819`），一个都没有就 `return`（「if new_slot_of_each_mapped_role.is_empty() {」，`:3826`），否则建一份 `(身份, 下标)` 的索引，逐项 `get` 改槽（`:3842`） | 是：同一身份取下标最小的那一项；`mapped_units` 里有重复身份时仍按 `mapped_units` 次序逐项改，后一项盖前一项，与改前一样 |
| 装要写的单元 | `research/prompts/_m4-publish-lookup-followup-code-r2-diff.md:359` 起 `units.iter().find(\|unit\| unit.identity == *identity).expect("重写的角色都装了单元")` | `units_by_role.first_entry_of(identity)`（`:14735`）、`.expect("重写的角色都装了单元")`（`:14737`），索引建在 `:14730` | 是；panic 文案与条件不变 |
| 记录点名的单元 | `research/prompts/_m4-publish-lookup-followup-code-r2-diff.md:371` `.find(\|unit\| unit.identity == identity)` | 同一份索引（`:14745`），panic 文案不变 | 是 |
| 释放判定路径、经映射释放的角色清单：留给快照的清单 | `research/prompts/_m4-publish-lookup-followup-code-r2-diff.md:150`、`:241`：`.filter(\|identity\| !kept_for_the_snapshots.contains(identity))` | 两处都走 `roles_other_than_the_ones_kept_for_the_snapshots`（定义 `crates/singlefs-core/src/transaction.rs:4580`，调用 `:4414`、`:4923`） | 是：成员判断，不取项；次序不变 |
| 记录点名的数据角色 | `research/prompts/_m4-publish-lookup-followup-code-r2-diff.md:295` 同形 `.contains` | `roles_other_than_the_data_roles_named_before_the_last_record`（`crates/singlefs-core/src/transaction.rs:11857`） | 是 |

### T1.3　`take_the_mapping_positions_over_the_location_hints` 的几支

两个调用方给的 `position_of_key` 都是只读映射表的闭包，不碰被改的那一版：恢复重建一版时调「take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {」（`crates/singlefs-core/src/recovery.rs:3425`，闭包体只是 `mapping_position_of_key.get(key).copied()`），整理发批对上一版的拷贝调「take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {」（`crates/singlefs-core/src/transaction.rs:13300`）。所以「先问完位置、再改」与「边问边改」只差问的次序，问的次数（每个 `mapped_units` 项一次）与结果相同。

| 支 | 改前 | 改后 | 一样？ |
|---|---|---|---|
| 位置全空 | 逐项问、都 `None`，一处不改 | 收成空 `Vec`，`return` 在函数最后一句，等于不改；不建索引、不进那一处 | 是 |
| 部分有位置 | 有位置的才在 `units` 上找并改 | 同一批项建一份索引改（`:3842`） | 是 |
| `mapped_units` 里有 `units` 里找不到的角色 | `if let Some(unit)` 找不到就跳过，不 panic | `if let Some(position) … get` 找不到就跳过，不 panic | 是；两边都没有 panic 文案，`locations[0]` 是 `[LocationEntry; 2]` 的下标，不会越界 |
| 同一角色两把 key | 后一把的位置盖前一把 | `Vec` 按 `mapped_units` 次序逐项应用，后盖前 | 是 |

索引存的是 `units` 里的下标（`:3819` 之后的 `enumerate`），不借 `units`；改槽不改身份、不改下标，所以建好之后不会过期。

### T1.4　`publish_admitted` 里索引会不会过期

不会，由借用检查守住：「let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity);」（`crates/singlefs-core/src/transaction.rs:14730`）借的是 `&units`，索引与 `named_unit_of` 闭包活着时 `units` 改不了；冻结副本能编过（基线 12 passed），说明这一段没有对 `units` 的改动。`FirstEntryOfEachIdentity` 的文档也这样写：「索引借着那张表（`'table`）：索引活着时那一版改不了」（`crates/singlefs-core/src/transaction.rs:3588`）。

### T1.5　三处 `.contains` 换索引：重复角色、空清单

`.contains(x)` 与「`first_entry_of(x).is_none()`」互为否定，对留给快照的清单里有重复角色（索引去重只留最先一项，成员关系不变）与清单为空（空索引恒查不到，全保留）两种都相同；被过滤的 `roles` 里有重复时两边都按 `roles` 的次序逐项判，保留重复。`roles_replaced_via_mapping` 先于过滤求值，改后变成先 `collect` 再迭代，过滤闭包只读 `kept_for_the_snapshots`，循环体改的东西读不到它，所以急切求值与惰性求值结果相同。

### T1.6　同一串合法操作写出的盘镜像逐字节相同（独立的第二条路）

读码之外再走一条不共用代码的路：`before` = 冻结副本倒打补丁，`after` = 冻结副本，两份各编 `tests/diff_history_remount.rs`（以第一轮攻方腿的差分用例为底，加了「随机历史里夹重新挂载」与每步一份 `units` 槽位摘要），按同一串种子跑随机合法操作，每步一行（操作、结局、两块盘写过的扇区的 sha256、留存条目数等、槽位摘要），两份输出逐行比：

```
name=diff_run copy=after rc=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 73.02s
name=diff_run copy=before rc=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 87.58s
name=differential seeds=256 steps=64 lines=16896 lines_before=16896 before_vs_after=identical
name=differential_coverage op=relocate steps=1594 ok=1594 err=0
name=differential_coverage op=remount steps=631 ok=631 err=0
name=differential_coverage panics_after=0 panics_before=0 remount_ok=631
```

（原样输出在 `out/trip2-raw.txt`；两份 `.lines` 的 sha256 都是 `fe8d434b8811d0a652f0eaee2f6e0e7cea790474eaabe1d863084b1056e126a6`，`cmp` 字节相同。覆盖：新建 inode 1741 步、整份写 5100、覆盖写 986、建快照 1213、删快照 758、unlink 511、整理 2326、直接发一批搬迁 1594、抬 F 1524、重新挂载 631，全部 ok，0 次 panic。）这条路与读码不共用任何代码或采样。

没走到的：`units` 或 `mapped_units` 里同一身份出现两次的表（产品里角色唯一，差分的随机历史造不出）只靠读码；256×64 步不是穷举；崩溃态没有逐个枚举（比的是每步之后的镜像）。

什么现象会推翻 T1：同一种子下改前改后哪一步的镜像摘要或槽位摘要不同；或造得出 `units` 里同一角色两项而改后取了不是最先的那一项；或 `mapped_units` 里有而 `units` 里没有的角色让改后 panic。

校验这条路会红（往被测对象里塞已知故障）：`tools/peer_trip6.sh` 在 `after` 的拷贝里把「映射位置换进 `units`」改成只改槽号为偶数的那一半，编同一份差分，跑种子 1–32、每段 64 步，与改前同一批种子（`out/diff-before-seeds1-32.lines`，2112 行）逐行比：

```
name=differential_red seeds=1-32 steps=64 lines=2112 lines_before=2112 faulty_vs_before=different
name=differential_red differing_lines=379
```

（原样在 `out/trip6-raw.txt`；第一处差异在种子 2 第 62 步的整理，槽位摘要变了。）差分会判出「镜像不同」，所以上面 `identical` 不是路子本身判不出来。

## 二、T2　产生结论的方法：计数与用例判的是不是那件事

判定：**部分成立**。「每遍建一份」与「每项建一份、每项都进一次」分得开，20 行变异各自红在名字说的那一处；但有四类坏写法不红：三棵多层码 2 树里记账树的照抄在所有取样里是 0 个节点、计数之外的额外线性、没点名那一格（`None`）里的重建、构建脚本里 cfg 写死成假（T4）。

### T2.1　`index_builds == times_entered` 分不分得开「每遍」与「每项」

计数：「count(|work| work.times_entered += 1);」（`crates/singlefs-core/src/transaction.rs:3459`，在 `entering` 里，记进刚推进栈的那一处）。判法：「work.times_entered >= 1 && work.index_builds == work.times_entered,」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1478`）。

这一条等式本身分不开：每项都进一次那一处、每项现建一份时，建索引次数恰好等于进的次数。真正分开靠随后的第二半：查的次数是「遍数 × 每遍该查的数」。我造了「每项进一次并每项现建」的变异（`mutations/alt-mutations.tsv` 第 1、2、10 行，改的是 `crates/singlefs-core/src/transaction.rs:6322-6329`、`:14728-14741`、`:11861-11875`）：

| 变异 | 现象 | 红在 |
|---|---|---|
| ALT1 删快照每条条目进一次并现建 | 「建索引 16 次、进 16 遍」等式成立；查的次数 16 ≠ 16 × 放掉的 8 条 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1922`（`released >= 4 && kept_entries_released.index_lookups == passes_that_release * released`，第 1923 行） |
| ALT2 装要写的单元每个角色进一次并现建 | 装了 24 次，每次 24 个角色 | `:1571`（`(assemblies * rewritten, 0, assemblies * named_roles)`） |
| ALT14 记录点名的数据角色每个角色进一次并现建 | 78 遍、每遍查 1 次 < 每遍至少 8 次 | `:1543`（`work.index_lookups >= passes * data_units_rewritten`） |
| ALT3 释放判定路径留给快照的清单每个角色现建（进一次） | 进 2 次、建 47 次 | `:1477`（等式的左半） |

结论：分得开，靠的是第二半，且只在重写单元数 ≥ 2 时成立（取样 8、32、4 单元，都 ≥ 2）；单元数取 1 时每项等于每遍，分不开——现有取样里没有 1 单元的重写走这几处的计数判。

### T2.2　作用域：有没有查找落在任何一处之外、或套着记两处

`count` 只记最里层那一处：「let site = state.sites_entered.last().copied();」（`crates/singlefs-core/src/transaction.rs:3484`），所以套着不会一次查记两处；退出是按栈顶弹，不按身份弹：「state.borrow_mut().sites_entered.pop().expect(」（`crates/singlefs-core/src/transaction.rs:3472`），所以两个守卫必须后进先出。第 11 批里显式 `drop` 的两处都满足：「drop(inside_the_file_lookups);」（`crates/singlefs-core/src/transaction.rs:6249`，它是在 `SnapshotJudgement` 之后进的、之前退）、「drop(inside_the_destruction);」（`crates/singlefs-core/src/transaction.rs:6337`，早退的 `?` 由局部变量逆序析构保证后进先出）。

没点名那一格（`None`）里实际有什么，用探针量（`tests/probe_scans.rs`，原样在 `out/trip5-raw.txt`）：

```
name=probe scenario=write_size n=8   … none_site=Some(LookupWork { index_builds: 5, index_lookups: 65, … times_entered: 0 })
name=probe scenario=write_size n=128 … none_site=Some(LookupWork { index_builds: 5, index_lookups: 65, … times_entered: 0 })
name=probe scenario=pool_files p=16  … none_site=Some(LookupWork { index_builds: 5, index_lookups: 236, … times_entered: 0 })
name=probe scenario=pool_files p=256 … none_site=Some(LookupWork { index_builds: 5, index_lookups: 3871, … times_entered: 0 })
```

`None` 里建索引次数是常数 5（池里文件数 64 时是 6；其中一份是「let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」，`crates/singlefs-core/src/transaction.rs:13458`，别的几份我没逐个对到调用点），查的次数随池里单元数涨（每个照抄的单元查一次）。但没有任何用例对 `None` 里的建索引次数下断言（`for (site, work) in &lookup_work.work_of_each_site {`，`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:240`，这一条把 `None` 也算进去，只判「每次查至多折半那么多项」，不判建了几份）。我造了「第一个文件之外的文件照抄单元时每个单元现建一份上一版 `units` 索引」（`mutations/alt4-mutations.tsv`，改 `crates/singlefs-core/src/transaction.rs:14371` 起那一句 `carried_unit(` 的实参，记进 `None`），整份用例文件 13 条全跑：

```
name=mutant row=1 rc=0 test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out :: ALT-None 照抄第一个文件之外的文件的单元：每个单元现建一份上一版units索引（记进没点名的那一格），整份用例文件都跑
```

即第一轮判决第三节第 3 条 ② 里「没点名那一处（`None`）的建索引次数在两档大小下不涨」，第 11 批选的是「把作用域包住建」（`build_extent_tree` 与 `build_multi_level_tree` 开头进一处：`crates/singlefs-core/src/transaction.rs:12085`、`:15074`），这两处之内的建被收住了；`None` 里剩下的那些没有断言，坏写法不红。

### T2.3　按树分开的三处：只照抄一棵、三棵都照抄、一棵都不照抄

判法有「一个都没照抄就断言不建、不查、不解叶」这一支：「if carried == 0 {」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1455`）。量三个取样里三棵树各照抄几个节点（`tests/carried_counts.rs`，在第 11 批用例文件的原样拷贝上多打一行；deadlist 池那条是用例自己打的）：

```
name=carried test=whole_rewrite_no_snapshot carried_accounting_deadlist_mapping=[0, 0, 1] …
publish_lookup_by_identity deadlist_pool_rewrite data_units=8 carried_accounting_nodes=0 carried_deadlist_nodes=2 carried_mapping_nodes=10 …
publish_lookup_by_identity deadlist_pool_rewrite data_units=32 carried_accounting_nodes=0 carried_deadlist_nodes=3 carried_mapping_nodes=10 …
```

记账树在三个取样里一直是 0 个节点，所以它那一支只验「不建、不查、不解叶」，退回线性（查的次数仍是 0）与正确写法在这一格里没有区别。变异证实（原样在 `out/mutants-summary.tsv`）：

| 变异（`crates/singlefs-core/src/transaction.rs:15102-15106` 那句 `let carried = …`） | 判它的用例 | 结果 |
|---|---|---|
| 只有记账树退回线性（ALT-Accounting） | deadlist 池用例 | `rc=0` 绿 |
| 只有记账树退回线性（ALT3a / ALT3b） | 无快照整份重写 / 快照下重写加删快照 | `rc=0`、`rc=0` 绿 |
| 只有 deadlist 树退回线性（ALT3c） | 无快照整份重写（deadlist 照抄 0 个） | `rc=0` 绿 |
| 只有 deadlist 树退回线性（ALT-Deadlist） | deadlist 池用例 | 红，`:1600`（deadlist 照抄 2 个，0 遍） |
| 只有中央映射树退回线性（ALT-CentralMapping） | 无快照整份重写 | 红，`:1600`（映射树照抄 1 个，0 遍） |
| 记账树与 deadlist 树退回线性（批内第 10 行，MA4） | deadlist 池用例 | 红，`:1600`（deadlist 那半） |

所以「三棵都照抄」在取样里实际只有映射树一棵稳定被照抄，deadlist 树只在 deadlist 池里被照抄（`carried_deadlist_nodes >= 2`，`:2310`）；记账树从没被照抄过。第一轮判决要的「让记账树或 deadlist 树照抄到不止一个节点、按树分开数」对 deadlist 树做到了，对记账树没做到（记账树要长到多片叶才有照抄，4 GiB 稀疏盘的测试池里长不到，这一句是推的）。

### T2.4　取样分不分得出三类、20 行变异各自红在哪

两档（8 与 32 单元）、三批（2、8、3 项）：每项现建的写法在最小一档就红（ALT1/2/3/14 都在 4–8 单元的取样里红）；退回线性的写法 0 次查（第 2、3 行红在 `:1571` 的查次数）；每遍重建与每项重建的区别靠「查次数 = 遍数 × 每遍该查的」。批大小三档里，第三批（3 项、没有留存项）专为「留存条目索引不等要找就建」设的：批内第 15 行把那个索引提前建，前两批（有留存项）不红，只有第三批红在 `:2045`。

20 行逐条（冻结副本行号 = 变异改的原文起止行；红在 = 变异日志 `out/mutants-summary.tsv` 里第一处 panic 的位置，全部是点名的那条用例）：

| 行 | 改的是（`transaction.rs`，除注明） | 红在 | 判的是不是名字说的那件事 |
|---|---|---|---|
| 1 | `:3842-3844` 映射位置换进 `units` 改回线性 | `:2064`（`(1, roles_relocated, 1)` 的查次数 0） | 是 |
| 2 | `:14735-14737` 装要写的单元改回线性 | `:1571`，左 `(0, 0, 24)` 右 `(24, 0, 24)`：要写的那处查次数 0 | 是 |
| 3 | `:14745-14747` 记录点名的单元改回线性 | `:1571`，左 `(24, 0, 0)` 右 `(24, 0, 24)`：点名那处查次数 0 | 是 |
| 4 | `:4414-4418` 释放判定路径留给快照的清单改回 `contains` | `:1477`：那一处进 0 次 | 是（红的原因是辅助函数被绕开、没进那一处；同类写法见 T2.5 ALT4 survive） |
| 5 | `:4923-4927` 经映射释放的角色清单同 | `:1477`：进 0 次 | 同上 |
| 6 | `:11829-11834` 记录点名的数据角色同 | `:1477`：进 0 次 | 同上 |
| 7 | `:6328` 删快照每条现建留存条目索引 | `:1477`：进 2 次、建 16 次 | 是 |
| 8 | `:6244` 快照判定那一圈每问一个角色现建文件读者 | `:1477`：进 2 次、建 18 次 | 是 |
| 9 | `:4967` 经映射释放的角色清单每个角色现建文件读者 | `:1477`：进 1 次、建 9 次 | 是 |
| 10 | `:15102-15106` 记账树与 deadlist 树退回线性 | `:1600`：deadlist 照抄 2 个、0 遍 | 是，只判到 deadlist 那半（记账树 0 个节点） |
| 11 | `:12128-12131` extent 树每个节点在计数那处之外现建 | `:1600`：extent 树 47 个节点，建次数非 0 | 是 |
| 12 | 与第 7 行改后的源文件逐字节相同 | `:1477`（同第 7 行） | 重复 |
| 13 | 与第 9 行相同 | `:1477`（同第 9 行） | 重复 |
| 14 | 与第 8 行相同，点名的是快照下重写加删快照那条用例 | `:1477`：进 3 次、建 15 次 | 重复（只多一条判它的用例） |
| 15 | `:10101-10102` 留存条目索引不等第一次要找就建 | `:2045`：「搬 3 项（其中留存的 0 项）的一批：建 2 份索引」 | 是 |
| 16 | `crates/singlefs-core/build.rs:28` 交出前把环境里的 `CARGO_FEATURE_*` 也算开着 | `:2394`、`:2371`（外带变量的子进程里不绿） | 是 |
| 17 | `crates/singlefs-unit-graph/src/build_features.rs:145` 特性表不核清单 | `crates/singlefs-unit-graph/src/build_features.rs:619`：左 `Ok([])` 右 `Err(FeatureTableDisagreesWithTheManifest…)` | 是 |
| 18 | `:11824-11834` C491 整行替换（共享的提交内生块挪到第一条记录里点名） | `crates/singlefs-harness/tests/sequential_write_split_into_one_unit_transactions.rs:157`，左 13 右 1 | 是；改前同一行（旧锚点）红在同一处，见下 |
| 19 | `:4924-4926` C394 整行替换（只走这次重写的角色） | `crates/singlefs-core/src/transaction.rs:11567`（产品里的对账断言「resolve 判生死的那一组用户可见单元…」） | 是；改前红在 `:11471` 同一条断言，见下 |
| 20 | `crates/singlefs-unit-graph/src/build_features.rs:158` 本包特性取反 | `crates/singlefs-unit-graph/src/build_features.rs:594`：左 `Ok([])` 右 `Ok(["counting"])` | 是 |

三对重复，我把改后的源文件逐字节比了：

```
7 12 mutated-file-identical= True
8 14 mutated-file-identical= True
9 13 mutated-file-identical= True
```

所以 20 行里改后的源只有 17 种不同。第 18、19 行是整行替换：实现员报告「改前红在哪条没对照」，我把旧行（`git show HEAD:crates/mutations.tsv` 第 374、386 行，`mutations/old-rows.tsv`）用在 `before` 副本上跑：

```
old-before.row01.log … sequential_write_split_into_one_unit_transactions.rs:157:5 … 前一条只点名一个单元
old-before.row02.log … crates/singlefs-core/src/transaction.rs:11471:9 … resolve 判生死的那一组用户可见单元，就是释放判定路径按这次换下的全部角色走出来的用户可见那一组
```

（`out/mutants-summary.tsv`）改前改后红在同一条断言，只有行号因补丁加的行而挪了 96 行。第 19 行的红来自产品里的断言（用例只是把它触发出来），名字说的「释放之前读盘核校验和只走这次重写的角色」由那条断言判，不是用例自己的断言。

### T2.5　换一种同样坏的写法还红不红（我自造的变异）

`mutations/alt-mutations.tsv`、`alt2`、`alt3`、`alt4`，在第二台上跑，原样 `out/mutants-summary.tsv`、`out/mut2-raw.txt`、`out/trip3-raw.txt`：

| 变异 | 判它的用例 | 结果 |
|---|---|---|
| ALT1/2/3/14 每项进一次并现建（见 T2.1） | 各自的 | 红 |
| ALT4 留给快照的清单：索引照用，再多做一遍逐个 `contains` | deadlist 池用例 | **绿**（`rc=0`）：计数只数经索引的操作，额外的线性数不到 |
| ALT5 映射位置换进 `units`：索引照用，再多做一遍线性 `position` | 搬迁批用例 | **绿** |
| ALT12 映射位置换进 `units`：一个有位置的都没有也照建 | 搬迁批用例 | **绿**（源码注释说「一把有位置的 key 都没有的不进、不建」，没有用例钉它；整理那一支只在目的地表非空时才调它（「(Some(previous_version), false) => {」，`crates/singlefs-core/src/transaction.rs:13298`），恢复那一支几乎总有位置，所以今天无害，推的） |
| ALT-Accounting / ALT3a / ALT3b / ALT3c | 见 T2.3 | **绿** |
| ALT-None 没点名那一格每个单元现建 | 整份用例文件 | **绿**（12 passed） |
| T4b 构建脚本里特性的 cfg 写死成假 | 外带变量那条用例 | **绿**（T4） |

绿的这几条不是 11 批改坏了什么，是判法的边界：这套判法数的是经索引的建、查、走，用例看不到计数之外的额外线性，也看不到记进 `None` 的重建，也看不到从没走到的那棵树。

什么现象会推翻 T2「部分成立」的下半：记账树在某个现有取样里照抄到 ≥ 1 个节点（我量的三个取样都是 0）；或有用例对 `None` 里的建索引次数下断言（我读过用例文件，`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:240` 是唯一遍历 `None` 的地方，且不判建次数）。

## 三、T3　从未被看过的地方：第 11 批之外、同一条路径上的查找与第 11 批改动的副作用

判定：**部分成立**。第 11 批没有引入可量的新代价（恢复里的新索引一次一份、40 字节一项；小发布整次挂钟反而更短；不开特性时 `Drop` 在优化后的 IR 里只剩调试元数据）；但同一条路径上还剩两处没被七·6 点名、随规模平方（或两个规模的乘积）涨的线性查找，一处在恢复里、一处在发布里。七·6 的范围是 `.claude/kb/milestone/04-fourth-txn.md:143` 那一行点名的那些处（「| 七·6 | 发布路径按身份查：」起头），下面两处不在其中。

### T3.1　扫描方法与量法

先扫 `transaction.rs` 与 `recovery.rs` 里的 `.iter().find(`、`.position(`、`.contains(`、`.any(` 与折行写的 `.find(|`，一共挑出能在发布、整理、恢复、释放路径上随项数重复调用的候选，再在探针副本（`tools/probe_patch.py`：在这几处线性查找里给每次比较记一次数，只在草稿副本里改）上量比较次数。每个候选量两个自变量：一次写的单元数 n、池里的文件数 p（再加第一个文件的数据单元数 d、重新挂载时的文件数）。原样输出：`out/trip5-raw.txt`、`out/trip2-raw.txt`。

### T3.2　还剩的两处平方

**(1) 恢复里按 inode 号找 inode 记录，文件数的平方。**「let inode_record_of = |inode: u64| {」（`crates/singlefs-core/src/recovery.rs:3716`）在所有 inode 叶容器的记录上线性找「.find(|inode_record| inode_record.inode == inode)」（`crates/singlefs-core/src/recovery.rs:3720`），而对第一个文件之外的**每个**文件调一次：「inode_record_of(file.inode.0).ok_or(RecoveryFailure::UnitMalformed {」（`crates/singlefs-core/src/recovery.rs:3766`）。重新挂载时的比较次数（探针原样）：

```
name=probe scenario=remount files=64   units=339  cmp[unit_find=22274  file_find=0 inode_record_find=2145]
name=probe scenario=remount files=256  units=1320 cmp[unit_find=87628  file_find=0 inode_record_find=33153]
name=probe scenario=remount files=1024 units=5241 cmp[unit_find=390220 file_find=0 inode_record_find=525825]
```

文件数 ×4，`inode_record_find` ×15.5、×15.9（≈ 文件数²/2：1024²/2 = 524288）。外推到 10⁵ 个文件约 5×10⁹ 次比较（推的，没量）。这是恢复路径上七·6 前几批与第 11 批都没碰过的按项线性找。

**(2) 发布里首文件的数据单元 × 这次重写的角色数。**「Some(_) if !rewritten.contains(&identity) => carried_unit(」（`crates/singlefs-core/src/transaction.rs:14342`）：首文件被照抄的发布（别的文件被写）里，对首文件的**每个**数据单元在 `rewritten`（`Vec<TransactionUnit>`）上线性 `contains`。探针（`first_file_contains` 是这一处的比较次数）：

```
name=probe scenario=first_file d=8   first_file_units=8   … cmp[first_file_contains=176  …]
name=probe scenario=first_file d=64  first_file_units=64  … cmp[first_file_contains=1280 …]
name=probe scenario=first_file d=256 first_file_units=256 … cmp[first_file_contains=5120 …]
name=probe scenario=write_size n=8   first_file_units=1   … cmp[first_file_contains=22 …]
name=probe scenario=write_size n=128 first_file_units=1   … cmp[first_file_contains=142 …]
```

d 固定一次写 8 单元时 ≈ 20·d（这次重写约 20 个角色）；首文件只有 1 个单元时随一次写的单元数线性涨（22→142）。两个因子都随规模涨：乘积 d × R。这一处也不在七·6 的名单里。

其余候选量出来都是线性或常数，不平方（原样同一份探针输出）：

| 处 | 现象 |
|---|---|
| `TransactionOutput::unit` 的线性找（`crates/singlefs-core/src/transaction.rs:4150`，批外单次调用） | 一次发布里 214/358/934 次比较（单元数 42/66/162），≈ 5–6 × 单元数；恢复里 22274/87628/390220（≈ 66–74 × 单元数）：调用次数与规模无关，每次 O(单元数) |
| 分配记录树节点按角色找重写的单元（`crates/singlefs-core/src/transaction.rs:14418`） | 28/28/28、45、195（分配记录树 7/7/7、9、21 个节点）：节点数 × 重写的节点数，后者每次发布有界 |
| 释放判定里 `extent_lower_nodes.iter().any`（`has_a_placement_in_the_previous_version`） | 8–11，常数 |
| 照抄文件下段节点按位置找 | 15/63/255（文件数 16/64/256），线性 |
| 装下段节点按孩子找（`crates/singlefs-core/src/transaction.rs:12163`、`:12239`） | 所有取样都是 0 次（下段只有一层）；形状上是下段节点数的平方，没走到，推的 |

### T3.3　新索引在恢复路径上的内存与次数

`take_the_mapping_positions_over_the_location_hints` 在一次重建里恰好建一份、查 `mapped_units` 那么多次（探针，场景 D 的 `positions_site` 就是 `LookupSite::UnitsOfTheMappedRolesTakingTheirMappingPositions` 那一处）：

```
name=probe scenario=remount files=64   units=339  … positions_site=LookupWork { index_builds: 1, index_lookups: 334, … times_entered: 1 }
name=probe scenario=remount files=256  units=1320 … positions_site=LookupWork { index_builds: 1, index_lookups: 1309, … times_entered: 1 }
name=probe scenario=remount files=1024 units=5241 … positions_site=LookupWork { index_builds: 1, index_lookups: 5204, … times_entered: 1 }
```

内存：索引一项 `(身份, 下标)`：`name=bench_sizes size_of_identity_slot_index_pair=40`；`units` 里每个 `PublishedUnit` 头 64 字节外还带整单元的字节（`size_of_published_unit_header=64`；单元是 16 KiB 槽，`crates/singlefs-core/src/pointer.rs:13` 的注释「16 KiB 槽号」，字节数按槽算是推的），所以临时索引占 `units` 的千分之几。整理发批那一支在调它之前已经整版拷贝：「let mut relocated_version = previous_version.clone();」（`crates/singlefs-core/src/transaction.rs:13299`），这点索引比拷贝小得多。

### T3.4　建索引排序本身在小发布上更慢到可量吗

函数级（`tests/bench_positions.rs`，release，单次，第二台是共用机，没重复）：`take_the_mapping_positions_over_the_location_hints` 每次调用的纳秒，K = 有位置的 key 数，改前 / 改后：

| 单元数 | K=0 | K=1 | K=2 | K=8 |
|---|---|---|---|---|
| 339 | 13682 / 9399 | 17382 / 15175 | 18978 / 15839 | 32324 / 21179 |
| 1320 | 53702 / 35958 | 68889 / 56172 | 74294 / 59556 | 126769 / 79483 |
| 5241 | 146950 / 155201 | 193020 / 252678 | 207040 / 265225 | 289791 / 362151 |

5241 单元、K=1、2 时改后慢约 60 µs（30%）；339 与 1320 单元时改后更快，方向不单调，我没重复量，不下结论说哪个方向是真的。整次小发布（整份重写一个 1 单元的文件，连发 24 次，微秒，最小 / 中位数，改前 → 改后）：

| 文件数（单元数） | 改前 | 改后 |
|---|---|---|
| 64（245） | 6535 / 7363 | 4278 / 4562 |
| 256（1224） | 24910 / 27975 | 16875 / 17474 |
| 1024（5145） | 101116 / 132011 | 84053 / 87670 |

整次挂载（毫秒）：64/256/1024 个文件 46/173/788（改前）→ 32/121/543（改后）。所以函数级最坏多出 60 µs，占整次发布 84 ms 的千分之一以内；小发布整次反而短了 17%–38%（最小值 17%–35%，中位数 34%–38%）。可量的「更慢」只在函数级、5241 单元、K ≤ 2 一格，未重复。

### T3.5　不开特性时 `InsideALookupSite` 也实现 `Drop`

「impl Drop for InsideALookupSite {」（`crates/singlefs-core/src/transaction.rs:3468`）在不开 `lookup-work-counters` 时函数体为空。把 `singlefs-core` 的库在 release、不开特性、`codegen-units=1` 下编成 LLVM IR 数（`tools/t3_drop.sh`）：after（第 11 批终稿）、dropless（after 里把 `impl Drop` 改回 cfg 之下，即第 11 批之前的写法）、before：

```
name=t3_drop copy=after    … ll_bytes=233703254 define=1797 landingpad=2649 drop_in_place_InsideALookupSite_refs=4 resume=547
name=t3_drop copy=dropless … ll_bytes=233704764 define=1797 landingpad=2647 drop_in_place_InsideALookupSite_refs=2 resume=547
name=t3_drop copy=before   … ll_bytes=231635573 define=1788 landingpad=2634 drop_in_place_InsideALookupSite_refs=1 resume=540
```

`InsideALookupSite` 在 after 里多出的 2 处出现是调试元数据行（`DISubprogram(name: "drop_glue<…InsideALookupSite>"` 与 `DICompositeType`），没有函数定义和调用（`out/trip4-raw.txt` 把每个出现的行都打了）。after 比 dropless 多 2 个 `landingpad`（2649 对 2647，千分之一以内），函数定义数相同（1797）。before 比 after 少 9 个定义是第 11 批别的新增函数。没有可见的多出分支或代价。

什么现象会推翻 T3：(1) 的恢复比较次数在文件数 ×4 时涨不到 ×10 以上（这就不是平方）；(2) 首文件被照抄的发布里 `first_file_contains` 不随首文件的数据单元数线性涨；T3.4 的「整次发布更快」被重复量推翻；after 的 IR 里出现了 `InsideALookupSite` 的 drop 函数定义或调用。

## 四、T4　构建脚本与判法单元键这一链

判定：**部分成立**。核心层构建脚本交出的特性确实只认 cargo 这一次真开的，几种编法都对；但「开着要报开着」这一个方向没有任何登记的检查钉住（cfg 写死成假照绿），代码注释里点名的一条单测不存在，另有一种手设 `RUSTFLAGS` 加 `--target` 的边界编法键与真编不符。

### T4.1　几种编法下构建脚本各报什么（第二台上真 cargo 编，原样）

每格一个编译目录，读 `target/**/build/singlefs-core-*/output` 里的 `cargo::metadata` 行，并看 `-vv` 里编库与编构建脚本的 rustc 命令行里的 `--cfg 'feature=…'`（`tools/t4_cells.sh`、`t4_cells2.sh`；`out/trip2-raw.txt`、`out/trip3-raw.txt`）：

```
name=t4 cell=environment core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features= ]
name=t4 cell=config_env core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features= ]
name=t4 cell=harness_normal core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features= ]
name=t4 cell=plain cargo_rc=0 lib_cfg=[] script_cfg=[]
name=t4 cell=plain core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features= ]
name=t4 cell=feature cargo_rc=0 lib_cfg=[--cfg 'feature="lookup-work-counters"' ] script_cfg=[--cfg 'feature="lookup-work-counters"' ]
name=t4 cell=feature core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS ]
name=t4 cell=harness_test cargo_rc=0 lib_cfg=[--cfg 'feature="lookup-work-counters"' ] script_cfg=[--cfg 'feature="lookup-work-counters"' ]
name=t4 cell=harness_test core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS ]
name=t4 cell=rustflags_cfg cargo_rc=0 lib_cfg=[--cfg 'feature="lookup-work-counters"' ] script_cfg=[--cfg 'feature="lookup-work-counters"' ]
name=t4 cell=rustflags_cfg core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS ]
name=t4 cell=rustflags_cfg_with_target cargo_rc=0 lib_cfg=[--cfg 'feature="lookup-work-counters"' ] script_cfg=[]
name=t4 cell=rustflags_cfg_with_target core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features= ]
name=t4 cell=stale second_build_ran_core_build_script=0 second_build_recompiled_core=0
name=t4 cell=stale core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features= ]
```

（前三行来自第一趟 `t4_cells.sh`（`out/trip2-raw.txt`），那一趟的 cfg 列因 grep 写错是空的、只取 `metadata` 列；其余来自第二趟 `t4_cells2.sh`（`out/trip3-raw.txt`），`environment`、`config_env`、`harness_normal` 三格的 cfg 列没取到。第一趟里 `plain`、`feature`、`harness_test`、`stale` 的 `metadata` 列与第二趟逐格相同。行尾空格去掉了。）

- harness 档 dev 依赖打开 `lookup-work-counters` 而普通依赖没开（`crates/singlefs-harness/Cargo.toml:18` 的 dev 依赖、工作区 `resolver = "2"` 在 根 `Cargo.toml` 的第 4 行）：同一次构建里 `singlefs-core` 只有一个编译单元、一个构建脚本输出，特性是并集——`cargo test -p singlefs-harness` 报 `LOOKUP_WORK_COUNTERS`，`cargo build -p singlefs-harness` 报空。不存在「同一次构建里两份构建脚本输出」的情形，所以 checker 档读到的就是它链进来的那一份核心层的那一个输出。
- 外带变量（环境、`--config env`）不改构建脚本的输出：「cfg!(feature = "lookup-work-counters"),」（`crates/singlefs-core/build.rs:20`）是 cargo 编构建脚本时给的 cfg（上面 `script_cfg`），不是进程环境。
- `cell=stale`（先不外带编一次、再外带变量编同一个编译目录）：第二次既没重编核心层也没重跑构建脚本，输出留着空。这是对的：特性没变，外带变量不再是构建脚本的输入（构建脚本只有「println!("cargo::rerun-if-changed=build.rs");」，`crates/singlefs-core/build.rs:39`），要重跑的只有 `build.rs` 变了或特性变了（特性一变，编构建脚本的 cfg 变、单元重编、重跑）。

### T4.2　特性表手写：清单加一个特性而表没加，谁红

- 核心层：构建脚本自己不核，不报错；它不依赖 `singlefs-unit-graph`。代码注释说「`build_features` 单测把两边逐个比（`the_core_build_script_lists_every_feature_of_its_manifest`）」（`crates/singlefs-core/build.rs:14`），**这条单测不存在**：`grep -rn 'the_core_build_script_lists_every_feature_of_its_manifest' crates scripts .claude`（在冻结副本里）只命中这一行注释本身。`build_features.rs` 里与核心层构建脚本有关的那条是「fn the_core_build_script_reports_under_the_key_read_here() {」（`crates/singlefs-unit-graph/src/build_features.rs:632`），只判构建脚本文本里含键名那句。真正逐个比表与清单的是 harness 档那条：「names_in_the_table,」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:2418`，与 `feature_names_of_the_core_manifest()` 比）。我造了把表里特性名写错一个字母（ALT T4a，`mutations/alt2-mutations.tsv` 第 1 行）：红在 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:2417`（「构建脚本的特性表与 singlefs-core 清单 [features] 逐个相同」）。所以核心层的表只在 harness 档测试被跑到时才红；不跑那条用例，漏写一个特性的后果是：那个特性开着、元数据里却没有它，checker 档的键按没开算。
- checker 档：「let features = features_of_the_crate_being_built(」后面接 `.unwrap_or_else(|error| panic!("本包这一次开了哪些特性取不了：{error}"))`（`crates/singlefs-checker-tier/build.rs:89-94`）在 `main` 里无条件执行，所以清单 `[features]` 与表（`crates/singlefs-checker-tier/build.rs:44` 起的三项）对不上时，该包的**每一种**编法都红，不是只在某一条上。清单是 `default`、`verdict-store`、`gpu`（`crates/singlefs-checker-tier/Cargo.toml:37-40`），表是三项，对得上。对应的单测会红：批内第 17 行（`crates/singlefs-unit-graph/src/build_features.rs:145` 的 `if` 写成 `if false && …`）红在 `crates/singlefs-unit-graph/src/build_features.rs:619`，原样在 `out/mutants-summary.tsv`。

### T4.3　「开着要报开着」没有登记的检查钉住

批内第 16 行（外带变量也算开着）红了，说明「外带变量不算」被钉住。反方向：把核心层构建脚本里那一格的 cfg 写死成假（`mutations/alt2-mutations.tsv` 第 2 行，T4b）：

```
name=mutant row=2 rc=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out :: T4b 核心层构建脚本：特性的 cfg 写死成假（开着也报没开）
name=mutant row=3 rc=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out :: T4c 核心层构建脚本：特性的 cfg 写死成真（没开也报开着）
```

harness 那条用例把构建脚本编进测试包（`#[path = "../../singlefs-core/build.rs"]`），而测试包没有这个特性，所以 cfg 在那里恒为假：写成真会红，写成假看不出来。判法单元键的 cfg 原子从「开着」变成「没开」，等于第一轮 T4 要修的那个洞重开，而没有任何登记的用例或变异红它；只有 `t4-result.txt` 那一次性的真 cargo 观测（主 agent 的观测，我没拿到那份文件）。我自己在第二台上做的 `cell=feature`、`cell=harness_test` 两格证明当前代码是对的，但它们是一次性的格，没进变异表。

### T4.4　checker 档三项与键里 cfg 原子的真值是不是同一份

cargo 给本包库和给本包构建脚本的 `--cfg feature=…` 是同一份（上面 `lib_cfg`、`script_cfg` 两列逐字相同），所以构建脚本里 `cfg!(feature = "default")`、`"gpu"`、`"verdict-store"`（`crates/singlefs-checker-tier/build.rs:45-47`）就是库真编出来的特性。旁证：另一个会话此刻编同一个包的 `rustc` 命令行里 `ps` 看得到 `--cfg feature="default" --cfg feature="gpu" --cfg feature="verdict-store"` 三个（我没有自己编 checker 档，rocksdb 要现编，也不跑 checker 档测试）。`features_enabled_on_the_dependencies` 对带特性的工作区依赖要求它是直接依赖（`crates/singlefs-unit-graph/src/build_features.rs:218`）并读 `DEP_<links>_ENABLED_FEATURES`（`crates/singlefs-unit-graph/src/build_features.rs:229`）；checker 档对 `singlefs-core` 是直接依赖（`crates/singlefs-checker-tier/Cargo.toml:10`）。

一种边界编法键与真编不符（第二台原样）：手设 `RUSTFLAGS='--cfg feature="lookup-work-counters"'`（不是 cargo 特性）且显式 `--target`：

```
name=t4 cell=rustflags_cfg cargo_rc=0 lib_cfg=[--cfg 'feature="lookup-work-counters"' ] script_cfg=[--cfg 'feature="lookup-work-counters"' ]
name=t4 cell=rustflags_cfg core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS ]
name=t4 cell=rustflags_cfg_with_target cargo_rc=0 lib_cfg=[--cfg 'feature="lookup-work-counters"' ] script_cfg=[]
name=t4 cell=rustflags_cfg_with_target core_build_script_outputs=1 metadata=[cargo::metadata=enabled_features= ]
```

不带 `--target` 时 `RUSTFLAGS` 也作用于构建脚本，两边一致；带 `--target` 时库有这个 cfg、构建脚本没有，元数据报空。这条编法键里 `CARGO_ENCODED_RUSTFLAGS` 不同于没设时的键（`crates/singlefs-checker-tier/build.rs` 文件头写了读它），推的是不会与「特性关着」的键撞，没验。仓里的脚本今天不这样设变量（没全仓搜）。

什么现象会推翻 T4「部分成立」：有一条登记的检查在 cfg 写死成假时红（我读到的只有 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:2361` 那一条与 `build_features.rs` 两条）；或 `the_core_build_script_lists_every_feature_of_its_manifest` 在别处存在。

## 五、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 修补本身：取同一项、写同一份字节 | 成立 | 索引取表里最先一项（稳定排序加 `dedup_by`，`Ord` 与 `==` 同源派生），位置为空 / 部分 / 找不到的几支与改前同；差分 256 种子 × 64 步、16896 行两份逐字节相同（含 631 次重新挂载、1594 次直接发搬迁批），塞进故障的版本在 32 种子里 379 行不同，证差分会红 |
| T2 计数与用例判的是不是那件事 | 部分成立 | 等式本身分不开「每项建且每项进一次」，靠第二半「查次数 = 遍数 × 每遍数」才分开（ALT1/2/3/14 红）；20 行各自红在名字说的那一处，但有 3 对改后源逐字节相同（17 种）；记账树照抄在所有取样里是 0 个节点，单独退回线性、`None` 里每单元重建、计数之外的额外线性都不红 |
| T3 没看过的地方与副作用 | 部分成立 | 第 11 批无可量新代价（恢复里一份索引、40 字节一项；小发布整次短 17%–38%；不开特性 `Drop` 在 IR 里只剩调试元数据，+2 个 landingpad）；同路径还剩两处随规模平方或乘积涨的线性查找：恢复 `inode_record_of` 随文件数平方（1024 文件 525825 次）、发布首文件 `rewritten.contains` 随首文件单元数乘重写角色数 |
| T4 构建脚本与判法单元键 | 部分成立 | 真 cargo 几种编法元数据都对（外带变量不影响、dev 依赖并集时报开、stale 行正确），checker 档表与清单对得上且每种编法都核；但 cfg 写死成假不红、`crates/singlefs-core/build.rs:14` 点名的单测不存在、手设 `RUSTFLAGS` 加 `--target` 键与真编不符 |

## 六、没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；T2、T3、T4 里「第 11 批之外」的发现是线索，交主 agent 现查（尤其 T3.2 两处平方是否并进或记进优化项）。
- 没跑门禁阶段、checker 档测试、GPU、崩溃点重放与变异整表；没编 `singlefs-checker-tier`（rocksdb 现编太重），所以「checker 档真编出来的键与 `rustc` 的 cfg 逐格对」只有读码和 `singlefs-core` 一侧的真编格，键对拍（主 agent 的 `groups_with_a_different_line=0 of 66`）我没复核、也没读那份 `t4-result.txt`：复核不了，原因是它在别的会话的草稿目录里、编 checker 档要 rocksdb。
- 第 11 批合入后验证的数（全包 299 个测试二进制、变异轻型、形式化、崩溃轻型）没复核，不属于我这一格。
- 差分是 256 种子 × 64 步的抽样，不是穷举；`units`、`mapped_units` 里同一身份出现两次的表只靠读码；崩溃态没逐个枚举。
- 微基准（T3.4）每个数只量了一次，在共用的第二台上，没重复，没有方差；方向不一致的那一格（5241 单元，K ≤ 2）我没下结论。
- T2.5 里绿的几条自造变异是判法边界的实测，不是说第 11 批改坏了什么；`mutations/alt*.tsv` 没往 `crates/mutations.tsv` 里加（写范围外），要不要登记交主 agent。
- 本机 `run-with-memory-cap.sh` 排队的几条（一条我自己因等得太久停了，另一条按主 agent 的消息停了；每次一个进程号、自底向上单独停），之后编译与测试都走第二台。第二台的驱动几次报「在用核太少」（我的跑法有大段是单线程的用例与逐条变异），那几条告警是我的跑法造成的，不是别的异常。
- 差分、探针、微基准、变异都是在冻结副本的拷贝上做的，冻结副本本身一个字节没改。
