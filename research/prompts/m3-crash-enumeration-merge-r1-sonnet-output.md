# 层 0 与崩溃放量合成一套：正推腿报告（设计轮第一轮，2026-10-10）

立场：正推。从今天的 `crates/` 与门禁推第三节那一种形态（F1–F10）落地后 K1–K7 每一格是什么样；不攻「要不要合」「提交跑不跑整域」「CPU 对拍跑不跑全部状态」。
基线：`git rev-parse HEAD` = 31e47115471ba5502a4531d25dffcc921be36899，工作区带大批已暂存改动；开工与收工各 `sha256sum -c` 快照两份清单，两次都是 96 份 crates + 4 份 kb 全 OK（不 OK 的行数 `grep -vc ': OK$'` 输出 0、0）。

## 0 先读这里

### 0.1 我自己跑了什么、没跑什么

- 仓副本 `~/.local/share/singlefs/work/m3-crash-enumeration-merge-r1-sonnet/repo-copy`（只拷 Cargo.toml、Cargo.lock、.cargo、crates），编译目录 `.../target`；交回前删。副本里加了探针（`research/prompts/m3-crash-enumeration-merge-r1-sonnet-model/apply_probes.py`），只往副本的测试文件里追加打印，不改判法。
- 跑过的（checker 档测试二进制，筛选词 `zz_probe`，只有「层 0 一侧」那几条探针，每个二进制经 `run-with-memory-cap.sh 6G` 与 `capped.sh 7`）：multi_record、unlink、fixed_script_stream、new_pool_file_creation_stream、position_addressed_trees 五个。输出原样在 `research/prompts/m3-crash-enumeration-merge-r1-sonnet-model/probe-l0.log`，下面 2.1 节贴摘要。
- **中途收到主 agent 的更正**：子 agent 一律不跑 checker 档（含不带 `--include-ignored` 的快用例）。收到时五个二进制已经跑完；此后没再起任何 checker 档测试二进制。放量一侧的探针（`AMP …` 行）因此没跑，写进第 8 节「要主 agent 跑的命令」。
- 中途收到「线程上限改成 7」：编译与上面五次运行都用 7（`capped.sh 7`）；开工时 `ps` 看到别的会话在跑一条 `cargo test -p singlefs-harness`（变异复跑），没有 qemu、fio、vm-bench、e152，照常 `nice -n 19` 起。
- 没跑：任何门禁阶段、全量、GPU、崩溃放量入口。放量一侧的所有数来自 records / 日志，标「记录」，不是我量的。

### 0.2 一眼看懂的几条（每条的出处在后面各节）

1. **复用键口径：代码是「流程进键、判法进判定行」**。`ReuseKey` 只由写表与父镜像算（`crash_identity.rs:89`），判定块按（块键，判法版本）存（`verdict_store.rs:47`、`:122`）。`crash_identity.rs:6` 文件头「复用键罩三样属性」是旧话，落 `crash_amplification.rs:7` 那一份。
2. **判法版本只罩 `crash.rs` 与 `crash_judge_gpu.rs` 出发的模块闭包**（`crash_identity.rs:839`），不罩 `core::mount`、`instance_table_sweep`、各测试文件里的观察者。F3 把「逐状态可写挂载、观察者」接进引擎之后，这些结果一旦随块复用，键就罩不全（K3）。
3. **放量只存「红项」**（`crash_amplification.rs:1162`），不存计数。层 0 的 `OracleCounts`、`states_by_publish`、`journal_differing_states` 这类绿状态上的计数没有存处；剪枝命中的块不会再给观察者看。F3 的「计数断言」要么扩库里每块存的东西，要么这些项只能冷跑（K1、K3、K5）。
4. **放量今天没有「GPU 对 CPU 逐状态比」这一路**：登记了 `crash_judge_gpu` 的类，第 ① 段整块由 GPU 判、CPU 不判；没登记才由 CPU 判、再把单元校验和另交卡对拍（`crash_amplification.rs:7120`、`:7135`）。GPU 与 CPU 的逐状态对拍今天只在专用对拍用例里（默认展开 8，`crash_judge_gpu_agrees_…rs:166`，只覆盖其头注释列的那几条流）。F4 要的「每个状态 CPU 也判」对放量是新增的一路，吞吐掉到 CPU 档（K5）。
5. **三个展开口径并存**：放量的「展开 N」（段写数 ≤ N 才展开，否则 `NotExpanded` 为 0 个状态，`crash.rs:1869`、`crash_amplification.rs:172`）、层 0 全量（`EveryProperSubset`）、层 0 快档的「甲二」（`InPlaceSubsetsWithCopyOnWriteNoneOrAll`，`crash.rs:1841`）。F6 的快档写「展开 6」，与今天 8 个快档用例以外的层 0 快档（甲二，9 个测试文件用它（`grep -c quick_tier_expansion\|enumerate_layer0_quick_tier` 命中非零的文件数））不是同一个域；实测态数不同（2.2 节）。
6. **今天两边的态数差 15728640 是同一个原因**：谱系记录追加使 D 的单元写 20→24（`crash_enumeration_fixed_script_stream.rs:96-97`）。第二条流、到回退 D、删除流三处「层 0 闭式 − 放量记录」都等于 2^24 − 2^20，算术核过（2.1 节）。
7. **整域盖不住的流不是「展开 30 盖不住」而是「全域在 u64 里算不出来」**：`ExtentLowerSegmentGrowsToTwoLevels` 在全展开下 `.expect("一段的状态数装得进 u64")` panic（`crash.rs:2011`），我的探针在这条流上就这么红了（log 第 44–47 行）。F7 的「本次未跑」是这类流唯一的出路，不是剪枝能救的。
8. **改名射程里，Rust 标识符改名会改判法版本**（词法摘要保留标识符，`crash_identity.rs:275`），等于强制一次全量重判；中文术语、文档注释、门禁名不会（K6、K7）。

## 1 K1　判别力不丢：逐项表

### 1.1 共同判法（两边都判，合成后一份）

每个状态的判法是 `Layer0StateJudgement`（`crash.rs:1261`）：两遍恢复（看 / 不看 journal）、两份 oracle、池级 checker（`check_pool_image`，`IMPLEMENTED_INVARIANTS` 92 条：`crates/singlefs-checker/src/image.rs:111`）、记录核对器两条判据。层 0 的 `enumerate_layer0*` 与放量 CPU 路（`judge_one_block`，`crash_amplification.rs:1266`）调同一个 `judge_layer0_state_range_against_the_version_table`（`crash.rs:1444`）；它交给回调的是（序号，崩溃镜像，判定）三样（`crash.rs:1452` 附近的参数 `judge_state: &mut dyn FnMut(u64, &CrashImage<'_>, &Layer0StateJudgement)`）。**所以 F3 的「观察者」口子在 CPU 路上不用动 `crash.rs`：回调已经把镜像与判定交出来了**（读码推的，没造出来试）。GPU 路不交镜像，观察者、再挂载做不了。

放量比层 0 多的：第 ②③ 段（事实表核对 `crash_facts`，`compare_judge_and_verifier` `crash_amplification.rs:3385`，只判 4 条结构判据，窄于池级 checker）、单元校验和上卡对拍（CPU 路）。层 0 比放量多的：下面逐项列的观察者、再挂载、计数断言。

### 1.2 13 条 `crash-case:`（`stage-inputs.tsv` 第 48–60 行，逐行 grep 过）

「合成后」列是 F1–F10 下这一项该落的位置；「丢」写会丢 / 变弱的东西与代码出处。

| 行 | 案 | 层 0 今天判什么、断言什么 | 逐状态观察者 / 可写挂载 | 计数断言 | GPU 对拍今天有没有 | 合成后落在哪、丢什么 |
|---|---|---|---|---|---|---|
| 48 | 第一条流 | 默认判法；全量 `layer0_enumerates_every_crash_state_…`（`new_pool…rs:639`）；甲二快档 `layer0_quick_tier_matches_the_full_tally_shape`（`:731`） | 观察者数 I-3.10 读集（`assert_allocation_generation_read_set_reached_through_the_record` `:708`）；无可写挂载 | `OracleCounts`（`:48`）逐项、`states_by_publish`、`assert_checker_counts`（`:133`）、闭式 16777260 | 有：对拍用例头注释列「新池新建文件」及三种改坏 | 放量里这条流已是四项的前缀点；新增一项 = `CrashFlow` 单版本表。**丢**：观察者与全部计数，除非 F3 的口子补齐。GPU 对拍保留（专用用例） |
| 49 | 第二条流（到 E 再卸载） | 默认判法 + `assert_the_rollback_floor_raise_was_judged`（`fixed_script…rs:646`）；闭式 1678377204 | 无观察者 | `states_by_publish` 逐项（`:109`）、闭式 | 放量一侧 through_unmount 断言单元校验和 `gpu_cpu_disagreements`（`through_unmount…rs:406`）；整判器对拍用例只列到回退 D | 与放量 `fixed-script-through-unmount` 合一项。**丢**：回退抬 F 那一段「判到了」的断言、`states_by_publish`（放量不存计数） |
| 50 | 会话推的抬 F | 默认判法；态数 ≤ 10^6 | **每个状态可写挂载一次**（`mount_writable_with_space_admission` `floor_raise…rs:271`），挂载后池级 checker | 闭式 | 无 | 放量 `floor-raise…` 只有一个点罩整串、**不挂载**。**丢**：逐状态可写挂载（引擎口子 ②） |
| 51 | σ（c561） | 只一遍看 journal 恢复 + 记录核对器 `claimed_state_missing_unit`（`record_checker…rs:461-473` 的 `judge`）；σ 一段 2^16 个子集（`:824`） | 无 | `enumerated.states == 65_536` | 有（σ 另一条，`:1064`） | 取放量的域与判法（判得更多：`:1235`「多判 oracle 与 checker」）；σ 之前、之后包成 ignored 点。**不丢**（判法是超集，域见 2.4）。快档放量 σ 在展开 6 下 16 写不展开 |
| 52 | 多记录 | 默认判法；全量无观察者；快档 `every_crash_state_between_the_records_…` 有发布边界观察者（`observe_the_publish_boundary` `multi_record…rs:231`，快档展开规则 `segment.len() < 10` `:296`） | 快档有观察者；无可写挂载 | 闭式 1358954634 | 无 | 放量 `multi-record-publish` 合一项。观察者本来只在快档：F6 快档要继续带它，且展开 6 与 `< 10` 在这条流上等价（段写数 24/28/30 都不展开，4 与 6 展开）→ 域相同，探针 `states6=141` |
| 53 | 删除（unlink） | 默认判法 + 观察者（发布边界、删除停在中间读回上一版）；闭式 1463812278（`unlink…rs:1028`） | **每个状态在工作线程上可写挂载一次**核续做走到空（`observe_every_state` `:736`、`observe_the_mount_continuation` `:545`） | `cut_in_*`、`mount_continuation_steps_*` | 有（删除线若干，头注释） | 放量 `unlink-after-the-multi-record-publish` 合一项（版本表带「没有文件的那几版」）。**丢**：逐状态可写挂载与续做计数；这是 14.6 亿态里最贵的一项，见 K5 |
| 54 | 树分裂七条 | 默认判法 `assert_clean`（`tree_split…rs:421`） | 无 | 各计数行 `exhaustive=` | 无（对拍用例里的树表条是别的流） | 放量 `tree-split-streams` 合一项；不丢 |
| 55 | 按位置寻址（四条） | 默认判法；`ENUMERABLE_IN_FULL` 四条（`position…rs:483`） | 无 | 枚举域闭式 | 无 | 放量跑 `ALL` 五条（`:515`）；第五条 F7 |
| 56 | io_min 8192 可写挂载 | 默认判法 + 观察者：两盘槽 0 都不可用的态上 checker 读槽 1 的槽距 8192（`enumerate_layer0_selecting_versions_observing_each_state` `io_min…rs:210`） | 观察者（无再挂载；挂载本身是被录的流） | 33 态闭式 | 无（`grep` 对拍用例里 io_min 只命中 I-7.5 的 12288） | 流不是 `build_pool`：自己的 `SharedStream`。**丢**：槽 1 核对；GPU 表几何从基线槽 0 读（`crash_judge_tables.rs:426` `base_geometry`），槽距是参数（`crash_judge_recovery.wgsl:154` `cfg.spacing`），读码推能判，没有对拍用例 |
| 57 | 崩溃注入快档 | `run_crash_injection_campaign`（`crash_injection.rs:4572`）：理想模型、挂载、二次崩溃；24 种子（`crash_injection_campaign.rs:45`），平常 cargo test 只有 4 种子的小快档（`:54`） | 随机历史 + 故障注入 | 无闭式（抽样） | 无（要真跑实现） | F5：不经录写流引擎。放量脚本明说它「不在表里」（`crash-amplification.sh:24`）。**不丢**，只是搬家 |
| 58 | 读故障态不持有见证 | 只判见证判据 W（`witness_criterion_holds_without_read_faults` `read_faulted…rs:63`） | 随机历史前缀 | 无 | 无 | 同 57 |
| 59 | 清扫挂载 | oracle/恢复/checker + 观察者 F5（`crash_points_of_the_instance_table_sweep.rs:511`） | 观察者 | 无闭式断言 | 无（`sweep|清扫` 在对拍用例里 0 处） | 基镜像是「有死实例的池」（`:235`）不是 mkfs 之后。抹头写是带字节的物理块长写（`instance_table_sweep.rs:605`），不撞 `ZeroFillInTheEnumeratedWrites`（`crash_judge_tables.rs:209`）；会不会撞 `WriteNotAlignedToALocation` 没核 |
| 60 | 清扫分阶段重挂 | 同 59 + **每个状态再起可写挂载**、挂载后只看 `INVARIANTS_JUDGED_AFTER_THE_REMOUNT`（`:69`、`:604`） | 再挂载 | 无 | 无 | 引擎口子 ②（再挂载）；GPU 不判这一截 |

### 1.3 8 项放量（`crash-amplification.sh` 第 227 行 `ITEMS=`，`--list-items` 事实表里整行抄过）

放量的共同点：判法 = 1.1 的 CPU 判法（CPU 路）或 GPU 判器（登记了 `crash_judge_gpu` 的类），只存红项；断言只有 red=0、态数录全、对账为空、（through_unmount）`gpu_cpu_disagreements == 0`；没有观察者、没有再挂载。

| 项 | 对应层 0 行 | 放量判什么 | 放量没有而层 0 有的 | 放量有而层 0 没有的 | 默认展开 / 全量深度 |
|---|---|---|---|---|---|
| fixed-script-through-rollback | 无 `crash-case:`（层 0 只在非 ignored 用例里钉段序列：`fixed_script_stream.rs:700` 起） | 全套 | 无对应 | 第 ②③ 段 | 6 / 30（`FULL_EXPAND_UP_TO` `crash-amplification.sh:219`） |
| fixed-script-through-unmount | 49 | 全套 | 抬 F 那段判到的断言、`states_by_publish` | 第 ②③ 段、单元校验和对拍 | 同上 |
| multi-record-publish | 52 | 全套 | 快档的发布边界观察者 | 同上 | 同上 |
| unlink-after-the-multi-record-publish | 53 | 全套（版本表带「没有文件的版本」） | 逐状态可写挂载、续做计数、发布边界观察者 | 同上 | 同上 |
| tree-split-streams | 54 | 全套 | 无 | 同上 | 同上 |
| position-addressed-trees | 55 | 全套 | 无 | 多一条 GrowsToTwoLevels | 30 盖不住它（F7） |
| floor-raise-pushed-by-the-session | 50 | 全套 | 逐状态可写挂载与挂载后 checker | 记录核对器与不看 journal 那遍 oracle（层 0 这条算了不断言） | 同上 |
| sigma-of-the-misaligned-reuse | 51 | 全套 | 无 | 两份 oracle、池级 checker、`root_without_record` | 同上 |

### 1.4 K1 的结论：F1–F10 下会丢、会变弱的，逐项指到代码

1. **计数断言（会丢，除非扩库）**：层 0 的计数来自 `Layer0Tally` 的字段（`crash.rs:747` 起：`root_persisted_states`、`no_file_states`、`file_read_states`、`journal_differing_states`、`verification_ran_states`、`checker_evaluated_states`、`states_by_publish`……）。放量库里每个状态只存红项的串（`crash_amplification.rs:1162` `EncodedVerdictVector(items.join(…))`，`red_items` 在 `crash.rs:1276`），绿状态存空串。所以「`verification_ran_states: 6`」「`journal_differing_states: 3`」（`new_pool…rs:695`、`:751`）这类绿状态上的计数在放量里没有存处，复用命中的块也不会再重算。F3 写「计数断言（闭式相等、`states_by_publish`、`OracleCounts` 逐项）」要成立，只有两条路：① 库里每块多存一份分类计数（`verdict_store.rs` 表结构要动）；② 带这类断言的项在默认剪枝下仍整条现判（K3、K5）。闭式相等（`tally.states == closed_form`）不受影响：放量按 `layer0_plan_state_count`（`crash.rs:1556`）对账，且已有「录的 + 库里已判的 = states」断言。
2. **观察者、再挂载（会丢，口子能补，但键要跟）**：CPU 路已有回调（1.1），补口子不需要碰 `crash.rs`。GPU 路不交镜像。再挂载要真跑 `core::mount` 与 `instance_table_sweep`，不在判法闭包里（K3）。
3. **GPU 对 CPU 逐状态比（放量今天没有，F4 是新增）**：见 0.2 第 4 条。今天 `ENABLE_GPU` 下放量的第 ① 段由 GPU 判、CPU 一个状态都不判；对拍只在专用用例里，列的流有限。第 ③ 段 `compare_judge_and_verifier` 比的是 GPU 判器对事实核对器的四条结构判据，不是对 CPU 全判法。
4. **σ 取哪个域**：取放量的（σ 之前 / 之后各一个 ignored 点，σ 那次发布一个点全展开）。判法是层 0 σ 的超集：层 0 σ 只有 `recover(Consult)` + `check_records`（`record_checker…rs:461-473`），放量全套（`:1235`）。域上层 0 σ = σ 一段 2^16 个子集（`:824`），放量 = 这一段的真子集 + 记录段 + 根槽 + 轮换 + 全持久（事实表 3.1 拆 65548 = 2^16 − 1 + 3 + 1 + 8 + 1，读码推的，没跑）。层 0 的「σ 全落」态 = 放量里单元段全落、记录段一个没落的那个态。**判法不丢；但快档放量 σ 在展开 6 下基本不展开**：快档的 σ 判别力要另给（F6 的「每项小规模」对 σ 是 6 写以内的段，即几乎只剩前缀点）。
5. **第 ②③ 段结构核对**：F3 只让固定流项走放量引擎，②③ 段保留在引擎里，不丢。F5 的随机历史两项不走引擎，也没有 ②③ 段（今天崩溃注入也没有）。
6. **崩溃注入、读故障**：判别力靠「理想模型 + 真挂载」，F5 说驱动不变，不丢；但「同一种快档 / 整域分法」今天就有现成的分法（4 种子小快档 `crash_injection_campaign.rs:54` / 24 种子 `:45`），合成时它不是展开深度而是种子数，F6 的「展开 6」对它没有意义（要另定一条等价说法）。
7. **快档域换了**：层 0 快档（甲二，9 个测试文件用 `quick_tier_expansion` 或 `enumerate_layer0_quick_tier_versions`）与放量快档（展开 6）是两个域。我的探针（2.2 节）：第二条流甲二 278 态 / 展开 6 261 态，多记录 144 / 141，删除 196 / 189，第一条流 46 / 45，到回退 D 132 / 125。F6 说快档 = 展开 6，意味着层 0 快档里那些「甲二」断言（例 `QUICK_TIER_STATES_THROUGH_THE_UNMOUNT: 278`，`fixed_script_stream.rs:118`）的数要重算或这些用例继续留着不并；没有写明哪一种，是 F6 的缺口。

## 2 K2　枚举域

### 2.1 我量的层 0 一侧：今天的树上每条流的态数（探针，原样取字段）

命令：`grep ZZPROBE research/prompts/m3-crash-enumeration-merge-r1-sonnet-model/probe-l0.log | sed -E 's/ sizes=\[[^]]*\]//; s/ table_digest=[0-9a-f]+//; s/ base_digest=[0-9a-f]+//; s/^ZZPROBE //'`，输出：

```
label=L0 multi-record writes=115 segments=19 states6=141 states28=285212811 states30=1358954634 states_full=1358954634 states_quick_tier_jia2=144
label=L0 unlink writes=231 segments=35 states6=189 states28=390070455 states30=1463812278 states_full=1463812278 states_quick_tier_jia2=196
label=L0 second-stream writes=481 segments=78 states6=261 states28=1678377204 states30=1678377204 states_full=1678377204 states_quick_tier_jia2=278
label=L0 through-rollback writes=195 segments=36 states6=125 states28=67502198 states30=67502198 states_full=67502198 states_quick_tier_jia2=132
label=L0 first-stream writes=41 segments=11 states6=45 states28=16777260 states30=16777260 states_full=16777260 states_quick_tier_jia2=46
label=L0 pat ExtentInlineToLowerSegment writes=35 segments=4 states6=25 states28=268435480 states30=268435480 states_full=268435480 states_quick_tier_jia2=26
label=L0 pat ExtentLowerSegmentBackToInline writes=29 segments=4 states6=13 states28=16777228 states30=16777228 states_full=16777228 states_quick_tier_jia2=14
```

`states6/28/30/full` 是探针对同一份写表按「段写数 ≤ N 才 `EveryProperSubset`，否则 `NotExpanded`」调 `layer0_plan_state_count`（`crash.rs:1556`）算的，规则与放量 `expansion_of`（`crash_amplification.rs:172`）一致；`jia2` 是 `quick_tier_expansion`（甲二）。第三条按位置寻址流（GrowsToTwoLevels）在 `full` 上 panic：

```
44:thread 'zz_probe_layer0_pat' (3075529) panicked at crates/singlefs-checker-tier/src/crash.rs:2011:14:
45:一段的状态数装得进 u64
```

（`crash.rs:2011` 是 `.expect("一段的状态数装得进 u64")`。）所以全域不是「30 盖不住」，是 2^约 300 根本没有 u64 的表示，F7 的登记不是权宜。

### 2.2 与放量记录比（记录是旧代码上的数，下面的差是算出来的）

| 流 | 层 0 今天（上面探针 / 代码里的闭式） | 放量记录（出处） | 差 |
|---|---|---|---|
| 第二条流到 E 再卸载 | 1678377204（`fixed_script_stream.rs:103` 与探针一致） | 2026-09-30 现判 1 610 875 015，全域 1 662 648 564（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35`） | 1678377204 − 1662648564 = 15728640 |
| 到回退 D | 67502198（探针；层 0 只在非 ignored 用例钉段序列） | 现判 51 773 558（同行） | 67502198 − 51773558 = 15728640 |
| 删除 | 1463812278（`unlink…rs:1028`；探针 35 段，最后四个单元段 26/24/22/24） | 2026-10-08 展开 30 `states=1448083638`（`records/2026-10-01-删除与并发事务.md:1292`） | 15728640 |
| 多记录 | 1358954634 | 2026-10-08 展开 30 `states=1342177375 reused=16777259`（同一行） | 0（1342177375 + 16777259 = 1358954634） |
| 多记录，展开 28 | 探针 285212811 | 2026-09-30 现判 268 435 552（`:35`） | 285212811 − 268435552 = 16777259 = 那趟复用的第一条流前缀，对上 |

2^24 − 2^20 = 15728640，四个差里三个恰好是它。第二条流的代码注释自己交代了原因：`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:96` 「回退那次发布追加一条谱系记录之后」、`:97` 「D 的单元写 20 → 24」。

### 2.3 合成后每条流的状态集合

合成后的整域（F6）= 每条流在「展开盖住每一段」下的域，用的就是 `layer0_plan_state_count` 与同一个枚举函数，所以在**同一份写表**上两边的域逐态相同（同函数，非巧合）。要问的只剩「写表是不是同一份」「两边各自多的几个态」。

| 流 | 合成后域 vs 今天两边的并集 | 两边各自多出的态 |
|---|---|---|
| 第一条流 | 等于层 0 全量（放量里是四项共有的前缀点，今天不单列） | 差一个态（推的）：层 0 的 16777260 含末尾「全部持久」那一个；多记录那趟只复用了 16777259（`records/2026-10-01-删除与并发事务.md:1292` 的 `reused=16777259`），第一条流的末态在后继流里是后一段的空子集、归后继的点。合成后若单列第一条流，这一个态要在新项里现判 |
| 第二条流 | = 层 0 全量 = 放量展开 ≥ 28 的域（探针 `states28 = states_full`）。放量展开 6 的域（261）是它的子集 | 无（同函数同写表的前提下）；前提见 2.4 |
| 到回退 D | 放量独有（层 0 没有这一项的全量案）。合成后要不要登记，F2 写「`fixed-script-through-rollback` 与第二条流按事实表核出来的关系合或分」：**核出来的关系是前者的 36 段是后者前 36 段**（探针两条段形的前 36 项逐项相同），但被判根与版本表不同，放量文件头说「只差被判根与版本表的两条流不互相复用判定」（`crash_amplification.rs` 文件头第 9–10 行，`JudgeInputsOfTheFlow` `:500`）→ 合成时到 D 要单列一项，不能当第二条流的前缀 | 放量多 67502198 个态，不与第二条流共享判定 |
| 多记录 | = 层 0 全量 = 放量展开 ≥ 30（`states30 = states_full`）；放量展开 28 盖不住 30 写那一段（探针 285212811 ≠ 1358954634） | 无 |
| 删除 | 全域要展开 ≥ 30（探针 `states28 = 390070455`，`states30 = states_full = 1463812278`；最长段是前 19 段里多记录共有的 30 写） | 无；旧记录里的 1448083638 是旧代码的数（上表） |
| 树分裂七条 | 同函数同 `prepare(stream)`；层 0 闭式 78905428 = 放量记录 78 905 428（`:35`） | 无 |
| 按位置寻址 | 层 0 四条（`position…rs:483`）⊂ 放量五条（`:515`）；第三条 GrowsToTwoLevels 全域无 u64 表示（2.1），所以合成后整域 = 四条全域 + 一条「本次未跑」（F7） | 放量多一整条 |
| 抬 F | 同一个 `record_the_floor_raise_pushed_by_the_session`（层 0 `:171` 与放量 `:315` 调同一函数），域相同（推，段形没量） | 无域差；层 0 多的是逐状态可写挂载，不是状态 |
| σ | 放量的域 ⊇ 层 0 的 2^16（推，事实表拆分：2^16 − 1 + 13） | 放量多 13 个态（记录段 3、根槽 1、轮换 8、全持久 1），层 0 的 σ 全落那一态与放量里「单元段全落」同集 |
| io_min 8192 / 清扫 / 崩溃注入 / 读故障 | 不进放量引擎的域（口子 ①②）或抽样，没有「两边并集」问题 | — |

删除流最长一段的核对：探针 `sizes` 里 35 段最长的 30 在前 19 段，所以展开 30 刚好盖住、展开 28 不盖，与 `crash-amplification.sh:219` 的 `FULL_EXPAND_UP_TO=30` 一致。

### 2.4 「第二条流两套造流代码」要补什么才敢说同一条流

读码结论（没跑放量一侧）：两份造流代码是逐步复制的，不是共用函数。内容生成器两边各抄一份（`SECOND_FILE_BYTES`、`later_content` 的 `3000 + seed`、`% 253`、`% 251` 在 `fixed_script_stream.rs:50-54`、`:169-177` 与 `through_unmount…rs:54-55` 相同，grep 过）；重开可写挂载、回退、抬 F、卸载在放量里经 `common_crash_points/mod.rs` 的步骤函数（`mount_writable_step` 等），层 0 里是内联；切段两边都经 `writes_and_segments_with_stream_indexes_and_entries`（层 0 `:246`，放量 `:173`）。

今天只断言段形（78 段）与写数（481）相同（`through_unmount…rs:179-192`，文件内注释 `:186` 写「与 crash_enumeration_fixed_script_stream 登记的相同」）。**字节相同没断言。**补法有两条，我只给读码能支持的：

1. 合成时只留一个造流函数（F3 说「同一条流今天两份全量用例合成一份」）：这样「是不是同一条流」是构造出来的，不用断言。代价：`fixed_script_stream.rs` 里 `Script` 的 5 个变体（只到 B、到 C、到 D、到 E 卸载、E 复用首槽）与残留记录、陈旧 journal 尾巴那几个靶向用例都靠同一个 `prepare`，统一的造流函数得带 `Script` 参数；放量一侧的 `RecordedEntrySpan` 入口段、`CrashPointRecorder` 要保留。
2. 先不动造流代码，各打一个**整张写表（带屏障）的摘要加基镜像摘要**并比相等：探针就是这么做的（`step_write_table_with_barriers(writes, segments, 0..writes.len())`，`crash_identity.rs:182`，与 `SectorImage::of_base(base).digest()`）。层 0 一侧我已量出（`probe-l0.log`）：第二条流 `table_digest=8623730353a96115ff825f1d0b9aa33f4cf345b69fb83b37233856d6d74069fc`，到回退 D `cc055c3b61b6c888f2ace6f5c30c5098f5489644b886601835c2c9688fec8a93`，多记录 `525fa9d2edecf140b4ea6c65d733e0611870af3c4f21171082003272f571de0a`，删除 `84ecfef945d2f6f7f6006e20c80e3acec2f48dfa57b3accd5aa07cc70cabd70f`，第一条流 `d2a4268b6d9b75392ac984145862f19d7a8a35b41e2fcba102895970c3e6fb77`；基镜像摘要五条相同（`4b6d45f9380b…89e9`）。放量一侧的同名摘要**没量**（第 8 节命令）。主 agent 量出来对不上的，就是这一格的反例。

### 2.5 删除流两边差 15728640 在今天的树上对不对得上

- 读码推：对得上，**两边今天应是同一个数 1463812278**。理由：放量的删除用例与层 0 调同一组函数（`publish_the_unlink_intent`、`publish_the_next_unlink_step`、`assert_the_shape_of_the_deletion_steps`：层 0 `unlink…rs:296`、`:334`，放量 `:1115`）；差值与第二条流的差同为 2^24 − 2^20，而第二条流的原因有代码注释（谱系记录追加）。
- 缺的一环：放量一侧只断言删除步的形状（`:1115`），**不断言 35 段的段形**；放量脚本头里的探针段形末尾是 `22,2,1,2,20,2,1,2`（`crash-amplification.sh:42`），层 0 现在断言的是 `…22, 2, 1, 2, 24, 2, 1, 2`。放量一侧的 `AMP unlink` 探针行一出（第 8 节）就能定：`states30=1463812278` 则对上；不是这个数则「同一函数」的推断被推翻。
- 旧数的现状：`stage-inputs.tsv:53` 仍写「枚举域 1448083638 个状态」，`:52` 仍写 12230590578（多记录今天 1358954634），`:51` 仍写 σ「2^18 个状态」（今天 2^16）。`fixed_script_stream.rs:865` 的 ignore 理由写「全量五十多亿个状态」，该流今天 1678377204（还有一处事实表第 6 节没列的旧数）。

## 3 K3　复用、剪枝与逐项

### 3.1 键今天罩什么、F6 的「按条复用」要罩什么（逐样输入）

一条判定的存储键 =（块键，判法版本）。块键 =（输入指纹 = 复用键，流程名，计划哈希，块起点）（`verdict_store.rs:122` 把块键字节接上判法版本）。

| 输入 | 今天谁罩、罩在哪 | 合成后（F3、F5、F6）罩得住吗 |
|---|---|---|
| 被测代码（写路径）| 复用键 = 写表（带屏障）+ 起步镜像两样（`StepAttributes` `crash_identity.rs:92`、`reuse_key` 在 `:100` 附近）。被测代码改了写出来的字节，键就变。读路径、恢复路径的改动写表看不出，靠判法版本 | 罩住 |
| 判法代码 | 判法版本 = 从 `crash.rs`、`crash_judge_gpu.rs` 出发的模块引用闭包摘要（`JUDGE_ENTRY_FILES` `crash_identity.rs:839`；`judging_code_of_the_judges` `:1013`），另拼 `Cargo.lock`、各 crate 的 `Cargo.toml` 与工具链；词法摘要去注释、保留标识符与串（`:275`）。闭包停止表把 `layer0_progress.rs`、两个 `lib.rs` 拦在外面；`crash_judge_dispatch.rs`、`gpu_unit_checks.rs` 在「不许引」表里，**改结论的毛病修在派活里时要手工抬** `GPU_JUDGE_REVISION`（`crash_identity.rs:864`、`crash_judge_gpu.rs:156`） | 判法本体罩住；派活靠手抬，是已有的缺口，不是合并带来的 |
| 参数（展开）| 计划哈希：每段的展开名（`Layer0SegmentExpansion::name` `crash.rs:1845` 一带）、每点态数、`judge_inputs`（`crash_amplification.rs:429` 之前的 `plan_text`）。展开深度不同 = 计划不同 = 块键不同，不会串 | 罩住 |
| 判器另读的流程输入（版本表、被判根槽写、之前的写表）| 进计划哈希、不进复用键（`:233` 的文档；`JudgeInputsOfTheFlow` `:500`）。有用例钉：`two_flows_share_their_prefix.rs:817`、`:841`、`:881` | 罩住 |
| 工具链 | 判法版本里（`layer0_progress::Layer0ToolchainIdentity` `layer0_progress.rs:194`）| 罩住 |
| **逐状态观察者的代码**（F3 口子 ①）| 不罩。观察者住在测试文件里，`crash_identity.rs:836-838` 的文档写明设点的测试文件、流水线「都不在——它们变了，判定不变」。今天层 0 靠 `stage-inputs.tsv` 整个 `crates/` 进输入（`:48`、`:53` 行第二列 `crates/ Cargo.toml Cargo.lock`），所以观察者改了标记就失效 | **罩不住**：观察者一旦把结论随块存、随块复用，改观察者代码不改键 |
| **再挂载、清扫挂载的被测代码**（F3 口子 ②）| 不罩。`crash.rs` 与 `crash_judge_gpu.rs` 都不 `use` `core::mount` 与 `instance_table_sweep`（`crash.rs` 第 6–42 行的 `use` 里没有；grep `mount_writable\|instance_table_sweep` 在这两个文件里 0 处），闭包里没有它们 | **罩不住**：`mount.rs`、`instance_table_sweep.rs`、`admission.rs` 改了，旧的挂载结论照样命中复用 |
| 随机历史的种子与生成器（F5）| 崩溃注入不进库，今天靠 `stage-inputs.tsv:57` 整个 `crates/` 进输入；`crash_identity.rs:890` 的 `RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED` 是另一件事（防随机历史经库模块溜进判法闭包） | F5 说「同一个判定库与复用键约定」：要罩的是（种子基、种子数、每段步数、故障比重、`history.rs`/`fault_injection.rs`/`model*.rs`/`crash_injection.rs`），这些**都不在**判法闭包里，也不在复用键里，要新增一类键输入 |
| 判法走的路（GPU / CPU / 两者对拍）| 不罩。判定行只有红项串，不记谁判的；GPU 路与 CPU 路用同一个判法版本（`crash_judge_gpu.rs` 在闭包里，两条路都过）| **罩不住 F4 的新要求**：库里早先 GPU 单判的块，F4 下「CPU 也判」没做过，却命中复用 |

### 3.2 两份文件头「判法进不进键」按哪份落

代码和 `crash_amplification.rs:7` 一致：`StepAttributes` 只有两样，注释自己写「只有流程进复用键」（`crash_identity.rs:89`），判法在判定行上（`verdict_store.rs:47`）。`crash_identity.rs:6` 的「复用键罩三样属性」「判法代码」是改成判定行属性之前的旧话，同一个文件里就和 `:89` 打架。**落 `crash_amplification.rs:7` 那份，改 `crash_identity.rs` 文件头第 6–8 行。**还有第三处字面像第三种说法：`two_flows_share_their_prefix.rs:485` 的用例文档写「判法版本不进判定的键，B 就拿到旧版本的红」。读它的上下文，这是那条用例要抓的毛病的条件句（故意先放旧版本下判红的 B，键若不含判法版本就会拿到旧红），不是第三种口径；读码读的，没造出来对照。三处里 `verdict_store.rs:122` 是判决性的代码，其余是文档。

### 3.3 剪掉的是不是只有「键同」与「等价」两种

- 今天的剪枝只有一种真剪：键同且判法版本同的块不重判（`CoverageState` 的「复用」，`crash_identity.rs:115`）。
- **「与已判状态等价」今天没有实现**：`grep -n '等价\|dedup\|equivalen'` 在 `crash_amplification.rs`、`crash.rs`、`crash_identity.rs` 里只有一处无关的（`crash.rs:4620` 讲 `catch_unwind` 与 `should_panic` 等价）。状态去重在 `src/bin/e161_crash_state_dedup_and_time_split.rs`，是装置，不在引擎里。所以「等价关系写成用例钉住」（`.claude/singlefs-ai-sop/rules/test-script-discipline.md:61`）今天没有对象；F6 若写「与已判等价的才剪」，是新增，要同时新增那条用例。
- 另两种「没判」：ignore 的点（`CrashPointSpan.ignored` `crash_amplification.rs:194`，只留全持久一个态）与展开上限没盖住的段（`NotExpanded` 贡献 0 个态，`crash.rs:1869`）。前者是按登记整批不判，规则明说不算剪枝（`test-script-discipline.md:62`），要在成功那句里报个数；放量今天已报 `ignored_points`/`states_skipped_by_ignore`（`crash-amplification.sh` 文件头）。后者在整域（展开盖住每一段）里不出现，在快档里出现——快档的成功句要报「展开 6 没展开的段」，不是 F6 说的「每项小规模」一笔带过。

### 3.4 点名单跑一项与整域里跑判定相同，怎么证

- 今天放量一侧的证据：`two_flows_share_their_prefix.rs:489` 的 `reused_verdicts_equal_state_by_state_the_verdicts_judged_without_the_library_or_the_prefix_split`（关掉库与前缀共享各判一次，逐态相同；`#![cfg(feature = "verdict-store")]`，快档第二趟编进来）。层 0 一侧每条用例独立有自己的标记（`singlefs-crash-case-green.<用例名>.<指纹>`，`admission.py:271`），没有「在整域里跑」这回事。
- 合成后要证的是：某一项在「库里已有别的项的块」时与「空库」时的逐状态判定（红项串，若扩库则含计数）相同。`t9` 只证了两条流的小例；要对 13+8 项逐项做一份同形的对照，量太大，只能：快档里对每项最小展开跑「空库 vs 先跑别的项」，整域里不证。**计数断言**依赖库里存了计数（见 K1 第 1 点），否则点名一项时 `OracleCounts` 随库内容变，这条性质不成立。

### 3.5 剪枝自证放哪一档

快档：`test-script-discipline.md:63` 明写「剪枝自证进快档」，`t9` 已在那里（`#![cfg(feature = "verdict-store")]`，`lib.rs:11-12` 把整个 `crash_amplification` 模块挂在特性后面，所以只在门禁快档的第二趟 `feature-gated-tests` 编进来；`cargo test -p singlefs-checker-tier --lib --tests` 一趟编不进）。合成后若快档仍是这两趟，等于「剪枝自证」与「引擎本身」都只在第二趟——和今天一样，不是新缺口。

## 4 K4　GPU 判不了的处置

### 4.1 今天「判不了」怎么处置、F4 改成什么

- 今天：配置登记了 `crash_judge_gpu` 的类，建不出表、有状态判不了，都**整趟报错、不退回 CPU 判器**（`crash_amplification.rs:7135` 一带的 `match (routes.crash_judge, &cards)`；文档在 `judge_my_blocks_on_gpu_cards` 的 `# Errors`；用户 2026-09-29 的原话在 `research/scripts/crash-amplification.sh:28`「不接受默认降级」）。没登记才由 CPU 判（`crash_amplification.rs:7120`）。
- `crash_amplification.rs:1836` 那个函数文档写「判不了的状态交回 `None`（回落到 CPU）」只是函数返回值，调用处在上游已拒过，事实表第 6 节第 5 点已指出，我读码同意。
- F4：每个状态 CPU 判（对拍参照），建得出表的 GPU 也判并逐状态比，建不出表的项在登记表里写「GPU 不判」+ 理由。这等于把「判不了」从**运行时判定**（建表时碰到才报错）搬成**登记表里的静态声明**。

### 4.2 哪几项会落进「GPU 不判」，各缺 GPU 判器的哪一判

| 项 | 缺什么 | 依据与状态 |
|---|---|---|
| 随机历史两项（崩溃注入、读故障见证）| 判器是（理想模型对拍、真挂载、二次崩溃、挂载透明、见证判据 W），都要真跑实现；放量引擎和 GPU 判器里 `fault` 一词除 `default` 外 0 处（`grep -n 'fault\|Fault' … | grep -vi default` 在 `crash_amplification.rs`、`crash_judge_tables.rs` 无输出；`crash_judge_gpu.rs` 0 处）| 读码 + grep，确定 |
| 逐状态再挂载那一截（删除 53、抬 F 50、清扫分阶段 60）| 要 `mount_writable` 真跑 core；GPU 内核只有恢复 + 池级 checker 两类判，没有可写挂载那一路 | 读码，确定（GPU 内核文件里没有 mount 逻辑，`crash_judge_recovery.wgsl` 只做恢复）。**注意**：这些项的**前半**（恢复 + oracle + checker）仍可 GPU 判，被登记「不判」的是挂载那一截而不是整项，F4 写成「项」一级太粗 |
| io_min 8192（槽距）| GPU 表几何从基线槽 0 取（`crash_judge_tables.rs:426` `base_geometry` 读 `fixed_structure_slot_spacing`）；恢复内核的槽 1 偏移是参数 `cfg.spacing`（`crash_judge_recovery.wgsl:154`）| 读码推能判，**没有对拍用例**（对拍用例里 io_min 只命中 I-7.5 的 12288 改坏流）；核不动：要主 agent 在有卡的机器上跑，我不跑 GPU |
| 清扫的 12 槽小环池与抹头写 | 抹头写是带字节的物理块长写（`instance_table_sweep.rs:605`，`ForceUnitAccess`），不是 `Zeros`，不撞 `ZeroFillInTheEnumeratedWrites`（`crash_judge_tables.rs:209`）；会不会撞 `WriteNotAlignedToALocation`（短于 16 KiB 落点的单元头写）、`TooManyWritesOnOneLocation`（上限 64，`:47`）没核 | 读码；**没有对拍用例覆盖清扫**；核不动 |
| 到 E 的第二条流 | 无拒绝：已在放量上全量跑过（2026-09-30 记录 `:35`「逐状态比过全部 0 不一致」，两台五张卡） | 记录 |
| 删除流 | 建表接得住「没有文件的那几版」（`build_judge_tables_against_the_version_table`）；2026-10-08 在 GPU 上跑完（记录 `:1292`）| 记录 |
| 按位置寻址第 3 条 | 建表之前就因全域无 u64 表示而停（2.1），不属于 GPU 判不了，属 F7 | 探针 |

建表拒绝的全集（`JudgeTablesRefusal` `crash_judge_tables.rs:205`）：基线槽 0 解不开、枚举写表里有清零写（`:209`）、写不对齐落点、一个落点被多于 64 次写盖住（`:47`）、一个落点多于 4096 个版本（`:49`）、写不在任何一段。

### 4.3 登记「GPU 不判」之后「GPU 与 CPU 逐状态比」落空，算不算违背「用上现有的硬件加速」

- 条款：`.claude/singlefs-ai-sop/rules/test-script-discipline.md:79` 「登记了的那几类只走加速路径，配置了几台机器就分到几台。」，`:80` 「不悄悄退回 CPU 或单机」，`:81` 「加速路径留一份 CPU 参照实现，两边在同一组输入上判定逐项相同；这组对拍进快档。」，`:30`（入口脚本一节）「只有个别项要的，那几项照 `show-me-test.md`「门禁不许假装通过」逐项报「本次未跑」」。
- 我的判法（读规则字面，没有「一致」可对）：**不构成违背，条件有三**：① 登记的粒度是**判定类**，不是项——条款里的「类」是配置里的 `GPU_COMPUTATION_CLASSES`（`crash-amplification.sh` 文件头 run-condition），今天的类 `crash_judge_gpu` 被登记了，类里某些流建不出表是**输入的性质**（表模型接不了），登记写成静态的、带理由的「这类流 GPU 不判」，且不是运行时降级；② 成功那句逐项报（项名 + 理由 + 该项的态数 N、「CPU 单判」），和 F7 的「本次未跑」一样列名，不报「全 GPU」；③ 条款的第三条（CPU 参照 + 对拍进快档）对**这些项本身**落空，所以这几项的 GPU 判别力要由别处证：用一条**小展开**的对拍用例把它们的小流喂给 GPU 判器（那就等于去掉「不判」——即证明它们其实建得出表）；建不出的，才允许「不判」。
- 反面：登记表若写成「这一项 GPU 不判」而其实是**没人试过能不能建表**（io_min 8192、清扫就是这种：读码推能判、没对拍），那是把没验过登记成性质，违背。规则里「GPU 判不了」今天全部来自运行时报错，没有一项是预先登记的；**用户 2026-09-29 的「不接受默认降级」若按「F4 登记」读就被放宽，这一条要向用户确认，不是我能定的**（`crash-amplification.sh:28`）。
- 还有 F4 对放量的第二个新要求：**逐状态比 GPU 与 CPU 在放量引擎里今天没有**（0.2 第 4 条）。这会让 GPU 路的吞吐变成「GPU 判 + CPU 判」，K5 给数。

## 5 K5　提交时的代价（计数模型；量过的与推的分开）

模型脚本：`research/prompts/m3-crash-enumeration-merge-r1-sonnet-model/k5-model.py`（纯算术，`python3 -I` 起，输出下面原样贴）。输入分两类：**我量的**（今天的树，层 0 一侧探针，2.1 节）与**记录里的**（旧代码上别人量的，不是我量的）。

### 5.1 态数

| 流 | 态数 | 来源 |
|---|---|---|
| 第一条流 | 16 777 260 | 我量（探针） |
| 第二条流（到 E 再卸载） | 1 678 377 204 | 我量；与 `fixed_script_stream.rs:103` 一致 |
| 到回退 D | 67 502 198 | 我量 |
| 多记录 | 1 358 954 634 | 我量；与 `multi_record…rs:395` 的断言一致 |
| 删除 | 1 463 812 278 | 我量；与 `unlink…rs:1028` 一致 |
| 树分裂七条 | 78 905 428 | 代码里的 ignore 理由与记录相同（`records/…:35`） |
| 按位置寻址 5 条 | 553 664 602 | 记录（旧代码；前两条我量到 268 435 480、16 777 228，日志里按两台分份、没逐条相加核；第三条 GrowsToTwoLevels 日志现判 10 态） |
| 抬 F / σ / io_min | 8 223 / 65 548 / 33 | 记录 / 记录 / 代码（`stage-inputs.tsv:56` 注释） |
| 清扫两项、崩溃注入、读故障 | 没数 | 清扫用例没有闭式断言；随机历史是抽样（`stage-inputs.tsv:57` 注释写 24 段、每段 4 个崩溃状态，即 96 个崩溃态，推的） |

各流之和 = 5 218 067 408（含重复计的第一条流前缀）。若每个公共前缀都共享（第一条流前缀 16 777 259 在第二条流与多记录之间只判一次；删除流前 19 段与多记录相同，只多判删除的四个单元段 104 857 644 态），distinct = 3 758 056 057。**后者是推的**：2026-10-08 那趟删除流 `reused=0`，原因是两项之间判法版本变了（`records/2026-10-01-删除与并发事务.md:1292` 「判法版本 c7c1e096 → 93e1e4e3」），没有在同一判法版本下量过删除流复用多记录前缀。

### 5.2 速率（都是记录，不是我量的）

- GPU 端到端：2026-09-30 七项 `CRASH_AMPLIFICATION_TOTAL items=7 failed=0 states=2563727926 red=0 seconds=27288`（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35`）→ 93 951 态/秒；到 E 那一项 99 003 态/秒；2026-10-08 展开 30：多记录 339 792、删除 309 420 态/秒（`records/2026-10-01-删除与并发事务.md:1292`）。端到端含录制、写库、对账。
- GPU 判段（日志 `~/.local/share/singlefs/crash-amplification/full-run-20260930-i.log` 的 `gpu_states_per_second=`）：同一趟里 71 849（按位置寻址的分配记录树流）到 1 183 784（按位置寻址第一条流）态/秒，每张流差 16 倍，所以「一个速率」模型本身就粗。
- CPU（层 0 路）：`records/2026-09-24-里程碑二收尾调度.md:318` 「本机 23,700 个/秒、第二台 24,800 个/秒」（2026-09-28，第二条流，31 线程）；2026-09-27 第一条流两片 `shard_elapsed_milliseconds=338323,310008` 对 16 777 260 态 ≈ 49 600 态/秒（标记原文，事实表 1.2 节）。
- 全部命中的流不是零成本：日志第 156 行 `tree_split_CentralMappingRootSplit … states_checked_here=0 … unjudged_blocks=0 … seconds=486.1`（记录与日志都写录制单线程约 480–520 秒一条流，`records/2026-09-28-…:35`）。

### 5.3 改到 core 的那次提交（判法版本变、整域重判）

判法版本罩 `core::recovery`、`checker::walk` 等闭包（`crash_identity.rs:839` 起，3.1 节）；改到它们，所有块的判定行换新版本，全部重判。F4 下 GPU 与 CPU 都判，墙钟由慢的一端定。`python3 -I research/prompts/m3-crash-enumeration-merge-r1-sonnet-model/k5-model.py` 原样输出：

```
naive_sum_of_domains 5218067408
distinct_if_every_common_prefix_is_shared 3758056057 unlink_extra_over_multi 104857644
GPU 2026-09-30 whole run (2563727926 states / 27288 s)     rate=     93951/s  naive=  15.4 h  distinct=  11.1 h
GPU 2026-10-08 unlink item (1448083638 / 4680 s)           rate=    309420/s  naive=   4.7 h  distinct=   3.4 h
GPU 2026-10-08 multi item (1342177375 / 3950 s)            rate=    339792/s  naive=   4.3 h  distinct=   3.1 h
CPU two hosts (2 x 24.8k/s, records 2026-09-28)            rate=     49600/s  naive=  29.2 h  distinct=  21.0 h
CPU one host 24.8k/s                                       rate=     24800/s  naive=  58.4 h  distinct=  42.1 h
CPU one host 23.7k/s                                       rate=     23700/s  naive=  61.2 h  distinct=  44.0 h
unlink per-state-mount CPU lower bound: two hosts 8.4 h, one host 16.4 h
12 tree/position flows x ~500 s recording = 1.67 h
```

读法（推的）：今天的 GPU 单判整域在 3 到 15 小时（取决于用 10-08 的快速率还是 09-30 的整趟速率，二者差 3.3 倍，09-30 那趟慢的原因记录自己写「没拆开量过」）；F4 加上 CPU 逐状态判之后，墙钟被 CPU 端定住，**21 到 29 小时（两台，共享前缀与否）**，一台 42 到 61 小时。CPU 路在放量引擎里的每态开销（verdict 串、写库）没量，所以这几个数是 CPU 路的下界，不含挂载。
**每个状态再起一次可写挂载的项**（删除 14.6 亿、抬 F、清扫）：挂载的单态成本没有任何记录；按 CPU 速率（只判不挂）的下界，删除一项两台就要 8.4 小时。若这类项因为键罩不住挂载代码（3.1 节）而不能剪，它们每次提交都整条现判，这部分在「只改测试 / 脚本」的提交里也省不掉。

### 5.4 只改测试或脚本的提交，剪枝能省多少

- 判法版本不变（测试文件、脚本、门禁都不在闭包里）、写表不变 → 块全部命中。成本 = 排计划 + 录制 + 查库。录制的下限是记录里的数：约 12 条树分裂 / 按位置寻址流 × 约 500 秒 ≈ 1.7 小时（日志第 156 行为证，单线程，库越大越慢），再加到 E 与多记录、删除三条大流的录制（没有记录，**没量过**）。所以「全部命中」也是小时级，不是分钟级；提交时门禁等它 ≥ 2 小时的量级（推的）。
- 若观察者 / 再挂载 / 计数断言不入键、又要每次现跑（3.1、K1 第 1 点），省不下来的部分由那几项的态数 × 每态成本定，下界见上（删除一项两台 ≥ 8.4 小时）。
- 层 0 今天的做法对照：13 条案的全绿标记以整个 `crates/` 为输入（`stage-inputs.tsv:48`），改任何一个 crates 文件 13 条全失效；放量的判法版本窄得多（只罩判法闭包），所以放量一侧提交粒度更细；合成后「按条复用」要用的是放量的这一套。
- 没量过的：快档（展开 6、甲二、两趟 cargo）一次提交的挂钟；整域在一次空跑（零判）下的总挂钟。要主 agent 在有卡的机器上跑（第 8 节）。

## 6 K6　改名与条款的射程

做法照 `.claude/rules/path-moves.md:24`「改一个全仓术语：正文之外还有五处会红」与同文件 `:40`「英文标识符与中文术语可以分批改」。下面的文件数是我现跑的：`git ls-files -co --exclude-standard | grep -v '^target/' | xargs -d '\n' grep -lI -e '<模式>'`，按路径前缀归组（含 `research/prompts/`、`research/results/`，事实表没扫这两处）。

### 6.1 命中分布（文件数）

| 模式 | `.claude/` 门禁本体 | 门禁样本 | hooks | skills/rules/agents/main-agent/CLAUDE.md | kb | research/scripts | crates | research/e7-index-bench | research/prompts（冻结） | research/results |
|---|---|---|---|---|---|---|---|---|---|---|
| `checker-tier-layer0-replay` | 10 | 56 | 4 | 3+2+3+1+1 | 9 | 14 | 20 | — | 564 | — |
| `layer0` | 12 | 84 | 5 | 3+2+6+1+1 | 10 | 22 | 49 | 39 | 1574 | 172 |
| `层 0` | 5 | 4 | 3 | 1+1+2+1+1 | 12 | 18 | 68 | 21 | 1526 | — |
| `crash[-_]amplification` | 3 | — | 4 | 1+2+0+0+0 | 5 | 11 | 29 | 2 | 618 | 37 |
| `崩溃放量` | 1 | — | 2 | 1+2+0+1+1 | 8 | 9 | 33 | 13 | 593 | — |
| `singlefs-crash-case-green` | 1 | — | — | — | 1 | 2 | — | — | 179 | — |

（kb 一列是该模式下 `.claude/kb/` 下命中的文件数；「skills/rules/agents/main-agent/CLAUDE.md」一列按这个次序分项相加。）

### 6.2 分类（照 path-moves 的「怎么做」与「留存产物分两类」）

**A. 活的、跟着改（一批做完）**

| 落点 | 要改什么 | 会红的闸 / 要核的 |
|---|---|---|
| 门禁名 `checker-tier-layer0-replay` | 脚本文件名、文件头 `gate-stage`/`gate-cell`（`checker-tier-layer0-replay.sh:8-9`）、`stage-inputs.tsv`（stage 行与 `crash-case:` 行的键）、`stage-owners.tsv:5-6`、`doc-registries`、`selftest-registry.tsv:128`、`lib_heavy_tests.py` 的 `STAGE_KIND`（`:389`）与 `STAGE_SAMPLE_FAKE_TOOLS`（`:380`）、样本目录 `fixtures/checker-tier-layer0-replay.sh/`（11 份样本）与 `.layer0-sample-tools` 目录名（`selftest-registry.tsv:128` 的 `samples-light-marker:` 登记它）| 门禁结构检查（`gate-structure-check.py`）、`heavy-test-guard.sh --selftest`（69 处命中）、样本档 |
| `# gate-covers: 崩溃点重放` 键与 `.claude/gate-not-implemented.tsv:4` | **不用改**：这个键今天就叫「崩溃点重放」（`checker-tier-layer0-replay.sh:7`、`gate-not-implemented.tsv:4`）| — |
| 全绿标记前缀 | `admission.py:271` 的 `singlefs-crash-case-green.`。标记在 `.git` 下，旧前缀的标记改名之后全部不认（今天只有第一条流有一份，2026-09-27 的，事实表 1.2）；`admission.py` 里「层 0 / layer0」112 处命中里，标记前缀、`LAYER0_PARALLEL_FINISHED_PREFIX`（`:273`）、`crash-case-command` 输出的环境变量名是**跨文件协议**（用例打的行 ↔ 准入读的行 ↔ 门禁核的行），要同批改 | `admission.py --selftest`、`stage-inputs.tsv` 的 `count-line=` 登记 |
| 环境变量 | `SINGLEFS_LAYER0_*` 14 个名（`THREADS` 100 处、`SHARD` 52 处……）、`SINGLEFS_CRASH_AMPLIFICATION_*` 12 个名（`crates/`、脚本、`multi-host.env.example`、`gpu-cards.env.example`）；仓根的私有 `multi-host.env`（git 忽略，不在仓里）只命中注释与对端目录名 `singlefs-layer0-shard` | 对端那台机器上的目录路径（`PEER_REPOSITORY_DIRECTORY`）在仓外，改名要两台各改 |
| 入口脚本 | `crash-amplification.sh`/`-item.sh`/`-lib.sh`/`-two-hosts.sh`、`layer0-shard-run.sh`/`-selftest.sh`/`layer0-shard-configuration-check.sh`、`gpu-quiesce-run.sh`（13 处命中放量名）；`hook-swap.py`、`host-dispatch.py`、`host-resources.py`、`agent-watch.py` 里的名字 | 脚本执行位、`selftest-registry.tsv:43-45,80`、`lib_heavy_tests.py:585` `CRASH_AMPLIFICATION_SCRIPTS`（按文件名认重型，改名要同批改并保 `LIB_HEAVY_TESTS_BREAK=crash-amplification-unnamed` 那条弄坏开关仍红）|
| 库目录 | `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library`（`verification.md:36`）。改默认路径 = 新目录 = **空库 = 下一次整域冷跑**；要改就 `mv` 旧库，不要重建 | — |
| 测试与模块 | `crates/` 49 份含 `layer0`：模块 `layer0_progress`（`verification.md` 模块表与 `//! checker 档模块：` 首行声明，15 份测试声明它、11 份声明 `crash_amplification`，`crash-case-check.py` 的 modules 一样判）、类型 `Layer0*`、计数行前缀 `LAYER0 …`（`stage-inputs.tsv` 的 `count-line=` 登记）、`crates/mutations.tsv`（249 处命中，其中 41 行点名层 0 全量用例函数、2 行点名放量用例；锚点腐化就红）、`e165-pinned-red-examples.tsv`（50 行含「层 0」）| 门禁 59 号（变异表锚点）、`mutation-tables` 格、E165 钉表复放 |
| 条款与规则 | `verification.md`（第 34-36 行三档表、第 100-104 行「门禁管哪一半」）、`implementation-workflow.md`、`CLAUDE.md`、`main-agent.md`、`agent-common.md`、`implementation-writer.md`/`three-way-attack.md`/`experiment-*`/`investigator.md`/`tooling-writer.md`、`skills/crash-test/commit-stages.md`（26+7 处）、`skills/gate/staged-gate.md`、D13 已定项 9/15、`kb/milestone/*`、`invariants.md`（62 处「层 0」）、`tooling.md`、`verification-build.md`、`checks-owed.md`（76+50+27+30，欠账行）、`README.md`、`briefs/`、`research/perf-by-milestone.md` | 门禁 doc-text term-renames、doc-lint 的编号与简称（简称 ≤ 48 宽，`path-moves.md:31`）、doc-decisions（`--write` 重生成分项清单，`:33`）、doc-experiments decision-links（`:34`）|
| 登记 | `.claude/kb/term-renames.md` 加一节与表格行（表在 `:22` 起，一行「旧 \| 新 \| 匹配 \| 是什么」）；变更史里写一条索引性的改名条目（`path-moves.md:38`）| doc-text term-renames |

**B. 冻结、原样留旧名（不改）**

- `research/prompts/` 整个目录（`term-rename-exempt:11`「冻结证据」）：1500+ 份文件含旧名，不动。
- `research/results/` 里已归档、今天跑不出来的产物（172 份含 `layer0`）：不改；`path-moves.md:36`「这份产物今天跑得出来吗」为判据。产物的**引用句**随之留旧名。
- `records/` 与 `*-history.md`：只改「现状句」，记「那一次发生的事」的句子留旧名（`path-moves.md:38`），逐个 `term-rename-exempt` 登记。
- `research/e7-index-bench/`（39 份含 `layer0`、13 份含 `崩溃放量`）：装置源码在树里、`replay.sh` 判 `exact` 的跟着源码换名并重生成产物；要虚机或已归档的不动。哪些属于哪类我没逐份核（核不动：要跑 `replay.sh`，是 checker 档 / 实验复跑）。

**C. 英文标识符会改判法版本，别混进别的批**

`crash.rs` 里 `Layer0*`、`layer0_*` 共 602 处命中，`layer0_progress.rs` 321 处。判法版本的词法摘要保留标识符与串（`crash_identity.rs:275`），而 `crash.rs` 是闭包入口（`:840`）：**任何一个标识符改名都换判法版本 → 整域冷重判**（K5 的 21 到 29 小时）。字符串字面量也算（例：`stage-inputs.tsv` 的计数行前缀 `LAYER0` 在 `crash.rs` 里是 `println!` 的串）。文档注释不算。所以：中文术语、门禁名、脚本名、条款、环境变量名可以独立一批，不动判法版本；`Layer0*`、`layer0_progress` 的改名放进「本来就要换判法版本」的那一批。还有 `src/bin/e158/e161/e170/e171*` 这些装置二进制引用 `Layer0*`（事实表 5.2：e170 16 处、e171 两份 9+26 处），E171 要冻结的树里它们必须一起编得过。

### 6.3 D13 已定项 9 的层表（层 0 / 层 1 / 层 2）

- 表在 `.claude/kb/decisions/13-验证路线.md:242-246`：「层 0 冒烟」「层 1 常规」「层 2 全量」。三层说**负载规模**，「触发」列只有层 0 写了触发（`checker 档的全量（已定项 15）：用户要求时跑；提交时默认只跑快档`），层 1 写「每次提交」、层 2「用户要求时 / 发布前」。`:259` 射程句自己说「层 1、层 2 在门禁里没有阶段」，`:270` 「**欠**：层 1、层 2 在门禁里没有阶段，没有 C 编号」。
- F6 的「快档 / 整域」是**同一种崩溃点重放按展开深度分**的两档，负载规模那条轴（写请求数十 / 千 / 更大）与展开深度正交；两条轴都叫「层」或「档」就会打架。F1 只说「怎么处置是这一形态要回答的一处」，没答。
- 我的推论（只能写这个，因为判决是主 agent 的）：表里真被门禁实现的只有「层 0」一行，其余两行是未实现的愿景。合成之后若保留负载规模表，**「层 0」这个名字必须改**（它是 `term-renames` 里要退役的装置名，同时也是这张表的第一行名，同一个串两个意思，`sweep-term` 无法区分）；最小改法是把表改叫按负载规模的「冒烟 / 常规 / 全量规模」三档，且明写与 F6 的展开深度两档正交；或把层 1、层 2 两行并进「欠」里删掉表。任一都要同批改 `:244-246`、`:253`、`:259`、`:270`、D13 变更史索引与 `verification.md:35` 的「分层照 D13 已定项 9」。
- 「全量指两样」（`:253` 「全量」指两样：checker-tier-layer0-replay `--full`）改成一样，同行紧跟的「动了崩溃点重放或判器」路径判法（`:254`）、「上一次全量」记法（`:255`）与 `:251`「全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快速档全绿（必要条件）」要整句改成「提交跑整域」。`:259` 里 12230590578 是旧数（今天 1358954634），`:395`（已定项 15 定案）「崩溃放量判器整套上了 GPU」「全量是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`」整句同改。

## 7 K7　次序与切批

### 7.1 约束（都是现查的）

1. **别的会话的快照**：`research/prompts/m3-batch52-54-code-r1-snapshot/crates-sha256.txt` 共 24 份，与合并要碰的文件相交的有 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs`（`:7`）、`crash_judge_dispatch.rs`、两个 `crash_judge_checker*.wgsl`、测试 `crash_enumeration_new_pool_file_creation_stream.rs`、`crash_enumeration_sharded_across_processes.rs`、`crash_enumeration_tree_split_streams.rs`、`crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`（`:12`）、`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，还有 core 的 `recovery.rs`、`mount.rs`、`transaction.rs`、`inode_tree.rs`。派发闸拒绝派 `kb-scribe`/`implementation-writer` 改「还没写判决的三方轮的开工快照里」的文件（`agent-common.md` 派发闸一条）。**这一轮自己的快照（96 份 crates）也算**：这一轮判决写出之前，这些文件同样动不了。
2. **另一批在飞**：`.claude/batch-scope`（现有 7 行：`CLAUDE.md`、`crash_amplification.rs`（`:15`）、`crash_facts.rs`、`crash_judge_common.wgsl`、`crash_judge_dispatch.rs`、`crash_judge_tables.rs`、`research/scripts/e152-tables.py`）是「判器组合掩码放到 64 位、崩溃放量全量重判与 E152 出表加 singlefs 行」那一批，提交前要清空。其中 `crash_judge_tables.rs`、`crash_judge_common.wgsl` 在判法闭包里（`crash_judge_gpu.rs:23` 引 `crash_judge_tables`），所以那一批提交就换一次判法版本。引擎口子要动的 `crash_amplification.rs` 与它撞。
3. **判法版本**：闭包 = `crash.rs` + `crash_judge_gpu.rs` 出发（含 `crash_judge_tables.rs`、各 wgsl、core 的 `recovery` 等）；不在闭包：`crash_amplification.rs`、`crash_facts.rs`、`crash_identity.rs`、`verdict_store.rs`、`crash_judge_dispatch.rs`（`crash_identity.rs:861` 起的「不许引」表）、所有测试文件。改闭包里的文件（含标识符改名）就换一次判法版本 = 一次全量重判（K5）。
4. **E171 冻结**排在这一题之后（派发材料原话）；E171 装置二进制引用 `Layer0*`（6.2 C 节）。冻结的那棵树必须是一个自洽的状态：不能门禁已改名、`stage-inputs.tsv` 还登着旧用例，也不能新登记表登着旧代码里没有的项。

### 7.2 切批（按文件不相交；编号是推荐的先后，不是并列）

| 批 | 动什么 | 与哪些约束相撞 / 先决 |
|---|---|---|
| S0 | 等待：① 这一轮的判决写出；② `batch-scope` 那一批（掩码 64 位）提交；③ batch52-54 的判决写出 | 都不在我手里。S0 之前不要派写 `crates/` 的活 |
| S1 引擎口子（只加不换） | `crash_amplification.rs`（`CrashFlow` 加逐状态回调字段，利用 `crash.rs:1452` 已有的 `judge_state` 回调，**不动 `crash.rs`**）、`tests/common_crash_points/mod.rs`（`amplify_*` 签名）、`verdict_store.rs`（若选「库里每块存分类计数」）、`crash_identity.rs`（给带观察者 / 再挂载的项另算一个键输入：测试文件闭包 + `core::mount`、`instance_table_sweep`，算法用 `crash_identity.rs:1033` 的 `judging_code_by_module_references`，从测试文件出发收闭包；`crash_identity.rs` 的单元测试已对登记用例的测试目标这样算过）。只加不改既有项的行为 | 撞 `batch-scope`（S0 ②）。`crash_amplification.rs`、`verdict_store.rs`、`crash_identity.rs` 都不在判法闭包 → **不换判法版本**，这是它排在最前的理由 |
| S2 逐流并入 | 每流一个小批，≤ 8 文件，每批只动该流的测试文件与它的放量函数：先做**不在 batch52-54 快照里**的（多记录、抬 F、按位置寻址、σ、第二条流 + 到回退 D、io_min、清扫、崩溃注入 / 读故障）；**后做**在快照里的（第一条流、树分裂、删除、对拍用例）。每批的验收：该流的态数、`states_by_publish`、观察者计数在新入口里与旧用例逐项相同（旧用例这一批里仍留着，下一批才删）| 依赖 S1。删除 / 第一条流 / 树分裂排在 S0 ③ 之后。每批要保证旧用例与新入口并存时登记表不重复判同一条流 |
| S3 登记、入口、门禁（原子一批）| 新入口脚本（`--list-items`/`--item`/不带参数全量）、`stage-inputs.tsv` 把 13 条 `crash-case:` 换成新登记、门禁改名（含 56 + 84 份样本、`.layer0-sample-tools`、`lib_heavy_tests.py` 的 `STAGE_KIND`/`CRASH_AMPLIFICATION_SCRIPTS`、`selftest-registry.tsv`、`heavy-test-guard.sh` 自检 69 处）、`admission.py` 标记前缀与读法、双机驱动合并（`layer0-shard-run.sh` ↔ `crash-amplification-two-hosts.sh`）、删旧用例 | 依赖 S2 全部。必须一批做完：`stage-inputs.tsv` 一边登 13 条旧案、一边门禁已按新项判，会让整道门禁红或悄悄不判。这是 `tooling-writer` 的活，样本与自检要同批改（`path-moves.md` 第 7 步）|
| S4 条款与旧数 | D13 已定项 9、15、`verification.md`、`implementation-workflow.md`、`main-agent.md`、`commit-stages.md`、`term-renames.md` 一节、F10 的 8 处旧数、`fixed_script_stream.rs:865` 的「五十多亿」、`crash_identity.rs:6-8` 的文件头口径 | 依赖 S3：条款写的是已经成立的做法，不先于做法。D13 是已定项，改它按推论纪律再走一轮；这是用户已定的 F9，仍要 kb-scribe 经推论门禁 |
| S5 英文标识符与模块名 | `Layer0*`、`layer0_progress`、`crash_amplification` 模块名、`SINGLEFS_LAYER0_*`/`SINGLEFS_CRASH_AMPLIFICATION_*`、`crates/mutations.tsv` 锚点（249 处）、`e165-pinned-red-examples.tsv`、各测试首行模块声明（26 份）、`src/bin/e158…e171`（含 E171 装置）| 换判法版本。可选：放最后，与最后一批本来就要换判法版本的改动同批，或整个推迟到 E171 冻结之后（`path-moves.md:40` 允许中英文分批）|

### 7.3 为什么这样排

- **只换一次判法版本**：S0 里两批已经要换（掩码批、batch52-54 的 core 与 `crash_judge_*`）；S1–S4 都不进闭包；S5 若做，放在这些之后一次换。整域的第一次冷跑（K5 的 21 到 29 小时量级）放在最后一个闭包改动之后，之前不要起。若在 S5 之前就先跑过整域，S5 的改名会把它整个作废（推的，来自 `crash_identity.rs:275` 的词法摘要）。
- **E171 冻结的树**：到 S4 完成时（S5 可有可无）是自洽的：旧用例已删、登记表已换、条款已写。S2 进行到一半、S3 没做时冻结，树里会同时有新旧两套登记，就是题面担心的「一半合了的装置」。所以冻结点只能落在 S0 之前（全部还是旧的）或 S4 之后（全部是新的），不能落在 S1 到 S3 之间。S1 只加不换，可以排在冻结前，也不影响冻结后的状态。
- **S2 不用等 S3 才开工**：每流并入时旧用例照留，门禁按旧登记继续判，两套并存；S3 才是切换点。S2 的每批在并存期间要有一条用例核「旧、新两边对这条流逐项相同」，否则切换点才发现不一致。

## 8 要主 agent 跑的命令（我按更正没有跑放量一侧）

目的：把放量一侧的同名探针行补出来，与 `probe-l0.log` 逐行对：`table_digest`、`base_digest`、`states6/28/30/full`。判法：同标签两边摘要不同 = 两条造流代码不是同一条流（2.4 的反例）；`AMP unlink` 的 `states30` 不是 1463812278 = 2.5 的推断被推翻。

- 工作目录：一个新建的仓副本（只拷 `Cargo.toml`、`Cargo.lock`、`.cargo`、`crates`），再 `python3 -I research/prompts/m3-crash-enumeration-merge-r1-sonnet-model/apply_probes.py <仓副本根>`。
- 环境前缀：`SINGLEFS_HEAVY_TESTS=user-request CARGO_TARGET_DIR=<编译目录> ROCKSDB_LIB_DIR=/home/fy5090/code/singlefs/target/release/build/librocksdb-sys-f7ce5abb7da4f505/out ROCKSDB_STATIC=1`（后两个省掉现编 RocksDB；该目录只读引用，没有写进去）。
- 编（我已编过一遍，1 分 19 秒 + 20 秒）：`bash research/scripts/capped.sh 7 cargo test --offline --release -p singlefs-checker-tier --features verdict-store --test crash_enumeration_unlink_after_the_multi_record_publish_stream --test crash_enumeration_multi_record_publish_stream --test crash_amplification_fixed_script_through_unmount --test crash_amplification_fixed_script_through_rollback --test crash_enumeration_tree_split_streams --test crash_enumeration_position_addressed_trees --test record_checker_judges_absence_by_the_persisted_set --test crash_enumeration_floor_raise_pushed_by_admission --no-run`。
- 跑（每条经 `bash research/scripts/run-with-memory-cap.sh 6G bash research/scripts/capped.sh 7 <测试二进制> <用例函数> --exact --nocapture`，二进制路径是编译输出里 `Executable` 那一行）：放量函数默认展开 6，是快档用例，会真判几十到几百个态：`crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean`、`crash_amplification_of_the_multi_record_publish_stream_is_clean`、`the_whole_fixed_script_through_the_unmount_is_recorded_checked_in_parallel_and_reused`、`the_fixed_script_through_the_rollback_is_recorded_checked_in_parallel_and_reused`（用例名以 `--list` 为准）。
- 看哪几行：`grep ZZPROBE <输出>`，取 `label=AMP …`。放量的 through_unmount / through_rollback 两个文件里我写的标签是 `AMP through-unmount`、`AMP through-rollback`，对 L0 的 `second-stream`、`through-rollback`；其余放量项标签 `AMP <flow 名>`（`multi-record-publish`、`unlink-after-multi-record-publish`、`sigma-of-the-misaligned-reuse` 等）。
- 另两样我量不了：① 有卡的机器上对 io_min 8192、清扫流跑 `build_judge_tables`（看会不会撞 `WriteNotAlignedToALocation`），② 一次「全部命中」的整域空跑的挂钟与一个状态再挂载的成本（5.4）。

## 9 判定一览

| 格 | 判定 | 一句话 | 什么现象会推翻它 |
|---|---|---|---|
| K1-a 计数断言 | 冲突 | F3 要的 `OracleCounts`、`states_by_publish` 等绿状态计数，放量库只存红项串（`crash_amplification.rs:1162`），剪枝命中的块不再给观察者看 | F3 实现里把分类计数随块存进库（`verdict_store.rs` 表加列），或在 F6 里写明带计数断言的项只冷跑 |
| K1-b 观察者、再挂载 | 一致（口子能补），键见 K3 | CPU 路回调已交出镜像与判定（`crash.rs:1452`），不用动 `crash.rs`；GPU 路交不出 | 造出一个需要 `crash.rs` 新字段才能写的观察者（读码推的，没造） |
| K1-c GPU 对 CPU 逐状态比 | 规则没说（F4 新增）| 放量今天 GPU 与 CPU 二选一（`crash_amplification.rs:7120`），对拍只在专用用例 | 找到放量里已有逐状态 GPU 对 CPU 全判法比较的代码 |
| K1-d σ | 一致 | 取放量的域与判法，判法是层 0 σ 的超集；快档下 σ 几乎不展开 | 层 0 σ 的 `judge` 里有放量没有的判据（`record_checker…rs:461-473` 只有恢复 + 记录核对器，已读） |
| K1-e 快档的域 | 冲突 | F6 快档「展开 6」≠ 层 0 快档甲二：第二条流 261 对 278，多记录 141 对 144，删除 189 对 196（探针）| 甲二断言的数（`QUICK_TIER_STATES…` 等）不需要改 |
| K2-a 域并集 | 一致（同函数同写表），到回退 D 要单列，按位置寻址第三条全域无 u64 表示 | 同一个 `layer0_plan_state_count`；`crash.rs:2011` panic 为证 | 放量一侧探针显示同标签 `states_full` 与 L0 不同 |
| K2-b 第二条流同一条流 | 核不动（放量一侧摘要没量）| 造流代码是复制的、字节相同没断言；层 0 一侧摘要已量 | `AMP through-unmount` 的 `table_digest` 或 `base_digest` 与 `L0 second-stream` 不同 |
| K2-c 删除流 15728640 | 一致（推）| 与另两处差同为 2^24 − 2^20，第二条流有代码注释交代原因 | `AMP unlink` 的 `states30` ≠ 1463812278 |
| K3-a 键罩 | 冲突 | 观察者、`core::mount`/`instance_table_sweep`、随机历史种子与生成器、判法所走的路都不在键里 | 查到判法闭包或复用键里已含这些文件（`crash_identity.rs:839`、`:89` 逐字核）|
| K3-b 文件头口径 | 一致 | 落 `crash_amplification.rs:7`，改 `crash_identity.rs:6-8` | 找到把判法摘要拼进 `StepAttributes::reuse_key` 的代码（我读到只有两样）|
| K3-c 剪掉的只有键同与等价 | 冲突 | 「等价」今天没实现；ignore 与 `NotExpanded` 不是剪枝是整批不判 | 找到按状态等价去重的引擎代码（我只在 `src/bin/e161…` 找到）|
| K3-d 点名单跑 = 整域里跑 | 规则没说（只有小例 `two_flows…rs:489`）| 计数断言存在时这条性质不成立 | 13+8 项逐项的空库 vs 暖库对照用例全绿 |
| K3-e 剪枝自证放哪档 | 一致：快档 | `test-script-discipline.md:63`；用例已在第二趟 | — |
| K4-a 哪几项 GPU 不判 | 一致 + 核不动 | 随机历史两项、逐状态再挂载那一截确定；io_min 8192、清扫读码推能判、无对拍 | 在有卡的机器上 io_min / 清扫流 `build_judge_tables` 返回 `JudgeTablesRefusal` |
| K4-b 登记 GPU 不判是否违背 | 有条件不违背 | 登记须是静态、带理由、成功句逐项报；而「不接受默认降级」（`crash-amplification.sh:28`）是用户原话，F4 登记是否放宽要问用户 | 用户确认「GPU 不判」登记就是他要的 |
| K5 提交代价 | 推 + 记录 | 改 core 整域：GPU 单判 3 到 15 小时，F4 加 CPU 21 到 29 小时（两台）；只改测试脚本也有 ≥ 1.7 小时的录制下限（没量）| 同判法版本下实跑空跑 / 整跑的挂钟 |
| K6 改名射程 | 一致 | A/B/C 三类；标识符改名会换判法版本；`gate-covers` 键不用改 | 改名一个标识符后 `PipelineJudgingCode::digest` 不变（读码推，没造）|
| K7 切批 | 推 | S0 等三件；S1 不进判法闭包最先；S3 原子；冻结点只能在 S0 前或 S4 后 | 查到 `batch-scope` 已清空且 batch52-54 判决已写 |

## 10 没做什么

- 不判「要不要合」「提交跑不跑整域」「CPU 对拍跑不跑全部状态」；不替主 agent 采纳或出判决。
- 没跑任何门禁阶段、全量、GPU、崩溃放量入口；放量一侧的探针没跑（更正后），5.4 的空跑挂钟与单态挂载成本没量。
- 没核：io_min 8192 与清扫流能不能建 GPU 判表；放量与层 0 的第二条流字节是否相同；13+8 项逐项的点名单跑对照；`research/e7-index-bench` 里哪些是 `exact` 产物；`research/results` 里 172 份含 `layer0` 的产物哪些今天跑得出来。
- 5.1 的放量态数（按位置寻址、树分裂、抬 F、σ）与全部速率是记录里的数，不是我量的；两处旧代码上的数（2026-09-30）与今天树上的闭式差 2^24 − 2^20，已在 2.2 节分开写。
- 事实表第 6 节以外我新看到的旧数：`fixed_script_stream.rs:865` 的「全量五十多亿个状态」（今天 1 678 377 204）。
- 仓副本与编译目录：交回前删（见交回消息）。

## 11 引文核对表（逐条 `grep -nF` 式现取行号，`cite-check.py` 逐条核）

- `crates/singlefs-checker-tier/src/crash_identity.rs:6`：「复用键罩三样属性」
- `crates/singlefs-checker-tier/src/crash_identity.rs:89`：「只有流程进复用键」
- `crates/singlefs-checker-tier/src/crash_amplification.rs:7`：「判法不进键」
- `crates/singlefs-checker-tier/src/verdict_store.rs:47`：「判定块与违例按（块键，判法版本）存」
- `crates/singlefs-checker-tier/src/verdict_store.rs:122`：「fn to_block_key_bytes_with_version」
- `crates/singlefs-checker-tier/src/crash_identity.rs:839`：「pub const JUDGE_ENTRY_FILES」
- `crates/singlefs-checker-tier/src/crash_identity.rs:275`：「去空白、行注释、块注释（可嵌套）与全部文档注释」
- `crates/singlefs-checker-tier/src/crash_identity.rs:864`：「改结论的毛病修在这里时抬」
- `crates/singlefs-checker-tier/src/lib.rs:12`：「pub mod crash_amplification;」
- `crates/singlefs-checker-tier/src/crash.rs:1869`：「Self::NotExpanded => 0,」
- `crates/singlefs-checker-tier/src/crash.rs:1841`：「InPlaceSubsetsWithCopyOnWriteNoneOrAll,」
- `crates/singlefs-checker-tier/src/crash_amplification.rs:172`：「if segment.len() <= self.0 {」
- `crates/singlefs-checker-tier/src/crash_amplification.rs:1162`：「EncodedVerdictVector(items.join」
- `crates/singlefs-checker-tier/src/crash_amplification.rs:7120`：「没登记由 CPU 判器判」
- `crates/singlefs-checker-tier/src/crash_judge_tables.rs:209`：「ZeroFillInTheEnumeratedWrites { write_index: usize },」
- `crates/singlefs-checker-tier/src/crash_judge_tables.rs:47`：「pub const MAXIMUM_WRITES_PER_LOCATION: usize = 64;」
- `crates/singlefs-checker-tier/src/crash.rs:1556`：「pub fn layer0_plan_state_count(」
- `crates/singlefs-checker-tier/src/crash.rs:1261`：「pub struct Layer0StateJudgement {」
- `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:96`：「回退那次发布追加一条谱系记录之后」
- `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:97`：「D 的单元写 20 → 24」
- `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:865`：「全量五十多亿个状态」
- `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:1028`：「assert_eq!(closed_form, 1_463_812_278);」
- `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:1115`：「assert_the_shape_of_the_deletion_steps(&deletion_steps);」
- `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:296`：「let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;」
- `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:281`：「let expansion = move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);」
- `crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs:483`：「for stream in PositionAddressedStream::ENUMERABLE_IN_FULL {」
- `crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:824`：「enumerated.states, 65_536,」
- `crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1235`：「多判 oracle 与 checker」
- `crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs:489`：「fn reused_verdicts_equal_state_by_state_the_verdicts_judged_without_the_library_or_the_prefix_split(」
- `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:166`：「const DEFAULT_EXPAND_UP_TO: usize = 8;」
- `crates/singlefs-core/src/instance_table_sweep.rs:605`：「&vec![0u8; physical_block_bytes],」
- `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:154`：「let offset = select(0u, cfg.spacing, slot == 1u);」
- `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:45`：「const FAST_TIER_SEEDS: u64 = 24;」
- `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:54`：「const SMALL_FAST_TIER_SEEDS: u64 = 4;」
- `.claude/gate.d/stage-inputs.tsv:48`：「crash-case:layer0-first-stream crates/ Cargo.toml Cargo.lock」
- `.claude/gate.d/stage-inputs.tsv:53`：「crash-case:layer0-unlink-after-the-multi-record-publish-stream crates/ Cargo.toml Cargo.lock」
- `.claude/gate.d/stage-inputs.tsv:51`：「σ 那一段 2^18 个状态」
- `.claude/gate.d/stage-inputs.tsv:52`：「枚举域 12230590578 个状态」
- `.claude/gate.d/stage-inputs.tsv:53`：「枚举域 1448083638 个状态」
- `.claude/gate.d/checker-tier-layer0-replay.sh:8`：「# gate-cell: layer0-replay」
- `.claude/gate.d/checker-tier-layer0-replay.sh:9`：「# gate-cell: feature-gated-tests」
- `.claude/gate.d/checker-tier-layer0-replay.sh:13`：「全量，只有 layer0-replay 一格」
- `research/scripts/admission.py:271`：「CRASH_CASE_MARKER_PREFIX = "singlefs-crash-case-green."」
- `research/scripts/crash-amplification.sh:219`：「FULL_EXPAND_UP_TO=」
- `research/scripts/crash-amplification.sh:24`：「崩溃注入（crash_injection_campaign）不在表里」
- `.claude/hooks/lib_heavy_tests.py:389`：「STAGE_KIND = {"checker-tier-layer0-replay.sh": "layer0-stage"」
- `.claude/hooks/lib_heavy_tests.py:585`：「CRASH_AMPLIFICATION_SCRIPTS = frozenset(」
- `.claude/hooks/selftest-registry.tsv:129`：「.claude/gate.d/checker-tier-layer0-replay.sh samples-light-marker:.layer0-sample-tools」
- `.claude/gate-not-implemented.tsv:4`：「崩溃点重放」
- `records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:35`：「CRASH_AMPLIFICATION_TOTAL items=7 failed=0 states=2563727926 red=0 seconds=27288」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:81`：「加速路径留一份 CPU 参照实现，两边在同一组输入上判定逐项相同；这组对拍进快档。」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:80`：「不悄悄退回 CPU 或单机」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:61`：「剪掉的只能是两种：键相同、已经判过的；与已判状态等价的。等价关系写成用例钉住。」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:63`：「剪枝要自证：在小规模输入上关掉剪枝与复用跑一遍，判定与开着时逐项相同。剪枝自证进快档。」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:62`：「按登记整批不判的一类不算剪枝」
- `.claude/kb/decisions/13-验证路线.md:251`：「**全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快速档全绿（必要条件）。**」
- `.claude/kb/decisions/13-验证路线.md:395`：「崩溃放量判器整套上了 GPU」
- `.claude/rules/path-moves.md:24`：「## 改一个全仓术语：正文之外还有五处会红」
- `.claude/rules/path-moves.md:40`：「英文标识符与中文术语可以分批改」
- `.claude/term-rename-exempt:11`：「research/prompts/  # 冻结证据」
- `.claude/batch-scope:15`：「crates/singlefs-checker-tier/src/crash_amplification.rs」
- `research/prompts/m3-batch52-54-code-r1-snapshot/crates-sha256.txt:7`：「crates/singlefs-checker-tier/src/crash_judge_gpu.rs」
- `research/prompts/m3-batch52-54-code-r1-snapshot/crates-sha256.txt:12`：「crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs」
- `research/scripts/crash-amplification.sh:28`：「不接受默认降级」
- `.claude/singlefs-ai-sop/rules/test-script-discipline.md:79`：「登记了的那几类只走加速路径，配置了几台机器就分到几台。」
- `records/2026-09-24-里程碑二收尾调度.md:318`：「本机 23,700 个/秒、第二台 24,800 个/秒」
- `.claude/gate.d/stage-inputs.tsv:49`：「crash-case:layer0-second-stream」
- `crates/singlefs-checker-tier/src/crash.rs:2740`：「pub fn enumerate_layer0_selecting_versions_observing_each_state(」
- `crates/singlefs-checker-tier/src/crash_amplification.rs:1266`：「fn judge_one_block(」
- `crates/singlefs-checker-tier/src/crash_amplification.rs:1836`：「pub fn gpu_verdict_items(」
- `crates/singlefs-checker/src/image.rs:111`：「[&str; 92]」
- `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:156`：「GPU_JUDGE_REVISION: u32 = 1;」
- `crates/singlefs-checker-tier/src/crash_identity.rs:92`：「pub struct StepAttributes」
- `crates/singlefs-checker-tier/src/crash_identity.rs:1013`：「pub fn judging_code_of_the_judges(」
- `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:736`：「fn observe_every_state(」
- `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:545`：「fn observe_the_mount_continuation(」
- `crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs:511`：「fn every_crash_state_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row」
- `crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs:210`：「enumerate_layer0_selecting_versions_observing_each_state(」
- `crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs:171`：「fn crash_states_inside_the_floor_raise_pushed_by_the_session_recover_and_remount_with_the_checker_green」
- `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:427`：「fn crash_amplification_of_the_multi_record_publish_stream_is_clean」
- `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:639`：「fn layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations」
- `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:731`：「fn layer0_quick_tier_matches_the_full_tally_shape」
