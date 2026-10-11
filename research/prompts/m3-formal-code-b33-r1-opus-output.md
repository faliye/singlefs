# 第 32–33 批代码轮第一轮：云端攻方（Opus）报告（2026-10-06）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 F1 F2 B3 B4 R1 R2 R3 R4 RV U11 M35 M36 M42 M45 M46 M47 G5 -->

立场：找反例。只读冻结副本 `/tmp/claude-1000/m3-formal-code-b33-r1/tree/`，所有改动、编译、运行都在草稿目录 `/tmp/claude-1000/m3-formal-code-b33-r1-opus/` 下拷出的副本里，副本上的数不是入库装置上的数。重型测试一条没跑（checker 档测试、GPU、变异整表、装置内联单测都没跑）；跑的是 harness 档测试与攻方自己的原型 crate。

## 复跑

```
bash research/prompts/m3-formal-code-b33-r1-opus-model/tools/rerun.sh <工作目录> [x fix p3 p2 c551]
```

每一段从冻结副本拷一棵新树、用新的编译目录（不复用旧产物）；`FROZEN`、`CAP`（默认 10）、`MEMORY`（默认 8G）可改。输出写 `<工作目录>/out/`。我这一趟的原样输出在模型目录 `out/`（26 个文件）。

| 文件 | sha256 |
|---|---|
| `src/proto/Cargo.toml` | 0ef9765acac3459934ee685d24f4173d6a33ca2bbef464b3fce5e853e657dee7 |
| `src/proto/src/main.rs` | fc5292c04203570da679327db1648f6347433f25be3d6c9e1e418e21e346107e |
| `src/x_records_applied_versions.rs` | b284ff071bf3f35f96305e6a90ab5737f5dd1bcc6bb82ade3512d91902ef99ee |
| `tools/install_proto.py` | 251d342370d8ce874b7b1190ec65e0f551c273620f9d420f5e52a5c1d6163d0e |
| `tools/p3_mutants.py` | e32c82fdc215b275d015de03c5375fb2554bde01b51c4983a7e66b9f5d4212e8 |
| `tools/p3_run_batch.sh` | bc28fb9b54222130047df44673ef2ae7a373ace76b69b889466b874bfed8349d |
| `tools/patch_builder.py` | 048c26141ae7d6d9c81437a5d121e45284d1bff0cc2ce4d66f85b9f6a445bafe |
| `tools/patch_c551_probe.py` | 8fddbfcc6de49558067a9758bfea7cd9ed9eeffd534689d1ae79bb6babdc0285 |
| `tools/patch_fix_rv.py` | f39992547e5199d2a1d08c8b4724b66e3e60e42186ff3aab06ace6becccc798b |
| `tools/rerun.sh` | 8c3d9ecdbe438d7eb1829130a259b46ec63aebd745431a376e0a57a0803d66a8 |

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P2-a 由记录施加出来的版本、下一次挂载先施加的那一版 | **打中**（副本量过） | 这两类版本整版不进 I-5.4 按版本那一层：同一种「两条记录罩住同一个槽」放在最新根上 I-5.4 红，放在由记录施加的那一版上 0 违例；放在下一次挂载先施加的那一版上 checker 0 违例、可写挂载以「同一块盘上两条分配记录罩住同一个槽」拒。F1、33-c 都碰不到这几格；攻方改法 RV 在副本上让它们改红或报部分没判，副本上 104 + 89 条 harness 用例全过（零轮） |
| P2-b 第二节末那一处（树表指针全零的根） | **打中，比已记的更宽**（副本量过） | 已记那一形复现（只列 txg 3、不列 txg 4）；另外最新那条根的树表指针整条写零、重封之后，checker 0 违例、I-7.2 成立，core 可写挂载报 `UnitUnreadable { slot: 0 }` 拒。checker 把这种根当「没有树表」，core 把它当指向槽 0 的指针 |
| P2-c F2 与 core 是不是同一组取值、同一判法 | 取值相同；**判的指针不同**（副本量过） | 三段的取值、偏移与 core 的 `judged` 一致；根记录里的树表 / 实例表 / 映射根指针，core 的根记录读者只判 extent 偏移，checker 从第 32 批起判这三段：算法类型写 1 的根上 checker 红 I-2.4（与 I-8.10），core 照样择它、挂载做成 |
| P2-d GPU `head_ok` 与 CPU | 没打中（读码，推的） | `head_ok` = MAC/nonce 全 0 ∧ 偏移 28、29 为 0 ∧ 偏移 30 两字节为 0，与 CPU 三段加 MAC/nonce 同；调用点五处与 CPU 五处一一对上 |
| P2-e F1、33-c 在合法历史上多报 | 没打中（一次抽样） | 树表 0 条一族 12 段历史（含写行、暖机、零单元发布、冷启动、系统配置槽读回全零的挂载、多实例），层 0 全量 210 147 个崩溃状态，I-5.4 部分没判 0 次、别的不变量违例 0 次；带第一个文件版本的 6 段每段约 1.68 × 10⁷ 个状态，超过一段 10⁶ 的上限，没跑 |
| P3-a 812 用例每版跑 checker 罩不罩得住第 31 批点名的回收错法 | 没打中，R4 照判决预期不罩 | 门槛 +1、−1、R3、B3 两条 812 用例都红，d 只红每一版跑 checker 的那一条（第 1 次覆盖写，txg 4）；R4 两条都绿（判决已知，`root_ring_turn_in_one_mount` 红在第 108 行） |
| P3-b R4、d 两行点名的用例是不是基线就红 | 没打中 | 基线 3 条全过；R4、d 下 `turning_the_root_ring_…` 红在第 108 行 |
| P3-c U11 那一行红在哪 | harness 档上红在 I-3.1（副本量过）；E171 U11 本身推的 | 同一处改法在 812 用例里红在 txg 26 的 I-3.1，不碰 core 断言；E171 装置那条是 checker 档、没跑 |
| P3-d 四支豁免 | 没打中（读码） | 每支只认一条理由串，判不了的几种（树表读不出等）都报别的理由串、照红；豁免只放「不适用」，不放「违例」 |
| P3-e 第 31 批没点名的回收错法 | 补一格（副本量过） | 「复用时不删被新记录罩住的已回收记录」（Z1-d，变异表第 105 行）两条 812 用例都绿：那条会话不出跨度变化的复用，每版跑的 checker 对这一形没有对象；它由已有的再挂用例抓 |
| P4 33-a、33-b | 没打中（读码；C551 那一格副本量过） | 812 两条用例末尾与每一版上 I-3.1 都是成立，不在「不适用」里；`Some(1 + 3)` 与 G5 同一个理由键；33-b 新造法下界只能从盘 0 的系统配置来，别的拒因是 `VersionWithoutFile`、分得开 |
| P4-f 旧第 38 行改名 | 补一格（读码，推的） | E171 两份 117 行表第 33 行按旧名找，在今天的树上取快照会走「按名字找不到 ⇒ 补位」；已冻结的几棵树不受影响 |
| P5 E156 门槛读法与 core | 没打中（读码，推的） | 钉的 txg（落到这一槽、小于新实例第一个 txg 的最小 txg）、三种解开时点与 core 是同一个谓词；core 这一判另有不依赖 E156 的 harness 用例与 10c 九行变异；M36、M42 点名的都是没有全零钉住的格，不互相掩护（M36 在前洞格上可能少一块判别力，推的） |
| P6 变异行 | 没打中（静态核过） | 副本上 `crates-mutation-rows.py static-check` 只报 1 行（litmus 文件不在冻结副本里，不是这一轮的行）；我按名字挑出的这一轮 25 行，点名的测试函数各在 1 个文件里 |
| P1 B3、B4 | 没打中（读码，推的） | 135 是 core 的扇出、单测先断言它；B4 的 6 个取样点含 135，任何「按位置」的错都落在目标或现行取 135 的 11 种组合里；只按片数触发的错罩不到（推的） |

## P2　F1、33-c 与 F2

### P2-a 由记录施加出来的版本、下一次挂载先施加的那一版整版不进 I-5.4（打中，副本量过）

**被判对象里许可它的那几句。** I-5.4 按版本那一层只走候选根：`crates/singlefs-checker/src/walk.rs:5082`：「for index in candidate_indexes.iter().copied() {」；候选下标只来自根环：`crates/singlefs-checker/src/walk.rs:10287`：「let walked = *index == newest_index || (!abandoned && !below_floor);」。同一个函数里由记录施加出来的那几版早就算好了（`crates/singlefs-checker/src/walk.rs:10332`：「let versions_applied_only_by_records = versions_applied_only_by_records(」），下一次挂载先施加的那一版的树表指针也算好了、只给 I-3.10 用（`crates/singlefs-checker/src/walk.rs:5308`：「tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);」），调 I-5.4 时没有传进去（`crates/singlefs-checker/src/walk.rs:10487`：「judge_allocation_records_disjoint(」后面只跟 `&candidate_indexes`）。

**历史**（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 已有的搭建 `image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`，我只给它的结构多带回那一版的 `TransactionOutput`）：mkfs → 取号 → 暖机 → A → B（实例 1、txg 4）→ 实例 1 再发一版 (1, 5)，只落单元与 journal 记录（根槽、系统配置槽一个没落）→ 可写挂载择 (1, 4)、施加那条记录、写行 txg 6、暖机 txg 7。故障只在最后一步：改的是 (1, 5) 那一版这次发布自己重写的一片分配记录叶（槽 50269，两盘同槽），把一对同盘相邻记录的前一条跨度加一，沿引用链补校验和（叶 → 内部节点 → 根 → 树表 → 中央映射树根），再把 (1, 5) 那条 journal 记录（计数器 5）的新根段与点名项里变了的校验和改成新的、重封，不动别的记录。用户动作不在这段里（不是装置写死的后缀）。

| 格 | 镜像 | checker 判红的 | I-5.4 | 可写挂载 |
|---|---|---|---|---|
| a3 | 挂载之后，(1, 5) 由记录施加出来 | 无 | 成立 | 没挂 |
| c3（对照） | 同一种改法改在最新那一版（txg 7）的叶上 | I-5.4 | 违例 | 没挂 |
| b3 | 挂载之前，(1, 5) 是下一次挂载先施加的那一版 | 无 | 成立 | 拒：`AllocationRecordOutsideThePoolGeometry` |
| a1 | 挂载之后，(1, 5) 的树表两份清零 | I-2.1、I-3.1、I-4.8、I-7.4 | 成立 | 没挂 |
| a2 | 挂载之后，(1, 5) 的分配记录树根两份清零 | I-2.1、I-4.8、I-7.4 | 成立 | 没挂 |

原样输出（`research/prompts/m3-formal-code-b33-r1-opus-model/out/x-records-applied-versions.log`）：

`research/prompts/m3-formal-code-b33-r1-opus-model/out/x-records-applied-versions.log:21`：「B33X label=a3-after-mount-record-version-overlapping-records violated=[] i54=Holds」

`research/prompts/m3-formal-code-b33-r1-opus-model/out/x-records-applied-versions.log:31`：「B33X label=b3-before-mount-next-version-overlapping-records violated=[] i54=Holds」

`research/prompts/m3-formal-code-b33-r1-opus-model/out/x-records-applied-versions.log:32`：「B33X mount label=b3-before-mount-next-version-overlapping-records outcome=err error=Recovery(AllocationRecordOutsideThePoolGeometry { what: "同一块盘上两条分配记录罩住同一个槽" })」

`research/prompts/m3-formal-code-b33-r1-opus-model/out/x-records-applied-versions.log:23`：「B33X label=c3-after-mount-newest-version-overlapping-records violated=["I-5.4"]」

a1、a2 是 F1 的同形：同样的「这一版树表 / 分配记录树读不出」放在候选根上，今天报部分没判（`checker_known_bad_images.rs` 第 32、33 批那几条用例）；放在由记录施加出来的那一版上照报成立。b3 是 I-5.4 立条时要拦的那件事本身（「下一次可写挂载重建分配器时」出事，今天 core 改成了拒、不再 panic）：池级 checker 在这张镜像上一条都不红。

**打中之后的四句。**

1. 分不分辨臂：F1、33-c 之前与之后两种判法一起不中（它们只改候选根那一圈里读不出的报法），病根是两者共用的前提「按版本那一层只走 `candidate_indexes`」——打中不分辨臂，这个前提要另立一笔账。
2. 系统当时看不看得到：看得到。`check_pool_image` 在调 I-5.4 之前已经认出了这两类版本、拿到了它们的新根段（上面三处引文），走读、I-3.10、I-5.5、删除态扫描都在用。
3. 满足的是判据字面的哪一个分句：I-5.4 那一行写「任何一条有效根的分配记录树里」、实现状态写「按回退候选集里每条根判」（`.claude/kb/invariants.md:198`：「任何一条有效根的分配记录树里，同一块盘上任意两条分配记录」）。由记录施加出来的那一版字面上不是「根」，所以按 I-5.4 的字面这可能在射程外；同一份文件里 I-5.5 写明含它们（`.claude/kb/invariants.md:199`：「走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）」），checker 自己的注释也把它们叫回退候选集的补充。我报的是格 P2 的问句「坏盘上还有没有同形的版本整版不进按版本那一层」——有，两类；射程归不归 I-5.4 由主 agent 判。
4. 跑前条款给的改法在这几格上中不中：F1、33-c 两支只认 `candidate_indexes` 里的根，在 a1、a2、a3、b3 四格上都不起作用（今天的树就是 F1 + 33-c 合入后的，四格照绿）。

**攻方改法 RV（只在我的模型上量过、被攻过零轮）**：`judge_allocation_records_disjoint` 多收一张「由记录造出来的版本」表——由记录施加出来的那几版，加下一次挂载先施加的那一版——两类各取新根段里的树表指针与「树表 0 条那一版的分配记录树根」，进同一个按版本循环（补丁 `tools/patch_fix_rv.py`，10 处定点替换，不改别的判法）。

| 格 | 今天 | RV 之后 | 标 |
|---|---|---|---|
| a1 | 成立 | 部分没判（树表读不出，点名「由记录施加出来的那一版（实例 1、txg 5）」） | 量过 |
| a2 | 成立 | 部分没判（带树一版的分配记录树走不到叶） | 量过 |
| a3 | 0 违例 | 只红 I-5.4 | 量过 |
| b3 | 0 违例 | 只红 I-5.4 | 量过 |
| b2（挂载之前把 (1, 5) 的分配记录树根清零） | 成立 | 部分没判；这一形 core 恢复时点名单元验不过、不施加那条记录（挂载择 txg 4、施加 0 条），RV 照抄了已有函数「点名单元验证不复刻」的读法，多报一句部分没判（不红） | 量过 |
| `checker_known_bad_images` 全部 104 条（含 F1、33-c、b33x 两条） | — | 全过 | 量过 |
| 另 7 份 harness 测试（删除态、回退区间与新根段、窄判与被抛弃根、洞与一圈窗口、影子账整环等，89 条） | — | 全过 | 量过 |
| GPU 判器同一处 | — | 要同改，没做 | 推的 |

原样：`research/prompts/m3-formal-code-b33-r1-opus-model/out/fix-rv-x.log:19`：「B33X label=a3-after-mount-record-version-overlapping-records violated=["I-5.4"]」；`research/prompts/m3-formal-code-b33-r1-opus-model/out/fix-rv-x.log:30`：「B33X label=b3-before-mount-next-version-overlapping-records violated=["I-5.4"]」；`research/prompts/m3-formal-code-b33-r1-opus-model/out/fix-rv-checker-known-bad-images.log:111`：「test result: ok. 104 passed; 0 failed」。

推翻条件：RV 之外有别的判法在 a3 或 b3 那张镜像上判红（今天的 checker 两张都 0 违例，见上），或者主 agent 判这两类版本按 D16 / I-5.4 的射程就不该被问「分配记录互不相交」——那时这一格降成「射程外，不是缺口」。

### P2-b 第二节末那一处：树表指针全零的根（打中，比已记的更宽，副本量过）

搭建用 33-c 那条用例自己的 `image_after_the_first_file_on_the_version_whose_rows_were_written`（mkfs → 挂载两次、第二次写行 → 同一进程发第一个文件版本；写行那两版 txg 3、4 树表 0 条、根记录直接持有分配记录树，txg 5 是文件版本）。根槽改法用同一份文件的 `rewrite_the_root_slot_of`（改字节、重封整槽、抄进同代记录新根段）。

| 格 | 改法 | checker 判红的 | I-5.4 | 可写挂载 |
|---|---|---|---|---|
| y1 | txg 4 那条根的树表指针整条写零 | 无 | 成立 | 做成，择 txg 5 |
| y2 | y1 再把根记录那一项指着的分配记录树根两份清零 | I-2.1、I-3.1、I-4.8、I-7.4 | 部分没判，只点名 txg 3、**不点名 txg 4** | 做成 |
| y3（对照） | 只清分配记录树根 | 同上四条 | 部分没判，点名 txg 3、txg 4 | 没挂 |
| y4 | 最新那条根（txg 5）的树表指针整条写零 | **无** | 成立 | **拒：`Recovery(UnitUnreadable { slot: SlotNumber(0) })`** |

`research/prompts/m3-formal-code-b33-r1-opus-model/out/y-all-zero-tree-table-pointer.log:22`：「B33X label=y4-newest-txg5-tree-table-pointer-all-zero violated=[] i54=Holds」

`research/prompts/m3-formal-code-b33-r1-opus-model/out/y-all-zero-tree-table-pointer.log:23`：「B33X mount label=y4-newest-txg5-tree-table-pointer-all-zero outcome=err error=Recovery(UnitUnreadable { slot: SlotNumber(0) })」

y2 就是正文第二节已记那一形（checker 只在 `crates/singlefs-checker/src/walk.rs:5092`：「if !tree_table_pointer.all_zero {」那一支里认「树表 0 条」，指针全零的根根本不进那一支），复现了。y4 是更宽的一形：checker 把根记录里全零的树表指针读成「没有树表」（`.claude/kb/invariants.md:315`：「树表指针全零的根没有树表，不算没判到」；删除态扫描同一读法，`crates/singlefs-checker/src/walk.rs:8460`：「树表指针全零、树表 0 条的那一版没有这几棵树，不判。」），最新根可完整遍历这一条也因此无物可走、报成立；core 的根记录读者不把全零当特例，挂载按它去读槽 0。第一版的写者没有一处会写出树表指针全零的根（`crates/singlefs-core/src/` 里 `tree_table: NodePointer::empty_root()` 只出现在 `root_record.rs` 与 `mount.rs` 的单测里，grep 推的），I-2.5 列出的「今天写出全零指针的有四处」里也没有根记录的树表指针（`.claude/kb/invariants.md:134`：「今天写出全零指针的有四处」，列的是树表里三条树的条目与 mkfs、暖机根记录里的中央映射树根指针）——这样的根只可能是写者的错。

四句：① 不分辨臂（F1、33-c 前后都不碰这一形）；② 看得到（指针字节就在根记录里）；③ 判据字面：没有一条不变量说「根记录的树表指针非零」，I-7.2 写的是「能走完全部权威态」，全零指针下没有可走的东西，字面不红——这是「判别子在、判据没写」，照 evidence-discipline 那张表要改判据、改完是新一轮；④ 跑前改法：33-c 报告「要你定的」那一问只问 y2 那一形该不该报部分没判，答哪一边都碰不到 y4（y4 上 I-5.4 走的是根记录那棵树、走得到叶）。可能的改法（推的，没实现）：checker 加一格「每条自证过的根的树表指针非全零」，或 core 的根记录读者把全零树表指针当自证不过。

### P2-c F2 判的三段与 core 读路径（取值相同、判的指针不同，副本量过）

取值：checker 三段都要求 0（`crates/singlefs-checker/src/image.rs` 的 `judge_pointer_encryption_and_compression_fields_take_the_first_version_values`），core 的 `PointerHeadAsRead::judged` 依次判 MAC/nonce、算法类型、压缩算法码、压后长度都为 0，偏移 28 / 29 / 30 两边相同——同一组取值。

判的指针不同：core 的根记录读者四条指针只判 extent 偏移（`crates/singlefs-core/src/root_record.rs:146`：「let tree_table = NodePointer::read_judging_the_extent_offset_from(&mut reader).ok()?;」，同函数里实例表、映射根、分配记录树根三条同写法；代码自己记着 `crates/singlefs-core/src/pointer.rs:239`：「只判 extent 偏移（C581）：今天只剩根记录（`root_record.rs`）走这一判，加密与压缩那几段在它上面还没换成判」），而 journal 新根段里同样四条指针 core 判全（`crates/singlefs-core/src/journal.rs:324`：「NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(」）；checker 从第 32 批起在走读读码 2 节点与实例表链时判这三段（`crates/singlefs-checker/src/walk.rs:1305`：「judge_pointer_encryption_and_compression_fields_take_the_first_version_values(」），根记录里那几条指针也走这两处。

量的：发布 B（txg 4）那条根的树表 / 实例表 / 映射根指针的算法类型写 1、重封根槽：checker 红 I-2.4 与 I-8.10（I-8.10 是我的造法带出来的：`JournalRecord::to_bytes` 不带这几段，抄进记录的指针仍是 0；照字节抄过去的话 core 的 journal 读者又会拒那条记录），core 照样择 txg 4、可写挂载做成。`research/prompts/m3-formal-code-b33-r1-opus-model/out/f2-root-record-pointers.log:12`：「B33X label=f1-tree-table-pointer violated=["I-2.4", "I-8.10"] i54=Holds」；`research/prompts/m3-formal-code-b33-r1-opus-model/out/f2-root-record-pointers.log:15`：「B33X mount label=f1-tree-table-pointer outcome=ok chosen_txg=4 effective_txg=4 records_applied=0」。

这不是 F2 在合法历史上误红（core 不会写出这种根），是 core 的根记录读者比 kb 写的规则松：I-2.4 那一行对指针头部 MAC/nonce 写的是「加密关着时恒 0、读者遇到非 0 一律判该结构损坏」（`.claude/kb/invariants.md:133`：「加密关着时恒 0、读者遇到非 0 一律判该结构损坏」），根记录读者对 MAC/nonce 与这三段都不判。checker 一侧这一格照 F2 判是对的；分歧点交主 agent：根记录的四条指针要不要在 core 里换成判全（与 journal 新根段同一判法），还是在 I-2.4 里写明根记录这几条照「各判各的」处置。

### P2-d GPU `head_ok` 与 CPU（没打中，读码，推的）

`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:513`：「return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;」，等于 CPU 的 MAC/nonce 一判加三段；走读内核里 `w_judge_pointer_head` 的五个调用点（码 2 节点读、实例表链、extent 数据指针、inode 内部条目子指针、容器树子指针）与 CPU 的五处一一对上（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1321`：「w_judge(INVARIANT_I_2_4, p.head_ok);」）。全零指针两边都在判之前返回（GPU `zero_pointer` 把 `head_ok` 置真）。GPU 对拍没跑。

### P2-e F1、33-c 在合法历史上会不会多报（没打中，一次抽样）

原型 `src/proto`（crate `opus-b33-proto`，二进制 `b33_treeless_crash_states`，名字不带 layer0）：mkfs 之后只走树表 0 条那一族——可写挂载 M、零单元发布 E、冷启动恢复 C、每盘最新系统配置槽读回全零的可写挂载 Z（经 harness 的 `execute_history_with` 执行，录制流留内容）；整条流按层 0 的枚举域取崩溃状态（`enumerate_layer0_selecting_versions_observing_each_state`，每一段都展开），每个状态在观察者里再跑一遍池级 checker，按 I-5.4 的判定与说明分类。用户动作那几步（挂几次、两次挂载之间发几次零单元发布、冷启动插在哪）按种子放开抽，只固定起点（mkfs）。

取样与上限：每段历史开跑前用 `layer0_state_count_with_torn_in_place_overwrites`（全量展开）算状态数，超过 10⁶ 的那段不跑，合计不越过 10⁷。写死的 12 段里带第一个文件版本 F 的 6 段每段约 1.68 × 10⁷（`MMF` 16 781 387），全部没跑；跑了 6 段写死的与 6 段按种子抽的（种子 0–3、4–5，长度 3–8），合计 210 147 个状态，三批挂钟约 2、4.3、7.5 分钟（第一批 37 413 个状态 124 秒，按每秒约 300 个状态估后两批，后两批跑过了前台时限、被工具转到后台跑完）。

| 批 | 历史 | 状态数 | I-5.4 成立 | 不适用 | 部分没判（F1 / 33-c / 别的） | 别的不变量违例的状态 |
|---|---|---|---|---|---|---|
| 写死 | MM、MME、MMM、MMEME、MZM、MCME | 37 413 | 12 579 | 24 834 | 0 / 0 / 0 | 0 |
| 种子 0–3 | MCZM、MEE、MMZM、MEZEM | 90 545 | 78 059 | 12 486 | 0 / 0 / 0 | 0 |
| 种子 4–5 | MCCCCMCM、MEZMME | 82 189 | 73 899 | 8 290 | 0 / 0 / 0 | 0 |

`research/prompts/m3-formal-code-b33-r1-opus-model/out/p2-random-0.log` 末行原样：「B33P total states=90545 full_states_sum=90545 i54_holds=78059 i54_not_applicable=12486 i54_violated=0 i54_partially_f1=0 i54_partially_33c=0 i54_partially_other=0 states_with_other_invariants_violated=0 skipped={}」（`research/prompts/m3-formal-code-b33-r1-opus-model/out/p2-random-0.log:502`）。

成立的 164 537 个状态里都有「树表 0 条、根记录直接持有分配记录树」的版本在候选集里（33-c 那一支的对象），一次都没走不到叶。这只说这 12 段、这一族；带文件的版本、回退、抬 F 之后的崩溃状态没进（状态数超上限），按每步之后的整份镜像（不是崩溃状态）也没另取——这一格只算一次抽样的「没打中」。

## P3　32-2

### P3-a、P3-b、P3-e 812 两条用例与 `root_ring_turn_in_one_mount` 在回收错法下（副本量过）

每种错法各拷一份 `crates/`、各用自己的编译目录，施加之后跑 `overwrite_in_one_instance` 的两条 812 用例（`allocated_statistic_equals…`：每一版跑 checker；`repeated_overwrites_go_past…`：末尾一版跑 checker）与 `root_ring_turn_in_one_mount` 三条（`tools/p3_run_batch.sh`，四批，每批 2–4 件并行、每件 2–3 线程，日志 `out/p3/`）。

| 错法 | 来源 | 每一版 checker（812） | 末尾 checker（812） | `root_ring_turn_in_one_mount` | 812 那一侧最早的红 |
|---|---|---|---|---|---|
| 基线 | — | 过 | 过 | 3 条全过 | — |
| R3 释放代少记一 | 变异表第 2788 行 | 红 | 红 | 2 条红 | 第 1 次覆盖写 txg 4，I-3.9 |
| B3 根环公式每区少一槽 | 第 2789 行 | 红 | 红 | 1 条红 | 第 18 次 txg 21，I-7.17 |
| R4 取落点之前记根 | 第 2790 行 | **过** | **过** | 红在第 108 行 | —（判决已知：只在发布中途的崩溃态上红） |
| d 环转之前提前回收释放代 3 | 第 2791 行 | 红 | 过 | 红在第 108 行 | 第 1 次 txg 4，I-2.1、I-3.1 |
| 门槛 +1 | 攻方 | 红 | 红 | 红在第 90 行 | 第 22 次 txg 25，I-3.1 |
| 门槛 −1 | 攻方 | 红 | 红 | 红在第 108 行 | 第 23 次 txg 26，I-3.1 |
| U11 那一处（记根时不回收） | 第 2792 行 | 红 | 红 | 红在第 108 行 | 第 23 次 txg 26，I-3.1 |
| 复用不换分配代 | 攻方 | 红 | 红 | 2 条红 | 第 24 次 txg 27，I-3.10 |
| 只在盘 0 上回收 | 攻方 | 红 | 红（第 24 次覆盖写取落点被拒 `UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported`，两盘落点分叉） | 2 条红 | 第 23 次 txg 26，I-3.1（盘 1） |
| Z1-d 复用时不删被罩住的已回收记录 | 第 105 行 | **过** | **过** | 红在第 271 行（再挂那一条） | — |

结论：第 31 批判决点名的几种（门槛 ±1、R3、B3、d）812 每一版跑 checker 都罩住；R4 照判决预期罩不住、由 `root_ring_turn_in_one_mount` 罩。R4、d 两行点名的用例基线 3 条全过、在这两种错法下红在第 108 行，不是基线就红。另补一格：Z1-d 那一形两条 812 用例都绿——那条会话里复用的落点跨度不变（数据单元 2 槽、节点 1 槽各回各的），每一版跑的 checker 对 I-5.4 那一形没有对象。它不在第 31 批点名里，已有再挂用例抓到，不算缺口；只说明「每一版跑全部不变量」罩的是那条会话走得到的形。

### P3-c U11 那一行

同一处改法（第 2792 行，记根时回收换成空，不碰断言）在 harness 档上红在 I-3.1（txg 26，「记账的已分配 Some(6078464)，遍历全部有效根得到 6062080」，`out/p3/p3-row2792.log`），没有一处红在 core 断言。E171 装置 U11 那条是 checker 档，没跑；推的：同一个会话形状下它也先在 txg 26 的 I-3.1 上红。

### P3-d 四支豁免（没打中，读码）

`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:284`：「InvariantVerdict::NotApplicable(reason) if *reason == "这个镜像上没有它判的对象"」——这是「这一条一次都没评估、也没给理由」的缺省串（`crates/singlefs-checker/src/image.rs:358`：「.unwrap_or("这个镜像上没有它判的对象"),」）。I-5.5 判不了的几格都经 `part_of_the_range_not_judged` 报自己的理由串，压过缺省串，这一支照红；I-4.7、I-4.6 两支的串在 checker 里各只出现一次，删除态扫描读不全的十种缺口各报自己的串。四支都只放「不适用」：任何一支被判成违例、成立或部分没判，`matches!` / `assert_eq!` 照红。能放过去的只有「checker 自己少判一格而不给理由」那一类，那是 checker 的变异该罩的，不是这条用例的。

## P4　33-a、33-b

- **C551 推翻条件**（副本量过，`tools/patch_c551_probe.py` 只加打印）：越过 812 条的末尾那一版 83 条不变量里 68 条成立、15 条不适用、0 条部分没判（`research/prompts/m3-formal-code-b33-r1-opus-model/out/c551-probe.log:53`：「B33C551 end_of_812 holds=68 total=83」）；不适用的 15 条全是「没有对象」一类（I-7.9、I-8.8、I-3.12、I-4.1、I-5.5、I-4.4、I-4.6、I-4.7、I-9.17–I-9.23）。每一版跑 checker 的那条 30 版里 I-3.1 一次都没落进「不是成立」（`grep -c 'I-3.1:'` → 0）。所以「同一条用例判全部不变量」按字面做到了，I-3.1 在内（`.claude/kb/checks-owed.md:425`：「`build_pool` 装上根环表之后，同一条用例判全部不变量。」）。没打中。
- **① `Some(1 + 3)`**（读码）：第一次崩溃那一态与它途中三个二次崩溃记的是同一个理由键（`crates/singlefs-checker-tier/src/crash_injection.rs:2733`：「"崩溃态注入过读故障（挂载透明只管不带故障的状态）";」），用例另钉 `second_crash_points == 3` 与「二次崩溃一个都没判」，三个变异各红在不同的数上。没打中；用例没跑（checker 档）。
- **② 33-b 新造法**（读码）：下界只可能来自盘 0 的系统配置（根上的 F 都是 0），目标 (1, 2) 是树表 0 条的暖机根；F_生效 算错时报的是 `VersionWithoutFile`，与 `BelowEffectiveFloor` 分得开（`crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs:471`：「"txg 2 低于 F 生效值 3：不是候选：{:?}",」）。「拒在任何写之前」从比盘面快照改成比录制流步数（`crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs:477`：「"拒在任何写之前：录制流不多一步"」）：盘上的写都经录制器，两者在这个装置上等价（推的）。没打中；用例没跑。
- **旧第 38 行改名**（读码，推的）：E171 两份坏实现表第 33 行按旧名「步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效」找（`scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/bad-implementations-117.tsv:33`：「步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效」）。驱动从所给快照根的 `crates/mutations.tsv` 按名字取行，找不到就走登记 5.2 的补位、换同文件组下一行。已冻结的几棵 E171 树里还是旧行，不受影响；之后从今天的树取快照重跑 E171，第 33 行跑的就是补位那一行，`replay.sh` 的 exact 比对会在那一行上不同。

## P5　E156 装置

读码，推的，没跑装置与内联单测（checker 档）。

- **同一个谓词。** 钉的 txg：core 取落到这一槽、小于新实例第一个 txg 的最小 txg（`crates/singlefs-core/src/mount.rs:1803`：「for txg in 0..first_txg.0.min(ring_length_in_txgs) {」），装置锚点模型取槽号本身（`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1216`：「pins = (0..ring_length.min(txg))」，锚点模型的槽号就是 txg mod 3S，最小 txg 等于它），旁路读数取同一个最小 txg 并进门槛（`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1881`：「.union(&all_zero_slots_still_pinned)」）。解开：这个进程写到那一槽（装置 `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1861`：「let released_by_write = occupant.is_some_and(|occupant_txg| occupant_txg >= first_txg);」），读出一条比第一个 txg 旧的根（装置 `:1862` 那一行），读出被抛弃的根（core 拿掉那一项、装置的有效根集合不收它）；重读读回全零照旧钉着（`crates/singlefs-core/src/allocator.rs:2027`：「| RereadOfARootRingSlotBadAtMount::AllZero => {}」）。core 只在钉着的槽成了最小那一项时才重读、装置每个观测点都读，差别只在读故障会变的盘上才看得见，这一族不造读故障（实验页「它答不了的」③ 已写）。
- **会不会一起错。** 两边共用的是「钉在可能住过的最旧 txg」这个读法本身；它来自用户 2026-10-04、10-05 的两次定案（`records/2026-09-30-里程碑三开工第一轮.md:624`：「② 读回全零的槽在回收门槛里照自证不过钉住，只在准入抬 F 的上限里不并」），不是装置自己的推导。core 这一判另有不经 E156 的 harness 用例与 10c 九行变异（变异表第 2577–2585 行，挂载时、进程内、抬 F 三处各有一行），装置与 core 一起偏、而这九行一起漏，要三处同时错，没找到这样的形。没打中。
- **PC-钉门槛与 PC-全零钉。** 一个证装置的判定对门槛偏移看得见，一个证 core 少钉全零槽时装置看得见，要分的不是同一件事，没打中。只补一句：PC-全零钉的补丁（`research/results/e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.patch`）在源头把读回全零的槽集合换成空集，挂载那一刻的回收门槛与根环表两处一起去掉；这一族的前洞只在第一圈，挂载那一刻环里最旧有效根本来就比洞旧，挂载那一刻那一处钉不钉都一样，34 个红点都在进程内那一处（txg h + 3S − 1）。挂载那一刻那一处的判别力在 10c 第 2577 行与它点名的 harness 用例上，不在 E156 上（推的）。
- **改调用点不改函数体与 M36、M42。** 两行点名的都是没有全零钉住的格（M36 点名后洞用例，M42 点名无洞用例），并集进不进函数体不影响它们的点名用例。推的另一面：M36 的替换式是「最大有效根 + 1 − 根数」，只在 txg 集合有缺口时与正确门槛不同；并进钉住的 txg 之后，前洞那一格的缺口被钉住的 txg 补上，前洞格里原本因为这个缺口而偏的观测点在 M36 下可能不再偏——是前洞格上判别力少了一块（推的，没量），不是两行互相掩护。

## P6　变异行

副本上 `python3 scripts/mutation/crates-mutation-rows.py static-check .` 末行原样：「CHECKED	2817	298	9」，只报一行问题：第 1352 行 litmus 文件不在冻结副本里（副本只有 `crates/` 与两份 Cargo 文件，不是这一轮的行）（`research/prompts/m3-formal-code-b33-r1-opus-model/out/mutation-rows-static-check.log:2`：「文件 litmus/publish-returns-after-the-rotation-barrier.litmus 不存在」）。按名字挑出的这一轮 25 行（第 2745、2746、2774–2791、2793–2795、2821、2822 行），点名的测试函数各在 1 个文件里。原文相同的几组：F1 三行（第 2777–2779 行）同一处改法分别点名 B、A、第 0 代三条用例；R3 两行（第 2788、2793 行）、B3 两行（第 2789、2794 行）同一处改法分别点名两条 812 用例——P3 表里这几种改法下各自点名的用例都红，没有一组是靠同一条用例一起错。没打中。

## P1　32-1 的 B3、B4

读码，推的。B3 先断言 core 的扇出就是 135（`crates/singlefs-harness/src/model_comparison.rs:1427`：「const LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS: usize = 135;」），扇出一变这条先红；胶水只有一处把叶序映到角色（第 80 行），没有反方向的映射。B4 取 {1, 2, 3, 4, 5, 135}（`crates/singlefs-harness/src/model.rs:4331`：「const CONTAINER_COUNTS: [u64; 6] = [1, 2, 3, 4, 5, 135];」）：任何按位置的错（只取回或只丢第 j 片）都落在目标或现行取 135 的 11 种组合里，分配代取错一版由 1000 + k 与 2000 + k 分开。罩不到的只有「只在片数等于某个 6–134 之间的值时才出错」那一类，模型里没有按片数分支的写法，推的、没量。B4 不判最左那片（`InodeLeaf`）与 inode 根，这两样不在 B4 的射程里。

## 没打中的形状

- F1、33-c 在合法历史上多报：树表 0 条一族 12 段历史、210 147 个崩溃状态，0 次部分没判（P2-e）。带文件版本的那几段每段约 1.68 × 10⁷ 个状态，没跑。
- F2 与 core 的取值、偏移；GPU `head_ok` 与 CPU 的判法与调用点（P2-c、P2-d；GPU 那一半读码）。
- 812 两条用例在门槛 ±1、R3、B3、d、U11 那一处、复用不换分配代、只在盘 0 回收这几种回收错法下（P3 表，11 种含基线）；R4、d 两行点名的用例基线不红。
- 四支豁免各只认一种理由串（读码）。
- C551 推翻条件（副本量过）；`Some(1 + 3)` 的口径、33-b 新造法的拒因分得开（读码）。
- E156 第 6 次门槛读法与 core `AllZeroAtMount` 的谓词、三种解开时点（读码）。
- 这一轮的变异行锚点（静态核过）与原文相同的几组（P3 表里各自红）。
- B3 的 135 上界与 B4 的 36 种组合对「按位置」的错（读码）。

## 这条腿自己的限度

- 一次抽样；「没打中」各格照三方规则只算这一次。
- 打中的两格（P2-a、P2-b）都在坏盘上：a3、b3、y4 是写者或盘面出错才会有的镜像，合法历史上我没造出它们。a3 / b3 这种坏实现（一次发布写出互相罩住的分配记录）若真出现，层 0 在那次发布根槽落盘之后的状态上照样由 I-5.4 抓到、b3 那一态的恢复也会拒，所以它们说的是「池级 checker 对这一张镜像判不出」，不是「层 0 整体放过这个坏实现」。
- P2-a 第 3 句的射程判断（由记录造出来的版本算不算 I-5.4 的「有效根」）我没有定论，交主 agent。
- 改法 RV 只在副本上量过：104 条坏镜像与 7 份 harness 测试 89 条；层 0、GPU、变异整表都没跑，GPU 判器的同一处没改。被攻过零轮。
- 副本上跑测时整机负载很高（load average 70–116），几段跑过了前台时限、被工具转到后台跑完（不是我起的后台）；结局都等到了、输出都在。
- `run-with-memory-cap.sh` 两次按理由放行：上限 8G 超过峰值表里那条命令的 2 倍（派发给的显式上限）、同一会话并行（正推腿同时在跑）。第二台试了一次：它的 cargo 注册表里没有 `vstd`，`--offline` 编不过（基线那一件退 101、编译一行没成），之后没再用第二台，全部在本机跑。

## 没做什么

- 重型测试一条没跑：checker 档测试（33-a ①、33-b ②、E171 U11、E156 内联单测 19 条与 M35 / M44–M47）、GPU 对拍与 F2 的 GPU 变异、变异整表。涉及它们的结论都写了「推的」。
- 带第一个文件版本、回退、抬 F 之后的历史没有按层 0 取崩溃状态（每段超过 10⁶）；没有按「每步之后的镜像」另取。
- B4 的「只按片数触发」那一类错、M36 在前洞格上的判别力，都没量。
- P2-c 的分歧（根记录四条指针 core 只判 extent 偏移）没有提改法去量。
- 没判正推那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。

## 草稿与清理

草稿目录 `/tmp/claude-1000/m3-formal-code-b33-r1-opus/` 里的仓副本（`base`、`w1`–`w4`，含 `w3/p3w/` 下 11 份变异副本与它们的编译目录）与三个编译目录（`t-w1`、`t-w2`、`t-w4`）交回前都删了；只留日志（已拷进模型目录 `out/`）。第二台那一趟的树由 `multi-host-run.sh` 第 ⑦ 步删了，取回目录 `/tmp/singlefs-multi-host-run/b33opus-p3-b1` 我已删。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-formal-code-b33-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b33-r1/tree --root . --background research/prompts/_m3-formal-code-b33-r1-background.md` 末行见下一行（交回前现跑）。

「✓ 核了 49 处引文，对上 49 处，没判 0 处（逐处列在上面）」
