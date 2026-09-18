# m2-step45-code-r2 核查报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

挑的引用：Opus 报告第 100 行「`.claude/kb/decisions/23-journal的角色与格式.md:1238`」，抄的原文
「3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。」

现查确认原文确实在 1238 行（`awk 'NR==1238'` 命中该句）。把行号故意加 1 变成 1239，
在草稿目录的副本 `/tmp/claude-1000/m2-step45-code-r2-verifier/selftest/23-copy.md` 上取该行：

```
--- 副本第 1239 行实际内容 ---
4. **切换时的 W 取被重发的那个 checkpoint 里的最大事务号**：……
```

与待核引文（「3. 计数器全池接着走……」）不同，方法判 ✗。**核查方法能分辨对错行号。**

## 一、云端攻方腿（Opus）核对表

`research/prompts/m2-step45-code-r2-opus-output.md`（350 行）。所有代码路径行号现查用
`awk 'NR==A,NR==B{print NR": "$0}' 文件`，kb 引用同法；命令与结果见下。

| 引用 | 核的结果 |
|---|---|
| `mount.rs:661-666`（highest_counter 代码块） | ✓ 逐字节相符 |
| `recovery.rs:726-731`（above 过滤闭包） | ✓ 逐字节相符 |
| `recovery.rs:426-432`（rollback_high_water_of_root） | ✓ 逐字节相符 |
| `.claude/kb/checks-owed.md:303`（C332 引文「或落回 R_old 并施加……已确认的写丢掉」） | ✓ 原文一致 |
| `.claude/kb/decisions/23-journal的角色与格式.md:1238`（「计数器全池接着走……」） | ✓（即判别力自证那条） |
| `mount.rs:673-680`（`prefix_applied: 0`） | ✓ 该字段确在区间内 |
| `.claude/kb/checks-owed.md:311`（C340 第三列「回退之后第一条记录的 jsn = R_old 覆盖的最后一条 + 1（P1）；判别力自证……」） | ✓ 逐字节相符 |
| `mount.rs:556-561`（mount_writable 的 next_counter） | ✓ 逐字节相符 |
| `mount.rs:235-250`（S2 代码块，含 240/243 行内引用） | ✓ 逐字节相符（Opus 只引到 239 行，属该区间内的合法截取） |
| `recovery.rs:399-409`（allocation_records_under_root，不看 level） | ✓ 确认无 `IndexNodeHeader.level` 检查 |
| `walk.rs:298`（TREE_KIND_EXTENT \| ALLOCATION \| ACCOUNTING if node.level > 0） | ✓ 逐字节相符 |
| `allocator.rs:193`（assert! 「跨度越过单元区末尾」） | ✓ 逐字节相符 |
| `allocator.rs:190-202`（DeviceFreeMap::isolate，不动 free_slots） | ✓ 函数体确认只动 isolated 相关字段 |
| `allocator.rs:150-155`（is_free） | ✓ 逐字节相符 |
| `recovery.rs:481-486`（rebuild_version 内 read_node 闭包） | ✓ 确认在 `rebuild_version`（439 行起）函数体内 |
| `.claude/kb/decisions/28-挂载期承诺量.md:30`（上界公式引文） | ✓ 但为截断引用：原句还有「、只住内存、按设备算……被抛弃的根被轮转覆写时清零」未抄，Opus 在「算出」处断句，属摘句但未改变已抄部分的含义 |
| `.claude/kb/checks-owed.md:296`（C318，仅作引用指向，非直接引原文） | ✓ 行号对应 C318 条目 |
| `.claude/kb/decisions/23-journal的角色与格式.md:1209`（窄读法引文） | ✓ 逐字节相符 |
| `.claude/kb/decisions/16-发布语义.md:375-376`（生效／回退候选集两行表格） | ✓ 逐字节相符 |
| `recovery.rs:351-371`（effective_rollback_floor 整个函数体） | ✓ 逐字节相符 |
| `recovery.rs:357-370`（「一块盘上一条根都没有就不算它」的出处） | **✗ 位置有误**：该注释实际在 348-349 行（函数头文档注释），357-370 是函数体代码（highest_per_device 循环与 min），不含这句注释文字。Opus 把「函数头的注」与「357-370」并列写在同一括注里，读者会以为引文就在该区间；实为「误标区间，原文实在 348-349 行」 |
| `mount.rs:627-638`（`target.checkpoint_txg < effective_floor` 检查） | ✓ 逐字节相符 |
| `mount.rs:460`（`rollback_floor: start.effective_floor`） | ✓ 逐字节相符 |
| `mount.rs:633-648`（两道候选检查：F 下界 + 实例表行判定） | ✓ 逐字节相符，确为两道独立检查 |
| `walk.rs:786-799`（checker 候选集按最新根自己的 F 收） | ✓ 逐字节相符 |
| `.claude/kb/invariants.md:107`（I-2.1「2026-09-17 起『被引用』按……」） | ✓ 逐字节相符 |
| `.claude/kb/milestone/02-second-txn.md:189`（「F 只在一块盘上时……是不是条款的意思」） | ✓ 逐字节相符 |
| 测试文件注释「隔离的槽数（独占 34 + 两边都引用的 mkfs 实例表 2）」（未给行号） | ✓ 找到于 `second_transaction_step_four_rollback.rs:190`，内容相符 |

**计数**：核了 27 处，✓ 26 处（含 1 处截断引用但未失真，计入 ✓），✗ 1 处（recovery.rs:357-370 与实际注释位置 348-349 不符）。

### Opus 复跑命令（在草稿目录副本上跑）

```
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ /tmp/claude-1000/m2-step45-code-r2-verifier/opus-rerun/repo/
cp research/prompts/m2-step45-code-r2-opus-model/opus_attack_m2_step45_r2.rs 副本/crates/singlefs-harness/tests/opus_r2_attack.rs
CARGO_TARGET_DIR=副本外的 target 目录 cargo test -p singlefs-harness --test opus_r2_attack -- --nocapture --test-threads 1
```

sha256（与 Opus 报告第 20-21 行逐字节比对）：

```
7a30c64686bffa4b250617759a3dbad0869a105713c1b8b7ab4a30a53d635d22  opus_attack_m2_step45_r2.rs（509 行，报告写 509 行）✓
34d9aaacb1612dc380ef6ba1540ebed651d1ee84c8032750a063d82ef2d6c5ae  p1-contrast.patch.txt（9 行，报告写 9 行）✓
```

8 条用例复跑原样末行（逐条与 Opus 报告里抄的对比，全部逐字节相符）：

```
test x1_a_after_p2_a_recovery_that_falls_back_to_r_old_replays_the_whole_abandoned_run ... X1-A 五个故障之后：所选根 Some((1, 6))、施加 3 条、读回 根 (InstanceGeneration(1), CheckpointTxg(3))、2600 字节、= 第四版 true
ok
test x2_a_corrupting_an_abandoned_roots_tree_table_makes_every_writable_mount_fail ... X2-A 可写挂载失败：Recovery(UnitUnreadable { slot: SlotNumber(50326) })
X2-A 只读恢复："根 (InstanceGeneration(3), CheckpointTxg(10))、内容 3000 字节"
ok
test x2_b_isolated_slots_that_r_old_also_referenced_are_counted_as_free_after_reclaim ... X2-B 盘 DeviceIdentity(0)：记账空闲 211908、位图上真能发的 211872、隔离 36、差 36
X2-B 盘 DeviceIdentity(1)：记账空闲 211908、位图上真能发的 211872、隔离 36、差 36
ok
test x3_b_control_damaging_every_root_on_device_one_without_any_reuse ... X3-B 对照（无复用、6 个故障）：新实例的根写 F = 11、违例 [("I-3.1", Violated("盘 0：记账的已分配 Some(1048576)，遍历全部有效根得到 819200"))]
ok
test x3_b_damaging_only_the_carrier_is_worse_than_damaging_every_root_on_that_device ... X3-B 只坏载体（1 个故障）：新实例的根写 F = 0、违例 ["I-2.1", "I-3.1", "I-5.1"]
X3-B 盘 1 全部根都坏（6 个故障）：新实例的根写 F = 11、违例 ["I-3.1"]
ok
test x3_c_rolling_back_onto_a_root_whose_units_were_legally_reused ... X3-C 退到 (0, 0)：Recovery(UnitUnreadable { slot: SlotNumber(50178) })
X3-C 退到 (1, 3)：成功，新实例 InstanceGeneration(4)
ok
test x3_d_rolling_back_to_a_root_whose_data_unit_was_already_reused ... X3-D E 落 50178、下一版落 50180
X3-D 翻字节之前的违例：[]
X3-D 退到 A 被拒：Recovery(UnitUnreadable { slot: SlotNumber(50180) })
ok
test x3_losing_the_only_floor_carrier_after_a_legal_reuse_writes_the_floor_back_to_zero_and_the_checker_goes_red ... X3-A 一个字节之后：新实例的根写 F = 0；违例 ["I-2.1", "I-3.1", "I-5.1"]
  I-2.1 Violated("树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上")
  I-3.1 Violated("盘 0：记账的已分配 Some(1474560)，遍历全部有效根得到 1425408")
  I-5.1 Violated("盘 0：树表单元（槽 50178 跨 1）与 数据单元（槽 50178）重叠")

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.82s
```

**P1 对照**（Opus 报告第 15-16、80-83 行给的补丁与结论）：在副本上按 `p1-contrast.patch.txt` 把
`next_counter` 换成 `own_record.counter + 1`，只跑 `x1_a_after_p2_a_recovery_that_falls_back_to_r_old_replays_the_whole_abandoned_run`：

```
X1-A 回退：新实例 InstanceGeneration(2)、D 的 (txg, jsn) = (7, 4)、暖机 [(8, 5)]
X1-A 环里的记录 (实例, jsn, txg)：[(1, 1, 1), (1, 2, 2), (1, 3, 3), (1, 6, 6), (2, 4, 7), (2, 5, 8)]
X1-A 零故障恢复：根 (InstanceGeneration(2), CheckpointTxg(8))、3000 字节、= 第一版 true
X1-A 五个故障之后：所选根 Some((1, 3))、施加 0 条、读回 根 (InstanceGeneration(1), CheckpointTxg(3))、3000 字节、= 第四版 false
thread '...' panicked ... assertion `left == right` failed: jsn 4、5、6 三条全施加
test result: FAILED. 0 passed; 1 failed
```

与 Opus 报告第 81-83 行「`所选根 Some((1, 3))、施加 0 条、读回 …3000 字节、= 第四版 false`」逐字节相符。
（该用例断言是照 P2 写的，P1 补丁下这条断言本身失败属预期——Opus 报告已说明「跑完改回去」，不构成矛盾。）

改回 P2 后复跑 `diff -q` 与原仓 `mount.rs` 逐字节一致；再跑
`second_transaction_step_four_rollback` + `second_transaction_step_five_reuse` 各 5 条全过，
与 Opus 报告第 23-25 行「副本里既有的……也全过」相符。

**计数（Opus 复跑）**：核了 3 组命令（8 用例主命令、P1 对照补丁、还原后 5+5 用例），✓ 3 组、✗ 0 组。


## 二、云端辩方腿（Sonnet）核对表

`research/prompts/m2-step45-code-r2-sonnet-output.md`（178 行）。

| 引用 | 核的结果 |
|---|---|
| `mount.rs:608`（choose_root 调用） | ✓ 逐字节相符 |
| `recovery.rs:287`（choose_root 定义） | ✓ 逐字节相符（doc 注释在 285-286，函数签名在 287） |
| `mount.rs:601-716`（mount_rollback 函数边界） | ✓ 601 行为 `pub fn mount_rollback`，716 行为闭合 `}` |
| `mount.rs:624-625`（`instance_table_of_root` 调用） | ✓ 逐字节相符 |
| `grep -n "InstanceTableMalformed" crates/singlefs-core/src/mount.rs` 「命中两处（625、670）」 | **✗** 实际命中 **5 处**（34、334、555、625、671），非「两处」；且具体行号是 **671**，不是 670（670 行是 `InstanceTableRecords::parse(...)` 那半句，`.ok_or(MountError::InstanceTableMalformed)?` 在下一行 671） |
| `recovery.rs:426-432`（rollback_high_water_of_root，同 Opus 已核） | ✓ 逐字节相符 |
| `mount.rs:408-519`（establish_instance 函数边界） | ✓ 408 行 `fn establish_instance`，519 行 `}` |
| `mount.rs:420`（`acquire_instance` 调用） | ✓ 逐字节相符 |
| `mount.rs:447-463`（row_publish = publish_version(...)） | ✓ 逐字节相符 |
| `mount.rs:694-699`（previous_row 构造，`instance: target.instance`） | ✓ 逐字节相符 |
| `mount.rs:429-435`（row 引用 `start.previous_row.instance.0`） | ✓ 逐字节相符 |
| `mount.rs:428`（`for row_instance in first_row_instance..instance.0`） | ✓ 逐字节相符 |
| `crash.rs:470-483`（check_records） | ✓ 逐字节相符，判定逻辑 `in_place(publish.root) && !publish.records.iter().any(...)` 在 481 行 |
| `crash.rs:427-428`（root_without_record 文档注释） | ✓ 引文前半逐字节相符（截去末尾括注「（E77…）」，未改变已抄部分含义） |
| `grep -rn "合法覆盖" crates/` 「no output」 | ✓ 复跑同一命令确认零命中 |
| `.claude/kb/milestone/02-second-txn.md` 步 4「现状」段含「记录核对器把……分开」 | ✓ 该句确在同一段落里、紧随「暖机一次」之后（中间隔两句，「紧跟」为宽松但不失实的描述） |
| `mount.rs:597`（doc 注释仍写 P1） | ✓ 逐字节相符 |
| `mount.rs:658-660`（行内注释写 P2 及理由） | ✓ 逐字节相符 |
| `second_transaction_step_four_rollback.rs:1-2`（模块文档注释「jsn……= 4」） | ✓ 逐字节相符；另确认同文件 111 行也有一处同样的旧描述（P1），406 行断言取值为 9（P2），Sonnet 只点名了模块级文档注释，未提 111 行，不算错误 |
| `.claude/kb/decisions/23-journal的角色与格式.md:691`（C143/CJ2 定案叙述） | ✓ 逐字节相符 |
| `mount.rs:147-165`（first_txg_of_new_instance，同结构取环内最大值） | ✓ 逐字节相符 |
| `.claude/kb/checks-owed.md:146`（C143 定案条目） | ✓ 行号对应 C143 条目本身；但「此前的做法是类似 P1 的『从 R_old 自己的值往下接』」一句为 Sonnet 自己的转述归纳，非该行直接引文（该条目正文讲的是 inode 号水位，CJ2 公式在别处），不算行号错误，但引文与转述界限写得不够清楚 |
| `.claude/kb/decisions/16-发布语义.md:376`（「回退候选集｜按实例表判仍然有效 ∧ txg ≥ F_生效」） | ✓ 逐字节相符 |
| `.claude/kb/invariants.md:120`（I-3.1「2026-09-17 起『有效根』=……」） | ✓ 内容逐字节相符，唯引号符号从原文「」被转写成『』（语义未变，标点差异） |
| `.claude/kb/decisions/16-发布语义.md:362`（「只数改过用户可见状态的根……」） | ✓ 逐字节相符 |
| `mount.rs:196-252`（rebuilt_allocator 函数边界） | ✓ 196 行 `fn rebuilt_allocator`，252 行 `}` |
| `.claude/kb/decisions/16-发布语义.md:371`（「可再分配｜已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」） | ✓ 逐字节相符 |
| `mount.rs:719-740`（reclaim_floor_tests 模块边界） | ✓ 719 行 `mod reclaim_floor_tests {`，740 行 `}` |
| `grep -rn "NoPublishedVersion" crates/singlefs-core/` 「命中 mount.rs:32、110、550、657、944」 | **✗** 实际只命中 **4 处**（32、110、550、657），全仓（含整个 crate）没有第 944 行这个匹配；944 是虚构行号 |

**计数**：核了 28 处，✗ 2 处（`InstanceTableMalformed` 计数与行号错误、`NoPublishedVersion` 行号 944 虚构），
✓ 26 处（含 2 处截断引用/标点差异但不失真、1 处引文与转述边界不清但行号本身对，均计入 ✓）。


### Sonnet 变异表复跑（第 33、45-49 行，副本 `/tmp/claude-1000/m2-step45-code-r2-verifier/sonnet-rerun/repo`）

复跑方式：用一个小脚本按 `crates/mutations.tsv` 对应行的「原文/替换文」栏做定点替换（命中 1 次才写，
与 `replace-once.py` 同义），跑该行第五栏指定的测试命令，记原样末行；再还原、重新跑一次确认转绿。
`CARGO_TARGET_DIR` 指向草稿目录外的 target，不复用原仓构建产物。

| 变异表行 | 变异名（与 Sonnet 报告表格比对） | RED 原样末行 | GREEN 原样末行 |
|---|---|---|---|
| 33 | 步 4：回退之后 jsn 接 R_old 那条之后（C340 的 P1） | `panicked ... assertion \`left == right\` failed: D 的 jsn 接在 C 的 8 之后 / left: 4, right: 9` / `FAILED. 0 passed; 1 failed` | `test ...isolation ... ok` / `test result: ok. 1 passed; 0 failed` |
| 45 | 步 4：普通重开不隔离被抛弃根引用的槽 | `assertion \`left == right\` failed: 按 D 那一版实例表判被抛弃的根……普通重开照样隔离 / left: [...0...], right: [...36...]` / `FAILED. 0 passed; 1 failed` | `test ...isolation ... ok` / `test result: ok. 1 passed; 0 failed` |
| 46 | 步 4：回退候选集的 F 用最新根自己带的 F | `F_生效 是 0，txg 9 的根仍在候选集里: RollbackTargetNotACandidate {...}` / `FAILED. 0 passed; 1 failed` | `test roots_below_a_floor_carried... ok` / `test result: ok. 1 passed; 0 failed` |
| 47 | 步 5：回收门槛不看环里最旧有效根 | `assertion \`left == right\` failed / left: CheckpointTxg(0), right: CheckpointTxg(25)` / `FAILED. 0 passed; 1 failed` | `test ...reclaim_floor_takes... ok` / `test result: ok. 1 passed; 0 failed` |
| 48 | 步 4：影子账把被抛弃根账里已释放的落点也隔离 | `assertion failed: device.is_free(SlotNumber(50180)) && device.is_free(SlotNumber(50181))` / `FAILED. 0 passed; 1 failed` | `test raising_the_floor_to_the_first... ok` / `test result: ok. 1 passed; 0 failed` |
| 49 | 步 5：重建分配器时不认第 0 版树表单元 | `assertion \`left == right\` failed / left: None, right: Some(Placement { slot: SlotNumber(50178), span: 1 })` / `FAILED. 0 passed; 1 failed` | `test rebuild_from_records_remembers... ok` / `test result: ok. 1 passed; 0 failed` |

六条全部复现「改坏 → RED，还原 → GREEN」，与 Sonnet 报告第 78-85 行表格逐条相符（变异名、点名测试、
RED→GREEN 结论一致）。全部还原之后 `diff -q` 副本的 `mount.rs`、`allocator.rs` 与原仓逐字节一致（无输出）；
合并跑 `-p singlefs-core --lib` + `--test second_transaction_step_four_rollback` +
`--test second_transaction_step_five_reuse`：

```
test result: ok. 11 passed; 0 failed ...（singlefs-core --lib）
test result: ok. 5 passed; 0 failed ...（second_transaction_step_five_reuse）
test result: ok. 5 passed; 0 failed ...（second_transaction_step_four_rollback）
```

与 Sonnet 报告第 87-96 行「11+5+5，全部 21 条测试通过」逐字节相符。

**计数（Sonnet 变异复跑）**：核了 6 条变异 × 2（RED+GREEN）= 12 次跑，✓ 12 次、✗ 0 次；另加还原后 diff -q 与合并跑
共 2 项，✓ 2 项、✗ 0 项。


## 三、本地攻方腿（样本 s1、s3）与转述核对表

`research/prompts/m2-step45-code-r2-local-attack-output-s1.md`、`-s3.md`
（对应提示 `m2-step45-code-r2-local-attack.md`；两份都是 4-bit 模型输出，未见复读/拼接/落单标记）。
转述核对表 `research/prompts/m2-step45-code-r2-local-attack-translation-audit.md`（20 行，6 条数据行）核对结果：

| 转述核对表条目 | 原文文件:行 | 核的结果 |
|---|---|---|
| G1（reclaim_floor 定义与门槛依据） | `mount.rs:167-168` | ✓ 逐字节相符 |
| G2（abandoned_by_table 判据） | `mount.rs:177-178` | ✓ 逐字节相符 |
| G3-1（effective_rollback_floor 取最大再取最小） | `recovery.rs:348-349` | ✓ 逐字节相符（含「一块盘上一条根都没有就不算它」半句） |
| G3-2（choose_root 择新序） | `recovery.rs:285` | ✓ 逐字节相符 |
| D2（分配记录改写不删除） | `decisions/03-空间分配.md:175-248` | ✓ 该区间为「已定项 7」整节（175 到已定项 8 起始 249 之前），引文「落点释放时条目不删……」确在 180 行 |
| D3（影子账窄读法） | `decisions/23-journal的角色与格式.md:1206-1244` | ✓ 区间覆盖「已定项 14」整节，引文「查账的集合……仍被有效根引用的槽不在其内」确在 1209 行内 |

Facts 与源码/条款是否一致：以上 6 条转述与源码/kb 逐字节比对均一致，未见丢限定词。
表中记录「首稿漏了『一块盘上一条根都没有就不算它』半句、定稿用 replace-once.py 补上英文对应表述」；
现读 `local-attack.md` 提示正文第 15 行，确认逐字节含定稿声称补上的英文句
「a device with no readable root assigned to it at all is excluded from what follows, not treated as if it read zero」
与「zero if there are no readable roots anywhere in the ring at all」，与该条转述记录一致（首稿缺、定稿补上均属实）。

## 四、本地辩方腿（样本 s3）与转述核对表

`research/prompts/m2-step45-code-r2-local-defense-output-s3.md`（对应提示 `m2-step45-code-r2-local-defense.md`，
唯一未被字词损坏闸拒绝的样本）。转述核对表 `research/prompts/m2-step45-code-r2-local-defense-translation-audit.md`
（25 行，19 条数据行）核对结果：

| 转述核对表条目（节选，逐条现查） | 原文文件:行 | 核的结果 |
|---|---|---|
| Candidate set definition | `decisions/23...md:1206`；`decisions/16-发布语义.md:358` | **部分**：1206 行确为「已定项 14」标题（区间起点，合理）；`16-发布语义.md:358` 是「### 已定项 1」**标题行**，候选集定义具体内容（「回退候选集｜按实例表判仍然有效 ∧ txg ≥ F_生效」）实在 **376 行**，而非 358 |
| F_effective 定义（per-device max 再取 min） | `decisions/16-发布语义.md:358`（标注「"生效"那一行」） | **✗ 位置有误**：358 是节标题，「生效」那一行实在 **375 行** |
| 影子账机制 | `mount.rs:186-252`；`allocator.rs:185-202` | ✓ 逐字节相符，含「保守读法……交 alloc-basis 那一轮定」原句 |
| D23 主句 / 窄读法定案 | `decisions/23...md:1206` | ✓ 区间起点正确，内容在其后（1209/1211 行）区间内 |
| 保守读法与窄读法字面不一致 | `milestone/02-second-txn.md:158-186` | ✓ 158 行为「## 步 4」标题、186 行为该节末尾空行，区间正确覆盖含引文的整段 |
| P1/P2 定义 | `mount.rs:597`（P1）、`:658`（P2） | ✓ 逐字节相符 |
| C340 | `checks-owed.md:311` | ✓ 逐字节相符（与 Opus 报告引用同一行，已核） |
| mount_rollback 文档与函数体不一致 | `mount.rs:597` 对照 `:658` | ✓ 逐字节相符 |
| shadow ledger 每次挂载算 | `mount.rs:188` | ✓ 逐字节相符 |
| F_生效 候选集判据 | `decisions/16-发布语义.md:358` | 同上「部分」：标题行而非具体行（376） |
| reclaim_floor 两处 oldest_valid_root 来源 | `mount.rs:221-233`；`:357-366` 与 `:333` | ✓ 逐字节相符（重建走 choose_root 得的 newest_table，抬 F 走 :333 解析的 current 自己的表） |
| rebuild_from_records 认第 0 版树表 | `allocator.rs:353-356`、`:376-389`、`:725` | ✓ 逐字节相符，`:725` 单测名确认 |

**说明**：`decisions/16-发布语义.md:358` 被三处条目重复引用为具体行号，实际都是「### 已定项 1」的**节标题**，
具体内容分别在 375（生效）、376（回退候选集）；这与判别力自证要拦的错误同类（行号偏离实际内容所在行），
但因为整节只有一个标题起点、内容确在标题之后不远，属**宽松的区间型引用**而非指错行——
是否接受「引节标题代表整节」这种写法不是本报告要判的推理问题，只据实指出三处具体内容并不在 358 行本身。

**计数（本地两条腿）**：核了 18 处转述条目，✓ 15 处，「部分」（区间型引用、内容不在标题行本身）3 处，✗ 0 处。


## 总计

| 部分 | 核了 | ✓ | ✗ / 部分 | 核不动 |
|---|---|---|---|---|
| 一、Opus 报告引用 | 27 | 26 | 1 ✗（recovery.rs:357-370 位置误标，实在 348-349） | 0 |
| 一、Opus 复跑 | 3 组命令 | 3 | 0 | 0 |
| 二、Sonnet 报告引用 | 28 | 26 | 2 ✗（InstanceTableMalformed 计数/行号；NoPublishedVersion 行号 944 虚构） | 0 |
| 二、Sonnet 变异复跑 | 6 变异 ×2 + 2 项 | 14 | 0 | 0 |
| 三、本地攻方转述核对 | 6 | 6 | 0 | 0 |
| 四、本地辩方转述核对 | 12 | 9 | 3 部分（decisions/16-发布语义.md:358 被当具体行号引，实为节标题，内容在 375/376） | 0 |
| **合计** | **90** | **84** | **3 ✗ + 3 部分** | **0** |

（90 = 27+3+28+14+6+12；84 = 26+3+26+14+6+9；3 ✗ = 1（Opus 表）+2（Sonnet 表）；3 部分 = 本地辩方转述核对表里
`decisions/16-发布语义.md:358` 那三处区间型引用。Sonnet 表里 `checks-owed.md:311`（C340）与 `recovery.rs:426-432`
两条引用恰与 Opus 表核过的同一处重合，两张表各自独立计数，此处不作交叉相减，按两表分别报的数汇总。）

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 没有跑门禁 54 号全量层 0 崩溃点重放、QEMU（55 号）、herd7（57 号）——两条腿报告都已注明未跑，
  本报告同样未复核这些装置本身（属「核不动：要虚机/要更长时间」一类，但因两条腿本身都已如实声明未跑，
  没有出现「声称跑过」需要拆穿的情形，故不单列「核不动」条目）。
- 没有对本地攻方/辩方全部样本（含被字词损坏闸拒绝的 void 样本）逐一核对损坏检测结果本身，
  只按主 agent 给定的两份「已判干净」样本（攻方 s1、s3，辩方 s3）核了转述内容。
- 没有核 Opus 报告「没打中的形状」一节（第 307-317 行）里列的按代码推、未构造用例的几种猜想——
  那些本身声明「按代码推，没跑」，不构成需要复跑的产物声明。
- 没有对 Sonnet 报告「没做什么」一节自陈的欠账逐条复核（例如未跑遍全部新增变异、未核 layout/01-first-txn.md 全文），
  这些是 Sonnet 自己申报的限度，不是待核的引用或产物声明。
- 没有判断「决策点是否真的够不着 / 够得着」这类语义分类本身对不对（例如 Sonnet 认为「影子账读法」
  不该算决策点、该算「代码违反已定条款」）——这是推理判断，不在核查范围。
- 没有修改任何 `crates/` 或 `.claude/kb/` 下的文件；两处副本（`opus-rerun/repo`、`sonnet-rerun/repo`）
  的临时改动均已在跑完对应用例后还原并用 `diff -q` 核对与原仓逐字节一致。
