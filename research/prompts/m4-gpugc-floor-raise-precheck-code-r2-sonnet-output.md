# m4-gpugc-floor-raise-precheck-code-r2 云端辩方（Sonnet）报告（2026-11-10）

轮名 m4-gpugc-floor-raise-precheck-code-r2；被复核的判决 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-main-verification.md` 第二节「槽对那一判吃掉被宽护卡住的出路（T3-4）」一行与第三节第 1 条。冻结副本 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2/repo/`，被判文件 `crates/singlefs-core/src/mounted_session.rs` 的 sha256 是 `5104b9b5e4089e269853a6acde3bc28cff028de72c2544f3a69a742e4593d4dc`，与此刻工作区那一份相同（我现算的）。
引文的行号是仓根相对路径下的行号，冻结副本与工作区这几份文件同 sha256，两处行号一样。

## 零、模型与复跑

模型目录 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-sonnet/model/`（下称模型目录），一个探针补丁加复跑脚本，**我没编译、没跑**：补丁和探针只做过 `bash -n`、`python3 -m py_compile`、`rustfmt --check` 的语法解析，以及在冻结副本三个文件的拷贝上试打补丁（三处旧串各命中一次、补丁打得上）。所以下面凡是「探针会量出什么」都写「推的，待主 agent 跑」；编译不过的话新派同立场腿接着改，不当判决。

| 文件 | sha256 |
|---|---|
| `tools/count_flips.py` | `60cd99a824adc47e8ea4abfe77bdf049234425dfe1a3583f21df3e7105ca582c` |
| `tools/patch_the_copy.py` | `c69e2e40086433845890fd6f89fe1e2f9be2aae35b1702c9a7013b053825e531` |
| `tools/rerun.sh` | `2685c7900fac5f4e4ce1064fcc38bf2f159044c69740ae4c5d04c37a02153cec` |
| `tools/side_by_side.py` | `046770e0f9341b20573da70c373addc5e33af8e224b87ee77b0ceb6c4a2286b5` |
| `probe/probe_scenario.rs` | `a9b2d4ac6f5742ceb9d97193a771b2a85a6e5b860039189e76dde14a331b385e` |
| `probe/probe_tests.rs` | `deb124d2a391185091707651e2007eff9c6283a43c17fa0d5591b7dd2aa25864` |

探针做什么（一份编译、新旧两种口径靠环境变量切）：在冻结副本的拷贝里，① 新那一项乘上「环境变量 `PROBE_OLD_BOUND` 没设」，设了就等于合入之前的上界；② 先算判出放不下前打一行 `PROBE_PRECHECK`（需求、可用、新上界、去掉新那一项的上界、各项槽数）；③ 验证包里追加三个探针测试：`probe_sweep_natural`（不带读故障的自然崩溃那一形，文件多长 g 个数据单元，g 从 1 扫到 100）、`probe_sweep_natural_warm`（同一形，先写成功一个长 1 个单元的写让这次挂载里有了发成的版与开过的聚簇段，再扫 g 从 2 到 100）、`probe_sweep_forms`（读回全零的三形，写 10 到 100 个单元七档）。每一格同一个写连发三次，三次都没写进就卸载重挂再发一次，每次打聚簇段里没挡的槽对数（`in=`）、段外槽对数（`out=`）与结局。③ 三种扫法各新旧并排（`side_by_side.py`），外加把已有三个测试目标带先算那一行跑一遍，数「靠新那一项翻面的行数」。

## 一、要主 agent 跑的命令

命令一条，在 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-sonnet`（`--cwd`）起；脚本里每条 `cargo test` 都带了自己的范围声明，外面这一条是启动脚本，按「驱动脚本」那一条读进去判：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table 等三个目标的探针; 范围=3 个测试目标; 为什么=量先算上界新旧口径在宽护卡住的状态上差在哪一格' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-sonnet -- bash /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-sonnet/model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-sonnet/run1
```

- 在哪个目录起：上面 `--cwd` 的草稿目录。`--work` 要给新目录（脚本拒非空）；服务撞内存上限重排同一条命令时，先删 `run1` 再交（脚本不在旧输出上接着写）。
- 看哪几行判结果：`TASK_SERVER_RESULT` 的 `type=` 与 `exit=`；`task-server/task log <号>` 里 `name=build done`、`name=exit_codes natural_new=0 natural_old=0 forms_new=0 forms_old=0 warm_new=0 warm_old=0`、三行 `name=precheck file=existing-… flipped_by_the_term=<数>`、`name=lines` 六行（`probe_lines` 不为 0）、`SUMMARY cells=… same=… lost_path_in_old=… recovered_in_new=… white_push_new=… inside_positive_cells=…`，末行 `name=rerun_done`。编译错或某趟退码非 0 时整条退 1，错在 `run1/out/*.out` 头几十行。
- 规模（推的，没量过）：补丁只动三个文件，编译 `singlefs-core` 加验证包测试目标、变量 `CARGO_TARGET_DIR` 由服务给；探针的内存与时间我没量，自然那一形一格是一次挂载加至多四次写加一次卸载重挂，盘都是内存里的稀疏盘，不碰 GPU；每格要多久我没量，扫 100 格的自然那一形可能比读回全零三形慢，不预估。
- 不跑的：不跑任何门禁阶段、不带 `--item`。
- 副本的去留：`run1/copy` 用完由主 agent 删（带 `target`，会大）。

## 二、判决条目复核表

「站得住」是我现查之后挑不出毛病；「够不着」是判决那句话够不到它想堵的东西；「同样打中别的臂」是修补自己带出来的同形问题。读码推出来、没量过的写「推的，没量过」。

| 判决条目 | 辩方判定 | 证据（我现查的） |
|---|---|---|
| 第二节 T3-4 的机制前提：再挂时分配器重建、这次挂载开过的聚簇段清空 | 站得住（读码） | 「cluster_segments: BTreeSet::new(),」（`crates/singlefs-core/src/allocator.rs:1955`）在 `PoolAllocator::new`；重建入口起步就是「let mut allocator = Self::new(devices);」（`crates/singlefs-core/src/allocator.rs:2270`）；字段文档写「只在内存，重开后按 D3（空间分配） 已定项 10 ①「挂载后新开一段」重来」（`crates/singlefs-core/src/allocator.rs:1887`）。第一轮正推腿写「我没找到一条「卸载再挂本来放得过、上界却 < 需求」的形」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r1-sonnet-output.md:177`）、也只看到聚簇段集合「只有 `insert`」，漏了每次挂载重建这一条；攻方那一句对 |
| 同一判：聚簇段里没挡的槽对不在合入之前的上界里 | 站得住（读码） | 合入前上界的槽对一支只有「slots.deferred + slots.isolated + slots.free_but_blocked;」（`crates/singlefs-core/src/mounted_session.rs:5992`）三样，都是「放开之后才没挡」的槽；已经没挡、却在聚簇段里的槽对哪一样也不算。段外槽对的读法是总数减各段数（「self.unblocked_slot_pairs - pairs_inside_the_cluster_segments」，`crates/singlefs-core/src/allocator.rs:599`） |
| 「违正文射程：保住被宽护卡住那条出路」 | **够得着主 agent 的射程，够不着用户条款的字面** | 射程来自「保住被宽护卡住那条出路」（`research/prompts/_m4-gpugc-floor-raise-precheck-code-r1-body.md:7`），那是主 agent 在用户定案里定的读法。kb 记的用户定案（原话在变更史，我没读）是「推抬 F 放得出的全放出来也放不下就一串都不推」（`.claude/kb/decisions/16-发布语义.md:275`）：聚簇段里已经没挡的槽对，抬 F 放不出它，卸载重挂才放得出。采纳等于把「抬 F 放得出的」放宽成「抬 F 或卸载重挂放得出的」。这是第二个读法，不是笔误，见第三节「加的那一截是不是卸载重挂放得出的那一截」 |
| 合入之前的槽对上界本来就是「抬 F 放得出的」吗 | **不是，所以采纳的修补不是头一回放宽** | 攻方第一轮自己写过「defer 里有一大截是聚簇段里换下的元数据，算进了槽对的上界，放开它们补不成段外的槽对」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-output.md:140`）：聚簇段里的 defer 槽，抬 F 放开也补不成段外的槽对，只有卸载重挂才补得成，合入之前的上界已经把它们算作一对。新那一项补上的，是同一批段里「已经没挡」的那一半，两半合起来，上界才一致地等于「卸载重挂之后全部没挡的槽对加每个放得开的槽各补一对」 |
| 「可达性没量」（判决第二节 T3-4 一行末、交用户节） | 够得着：仓里两种造法都有，不必读坏的被抛弃根 | 攻方说「宽护那一形要读坏的被抛弃根才造得出，探针造不出」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-output.md:35`）是它自己探针的限度。仓里有不带读故障的自然崩溃那一形：「（不带任何读故障的自然崩溃点：可写挂载择 P，R 那几条记录在水位之上、没施加，」（`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3590`），挂上就是「被宽护卡住」；读回全零的三形在 `pool_stuck_behind_the_wide_protection`（同文件 2710 行）。本轮探针两种都扫 |
| 采纳的改法：上界加上聚簇段里没挡的槽对（第三节第 1 条） | 打中攻方提的缺口；**不分宽护，非宽护状态同样放大** | 报「被宽护卡住」要同时满足 F 已在上限和 C 那一圈隔离着槽：「FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}」（`crates/singlefs-core/src/mounted_session.rs:1974`）之后才读「let Some(target) = root_ring_push_target(allocator, current.root.checkpoint_txg) else {」（`crates/singlefs-core/src/mounted_session.rs:1980`）。没有隔离着的槽时推满报 `NotWhatStopsTheAdmission`，不列卸载重挂；新那一项在这些状态里也让上界变大，多放行的写白推一串，换不来出路。代价见第三节「宁大勿小之后白推一串的代价」 |
| 配一条判法小用例钉「聚簇段里空着的槽对算进上界」，配变异行 | 站得住（字面）；**够不着读分配器那一半** | 小用例在 `crates/singlefs-core/src/mounted_session.rs:8374`，变异行在 `crates/mutations.tsv:7580`（只改上界式子）。新那一项从真分配器读出来的那一段（`crates/singlefs-core/src/mounted_session.rs:5965`起）没有变异行，见第四节「新那一项读成 0 时，判法模块有没有拦」 |
| 第三节第 2 条：文档「八种」改「九种」 | 站得住 | `UserChangeRefused` 现有九个成员（`crates/singlefs-core/src/mounted_session.rs:590`、600、613、619、630、635、643、646、652 各起一个）；文档在 `crates/singlefs-core/src/mounted_session.rs:2913` |
| 第三节第 3 条：运行库两条崩溃循环用例补不并行的理由 | 站得住 | 「// 不切片并行：每次挂载接着上一次崩溃留下的盘，次序本身就是被测对象，没有可切的区间；两种填法各是一个测试函数，由测试框架并行。」在 `crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1093` 与 `:1101` |
| 第三节第 4 条：D16 已定项 10 射程补「头一次被拒就判出时」与先算不罩的三形 | 站得住；**新那一项没写进射程** | 射程里有「头一次被拒就判出时盘上逐字节不变」「短在段外槽对那一判同样先算」（`.claude/kb/decisions/16-发布语义.md:275`），整行没有「聚簇段」三个字。代码里槽对那一判的上界已经多了一截，kb 射程没跟，下一批要补一句 |

## 三、T1　第一轮的修补本身

### T1　加的那一截是不是「卸载重挂放得出」的那一截

- 口径同：先算读的和准入读的是同一个函数。准入的落得下那一判用 `allocator.cluster_segments()` 调「.unblocked_slot_pairs_outside_the_cluster_segments(」（`crates/singlefs-core/src/admission.rs:994`）；新那一项在 `of_the_device` 里用同一个函数、同一个分配器，传空集得全部槽对，传这次挂载的聚簇段得段外槽对，两次相减就是段内槽对（`crates/singlefs-core/src/mounted_session.rs:5965`起）。重挂之后聚簇段集合是空的（`crates/singlefs-core/src/allocator.rs:1955`），准入读出的段外槽对就等于现在的全部槽对。所以新那一项加进来之后，上界里「段外槽对」这一项等于「重挂之后全部没挡的槽对」，口径一致（读码）。
- 总数等于各段之和是构造出来的：「unblocked_slot_pairs: unblocked_slot_pairs_per_segment.iter().sum(),」（`crates/singlefs-core/src/allocator.rs:489`），增量维护在 `set_blocking_bit` 里两份同进同出（`crates/singlefs-core/src/allocator.rs:551`起）。聚簇段一个槽对不会横跨两段（段长 64、偶数起点）。
- 重挂之后「当前没挡」不一定全还没挡（推的，没量过）：重建的分配器位图是重算的，分配位照记录、隔离位照影子账、扣住位是内存的不带过去；重挂择根若不是现在这一版（读坏退到别的根）隔离集会不同。方向是上界偏大，不是偏小：当前没挡的槽不挨任何一条仍在窗口里的根引用（引用着的在 defer 里，仍置分配位），重挂不会新多出隔离。这是「推的」，不是「量的」。
- 新那一项只补「已经没挡」的槽对；「聚簇段里仍被挡着的槽、重挂之后放开」的那一半不用补：在 defer、隔离、扣住三项里，按「一个槽至多补成一对」已经算进去（`crates/singlefs-core/src/mounted_session.rs:5992`），也就是攻方一轮说的「聚簇段里的 defer 算进了槽对的上界」。

判：口径是同一个，加的就是「重挂放得出」的那一截。**站得住**（读码）。会推翻它的现象：探针里某一格新口径放行、重挂之后同一个写仍写不进，且原因是聚簇段里的槽对重挂后还是被挡着（不是别的原因）。

### T1　会不会多算：聚簇段里的槽对卸载重挂也放不出

我能列出的，都小，都在「宁大勿小」许可的一侧（推的，没量过）：

1. 卸载自己的那一串（抬 F 到现行那一版的空发布）要在开放段里占几个槽，重挂又要写行、发空发布，也占；新那一项没扣。占几个槽（一次空发布重写固定点单元、不碰用户树，见 `crates/singlefs-core/src/mount.rs` 里 `push_one_empty_publish_keeping_the_floor` 的文档）；占多少我没数，探针里重挂之后的 `after_the_mount=[out=… in=…]` 会给出。
2. 卸载本身被拒（卸载也要过空间）：`crates/singlefs-core/src/mount.rs:1121` 写「正常卸载照常做成：靠的是卸载那一串的预演算上新 F 回收出来的槽」，那一串的槽取不到时卸载拒，列了卸载重挂也走不通。这与新那一项无关，合入之前的上界对它一样，不算修补带出来的。
3. 合入之前的上界已经把聚簇段里的 defer 槽各算一对（第二节表的第四行）。新那一项和它同方向，不是头一个把「抬 F 放不出的」算进去的。

判：**有多算，量级小，方向许可**。会推翻它的现象：探针里 `lost_path_in_old` 为 0 而 `white_push_new` 很大，即新口径放行的格大多重挂也写不进。

### T1　宁大勿小之后白推一串的代价：什么形、多大

- 形：写要的槽对数 N 满足 合入之前的上界 < N ≤ 新上界，也就是 N 落在「段内空着的槽对」那一条带里。带宽等于段内没挡的槽对数，一个全空聚簇段 64 槽、32 对；这次挂载开过几段，上限就是 32 乘段数（推的）。聚簇段集合只在「OpenEmptySegment」那一支加（`crates/singlefs-core/src/allocator.rs:2625`），被回收空的聚簇段还会被最低全空段重新选中（「段里的块被回收空了也仍算聚簇段：它还能被 `lowest_empty_segment` 重新开来装提交内生块。」，`crates/singlefs-core/src/allocator.rs:1890`），所以段数增长慢于发布数（推的）。
- 这条带里的写，推完也写不进（先算的上界是真上界），除非报「被宽护卡住」之后用户卸载重挂。**在有宽护的状态里，推的每一次空发布写掉根环一槽，是向「宽护解开」走的**（`crates/singlefs-core/src/mount.rs:4956`起那一段：上限卡在读坏或读回全零槽上时推不抬 F 的空发布，写掉那几槽上限就不再被它们卡住），不全是白写；**没有宽护的状态里就是白写**。
- 一次的量：kb 记的是另一种形（从不放得下的写）量到的：「每次 `series=2 empties=6`，一次白写 `writes=40 barriers=16 written_bytes=500736`」（`.claude/kb/decisions/16-发布语义.md:275`）。这条带里的白推没量过，量级大概率同一档（推的）；同一次挂载里 F 到上限之后再重试不再推（「上限不高于现行 F、也没卡在那几槽上：不推（`FloorAlreadyAtTheCeiling`）。」，`crates/singlefs-core/src/mount.rs:4963`），所以是每次挂载一次，不是每次 `accept` 一次。崩溃循环每次挂载都重来一次，这是 kb 量过的那个量级的来源。
- 崩溃循环用例对新那一项不敏感：它们的写比整个单元区还大（`write_that_never_fits`，`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1056`），上界加几十对也远不到。所以「332 条没有用例改判」不能当作这条带不存在的证据，只说明已有用例落不进这条带。

判：**代价存在、形可识别、量级小于一次崩溃循环的整串**；没量过。探针量两个数：`flipped_by_the_term`（已有三个目标里靠新那一项翻面的先算行数）与 `white_push_new`。会推翻它的现象：`flipped_by_the_term` 为 0 且 `white_push_new` 为 0，即带宽在所有格上都没有写落进去。

### T1　被宽护卡住那条出路在这一形上走不走得到

走得到要过三道门，新那一项只开第一道：

1. 先算放行：新那一项让段内空着的槽对不再被算少。**开了**。
2. 推到停在「F 已在上限」：预算做满（一次准入至多 `PUBLISHES_PER_ADMISSION_AT_MOST` 次发布，`crates/singlefs-core/src/mount.rs:4743`）停在 `PublishesPerAdmissionWouldBeExceeded`，则「别的都照旧」，不报宽护（`crates/singlefs-core/src/mounted_session.rs:1975`那一支返回 `NotWhatStopsTheAdmission`）。这一道不是新那一项管的，合入之前就是这样。
3. C 那一圈还隔离着槽，才报「被宽护卡住」并「出路：卸载重挂一定列」。同上，不是新那一项管的。

判：**走得到的充要条件没变，新那一项只补了第一道上被聚簇段吃掉的那一截**。**够得着**那一条缺口，**够不着**第二、三道（本来就不归这一项）。

## 四、T2　产生结论的方法与从未被看过的地方

### T2　可达性：这一轮能不能造出来

**能造，不必读坏的被抛弃根**（读码，探针待跑）。

- 不带读故障：自然崩溃那一形（`pool_crashed_after_the_first_record_of_a_long_sequential_write`，`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3599`）崩在长顺序写的第一条记录之后，再挂时「C 那一圈只有 R 那一槽（阳性对照）」（同文件 3683 行），挂上就被宽护卡住；文件多长 g 个单元由 `bytes_of_the_file_grown_by`（同文件 3707 行）定。已有用例把 g 取 13、20、36（同文件 3714 行起的常量），其中 g = 20 与 36 推完仍拒、卸载重挂放得下（同文件 3904 行的断言「卸载再挂之后同一个写放得下」）。
- 带读故障：`pool_stuck_behind_the_wide_protection`（同文件 2710 行）三形，已有 60、80 个单元推完仍拒的用例（`crates/singlefs-verification/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:1603`）。
- 这些已有用例不能说明新那一项有没有用：它们在合入之前的上界下就放行了（验证包 332 条没有用例改判），也就是已有的 g 没落进带宽里。下面是本腿的探针要补的。
- 新旧口径分得出来，要这一次挂载里聚簇段已有空着的槽对：重建之后聚簇段集合是空的（`allocator.rs:1955`），只有这次挂载里有过一次需要提交内生块的发布才开段（`allocator.rs:2625`）。可写挂载写行那次发布与推的空发布都会走分配器取提交内生块的落点，所以第一次被拒时段内槽对数是不是 0，我没法读码断定（「推的，没量过」）：探针每次被拒前打 `in=`。`probe_sweep_natural_warm` 先写成一个成功的写，保证这次挂载里有发成的版，是为了不靠这一条猜测。

会推翻「能造」的现象：探针里每一格 `in=0` 且 `flipped_by_the_term=0`；那就是已有的状态里新那一项恒为 0，只能靠构造更满的池。

### T2　产生结论的方法

采纳依据是一条腿的读码（`research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-output.md:161` 一行，自标「推的」）；第一轮另一条腿读同一处得出相反结论（`research/prompts/m4-gpugc-floor-raise-precheck-code-r1-sonnet-output.md:177`）。两条都没跑。我读码核了机制那一半对（第二节表的第一行），所以采纳不是靠运气，但**带宽有多大、落不落进已有用例、白推多少，三样都没有读数**。攻方那一腿自己的改法写的是另一条（新成员带宽护报告）并明说：「槽对上界不该加聚簇段里的槽对（抬 F 放不出它们），加了就会推一串白写。」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-output.md:167`）。判决采的是它**没推荐**的那条改法。这不是错，是要在交用户时说清：这条改法被攻过零轮，提议它的腿自己没推荐它。

### T2　新那一项读成 0 时，判法模块有没有拦

- **判法模块只验了算术，没验接到真分配器上**（成立，读码）：模块 `everything_a_floor_raise_could_release_tests`（`crates/singlefs-core/src/mounted_session.rs:8203`）里的用例都直接构造 `SlotsAFloorRaiseCouldRelease`；`of_the_device` 只有 `devices_short_even_with_everything_a_floor_raise_could_release` 一处调用（「SlotsAFloorRaiseCouldRelease::of_the_device(」，`crates/singlefs-core/src/mounted_session.rs:6044`），模块里没有用例走到它。
- 变异行 `crates/mutations.tsv:7580` 只改上界式子里乘的那一项，对应的是式子。同一个文件里另外三项读分配器的读侧，各有一行变异，用的是集成用例：`读分配器时 defer 读成 0`（`crates/mutations.tsv:7492`）、隔离（7493）、扣住的槽（7494），分别指到验证包 `accept_raises_the_floor_for_space` 的用例。新那一项读侧没有对应的一行。
- 所以「新那一项读成 0」「两次相减写反」「传错聚簇段」这三类变异，单元测试看不见；写反时会在 `.expect` 上 panic，但只有某条已有的集成用例里段内槽对大于 0 才红；读成 0 则一定要有一条落进带宽的集成用例才红（推的）。是否有，看探针里 `flipped_by_the_term`：为 0 就是没有。
- 我建议的变异行（只是文本，我不写 `crates/mutations.tsv`）：旧串「.unblocked_slot_pairs_outside_the_cluster_segments(&BTreeSet::new())\n                .checked_sub(」，新串把 `&BTreeSet::new()` 换成 `cluster_segments`，段内槽对恒为 0；指到探针里挑出来的那一格做成的集成用例。

判：**成立**。会推翻它的现象：`flipped_by_the_term` 大于 0，即已有三个目标里有一格靠新那一项翻面。

### T2　有没有别的修法

| 修法 | 保住出路 | 白推 | 代价与风险 | 被攻过 |
|---|---|---|---|---|
| 无条件加段内槽对（判决采的） | 是 | 非宽护状态也放大 | 一处读、一处加；上界与 kb 射程要补一句 | 零轮 |
| 有隔离着的槽才加（本盘或任一盘 `isolated > 0`，字段已在 `SlotsAFloorRaiseCouldRelease`，`crates/singlefs-core/src/mounted_session.rs:5947`） | 是（报宽护要隔离着槽，`mounted_session.rs:1980`起） | 非宽护状态不放大 | 多一个分支和一行变异；`isolated` 是全部隔离，是 C 那一圈的超集，仍是宁大勿小（推的） | 零轮，我提的 |
| 新成员也带宽护报告（攻方原提） | 是 | 不变 | 报「被宽护卡住」要 F 已在上限（`mounted_session.rs:1974`），先算处还没推，不知道；要读盘算上限，而这一支承诺只读分配器（`crates/singlefs-core/src/mounted_session.rs:6025`）；改公开错误成员 | 零轮 |
| 读侧改用总数：加 `unblocked_slot_pairs` 的总数访问器，段内槽对 = 总数 − 段外槽对 | 同「无条件加」 | 同「无条件加」 | 去掉两次调用和一处 `.expect`；行为不变；要在 `allocator.rs` 加一个公开方法（推的） | 零轮 |
| 不改，射程写明：先算只拦抬 F 与现有段外槽对都不够的，聚簇段里空着的要卸载重挂才放得出，ENOSPC 之后由调用方自己卸载重挂 | 否：宽护状态下被先算拦掉的写报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，没有出路 | 最少 | 与「保住被宽护卡住那条出路」冲突；等于收窄主 agent 的射程，要用户定 | 即合入前的形态 |

「有隔离着的槽才加」在白推一侧严格不比「无条件加」差，也不吃出路；代价是多一个分支。**但它的优势只在「非宽护状态且需求落进带宽」这一小块**，如果探针里 `white_push_new` 全落在宽护状态，两者一样。哪个更好等读数；我不替主 agent 选。交用户的岔路是「无条件加」对「不改、射程写明」（要不要把「保住出路」当作先算的目标之一），「有隔离着的槽才加」与「读侧改用总数」是同一目标下的实现选择。
同类先例：仓内有：合入之前的上界已把聚簇段里的 defer 槽按一对算（`research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-output.md:140`）；仓外的我没找，也不引（别的项目的做法只是线索）。

## 五、检查单八格

| 格 | 判 | 依据（我追的调用点与现查的） |
|---|---|---|
| 走不到的理由二次查实 | **成立** | 实现员报告（`~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-fixes-writer/result.md` 的「走不到的理由」一节）写「无」的同时，把新加的一处 `.expect` 依赖的不变量写了：传聚簇段算出的段外槽对不大于传空集算出的。我追了：`of_the_device` 只有「SlotsAFloorRaiseCouldRelease::of_the_device(」（`crates/singlefs-core/src/mounted_session.rs:6044`）一处调用，被 `devices_short_even_with_everything_a_floor_raise_could_release`（同文件 6098 行起的调用）调，再被 `push_one_floor_raise_for_a_refusal_for_space` 的三个调用点（`crates/singlefs-core/src/mounted_session.rs:3795`、6213、7048）调，三处传的都是会话自己的分配器和刚被它拒的那一次。不变量的来源是总数等于各段之和（`crates/singlefs-core/src/allocator.rs:489`）；第二次调用里「聚簇段在单元区里」「在段边界上」的断言准入读同一个函数时已经跑过（`crates/singlefs-core/src/admission.rs:994`）。唯一新的 panic 形态是总数与各段之和对不上；那时准入读同一个函数在 `crates/singlefs-core/src/allocator.rs:599` 那一减法上早已不对，这条 `.expect` 只是第二个暴露点，不是新增的可达路径。会推翻它的现象：`PlacementRoomOnOneDevice` 与先算读到的聚簇段集合不是同一份（比如两次读之间分配器被换回发布之前那一份而集合不同），探针没有造这个，读码上两次之间只有 `publish` 的失败退回，退回的是整份分配器 |
| 类型与名字 | **成立（带两条记）** | 新字段名 `unblocked_slot_pairs_inside_the_cluster_segments`（`crates/singlefs-core/src/mounted_session.rs:5953`）与已有的 `unblocked_slot_pairs_outside_the_cluster_segments` 成对，本地变量 `slot_pairs_a_remount_frees_from_the_cluster_segments`（5995 行）把单位写进名字，不用序数、不缩写。两条记：① 槽与槽对共用 `u64`，同一结构里 `deferred`、`isolated`、`free_but_blocked` 是槽数，新字段是对数，编译器拦不住互相赋错；换算只有两处各乘 `slots_of_a_data_unit`，靠名字和变异行守；对数乘它的那一处没有单独的变异行，被新小用例的「需求一个数据单元」间接钉住（读码，推的）。② 结构名 `SlotsAFloorRaiseCouldRelease`、函数名 `most_a_floor_raise_could_make_available` 与公开错误成员的字段 `most_a_floor_raise_could_make_available` 说的是「抬 F 至多放得出」，现在含一截抬 F 放不出、卸载重挂才放得出的量；文档已改（`mounted_session.rs:5940`起的结构文档），名字没跟。它仍是「至多」的上界，只是更松，不算错，要不要改名交主 agent |
| 不兼容旧代码与旧格式 | **成立** | 补丁里没有认旧版本的分支、旧布局号、迁移、`legacy_ok` 一类；新加的只有一个字段、一个参数、一条相减和一个单元测试（改动全文 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r2-diff.md`）。新增注释全是现状句，没有「原先」「此前」「后来改成」 |
| 不新建判定库 | **成立** | 没有新目录、TSV、sqlite、RocksDB；新字段在内存分配器的读数里，变异行进 `crates/mutations.tsv`（已有登记位） |
| 所有任务多机可用 | **成立** | 新小用例（`crates/singlefs-core/src/mounted_session.rs:8374`）只构造结构、不读活树、不读主机状态；变异行命令是 `-p singlefs-core --lib -- … --exact`，两台一样 |
| 测试优先多线程 | **成立** | 两条崩溃循环用例各有一行「不切片并行：每次挂载接着上一次崩溃留下的盘，次序本身就是被测对象，没有可切的区间；两种填法各是一个测试函数，由测试框架并行。」（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1093` 与 `:1101`）；我对这一行的判是：理由站得住（次序是被测对象），且已经说了测试函数之间并行 |
| 带 --item 的命令与项 | **成立** | 这一批没有新的崩溃点重放项。新变异行 `crates/mutations.tsv:7580` 六列，旧串在 `crates/singlefs-core/src/mounted_session.rs` 里恰好出现一次（我 grep 的，计数 1），指到的测试 `an_unblocked_slot_pair_inside_the_cluster_segments_counts_toward_the_slot_pair_bound`（8374 行）在。**这一格只判登记，不判覆盖**：覆盖的缺口在第四节「新那一项读成 0 时，判法模块有没有拦」 |
| 新崩溃流多罩了什么 | **成立（不需要新崩溃流）** | 改动只改先算的读数和上界，不新增写、不新增落盘顺序；放行更多的写走的是已有的推串，崩溃点已被已有的崩溃循环用例与推串用例罩着（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1056` 一族）。**推不出**的一点：带宽里被放行的写，其推串在宽护状态下停在上限后再卸载重挂的整条路径（推串、报宽护、卸载、重挂、写进），有没有一条崩溃枚举罩着，我没查；已有的验证包用例是顺序场景加给定状态镜像，不枚举 |

## 六、给主 agent 的总判（辩方腿的结论，主 agent 定）

1. T3-4 那一条的**机制**站得住，**射程**有两个读法，采纳的是「保住出路」读法，不是用户条款字面的「抬 F 放得出」读法；合入之前的上界就已经把聚簇段里的 defer 槽算作一对、并不是严格的「抬 F 放得出」，所以采纳不是第一次偏离字面（第二节表的第三、四行）。
2. 采纳的改法**够得着**缺口，**也打中非宽护状态**：新那一项无条件加，没有宽护时多放行的写白推、不换出路。代价小、形可识别、没量过（第三节「宁大勿小之后白推一串的代价」）。有一条更窄的实现（有隔离着的槽才加），是我提的，零轮。
3. 可达性**不是**「要读坏的被抛弃根才造得出」，仓里有不带读故障的自然崩溃那一形；这一轮探针两种都扫。
4. 判法模块只验算术；读分配器那一半没有变异行，是否有集成用例拦它，等 `flipped_by_the_term`。这是下一批要补的一行变异和（若 `flipped_by_the_term` 为 0）一条落进带宽的集成用例。
5. kb 射程要补一句新那一项（`.claude/kb/decisions/16-发布语义.md:275`）。

## 七、没做什么

- 没替主 agent 撤销或维持判决，只交复核结果。
- 没往统一任务服务交任务，没起 cargo，没编译，没跑模型：判决要引的数（`flipped_by_the_term`、`lost_path_in_old`、`white_push_new`、`inside_positive_cells`）都没复跑，命令在第一节「一、要主 agent 跑的命令」。探针补丁只在冻结副本的拷贝上试打过（打得上），拷贝已删。
- 没改 `crates/`、`crates/mutations.tsv`、kb；建议的变异行与 kb 补句都只是文本。
- 没读禁读清单里的文件（`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-output.md`、`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/`）。
- 没读用户原话（变更史里），所以第二节「射程」一行里关于用户本意的判断只到 kb 的写法为止。
- 没查仓外的先例。
- 草稿目录里留下：`model/`（探针、复跑脚本、sha256）。没有仓副本与编译目录，没东西要删。
