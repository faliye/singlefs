# m3-close-b20-batch-code-r1 攻方腿报告（第 20 批修补之后七份产品补丁的代码轮，2026-11-10）

用例在模型目录里写好、没编译、没跑（子 agent 不交任务）。凡用例结局都是「推的，待主 agent 跑」；标「量的」的只有我现读源码、`grep`、`sha256sum` 与草稿目录里一个只读脚本的读数。被判的四份文件与开工快照 `research/prompts/m3-close-b20-batch-code-r1-snapshot/crates-sha256.txt` 逐份相同（量的：655bb6f5…、6d006f68…、02b07348…、87290b5b…），冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-batch-code-r1-opus/copy` 里这四份也相同。

## 复跑

```bash
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b20-batch-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-b20-batch-code-r1-opus/copy /home/fy5090/.local/share/singlefs/work/m3-close-b20-batch-code-r1-opus/attack-1-run /home/fy5090/code/singlefs/research/prompts/m3-close-b20-batch-code-r1-opus-model
```

模型目录每个文件的 sha256（量的，`sha256sum`）：

```text
9bc3d6ac8ce4232f29c1bcc66ba16b188b60cd1abb87769768f326c8948015a8  attack_cases_appended_to_the_fall_back_test_target.rs
c7ae768eada948374fb3eda1a952379adfbaba184d8f0cf66dc09ee32e526067  tools/rerun.sh
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：把冻结副本拷进运行目录，把两条攻方用例追加到测试目标 `recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed` 末尾（借它的判退造盘与「某根槽第几次读读回全零」那层盘），只跑名字以 `attack_` 起头的两条，打出 `ATTACK_` 行。盘全是内存稀疏盘（`SparseBlockDevice`，造盘函数 `record_one_publish_after_overwrites`），每格从内存里那一份起点盘拷。不调崩溃枚举函数，全量状态数 0（这一腿没有跑崩溃态）。估时（推的）：编译一个测试目标为主；跑的部分是 17 + 13 次 384 槽小盘上的可写挂载、17 次卸载与至多 104 次覆盖写，同目标里同量级的入库用例一条 30 次挂载，按它推不到 1 分钟；没有分批。探针的内存与时间没量。

## 背景材料对不上的一处（当场说）

正文第 11 行说 `mount.rs` 第 3541 行的 `roots_of_the_rebuild_pass_with_the_roots_read_by_the_read_stage` 与 `recovery.rs` 的 `choose_root_keeping_every_self_verified_root_read` 出自补丁 `read-stage-roots`，而 diff 文件里这一份补丁那一节是空的（`_m3-close-b20-batch-code-r1-diff.md` 第 541–544 行只有空围栏）。量的：`~/.local/share/singlefs/work/m3-close-b20-read-stage-roots/patch/crates.patch` 是不带 `diff --git` 头的统一 diff（文件第 1、2 行直接是 `--- a/…`、`+++ b/…`），摘 diff 的那一步多半按 `diff --git` 切，漏了整份。我按那份补丁原文与今天的代码判，没有靠 diff 文件。

## 各格判定一览

| 面 / 格 | 判 | 一句话 | 量的还是推的 |
|---|---|---|---|
| 面一 读阶段全集进重建那几遍 | **打中 A1** | 判退那一形，R 读阶段自证过、重建读根环那一遍读坏一次：C 照读阶段那一份护住 R，根环表在 R 那一槽却记成读坏或全零；写行之后 R 被抛弃，正常卸载那一串借 C（H2）要「那一槽记成同一条有效根」，借不到就读盘读 R 的账，R 的账读不出、没有凭据 ⇒ 卸载被拒；同一张盘不注这一次读错时卸载借 C、做成 | 读码量的；结局推的，待主 agent 跑 |
| 面一（影子账那一侧） | **打中 A2**（C554 已登记那一形的子形） | 被抛弃根 R 读阶段读到、重建那一遍读坏一次：并根只进 C，C 又滤掉被抛弃的；影子账只看这一遍，R 独占的槽不隔离。手里已有读阶段那一份，R1「按槽并两遍」防的正是「一遍读出、另一遍读坏的被抛弃根掉出影子账」 | 同上 |
| 面一 去重键 | 没打中 | (实例代号, checkpoint_txg) 在一次挂载里唯一；漏掉的只是判退两问里读出、读阶段与重建那一遍都没读出的根，而那样的根判退会退到它上面 | 推的 |
| 面二 删掉读阶段之后那次现择 | 没打中（一形推的、没量） | 判退那一形照 R 的表与判退本身一致；施加过记录、新实例写行那次发布由记录重建时，所选那一版的表比 R 的表多一行，要两次瞬时读错才分出差别，旧代码同样如此 | 推的 |
| 面二 读次数少一环 | 产品承诺没变；随机历史与故障注入那一项推不出 | 实例表那次重读照旧；去掉的只有「择不到根」那一支，读阶段先拒在 `NoValidRoot` | 推的；崩溃项待跑 |
| 面三 不可为空之后的测试断言 | 成立（轻：一行旧注释） | 判官与模型映射都按 `{ .. }` 认这一拒、不读字段，复算按 `choose_root`；改成真值没让断言变空 | 量的（读码） |
| 面四 H6 第 0 代出路 | 没打中 | 暖机根、写行记录、写行单元读得出都拒；只读挂载不写盘；跨实例只撕取号的盘面写出的值与 mkfs 同；瞬时读不出的较新东西放得过闸，但自证槽留着的 tail 不降 | 推的（读码） |
| 面五 H3 新旧入口并存 | 没打中（产品）；验证侧一处轻 | 产品 crate 里旧两个入口 0 个调用方；core 里不筛 fsid 择系统配置的几处是挂着之后的入口（故意拿它认换进来的别池盘）与取号之后的清扫 | 量的（grep） |
| 检查单 走不到的理由 | 成立 | 七份补丁 core 新加行没有 `expect`、`unreachable!`、`todo!`、`assert`、`_ =>` | 量的 |
| 检查单 类型与名字 | 成立（轻） | 实例代号、txg 是 newtype，fsid 是 `[u8; 16]`（与存量同型）；一条文档说过头、一行测试注释过期、H6 借用的拒绝成员管理员出路不对 | 量的 |
| 检查单 不兼容旧代码与旧格式 | 推不出 | 旧入口不是兼容旧格式；产品里没调用方，验证包 390 行调用 | 量的 |
| 检查单 不新建判定库 | 成立 | 七份补丁 core 改动行没有存判定的地方 | 量的 |
| 检查单 所有任务多机可用 | 成立（正文一句不实） | 实例表那条新用例用的是 `std::env::temp_dir()` 下的镜像文件，不是内存盘；每进程自建自删，不绑机器 | 量的 |
| 检查单 测试优先多线程 | 成立（轻） | 判退那三条新用例各在一个测试函数里串行扫 30 格 | 量的；耗时推的 |
| 检查单 带 --item 的命令与项 | **不成立（轻）** | 三份实现员报告的变异命令写的是 `--item <项>` 占位，没有项名；静态锚点核 840 行 0 行找不到 | 量的 |
| 检查单 新崩溃流多罩了什么 | 成立；结局推不出 | 没有新流；读次数挪了，按读次数注入的那一项罩到哪些读要跑才知道 | 推的 |

## 一、面一：读阶段读到的全部根进了重建分配器那几遍

### A1：判退之后 R 的根槽在重建那一遍读坏一次，正常卸载被拒（对照做成）

历史与故障（每一步指到许可它的代码）：

1. 造盘同入库用例 `a_root_whose_units_read_back_zero_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts`：384 槽小盘、第一个文件之后覆盖写 5 次（P 是最后一次），再覆盖写一次 R，崩在 R 的根槽之后、轮换之前（R 的单元、记录、根槽都落了，系统配置见证到的是 P）。这一次挂载里 R 的单元读回全零（同那条用例的读法），R 的记录读得出、齐。
2. 读阶段择到 R，`ReadStage::self_verified_roots_read` 里有 R。R 走读失败、没见证 ⇒ 判退到 P（`mount.rs` 第 9923 行起那一段）。
3. 故障：R 的根槽只在重建分配器读根环那一遍读回全零一次（之后照读）。每块盘别处都读得出有效根，这一遍不整环重读（`mount.rs` 第 3362–3372 行）。
4. C：`crates/singlefs-core/src/mount.rs:3140`：「let roots_for_the_newer_readable_roots =」并上读阶段那一份，R 比 P 新、按 R 自己的表有效，进 `placements_in_the_accounts_of_newer_readable_roots`；R 的账读不出、记录齐 ⇒ 丙那一支，C 在 R 那一槽（`target_for_publish(R 的 txg)`）记着落点。挂载做成（同入库用例的结局）。
5. 根环表：`root_ring_occupants_of_the_rebuild` 只按这一遍读数建（`crates/singlefs-core/src/mount.rs:3540`：「影子账、根环表与回收门槛不用这一份：它们说的是这一遍根环槽里读到了什么。」），R 那一槽记成读回全零（设备报错时是读坏）。
6. 写行那次发布的实例表给 R 的实例写一行，R 从此被抛弃。用户紧接着正常卸载（`unmount`，`mount.rs` 第 4562 行），走 `raise_the_floor_through`。
7. 抬 F 那一串这一遍读得出 R（故障只一次），R 按现行那一版的表被抛弃。取表里记着的落点：`placements_recorded_for_the_abandoned_root` 只认那一槽记成同一条被抛弃根；再借 C：`crates/singlefs-core/src/mount.rs:5488`：「.placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(」，它要那一槽记成 txg 相同的有效根，读坏与全零交 `false`（`crates/singlefs-core/src/allocator.rs:1456`：「RootRingOccupant::AllZeroAtMount { .. } => false,」）。两样都取不到 ⇒ `crates/singlefs-core/src/mount.rs:5512`：「PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(」那一臂读盘。
8. R 的账（树表、分配记录树）与映射树都读回全零 ⇒ `placements_of_an_abandoned_root_whose_account_is_unreadable`：映射现算不成、重读一次仍不成；凭据要现行账里一条代大于 R 的 txg、罩住 R 树表槽的记录，R 的单元在 C 里隔离着、没被分配，没有这条 ⇒ 卸载报 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`，分配器换回去。
9. 对照（第 3 步不注错）：R 那一槽记成有效根、txg 相同，第 7 步借到 C 那一槽，不读账、不拒，卸载做成。

条款：`.claude/kb/decisions/23-journal的角色与格式.md:393`：「被抛弃根的账读不出时，正常卸载、会话写与再可写挂载都该做得成」；D16 的 H2 那一句 `.claude/kb/decisions/16-发布语义.md:39`：「在那一槽记着同一条根（那一槽记的是有效根且 txg 相同）时，照 C 那一圈记着的落点算、不读账、读不出不拒」。条款写的是「那一槽记的是有效根」，打中满足的是 D23 那一句「正常卸载……都该做得成」的分句：同一张盘、同一串用户动作，结局只差重建那一遍一次瞬时读错。

后果的边：卸载被拒之后再可写挂载，R 被抛弃、账读不出、没有凭据，照 B 拒（D23 同一行「拒只在没有凭据时留着」），两臂在下一次挂载上结局相同；分得出的只在这一次卸载与挂着时的准入抬 F（同一个函数）。R 的单元在 C 里一直护着，没有数据丢失，打中的是活性。

四句：
- 分不分辨臂：分辨「C 并读阶段那一份」与「根环表也并」两种做法；不分辨这一批改没改（改之前这一格 R 不进 C，挂载那一刻 I-7.4 就有风险，卸载那一步走同一条读盘路）。
- 被判的系统看不看得到：看得到，读阶段那一份就在 `rebuilt_allocator` 的参数里（`roots_read_by_the_read_stage`），R 的那一条记录也在。
- 满足的是判据的哪一句：D23 第 393 行「正常卸载……都该做得成」；不是 H2 字面那一句（H2 字面只罩「记成有效根」的槽，这一形在它外面）。
- 改法在这一格上中不中：见第六节。

量法：模型用例 `attack_unmount_after_the_fall_back_with_the_root_slot_of_r_read_bad_on_one_read`，第几次读坏从 1 扫到 16（只坏这一次），另一格不注错；每格打 `ATTACK_UNMOUNT_CELL`，带根环表在 R 那一槽记的有效根 txg、是否记成读坏或全零、C 是否在那一槽有落点、H2 借不借得到、卸载结局、R 的单元有没有被写掉、卸载之后池级 checker 的违例。预期（推的）：对照与读坏落在重建那一遍以外的格 `unmount=raised`；落在重建那一遍那一格 `borrow_of_c_for_r=None`、`unmount=refused:…AccountOfAnAbandonedRoot…`。哪一次读是重建那一遍我没写死（入库同目标第 1213–1215 行的计数说读阶段之后从第 2 次起是判退两问、取 txg、取水位、F 生效值与重建那一遍），所以整段扫。

### A2：被抛弃根读阶段读到、重建那一遍读坏一次，影子账不护它（C554 子形）

历史与故障：

1. 起点同 A1 第 1、2 步，那一次可写挂载（M1）不注根槽错，判退到 P、写行与暖机发成，R 被写行那一行抛弃；M1 交回之后断电（不卸载）。断电这一刻是合法的崩溃态（M1 交回之前写都过了屏障）。
2. 再挂 M2：R 的单元这一次读得出。最新那条根是 M1 的，按它的表 R 被抛弃。读阶段读到 R，R 在 `roots_read_by_the_read_stage` 里。
3. 故障：R 的根槽只在 M2 重建读根环那一遍读回全零一次。
4. C 并读阶段那一份，但 `placements_in_the_accounts_of_newer_readable_roots` 先滤掉被抛弃的根（`mount.rs` 第 2223–2227 行）；影子账只拿这一遍：`isolate_slots_referenced_only_by_abandoned_roots` 的根表是 `crates/singlefs-core/src/mount.rs:3164`：「&sorted.roots,」。R 不在这一遍里，它独占的槽不隔离、不计进 `abandoned_roots_unreadable`；根环表那一槽记成全零，不记成被抛弃根，之后抬 F 那一串也没有它的落点。
5. R 的单元在 M1 那一版的账里是空闲的（它们是 P 之后 R 自己写的），M2 的写行、暖机与之后的用户写都可以分到它们。

条款：`.claude/kb/decisions/23-journal的角色与格式.md:393`：「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」。这一形就是欠账 `.claude/kb/checks-owed.md:339`：「可写挂载那一刻被抛弃根的根槽读回全零或读错，根环表不记它，它的单元在现行账里空闲，影子账与 C 那一圈都不护」——不是新机理。新在两处：

- 这一批之后可写挂载手里已经有 R 的那一份（读阶段读到、自证过），却只交给 C；C 又不收被抛弃的根。同一份数据进不进影子账，取决于 R 被不被抛弃——这正是面一问的「待遇不一致」。
- 项目已经为同一件事定过做法：重建那一遍两遍之间读坏的被抛弃根要并回来，`crates/singlefs-core/src/mount.rs:3334`：「只拿重读那一遍时，第一遍读出、重读读坏的被抛弃根掉出影子账」（R1「按槽并两遍」）。读阶段与重建那一遍之间是同一种「两遍之间」，没有并。
- `crates/singlefs-core/src/recovery.rs:2063`：「可写挂载的读阶段拿它择根，读到的全部根留给读阶段之后的几遍（`mount` 的 `rebuilt_allocator` 里 C 并的较新可读根），之后那几遍读坏的根不因此漏掉。」——对被抛弃的根不成立，文档说过头。

四句：分辨「读阶段那一份进不进影子账」两种做法，不分辨这一批改没改（改之前同样掉）；系统看得到（同上）；满足的是 D23 第 393 行主句「不许重新分配」；改法见第六节。

量法：`attack_abandoned_root_read_by_the_read_stage_and_read_bad_on_one_later_read`，第几次读坏从 1 扫到 12，另一格不注错；每格打 `ATTACK_ABANDONED_CELL`：根环表那一槽记没记成被抛弃根、影子账隔离了几槽、`abandoned_roots_unreadable`、M2 自己写掉 R 的单元几份，之后在 M2 的会话上覆盖写至多 8 次，R 的根还在根环里时第一次写掉 R 的单元是第几次。预期（推的）：对照 `slot_of_r_recorded_as_abandoned=true`、`isolated>0`、`first_overwrite_writing_r_while_r_in_the_ring=None`；重建那一遍那一格 `slot_of_r_recorded_as_abandoned=false`、隔离数比对照少；会不会在 8 次之内真写到 R 的单元取决于分配次序，没推出来——不写到也只说明这 8 次没分到，不说明护住了。

### 去重键：没打中

- 键 (实例代号, checkpoint_txg)：同一次挂载里读阶段与重建那一遍之间不写盘，同一槽两次读出的自证根要么同一条、要么一遍读坏；同一 txg 只有一次发布（新实例第一个 txg 大于读阶段择到的根，`first_txg_of_new_instance` 收了它）；冻结之后原样重发的是同一条。去重时这一遍的那一份在前、读阶段的后补，两份内容相同。
- 第 0 代根三槽各一份，并的时候按键去重只留一份；C 只看比所选那一版新的，第 0 代根进不来。
- 漏掉的只有：判退两问里读出（`readable_roots_rereading_every_bad_ring_slot_once`）、读阶段与重建那一遍都没读出的根。判退退到 [见证到的那一版, 所选根) 里最新一条读得出、不被抛弃的根，这样的根读得出就会被退到，退到的就是所选那一版，不在 C 要护的「比所选那一版新」里；读得出而被抛弃的，C 本来不收。读阶段的重读（N-配置）经同一个读缓存，第二遍是第一遍的超集，不丢。
- 试过的形：同一键两份内容不同（找不到一次挂载里造得出的写序）；第 0 代根重复；判退两问独有的根；读阶段两遍之间读坏。

## 二、面二：删掉读阶段之后那次现择

**实例表照读阶段择到的根取，判退那一形：没打中。** 判退本身按所选根 R 的表滤候选（`recovery.rs` 第 8293–8295、8321–8327 行），重建分配器判抛弃也用 R 的表，前后一致。R 在 R 自己的表里有效，于是进 C（A1 那一条路）；写行之后换成新实例的表，`instance_rows_to_write`（`mount.rs` 第 6485 行）给 R 所在的实例写一行（是上一个实例就写到退到的那一版的 txg，是中间实例就写 (i, 0, 0)），R 一定被抛弃，不会留成「谁都不抛弃它」的根。试过：R 与 P 同实例、R 是 P 之后另一个实例的第一条根两种。

**施加过记录的那一形（推的，没量，旧代码同样如此）。** 所选那一版 E 是由记录重建的新实例写行那次发布（D23 已定项 15），E 的表比读阶段择到的 R 的表多一行 (R 的实例, R 的 txg)。R 的实例里 txg 大于 R 的根 X 按 E 的表被抛弃、按 R 的表有效，影子账就不隔离它独占的槽。要分出这一差别，X 得在取新实例那次挂载与这一次读阶段都读不出、在重建那一遍读得出——两次瞬时读错；而改之前的现择同样落到 X 自己的表上（X 是那一刻环里最新的），结局相同。不归这一批，记在「没打中的形状」。

**它的槽在同一次挂载里被写盖：没打中。** 读阶段到重建分配器之间不写盘；之后第一个写根的是写行那次发布，它落在 `target_for_publish(first_txg)`，与 R 同槽只在 first_txg − R 的 txg 恰是 3S 的倍数时（被抛弃时间线上的记录把 first_txg 抬了一整圈），那时实例表早已读完，C 在那一槽的落点也正是在这个进程写掉那一槽时解开，R 离开根环，没有要护的了。

**读次数少一环。** 产品一侧：`instance_table_of_the_newest_root_read_at_most_twice` 的「读一次、读不出调钩子重读一次、仍不出拒」原样留着；删掉的只有「择不到根」那一支，读阶段择不到根已先报 `Recovery(NoValidRoot)`（`mount.rs` 第 9363 行），走不到。D16 已定项 1 里「重读一次」的几处（根环重读、被抛弃根的账、F 生效值那张表）读法没动。测试一侧：按读次数注入故障的入库用例改了 5 个常量（正文第 22 行），我只核到它们改在实现员说的那几个位置；崩溃枚举用例 `random-histories-and-fault-injection-given-states` 按每一步的读数抽注入点，原来落在「读阶段之后再择一次根」那一读上的注入点现在挪到 F 生效值与重建那一遍上，罩的读换了，结局推不出，命令见第八节第 2 条（实现员报告 `m3-close-b20-instance-table-read-stage/result.md` 第 129 行已列，没交）。

## 三、面三：`root_chosen_by_the_read_stage` 不可为空之后的测试断言

量的（读码）：

- 判官与模型映射都不读这个字段：`refusal_reason_on_the_d23_list` 按 `InstanceTableOfTheNewestRootForTheShadowLedger { .. }` 认（`crates/singlefs-verification/src/mount_transparency.rs` 第 2282 行），复算 `instance_table_of_the_newest_root_unreadable_on_the_crash_image` 用 `choose_root`（同文件第 2905 行），`model_comparison.rs` 第 1427、1543 行同样 `{ .. }`。原来给 `None` 的两处（`writable_mount_refusal_judged_against_the_legal_refusal_list.rs` 的 `an_unreadable_instance_table_refusal_on_a_healthy_pool_is_a_refusal_for_the_wrong_reason`、`model_comparison.rs` 第 2019 行）改成真值，断言判的还是「这一拒映射到哪一条」，字段值不进判法。`model_comparison.rs` 那一处填的是 `selected_version`（所选那一版），不是择到的根；字段不进判法，不影响结局，只是名不副实。
- 原来带 `Some(…)` 的四处（`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`、两个 `fall_back_…` 目标、`model_comparison.rs` 第 1920 行）断言的值没变：改之前是重读那一遍现择出来的根，那几张盘上现择与读阶段择到的是同一条。它们因此丢了「重读那一遍择到哪一条」这一判，而那一步已经删了，不算丢。
- 一行注释过期：`crates/singlefs-verification/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:367`：「点名重读那一遍择到的最新根 (3, 10)」——现在点名的是读阶段择到的根，重读只重读那张表。
- 没打中的形：找过断言里拿这个字段与恢复报告的 `effective_root` 比的（施加过记录时两者不同），没有。

## 四、面四：H6 第 0 代出路会不会被别的盘面走进

读码推的，没写用例。出路的门（`crates/singlefs-core/src/system_configuration_repair.rs` 第 274–341 行）：断言 (0, 0)、第 0 代根读得出、没有末条记录；单元区里没有 `birth_txg` > 0 的单元；`crates/singlefs-core/src/system_configuration_repair.rs:326`：「let something_newer_than_the_genesis_is_readable_in_the_rings = !records.is_empty()」且根环里读得出的根最大 txg 恰是 0。过了门写 tail 0、实例代号取自证槽、根环、journal 环与单元写序的最大值。逐形：

| 盘面 | 走不走得进 | 写出来对不对 |
|---|---|---|
| 新格式化池，取号写两轮撕了一轮（H6 本形） | 走进 | 对：取号写的 tail 就是 0（环里没有记录），实例代号取到撕下的那个号 |
| 暖机根或写行根落了 | 不进：根环里有 txg ≥ 1 的根，按第 333–341 行拒 | — |
| 写行那次发布的记录落了、根没落 | 不进：`records` 不空，拒；管理员该断言 (1, 1)，末条读得出走正常那一支 | — |
| 写行那次发布只有单元落了 | 不进：`birth_txg` 闸 | — |
| 只读挂载过 | 只读挂载一个字节不写，盘面与没挂过相同，归上面几行 | — |
| 跨实例：取号连撕几次、每次都在写行之前断 | 走进 | 对：tail 仍是 0，实例代号取各槽里最大的那个，下一次可写挂载再加一 |
| 有真历史的池，较新的根、记录、单元这一刻都瞬时读不出 | 走进（门只看此刻读得出的） | 不降见证：R2 那一形至少有一槽自证过，择系统配置要它；那一槽带着真 tail N 留着不重写，下次挂载 c_见证 取各自证槽的最大值，仍是 N；读错撤掉之后环里的根与记录照常择到。写进去的 F 取整池生效值，同样不降。代价是这时断言第 0 代本身是错的断言，门拦不住——与别的版本「`birth_txg` 闸数不到读不出的单元」是同一种代价，模块文档已写 |

借用的拒绝成员（轻，归检查单「类型与名字」）：「第 0 代之后读得到更新的东西」借的是 `TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion`。这个枚举按管理员下一步分成员（`crates/singlefs-core/src/system_configuration_repair.rs:142`：「换一个断言的版本、先修盘、或接受只读。」），原成员说的是「第一版不支持」（接受只读）；这一形管理员该做的是换断言（断言读得到的那一版），成员名把它报成了「不支持」。

## 五、面五：H3 新旧入口并存

量的（`grep`，不含注释行）：

- 旧两个入口 `recover`、`mount_read_only` 在 `crates/` 下验证包之外 0 个调用方（命中的只有两处定义，`recovery.rs` 第 8365 行、`mounted_read.rs` 第 651 行）；挂载运行时那个 crate 只调可写挂载。验证包里 390 行调用。
- core 里不筛 fsid 择系统配置的产品路径与它们为什么不中：
  - `crates/singlefs-core/src/mount.rs:7711`：「let system_configuration = choose_system_configuration(devices)」，挂着之后收参数与盘表的入口（抬 F、准入、卸载、管理员回退）。不筛是有意的：同一函数文档 `mount.rs` 第 7698 行写着靠它认出换进来的别池盘；而 H3 那一形（一块盘上一槽高世代别池槽）在可写挂载取号写两轮时两槽都被重写成本池的，挂着之后不再有别池槽。
  - `crates/singlefs-core/src/instance_table_sweep.rs:340`：「let system_configuration = choose_system_configuration(reader)」，可写挂载交回之前那一判清扫，在取号之后，同上。
  - `mount.rs` 第 968 行 `of_a_pool_on_disk`：调用方只在测试与实验装置里。`mount.rs` 第 9845 行可写挂载筛不出才不筛再择，后面有参数核，设计如此。
  - `recovery.rs` 里不带单元区起点的 `walk_to_file`、`rebuild_version` 等：文档写明给用例用，产品路径走带起点的那一份。
- 验证侧一处（轻）：`crates/singlefs-verification/src/mount_transparency.rs:2664`：「match (mount_read_only(&refused.devices), &report.outcome) {」所在的观察者函数参数里就有 `filesystem_identifier`，却调不筛的旧入口；它判的是一个池的写流造出的崩溃态，盘上没有别池槽，今天两种入口结局相同。

没打中的形：一块盘一槽别池高世代、可写挂载做成之后再抬 F 或卸载（取号已把那一槽写成本池的）；挂着时换进一块别池盘（照拒，有意）；只读挂载之后再可写挂载（只读不写，可写自己筛）。

## 六、改法（只在我的模型上想过、没实现、被攻过零轮）

| 改法 | 修哪一格 | A1 | A2 | 状态 |
|---|---|---|---|---|
| G1：读阶段择根那一遍连槽一起留（`choose_root_keeping_every_self_verified_root_read` 今天把槽丢了，只交根）；重建那一遍某槽读坏或全零、读阶段在那一槽读出过自证根的，根环表照 T1 甲那一类记（`RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`，带读阶段那一份整条根记录），影子账与回收门槛照「并出来照读出来的算」 | 读阶段与重建那一遍之间的读坏，与 R1 对重建两遍的处置同一个口径 | 推的：H2 借 C 那一臂认这一类里有效的那一半（`allocator.rs` 第 1441–1449 行），卸载借得到 | 推的：被抛弃那一半进影子账、记落点 | 推的，待主 agent 跑（没写补丁） |
| G2：只给影子账加读阶段那一份（`isolate_slots_referenced_only_by_abandoned_roots` 的根表也用并过的那一份） | A2 | 推的：不中（根环表那一槽仍是读坏，H2 照旧借不到） | 推的：中（隔离到；之后抬 F 那一串表里没记着它的落点，照读盘） | 推的，待主 agent 跑 |
| G3：H2 借 C 那一臂放宽到「那一槽记成读坏或全零而 C 在那一槽有条目」 | A1 | 推的：中 | 推的：不中 | 推的；放宽了 H2 的字面，要用户定，我不推荐 |

## 七、检查单八格

1. **走不到的理由二次查实：成立。** 量的：七份补丁 `crates.patch` 里 `crates/singlefs-core/src/` 下新加的行，按 `expect(`、`unreachable!`、`todo!`、`unimplemented!`、`assert`、`panic!`、`_ =>` 数，`repairs-fixes` 3 行、`repairs-fixes-2` 2 行，其余 0；这 5 行都是标识符 `asserted…` 里带 `assert` 的误中，没有一处真的断言或通配臂。`choose_root_keeping_every_self_verified_root_read` 里那个 `expect` 是从 `choose_root` 原样挪过来的（`read-stage-roots` 补丁第 214–219 行只改了接收它的写法）。A1 的拒走的是已有成员 `AccountOfAnAbandonedRoot`，不是新成员。
2. **类型与名字：成立（轻，三处）。** 实例代号 `InstanceGeneration`、txg `CheckpointTxg` 是 newtype；fsid 是 `&[u8; 16]`，与 `choose_system_configuration_of_the_pool` 等存量同型；新加行没有 `_ =>` 与缩写。三处轻：`recovery.rs` 第 2063 行文档说「之后那几遍读坏的根不因此漏掉」，对被抛弃的根不成立（A2）；测试注释第 367 行过期（第三节）；H6 借用的拒绝成员管理员出路不对（第四节）。
3. **不兼容旧代码与旧格式：推不出。** 旧入口留着不是为旧格式或旧盘，是不筛 fsid 的另一个入口；产品 crate 里 0 个调用方（第五节），验证包 390 行调用，多数判的是一个池的崩溃态。用户 H3 定「管理员需要只读」收 fsid 之后，留一个不收 fsid 的公开只读入口算不算「只为兼容旧调用方」，要主 agent 照用户原话判；验证侧至少 `mount_transparency.rs` 第 2664 行那一处手里有 fsid。
4. **不新建判定库：成立。** 七份补丁 core 改动行没有打开库、写 TSV 或 sqlite 的（量的，读 diff 与 `read-stage-roots` 补丁原文）。
5. **所有任务多机可用：成立；正文「新用例都用内存盘」一句不实。** `abandonment_at_a_writable_mount_is_judged_by_the_instance_table_of_the_root_the_read_stage_chose_even_when_its_slot_reads_bad_afterwards` 经 `pool_after_a_recovery_abandoned_the_third_version` 起池：`crates/singlefs-verification/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:301`：「let mut pool = build_pool(tag);」，镜像文件放在 `crates/singlefs-verification/tests/common/mod.rs:57`：「std::env::temp_dir().join(format!(」下，文件名带进程号与序号。每进程自建自用，不读别处的状态，两台都跑得起；只是不是内存盘。
6. **测试优先多线程：成立（轻）。** 判退那三条新用例各在一个测试函数里串行扫两种读坏法 × 15 次读（`recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` 第 926、1020、1092 行），一格一次小盘挂载，耗时推的、不长；拆成独立测试函数或按格并行能让 libtest 并起来，不急。
7. **带 --item 的命令与项：不成立（轻）。** 量的：`m3-close-b20-fallback-i74/result.md` 第 120 行、`m3-close-b20-read-stage-roots/result.md` 第 160 行、`m3-close-b20-instance-table-read-stage/result.md` 第 132 行的变异命令都写 `--item <项>` 占位，没有项名，照抄交不出；`repairs-fixes-3` 第 105 行要从 `notes/replay-items.txt` 粘。项名列不列得出要跑 `--list-items`，推不出。我另做了一个只读静态核：`crates/mutations.tsv` 里文件是被判四份之一的 840 行，锚点在今天的文件里 0 行找不到、0 行命中多处（草稿目录 `anchor-check.txt`，末行 `ANCHOR_CHECK checked=840 missing=0 multiple=0`）。这只说明锚点在，不说明被抓。
8. **新崩溃流多罩了什么：成立（没有新流）；读次数挪动之后那一项结局推不出。** 见第二节末段与第八节第 2 条。

## 八、要主 agent 跑的命令

1. 攻方两条用例的整份复跑（编译与跑都在 `rerun.sh` 里；`--cwd` 不需要，路径全是绝对的）：

   ```bash
   task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b20-batch-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m3-close-b20-batch-code-r1-opus/copy /home/fy5090/.local/share/singlefs/work/m3-close-b20-batch-code-r1-opus/attack-1-run /home/fy5090/code/singlefs/research/prompts/m3-close-b20-batch-code-r1-opus-model
   ```

   在哪起：仓根。跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0` 算跑通（两条用例只打行、不断言结局，编不过或 panic 才非 0）；`task-server/task log <号> --all` 里的 `ATTACK_UNMOUNT_CELL`（17 行）、`ATTACK_ABANDONED_SETUP`（1 行）、`ATTACK_ABANDONED_CELL`（13 行）与末行 `name=rerun_exits cases=0`。判 A1：对照那一行 `unmount=raised`、有一格 `valid_root_recorded_on_the_slot_of_r=None slot_of_r_recorded_bad_or_all_zero=true c_holds_the_slot_of_r=true borrow_of_c_for_r=None` 且 `unmount=refused:…AccountOfAnAbandonedRoot…` 就是打中；没有这样一格就是没打中（照实写回）。判 A2：对照 `slot_of_r_recorded_as_abandoned=true`，有一格 `slot_of_r_recorded_as_abandoned=false` 且 `isolated` 小于对照就是打中。这是副本上的数，不进 kb。
   编不过的话，把 `<运行目录>/out/attack-cases.out` 的 `error` 段交下一条攻方腿修（类型签名我按源码现读写的，没编过）。

2. 读次数挪动之后的那一项（实现员报告已列，正文说 `--item` 那一趟还没交）：

   ```bash
   SINGLEFS_RUN_SCOPE='种类=items; 跑什么=崩溃点重放入口带 --item; 范围=点名的 1 项; 为什么=可写挂载每次少读一整环根槽，按读次数抽的注入点挪到 F 生效值与重建那一遍上' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/crash/crash-point-replay.sh --item random-histories-and-fault-injection-given-states
   ```

   在哪起：仓根。跑完看末行 `TASK_SERVER_RESULT` 的 `exit=` 与日志里那一项的结局行。

## 没打中的形状

- 去重键：同键两份内容不同、第 0 代根三份、判退两问独有的根、读阶段两遍之间读坏（第一节末）。
- 实例表照读阶段择到的根：判退到同实例与跨实例两形；同一次挂载里 R 的槽被写盖（第二节）。施加过记录的新实例写行那一形要两次瞬时读错、旧代码同样，没写用例。
- 测试断言：找过拿字段与 `effective_root` 比的断言，没有（第三节）。
- H6：表里七行盘面（第四节）。
- H3：挂着之后的入口、取号之后的清扫、测试专用入口、可写挂载筛不出再择（第五节）。
- 取样范围：A1 扫「只坏一次」的第 1–16 次读与对照，用户动作只取「挂上就正常卸载」；A2 扫第 1–12 次读与对照，用户动作只取覆盖写至多 8 次。没扫「从第 k 次起一直坏」，没扫挂着时准入抬 F 与管理员回退。

## 这条腿自己的限度

- 两个打中都只到读码与「推的」结局，用例没编、没跑；结论以主 agent 第 1 条命令的读数为准。什么现象推翻 A1：第 1 条里每一格的卸载都 `raised`，或重建那一遍读坏的那一格 `borrow_of_c_for_r` 是 `Some`。推翻 A2：每一格 `slot_of_r_recorded_as_abandoned=true` 或 `isolated` 都等于对照。
- A2 是欠账 C554 已登记机理的子形；A1、A2 都不分辨「这一批改没改」，分辨的是「读阶段那一份只给 C」与「也给根环表、影子账」。
- 盘面借入库用例的造法：R 的单元「这一次挂载读回全零」是读故障，不是崩溃态本身；崩溃点是合法的（R 的根槽之后、轮换之前）。
- 第二节施加过记录那一形、第四节最后一行都只推了，没量。
- 用户动作只放开了「卸载」与「覆盖写」两种，第 3 步「放开扫」没做全；两个打中的说法因此是「有一格中」，不是「每种用户动作都中」。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；副本上的数不算入库装置上的数。
- 没往统一任务服务交任务；用例没编、没跑；判决要引的数没复跑（命令在第八节）。
- 没跑崩溃枚举（全量状态数 0），没跑门禁、没跑 `--list-items`。
- 删了：草稿目录里 rustfmt 语法核用的 `syntax/`（两份文件，几十 KiB，删前没 `du`）。没删：`/home/fy5090/.local/share/singlefs/work/m3-close-b20-batch-code-r1-opus/copy`（冻结副本，第八节第 1 条要用，已在同目录 `.deliverables` 写成「交接」，主 agent 跑完删）；同目录 `rerun-head.sh`、`rerun-steps.sh`、`rerun-tail.sh`、`anchor-check.txt`、`minimal-copy.txt` 是拼 `rerun.sh` 的片段与读数，普通文件。
