# m4-validate-code-r3 攻方腿报告（Opus，2026-10-26）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 -->

立场：假设改法甲（`of_a_file_without_data_units` 记录在就写标签 0、不看 size）与 D8 已定项 14 写回那一句是错的，在冻结副本的拷贝上造历史去打。被判对象照冻结副本 `~/.local/share/singlefs/work/m4-validate-code-r3/repo/`；我引的十份文件与主工作区此刻逐字节相同（`crates/mutations.tsv` 除外，别的会话改过，下文不引它的行号）。

## 复跑

模型目录 `research/prompts/m4-validate-code-r3-opus-model/`。复跑命令（整条经内存包装；`tools/rerun.sh` 拷自 `scripts/agents/three-way-rerun-template.sh`，只改了 `rerun_steps`，仓根与冻结副本从参数接）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 r3_attack_probe 与六份删除和标签 0 用例; 范围=7 个测试目标; 为什么=复跑 m4-validate-code-r3 攻方腿的扫与变异' \
  bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-validate-code-r3-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-validate-code-r3/repo --work <新的草稿目录>
```

它把冻结副本拷进 `--work`、打上 `src/probe-core.patch`（只加探针开关，开关全关时与冻结副本同一行为）、放进 `src/r3_attack_probe.rs`，依次跑 `tools/peer-b2.sh`、`peer-b3.sh`、`peer-b4.sh`，每份输出规范化之后与 `out/` 里那一份比，打 `name=<批>/<文件> same_as_model_out=yes|no`。规范化去掉 `seconds=`、`examples=`、只列样例的几种行（样例是哪几段随线程先后变）与 `name=compare` 行。模板自带的 `--selftest` 第一格在改过的 `rerun_steps` 上不成立（它在假仓里找 `src/probe-core.patch`），这一份没跑通自证。

我的三批是在第二台跑的（本机内存 slice 被别的会话占满，`run-with-memory-cap` 排队 486 秒以上，照共用约束改用 `scripts/hosts/multi-host-run.sh --peer`）；`rerun.sh` 在本机跑同样三份脚本。

`SHA256SUMS`（`sha256sum -c` 24 行全 OK）：

```
a6f269c7e21f596000d6119151f3b39095b5670568e84ae82f37ef22786ececb  ./out/b2/mutant-0.txt
c3b0817830eddac63c6aa8520007212c7ea45d9c96cc1a02f71a46a1b8d4412f  ./out/b2/mutant-1.txt
242dc6c804ea68dfc193e82494c2413c8221fae529f15065b96a7660c1427e03  ./out/b2/mutant-2.txt
ba5fefd3eeeb5d58b4278a7741e87fdde7532ec100942cc82ce56ddb7e782175  ./out/b2/mutant-3.txt
f007fc6b8ecc4480fa866f03edb7ccadebc055f6d43401fca4ef50f7d45ff97c  ./out/b2/mutant-4.txt
239982f1625a278cf070a3cf8b7fc022be2809dba9269e4af07f9931b92f28dd  ./out/b2/mutant-5.txt
d10e12d81523ccd096a9f1eb1446d3979c6eaa763dde071a6f18c9828f08cea2  ./out/b2/mutant-6.txt
0c4df15e4254135fe68464aa6a7c92ae65065d4be6e9d942312deff0ef702269  ./out/b2/mutant-7.txt
7305e9f4d5a6c0ca4d8feb4294689364c3f7cfa34db5dcd5dd7ba62fe1379e86  ./out/b2/sweep.txt
dd66760497f59f4f138035313621dab186e2d2b5e06f07ff6bff3366fa968658  ./out/b3/sweep-mutant-6.txt
2a821c1a7b949b7ebe6183c802bc93322bfa05ee4eff59bb074a7eb488754ee5  ./out/b3/sweep-scrubbed.txt
3474df8a70ffb40a14b3ad6dd30fbeaafdb3b95348c8b7a54d97c8c580c1a369  ./out/b4/detail.txt
0f460059798ecaf93fbfb1a89dbea89015ecb4c273ab08f126299261420c7d5e  ./out/b4/mutant-6-unlink_in_histories_judged_by_the_model.txt
a1a6a1699f20155c40c2ad55788ee31298ae059f7b5a0e19bebc2771b9f1183a  ./out/b4/mutant-6-unlink_mount_continuation_and_the_emptied_version.txt
988366d667f8da899707acfd716d3603954a5b939843bc984451aae5bf43910f  ./out/b4/mutant-6-unlink_of_files_other_than_the_first.txt
8ece73916d241cffa02f1e0969481d5c2e74b421eb3a96b777cc47a71c17ac9a  ./out/b4/mutant-6-unlink_single_file_publishes.txt
5742fd8ddcd6e9b22862af680a69acf0a80de00bd312ee0afd46f1dba99f385c  ./out/b4/mutant-6-writes_and_unlinks_after_the_first_file_was_deleted_in_histories_judged_by_the_model.txt
85192a1dfb3441654b318e6d216882283886413f7637a5233fd48bd7e9fb6a16  ./out/b4/sweep-batch-one.txt
acafe7900bbd39d97f958459a6dd8091fe499f6a2220ce2da19f0bea9306ae63  ./src/probe-core.patch
9df6a1c9890d5880b129107178022601ccc707d93af049f6ae40ce3a112bf96f  ./src/r3_attack_probe.rs
6a7bdf1197c583edcbf7a2680d0e6fb35b1716d8ec366991f91cca2b39a4a5b0  ./tools/peer-b2.sh
4ad45f303b4ba4fa62a799b698f13f7083625a8f2ca5cafe2c469850da8efb56  ./tools/peer-b3.sh
6689d8d75c59c748146893a2afa2f379fcd07636c7bbfbf1487c516f15e636b7  ./tools/peer-b4.sh
a6b5f4e91a9cbaa1ab1367b22418101a2ff6ff655658ced61460ff9d1a53f854  ./tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 改法甲本身 | 没打中 | 正常起点 4369 段（一批一个单元那一档另 5220 段）、23748 版里「有记录、0 个单元」0 版，改法甲与旧规则写出的字节 0 段不同；标签 0 起点上不同的那几段，改法甲之后四处读者全收 |
| T2 已定项 14 写回那一句 | 没打中 | 标签 0 配 size 0 在四处读者上 0 次拒；`with_the_tail` 与「标签 3、4」那一句在 0 个单元、size 0 上只给一个答案 |
| T3 C900 来路与判据 | **打中（清单不全，两处）** | ① 反方向「上段有第一个文件的标签 0 条目而 inode 树没有它的记录」池级 checker 同样判不出，C900 只写了一个方向；② 标签 0 的第一个文件只要被放掉的单元 0 还在盘上，I-1.13 的读法 Z 就把它认回来、判红，没有欠账写这一形。C900 要的判据在 23748 版产品路径版本与改法甲之后的标签 0 起点上都不误判 |
| T4 第二轮的数 | **打中（归因）**；数照得上 | 改法甲之后拒收 0 种、读出的长度与内容都对；但第二轮判决把 I-1.13 算进「手造泄漏」那四条红不成立：不带泄漏的起点照样 I-1.13 红，病根是盘上留着的单元 0 被扫描重建认回 |
| T5 用例与变异 | 没打中（整个 harness 档上） | 八种改坏里修补一那份用例文件抓到 6 种；漏掉的一种（上段形状把删完的第一个文件也算进去）由另两份 harness 用例抓到；修补一那份文件缺一个取样点 |

## 装置与取样

- 起点镜像（每个只建一次，内存稀疏盘 `SparseBlockDevice`，之后每段历史从内存里那一份拷）：
  - `normal`：mkfs → 取号 → 暖机 → 新池新建文件（3000 字节、标签 2），同 `common::build_pool`，只是不建文件镜像。
  - `hand_made` / `hand_made_zero`：第二轮两条腿与修补一用例那一种手造标签 0（size 3000），后者再在会话里截到 0；带一条已分配无引用的手造泄漏。
  - `generated_3000` / `generated_zero`：副本里的生成器开关（截断最后一批一个单元都不留，`src/probe-core.patch` 改 `truncate.rs` 两处）打开，由写者自己截掉单元 0，写出标签 0；分配记录与记账由写者算，不带泄漏。开关只在造起点那一次发布打开，之后关上。
  - `scrubbed_3000` / `scrubbed_zero`：同上，再把被放掉的单元在盘上的字节清零。
- 用户动作（放开扫，只固定起点）：截断到 0、100、3000、7000、三个净荷，建 1 个、142 个 inode，按号写 inode 2、143，覆盖写第一个文件 100 字节，顺序写第一个文件一个净荷加 500 字节，删 inode 1、2，建快照，回退到起点那一版，重挂（进程重启）。一批一个单元那一档另加「顺序写第一个文件五个单元」，删除与截断分几批走。长度 0 到 3 的全部序列：16 种动作 4369 段，17 种动作 5220 段。
- 每一版（`versions_published` 里每一版，删除与截断的中间版都在内）：内存里 `inodes_with_an_extent_upper_leaf_entry` 与盘上沿这一版上段根读出的条目集合比；第一个文件是不是「有记录、0 个单元」；C900 那一条判据（有记录而盘上没有条目）；反方向（没有记录而盘上有条目）。
- 每段末尾：冷走读、冷恢复（看 journal）、可写重挂、只读挂载打开第一个文件从 0 读到 size，与按动作推出来的内容比；池级 checker（`check_pool_image`）比起点多出来的违例。
- 同一段在变异 0（改法甲，冻结副本行为）与变异 1（两个调用方都改回「size 0 取 None」）上各跑一遍，比最后的盘字节；带 `R3_PROBE_JUDGE_OLD=1` 时旧规则那一份也判读者与 checker。
- 估时：先跑 5 个起点各 40 段（`R3_PROBE_LIMIT=40`，长度 2），每个起点 0.2 到 0.3 秒（31 线程，每段两种变异各一遍）；按它估全量 4369 段一个起点约 30 秒，没超 40 分钟，没缩。实跑一个起点 22.5 到 37.4 秒。
- 崩溃态：这一条腿没有调 `enumerate_crash_states`（见末节「限度」）。删到一半崩溃的落点用两样代：每一批之后的那一版都核（一批一个单元那一档），与序列里的「重挂」一步（重挂走可写挂载的重建与续做）。

## T1　改法甲本身

**没打中。**

量过（第二台上跑的副本，`out/b2/sweep.txt`、`out/b4/sweep-batch-one.txt` 原样）：

```
$ grep -E "^name=summary" out/b2/sweep.txt out/b4/sweep-batch-one.txt
out/b2/sweep.txt:name=summary start=normal length=3 histories=4369 seconds=23.0 threads=31 versions_seen=10420 record_zero_units_size_zero=0 record_zero_units_size_positive=0 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=0
out/b2/sweep.txt:name=summary start=hand_made length=3 histories=4369 seconds=22.5 threads=31 versions_seen=10420 record_zero_units_size_zero=1236 record_zero_units_size_positive=6193 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=37
out/b2/sweep.txt:name=summary start=hand_made_zero length=3 histories=4369 seconds=29.1 threads=31 versions_seen=9632 record_zero_units_size_zero=3583 record_zero_units_size_positive=3058 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=502
out/b2/sweep.txt:name=summary start=generated_3000 length=3 histories=4369 seconds=30.0 threads=31 versions_seen=10420 record_zero_units_size_zero=1236 record_zero_units_size_positive=6193 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=37
out/b2/sweep.txt:name=summary start=generated_zero length=3 histories=4369 seconds=30.2 threads=31 versions_seen=9632 record_zero_units_size_zero=3583 record_zero_units_size_positive=3058 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=502
out/b4/sweep-batch-one.txt:name=summary start=normal length=3 histories=5220 seconds=28.4 threads=31 versions_seen=13328 record_zero_units_size_zero=0 record_zero_units_size_positive=0 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=0
out/b4/sweep-batch-one.txt:name=summary start=hand_made_zero length=3 histories=5220 seconds=36.2 threads=31 versions_seen=12460 record_zero_units_size_zero=3930 record_zero_units_size_positive=3264 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=546
out/b4/sweep-batch-one.txt:name=summary start=generated_zero length=3 histories=5220 seconds=37.4 threads=31 versions_seen=12460 record_zero_units_size_zero=3930 record_zero_units_size_positive=3264 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=546
out/b4/sweep-batch-one.txt:name=summary start=generated_3000 length=3 histories=5220 seconds=37.2 threads=31 versions_seen=13328 record_zero_units_size_zero=1330 record_zero_units_size_positive=6732 unverifiable_versions=0 shape_mismatches=0 record_without_entry=0 entry_without_record=0 histories_whose_bytes_differ_from_the_old_rule=38
```

- 正常起点：两档合 23748 版里 `record_zero_units_size_zero=0`、`record_zero_units_size_positive=0`，`histories_whose_bytes_differ_from_the_old_rule=0`。这个取样域里，产品写路径写不出「有记录、0 个单元」（不论 size），改法甲不改写出的任何一个字节。实现员那句「今天的写路径到不了这一形」在这个域里成立。
- 标签 0 起点上改法甲改了字节的：size 3000 起点 37 段（一批一个单元 38 段），size 0 起点 502 段（546 段）。这些段的最后一版，改法甲上四处读者全收、内容对（每个起点只有一行 `readers cold_walk=ok cold_recovery=ok writable_remount=ok mounted_reader=ok`），池级 checker 比起点不多一条红（没有 `new_violations` 行）。
- 读码对得上的几处（冻结副本）：截断最后一批 `crates/singlefs-core/src/truncate.rs:168`「data_units_existing.min(data_units_a_size_needs(target_size_in_bytes));」留至少一个单元；`crates/singlefs-core/src/truncate.rs:238`「let keeps_at_least_one_unit_or_had_none = data_units_after >= 1 || data_units_before == 0;」；删除的一批 `crates/singlefs-core/src/unlink.rs:193`「删除的一批留下至少一个单元、至少截掉一个（留下 0 个是最后一批）」；建 inode 从上一版水位起号 `crates/singlefs-core/src/transaction.rs:9210`「let first_new_inode_number = previous.inode_number_watermark();」，没有上一版时水位 `crates/singlefs-core/src/transaction.rs:330`「InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => FIRST_INODE_NUMBER + 1,」；不写内容的发布（建 inode、整理、建快照）extent 树整棵照抄 `crates/singlefs-core/src/transaction.rs:7101`「.expect("不写文件内容的发布接在带文件的一版之后：extent 树整棵照抄")」。
- 推的、没量：只供测试的水位开关（C479）把水位压回 1 时，`publish_new_inodes` 会建出 inode 1 的记录（size 0、0 个单元），extent 树整棵照抄、没有它的条目。两种规则下这一版字节相同（照抄不经 `of_a_file_without_data_units`），冷读一样拒。仓里用这个开关的场景压到 2、3、4，没有压到 1 的。

没打中的形状：上面 16 / 17 种动作长度 ≤ 3 的全部序列，起点 7 种；扫描重建之后的第一次发布没有单独造（池级 checker 的扫描重建不写盘，核心层没有扫描重建写回的入口）；整理换下上段叶这一路靠会话里自动起的整理，这一档里有没有起过没数。

## T2　D8 已定项 14 写回那一句

**没打中。**

对象是 `.claude/kb/decisions/08-核心索引结构.md:451`「标签 0 全零：第一个文件一个数据单元都没有（size 0，或 size > 0 整个文件都是洞）时，写者重写上段叶照写这一条」。

- 读者一侧：冷走读与冷恢复经 `data_unit_count_matches_the_inode_size`，标签 0 交「之后到 size 是洞」，0 个单元、size 0 落在 `crates/singlefs-core/src/write_request_split.rs:113`「if data_units_on_disk < data_units_implied_by_the_file_size =>」那一臂（0 < 1）、收；挂载态读者把标签 0 映成同一种。量过：`hand_made_zero`、`generated_zero`、`scrubbed_zero` 三个 size 0 起点，加上别的起点里截到 0 的那些段，四处读者每段都收（每个起点只有一行全 ok 的 `readers`）。size 0 起点上 `record_zero_units_size_zero` 合计 3583 + 3583 + 3583（一批一个单元档另 3930 + 3930）版，盘上条目与内存形状 0 处不等。
- `with_the_tail`：`crates/singlefs-core/src/extent_tree.rs:403`「(ExtentUpperLeafTarget::NoDataUnit, _) => ExtentUpperLeafTarget::NoDataUnit,」，不看尾部；写者给 0 个单元、size 0 算的尾部是 `crates/singlefs-core/src/extent_tree.rs:335`「if data_units < size_in_bytes.div_ceil(payload_capacity_in_bytes()).max(1) {」，0 < 1，交「有洞」，与标签 0 的 `tail()` 同一个答案。
- 「最后一个单元之后到 size 是洞的换成标签 3、4」：3、4 只从标签 1、2 换来，0 个单元时 `(None, None)` 那一臂交的是标签 0，`with_the_tail` 不把它换成 3、4。在 0 个单元、size 0 上这两句不给两个答案。
- 「这一形上 size 改大、改成原值、截短（截到 0 也算）都只改 inode 记录、标签留 0、不加单元」：截到 0 再变长到 7000、三个净荷，标签留 0，扫里没有一段出现第一个文件单元数从 0 变多（覆盖写与顺序写第一个文件除外，那两样是整份重写）。

没打中的形状：size 0 的标签 0 文件上做截断（到 0、100、3000、7000、三个净荷）、覆盖写、顺序写（一个与五个单元）、建快照之后再写、回退、删除、重挂，长度 ≤ 3 的全部序列。没造：快照里那一版的标签 0 文件经快照读（核心层没有按快照读的入口，只读挂载读的是现行那一版）。

## T3　C900 与第二轮打中那一形还剩哪些来路

**打中：未定项清单不全，两处。** C900 要的那一条判据本身没误判。

### C900 的判据在合法镜像上误不误判：没误判

C900 要的判据：`.claude/kb/checks-owed.md:579`「第一个文件的 inode 记录还在而上段叶没有它的条目，池级 checker 判红」。探针在每一版上按这一句判（`record_without_entry`）：正常起点 23748 版、改法甲之后七个标签 0 起点全部版本，命中 0 版。这里面包括删除与截断的每一批之后那一版（一批一个单元，五个单元的文件分五批）、重挂之后重建出来的那一版、回退之后的那一版、建快照之后的那一版。命中只出在旧规则（变异 1）上，例如 `hand_made_zero` 起点 `[NewInodes(1), OverwriteInode(2), Unlink(1)]` 等段（`out/b2/sweep.txt` 的 `old_rule` 行，`record_without_entry=2`），那几版冷读本来就拒。

旧版写者写出的池（第二轮那一版写者、HEAD 841b3beab 的写者）：第一个文件有记录而没有条目的镜像只来自它们在标签 0 镜像上的那几种发布，冷走读、可写重挂、冷恢复已经拒成「这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符」（`out/b2/sweep.txt` 的 `old_rule readers: cold_walk=refused(...)` 几行）。C900 的判据把它们判红，与读者同向，不算误判。

### 打中一：反方向的孤条目，池级 checker 同样判不出

形状：第一个文件没有记录，上段叶里却还有它的标签 0 条目。旧规则下截到 0 再删第一个文件就留下这一条（第二轮判决记过的那一形）。量过：

```
$ grep -nE 'entry_without_record=1 ' out/b3/sweep-scrubbed.txt | cut -c1-330
35:  old_rule readers: cold_walk=ok cold_recovery=ok writable_remount=ok mounted_reader=ok new_violations={} record_without_entry=0 entry_without_record=1 examples=["[Truncate(0), Unlink(1)]", "[Truncate(0), Truncate(0), Unlink(1)]", "[Truncate(0), NewInodes(1), Unlink(1)]"]
78:  old_rule readers: cold_walk=ok cold_recovery=ok writable_remount=ok mounted_reader=ok new_violations={} record_without_entry=0 entry_without_record=1 examples=["[Unlink(1)]", "[Truncate(0), Unlink(1)]", "[NewInodes(1), Unlink(1)]"]
```

`new_violations={}`：池级 checker 比起点不多一条红（这两个起点的基线不含 I-1.13，扫描重建那一条也没响）；四处读者都按「没有文件」收。池级 checker 的标签判法只从记录出发（`crates/singlefs-checker/src/walk.rs:12759`「(None | Some(DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit), _) => (」在逐条记录的循环里），没有记录的号上的条目它不看。改法甲之后探针 `entry_without_record` 在全部起点上是 0：这一形与 C900 那一形一样，改法甲之后只剩「写者漏删」一种来路，C900 只写了一个方向。

四句：
1. 分不分辨臂：不分辨第二轮的甲、乙、丙。它是池级 checker 的缺口，三个改法都碰不到。
2. 系统看不看得到：看得到。池级 checker 手里有上段叶全部条目与 inode 树全部记录，比一下就判得出。
3. 满足的是判据哪一句：C900 那一行「要拦什么」「怎么拦」两列只立了「有记录而无条目」那一半；「前置」一列的理由（改法甲之后只剩写者漏写一种来路）对反方向同样成立，反方向没有登记。
4. 改法在打中的格上还中不中：甲、乙、丙都还中（推的：它们都不动池级 checker）。我提的改法：池级 checker 在每一版上另判「上段叶条目的 inode 号在 inode 树里有记录」（只在我的模型上量过、被攻过零轮；推的，没实现）。

推翻条件：池级 checker 里另有一条不变量在这一形上判红，而探针因为只比「比起点多出来的违例」漏看了它（起点基线是 I-2.1、I-4.8、I-7.4）。拿一张不带基线红的镜像（例如正常起点用旧规则造不出来，要另造）复核能排掉这一条。

### 打中二：标签 0 的第一个文件与读法 Z（I-1.13），清单里没有这一形

形状：第一个文件是标签 0（size 0 与 size 3000 都一样），被放掉的单元 0 还在盘上、单元头的对象出生代就是这条记录的。写者自己写出的标签 0 起点（`generated_zero`、`generated_3000`，分配记录与记账由写者算、没有手造泄漏）一建好就 I-1.13 红：

```
$ cat out/b4/detail.txt   （R3_PROBE_DETAIL_CHARS=3000，节选 generated_zero 那一段）
name=start start=generated_zero baseline_violations={"I-1.13"} readers: cold_walk=ok cold_recovery=ok writable_remount=ok mounted_reader=ok
  baseline_violation I-1.13: 扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 1 处，第一处 key （码 1、出生树 65536、出生 txg 6、实例 2、事务号 1）：扫描重建 落点 {盘 0 槽 50182、盘 1 槽 50182}、遍历方向没有这一条；「每个 inode 的数据单元集合」差 1 处，第一处 key （inode 1、偏移 0）：扫描重建 （码 1、出生树 65536、出生 txg 6、实例 2、事务号 1）、遍历方向 没有这一条
```

txg 6 那个单元是截断意图重写的尾单元，最后一批把它放掉。读法 Z 的认领条件只看记录：`crates/singlefs-checker/invariants/01-块自描述.md:19`「有同一对象出生代的 inode 记录时三条都满足才认领——对象出生代与记录相同、偏移在 ⌈size ÷ 净荷⌉.max(1) 个单元之内」。size 0 与 size 3000 时 ⌈size ÷ 净荷⌉.max(1) 都是 1，偏移 0 的单元恒在里面，于是被认回来；遍历方向按标签 0 说「一个单元都没有」。把这两个单元在盘上的字节清零（`scrubbed_*`）I-1.13 就不响了，换成更早那条候选根（txg 3）读不通：`baseline_violations={"I-2.1", "I-4.8", "I-7.4"}`（`out/b3/sweep-scrubbed.txt` 第 1 行）。

四句：
1. 分不分辨臂：不分辨。标签 0 本身（第一轮「丑」以来的形）与读法 Z 之间的缺口，改法甲、乙、丙都碰不到；size > 0 也中。按「打中不分辨臂」办：这一格不拿来判甲，另立一笔账。
2. 系统看不看得到：扫描方向看不到。读法 Z 照设计不读 extent 树（扫描方向与遍历方向只共享格式解析），能分辨「单元 0 因标签 0 已死」与「遍历方向丢了单元 0」的只有上段叶那一条标签，扫描方向手里没有；记录的截断字段也判不死它（字段只判 ⌈N_w ÷ 净荷⌉.max(1) 及以后的单元，单元 0 不在里面）。属「判别子观测不到」：要改判据或盘上记号，改完是新一轮。
3. 满足的是判据哪一句：I-1.13 第三格「数据单元按读法 Z 加写序判死活」那一段的认领三条，加上「与遍历方向从最新根读出的三张表……逐项相同」。
4. 改法在打中的格上还中不中：甲、乙、丙都还中（推的）。

它在不在这一轮的范围里：D8 `.claude/kb/decisions/08-核心索引结构.md:465`「⚠️ 标签 0 配 size > 0 这一形，现有写路径自己写不出来（每条写路径都至少落一个单元），只来自读进来的那一版」。读进来的那一版里旧单元 0 还在不在盘上，取决于写它的那个写者；手造的、我的生成器造的都在。推翻条件：找到一种写出标签 0 的写法，它放掉单元 0 时盘上留下一样东西让读法 Z 判它死（或把单元 0 抹掉而不坏早先的候选根），那这一形就有不红的合法来路，打中二收回成「只是这几种造法的问题」。

## T4　第二轮报出的数能不能核

**数照得上；I-1.13 的归因打中。**

### 拒收 0 种、长度与内容对

`hand_made_zero` 就是第二轮两条腿的起点截到 0 之后那一版。旧规则（变异 1）下冷读拒收的段，扫里抓到的样例（`out/b2/sweep.txt` 里 `old_rule readers: cold_walk=refused` 那几行，只摘起点与样例三段）：

| 第二轮判决点名的后续 | 扫里对应的段 | 旧规则 | 改法甲 |
|---|---|---|---|
| 覆盖写号 2 | `[NewInodes(1), OverwriteInode(2)]` | 冷走读、冷恢复、可写重挂拒，挂载态收 | 四处都收、内容对 |
| 上段长高 | `[NewInodes(142), OverwriteInode(143)]` | 同上 | 同上 |
| 写号 2 再删 | `[NewInodes(1), OverwriteInode(2), Unlink(2)]` | 同上 | 同上 |
| 写号 2 再变长到 7000 | `[NewInodes(1), OverwriteInode(2), Truncate(7000)]` | 三处拒，挂载态 `open_refused` | 同上 |
| 截到 0 再删第一个文件 | `[Unlink(1)]` | 读者收，上段留孤条目（`entry_without_record=1`） | 不留（`entry_without_record=0`） |

改法甲之后：七个起点、全部 4369 / 5220 段，四处读者每段都收、内容等于按动作推出来的那一份（读出的长度就是 size，标签 0 读出全 0）。「12 种后续里 5 种拒收」的 12 种第二轮判决没逐条列；顺序写号 2（`SequentialWriteOfAnInode`）不在我的动作表里，那一种没核。

### 「手造泄漏 → 四条红」的归因：I-1.13 不归泄漏

第二轮判决 `research/prompts/m4-validate-code-r2-main-verification.md:34`「攻方腿报的起点镜像带一条手造带来的已分配无引用记录（I-3.1、I-3.11、I-1.13、I-3.15 四条红）」。三种起点的基线（每个起点那一行 `name=start` 原样）：

```
name=start start=hand_made baseline_violations={"I-1.13", "I-3.1", "I-3.11", "I-3.15"} readers: cold_walk=ok cold_recovery=ok writable_remount=ok mounted_reader=ok
name=start start=generated_3000 baseline_violations={"I-1.13"} readers: cold_walk=ok cold_recovery=ok writable_remount=ok mounted_reader=ok
name=start start=scrubbed_3000 baseline_violations={"I-2.1", "I-4.8", "I-7.4"} readers: cold_walk=ok cold_recovery=ok writable_remount=ok mounted_reader=ok
```

不带泄漏（分配记录照写者放掉）之后 I-3.1、I-3.11、I-3.15 没了，I-1.13 还在；手造起点的 I-1.13 说明同一种（`out/b2/sweep.txt` 第 32 行：扫描重建认回槽 50180 那个出生 txg 3 的单元）。I-1.13 跟着「盘上还留着单元 0」走，不跟着分配记录走（见 T3 打中二）。

结论变不变：不变。三种起点上，改法甲改了字节的段数相同（size 3000 起点都是 37 段），四处读者全收，池级 checker 比各自基线都不多一条红；旧规则下拒收的段落在同一批序列上。判决里「拒挂的那几层判的是 extent 树对 size，不读分配记录」那一句照得上；要改的只是括号里四条红的归因。

四句：
1. 分不分辨臂：不分辨，它只改基线红的来历，不改哪条臂中。
2. 系统看不看得到：池级 checker 看得到盘上那个单元，按读法 Z 认领。
3. 满足的是判据哪一句：同 T3 打中二。
4. 改法：不涉及。

推翻条件：在不带泄漏、单元 0 的字节也不在盘上、早先候选根也读得通的标签 0 起点上，I-1.13 仍红，那它另有来历。

## T5　测试与变异钉得住改法甲吗

**没打中（按整个 harness 档算）；修补一那份用例文件自己漏一种改坏，缺一个取样点。**

改坏的办法：副本里两个调用方（上段形状 `inodes_with_an_extent_upper_leaf_entry`、装上段叶 `build_extent_tree` 的 `(None, None)` 那一臂）各经一个探针入口，按 `R3_PROBE_MUTANT` 换规则；冻结副本那一份规则是 `crates/singlefs-core/src/extent_tree.rs:420`「Some(_) => Some(ExtentUpperLeafTarget::NoDataUnit),」。修补一的用例文件 `crates/singlefs-harness/tests/whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs`（9 条，其中修补一新加 4 条）在每种改坏下跑一遍（量过，`out/b2/mutant-<号>.txt` 原样的 `test result` 行）：

| 号 | 改成什么 | 修补一那份文件 | 另跑的 harness 用例 |
|---|---|---|---|
| 0 | 不改（改法甲） | 量过：`test result: ok. 9 passed; 0 failed` | — |
| 1 | 两处都改回 size 0 取 None（`crates/mutations.tsv` 里「m4-validate-code-r2 T1 甲」那一行的改法） | 量过：`FAILED. 5 passed; 4 failed`（修补一新加的 4 条全红） | — |
| 2 | 只有上段形状改回 size 0 不算 | 量过：`FAILED. 7 passed; 2 failed` | — |
| 3 | 只有装上段叶改回 size 0 不写 | 量过：`FAILED. 7 passed; 2 failed` | — |
| 4 | 只在 size 0 时写、size > 0 不写（派发提示点名的「只在 size 0 时写」） | 量过：`FAILED. 7 passed; 2 failed` | — |
| 5 | 记录没了也写标签 0（两处） | 量过：`FAILED. 8 passed; 1 failed` | — |
| 6 | 只有上段形状把记录没了的第一个文件也算进去 | 量过：`test result: ok. 9 passed; 0 failed` | 量过：`unlink_of_files_other_than_the_first` 红 1 条（`deleting_both_files_in_either_order_ends_in_one_zero_entry_upper_root`），`writes_and_unlinks_after_the_first_file_was_deleted_in_histories_judged_by_the_model` 红 1 条（`the_fast_inode_write_histories_delete_another_inode_after_the_first_file_was_deleted`）；另三份照绿 |
| 7 | 只有装上段叶在记录没了时也写标签 0 | 量过：`FAILED. 8 passed; 1 failed` | — |

派发提示举的「size 0 写标签 1」：标签 1、2 要一条下段根或数据指针，0 个单元时手里没有可填的指针，写不成合法的一条（推的，没造）。

变异 6 在镜像上看得见（量过，`out/b3/sweep-mutant-6.txt`，正常起点、对照变异 6）：`[NewInodes(142), Unlink(1), OverwriteInode(143)]` 一段写出的那一版，冷走读、冷恢复、可写重挂都拒成「extent 树节点一条条目都没有（缺席的一段不写节点）」（I-1.1）：删完第一个文件之后形状里还留着它那片叶，写 143 号那次把它写成零条目的非根叶。它走的是正常起点、产品写路径，与标签 0 无关。

补哪一个取样点（只在我的模型上量过、被攻过零轮）：修补一那份文件加一条「删完第一个文件之后，建到 143 号并写它，四处读者都收、上段第一片叶不在」。理由是改法甲把「记录在不在」变成了形状的唯一判别，那份文件里只有「截到 0 再删」一条从删除那一侧钉它，钉不住「删完之后再长高」。`crates/mutations.tsv` 里点名这份文件的五行（「m4-validate-code-r2 T1 甲」一行与「m4-validate-code-r1 T5 丑」四行）都不是这种改法，可另留一行。

没打中的形状：8 种改坏、修补一那份文件 9 条加五份删除用例；没在整个 harness 档 290 个测试目标上跑变异 6（只跑了五份删除用例），所以「别处还有没有抓到它的」只到这五份。

## 没打中的形状（合起来）

- 起点 7 种：正常、手造标签 0（size 3000、截到 0）、写者生成的标签 0（size 3000、0）、清零放掉单元的标签 0（size 3000、0）。
- 动作 16 种（一批一个单元档 17 种），长度 0 到 3 的全部序列；每个起点 4369 或 5220 段，合 14 次起点扫（第一批 5 个起点、清零 2 个、变异 6 对照 3 个、一批一个单元 4 个）。
- 每一版核四样（形状对盘、第一个文件形态、C900 判据、孤条目），每段末核四处读者与池级 checker；每段在改法甲与对照变异上各跑一遍比字节。
- T1、T2 两格在这些形状上一次都没打中。

## 这条腿自己的限度

- 崩溃态没有经 `enumerate_crash_states` / `enumerate_crash_states_versions` 枚举：第 3b 步要的那一种我没做。删到一半崩溃、回退到截断意图中间那一版这两问，我只核了「每一批之后发布出来的那一版」与「重挂之后重建的那一版」：崩溃恢复落到的根是这些版本之一，这一句是推的；撕裂写、屏障之间的子集、恢复之后的续做在撕裂态上没有跑。要补的话交主 agent：在这份探针的起点与序列上录写流、按崩溃点重放的枚举域展开、每个状态恢复之后跑池级 checker 与「有记录而无条目」「无记录而有条目」两条，状态数先用 `crash_state_count_with_torn_in_place_overwrites` 算。
- 跑在第二台上：本机 `run-with-memory-cap` 的内存 slice 被别的会话占满（排队里记的「slice 已占 40.1 GiB，总上限 40.1 GiB」），三批都经 `multi-host-run.sh --peer` 起；第二台的工具链与本机相同（`rustc 1.98.0`，驱动那一步核过）。`rerun.sh` 写的是本机跑法，我没有在本机实跑过它，`same_as_model_out` 那几行没见过。
- 生成器与清零都是副本上的手段：生成器开关让截断最后一批不留单元，这一步本身产品写路径做不出；清零放掉的单元会坏早先的候选根。三种标签 0 起点各有自己的基线红（T4 那三行），没有一种是池级 checker 全绿的标签 0 镜像。
- 动作表里没有顺序写号 2、建快照之后销毁快照、整理的显式触发、管理员回退到中间那一版（只回退到起点那一版）；回退被拒的（起点之后建过快照、文件删了）照拒数记，没换目标再试。
- 副本上的数不进 kb：要引的由主 agent 在入库装置上重做。

## 要主 agent 跑的命令

- 判决要引这几格的数时，在本机复跑模型（命令见开头「复跑」一节，工作目录随意，内存上限 8G、线程 31 由脚本经 `scripts/hosts/capped.sh 31` 设）。看 `name=<批>/<文件> same_as_model_out=` 那 18 行与末行 `name=rerun_finished scripts_rc= differ=`。这份脚本我没在本机实跑过。
- 崩溃态那一半（第 3b 步）没做，要的话新派一条同立场的腿接着做，起点与序列照 `src/r3_attack_probe.rs`。

## 运行环境与清理

- 开跑前 `ps` 看到别的会话在本机跑 `singlefs-checker-tier` 的 `cargo test`（经 12G 的内存包装）；我本机那一次 `cargo test --release --no-run` 在 `run-with-memory-cap` 里排队（记到「已等 486 秒」，slice 40.1 GiB 占满），我停了它（底层那一个 admit 进程，`proc.py stop`），改在第二台跑。第二台四趟（一趟估时、三趟正式）的树由 `multi-host-run.sh` 第 ⑦ 步删掉、回读不在。
- 删之前 `du -sh`：仓副本 `~/.local/share/singlefs/work/m4-validate-code-r3-opus/repo` 73M（含拷进去给第二台用的 `scripts/` 与 `.claude/` 几份），取回目录 `fetch`、`fetch-b2`、`fetch-b3`、`fetch-b4` 合 392K；本机没有建过编译目录。都删了，草稿目录剩 72K 的日志与草稿脚本。

## 没做什么

- 不判辩方那几格，不替主 agent 采纳；T3 打中二、T4 归因这两处是这一轮里新冒出的形，按第三轮的规矩是「零轮」，写进判决的交用户表还是另立一题由主 agent 定。
- 没跑门禁阶段、没跑 checker 档测试、没起 GPU；没在 290 个 harness 测试目标上整跑变异 6。
- 没读禁读清单里的文件（别的轮的腿报告与模型目录、这一轮辩方腿的报告与草稿目录、`records/`）。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-validate-code-r3-opus-output.md` 末行（写这一节之前跑的）：

```
  ✓ 核了 16 处引文、0 处裸行号，对上 16 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```
