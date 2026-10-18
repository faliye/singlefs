# m3-b44-code-r1 正推腿报告（Sonnet，2026-10-15）

立场：正推。从已定条款与冻结副本里的代码推出「应该是什么」，与被判改动逐格比。冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r1/repo/`（下称冻结副本），代码行号都按它。
量法在我自己的副本里做（拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`），变异按冻结副本 `crates/mutations.tsv` 的行改、跑点名的测试、还原；模型目录 `research/prompts/m3-b44-code-r1-sonnet-model/`。

## 〇、前提核对

- 冻结副本与快照：`sha256sum -c --quiet research/prompts/m3-b44-code-r1-snapshot/crates-sha256.txt`（在冻结副本里跑）无任何失败行输出，即全对上。
- 被判改动里哪些文件只动注释：`comment-only-change.py /home/fy5090/code/singlefs HEAD <diff 里 24 份 core 与 format 文件>` 打出 4 份：`admission.rs`、`block_device.rs`、`journal.rs`、`make_filesystem.rs`。其余 20 份有代码词元的改动（含测试），K7 逐处看。
- 基线：冻结副本上 `cargo test -p singlefs-core --lib` 269 条全过；`-p singlefs-harness` 的三个目标（`newer_unreadable_root_passes_only_when_both_copies_of_one_pointer_are_newer` 8 条、`system_configuration_slot_with_a_foreign_node_size_is_not_chosen` 4 条、`opening_a_container_past_the_largest_number_returns_an_error` 1 条）全过。原样行在 `research/prompts/m3-b44-code-r1-sonnet-model/out/summary.txt` 前四行。
- 开跑前 `ps` 看到别的会话在跑 `cargo test -p singlefs-checker-tier`（变异复跑）与 `cargo test -p singlefs-harness`，没有 qemu、fio、vm-bench、e152；我的每条命令经 `nice -n 19` 与内存包装，机器 loadavg 一度 140 以上，所以有几条在包装排队里被我的外层 `timeout` 截断（见「没做什么」里的一条教训）。

## 一、K1 `mount.rs` 那一判换成 `one_pointer_has_both_copies_overwritten_by_a_later_instance`

### 1.1 对照表（条款句：用户定案，`research/prompts/c834-r3-main-verification.md` 第 43 行；代码：冻结副本）

| 条款句 | 代码行 | 判定 |
|---|---|---|
| 「被判的较新根 R 的根记录里，树表与中央映射树根两个指针中有一个」（`research/prompts/c834-r3-main-verification.md:43`） | 「[&root.tree_table, &root.mapping_root]」（`crates/singlefs-core/src/mount.rs:2162`）；`.any(`（第 2165 行，转述：闭包写法含竖线，表格里不抄） | 一致：只看这两个指针，实例表指针与分配记录树根指针不看，条款也没点它们 |
| 「它两条位置条目指着的两个槽都读出自证过、实例代号比 R 大的单元头」（`research/prompts/c834-r3-main-verification.md:43`） | 第 2166 行 `locations.iter().all(...)`（转述：两条位置条目逐个判、全真才真）；第 2168 行 `is_some_and(... instance > root.instance)`（转述：严格大于 R 的根记录实例代号） | 一致：两条都要；相等不算 |
| 「索引节点读 16 KiB」（`research/prompts/c834-r3-main-verification.md:43`） | 「usize::try_from(NODE_BYTES).expect("16384"),」（`crates/singlefs-core/src/mount.rs:2188`）；「crate::unit::parse_index_node(&bytes).ok()」（`crates/singlefs-core/src/mount.rs:2190`）；`NODE_BYTES` = 16384（`crates/singlefs-format/src/lib.rs:17`） | 一致 |
| 「数据单元与打包记录单元读 32 KiB」（`research/prompts/c834-r3-main-verification.md:43`） | 「usize::try_from(DATA_UNIT_BYTES).expect("32768"),」（`crates/singlefs-core/src/mount.rs:2199`）；「crate::unit::parse_data_unit(&unit_bytes)」（`crates/singlefs-core/src/mount.rs:2201`）；`parse_packed_unit` 在第 2204 行；`DATA_UNIT_BYTES` = 32768（`crates/singlefs-format/src/lib.rs:20`）；两个解析函数入口都核长度等于 32768（`crates/singlefs-core/src/unit.rs:477`、`:543`） | 一致。条款没写先后：代码先读 16 KiB 试索引节点（第 2192 行 `if index_node_instance.is_some() {`），解不开再读 32 KiB 依次试数据单元、打包记录单元；读之前不知道槽里是哪一类，所以是并集，不是漏读 |
| 「自证过」 | `parse_index_node`、`parse_data_unit`、`parse_packed_unit` 都是共同前缀四关加头校验和加载荷校验和（`crates/singlefs-core/src/unit.rs:345`、`:374`）；索引节点另判声明长度、条目宽（第 403 行起） | 一致，且多一层：结构判不过的单元（校验和过、条目宽乱）也交 `None`，只会让变体多拒，方向保守 |
| 「实例代号比 R 大」：比单元头里的哪一个 | 索引节点：「68+2k 写序 4」（`crates/singlefs-core/src/unit.rs:126`），代码读 `header.instance`（第 2191 行）；数据单元、打包记录单元：`header.write_order.instance`（第 2202、2204 行）；R 一侧是 `root.instance`（第 2168 行） | 一致：三类单元比的都是单元头写序里的实例代号；不是出生代号、不是 txg |
| 空指针（条款没写，题目要问） | 「.filter(|pointer| **pointer != crate::pointer::NodePointer::empty_root())」（转述：第 2164 行，全零指针不进判；闭包含竖线，不作引文） | 条款没写这一格；代码的处置合理：全零指针两条位置条目都是 (盘 0, 槽 0)，没有「它指着的槽」，不是证据。树表或映射根为空指针时，另一个指针照判 |
| 「否则照…那一支办，不读系统配置里的号」（`research/prompts/c834-r3-main-verification.md:43`）：读不出走哪一支 | 槽读不出（越过盘尾、没这块盘）或三类都解不开 ⇒ `instance_in_the_self_verified_unit_header_at` 交 `None`（`crates/singlefs-core/src/mount.rs:2179`）⇒ `is_some_and` 假 ⇒ `all` 假 ⇒ 守卫 `if !one_pointer_has_both_copies_overwritten_by_a_later_instance(`（第 2078 行）为真 ⇒ 「placements_of_an_unapplied_publish_whose_account_is_unreadable(」（第 2082 行，与「记录有缺口」那一支同一个函数，第 2095 行） | 一致：读不出一律落到「记录不齐」那一支，那一支重读一次、看现行账里 C694 的 B 凭据，仍不成在取号之前拒 |
| 「不读系统配置里的号」（同上） | 函数 `placements_in_the_accounts_of_newer_readable_roots` 的参数表（第 2031 至 2041 行）里没有 `journal_instance_in_the_system_configuration`；全仓 `grep -rn journal_instance_in_the_system_configuration crates` 零命中（见 1.3） | 一致 |
| 放过的动作 | 守卫不成立时落到「None => continue,」（`crates/singlefs-core/src/mount.rs:2090`）：这条根不交账、不拒、不重读 | 一致：与被替换的 Y2 那一判放过时同一个动作 |
| 「被攻过零轮：实现后随代码轮再攻，入库 harness 用例（含 `urJ` 那一条）要在它下面跑绿」（`research/prompts/c834-r3-main-verification.md:43`） | 入库用例 `crates/singlefs-harness/tests/newer_unreadable_root_passes_only_when_both_copies_of_one_pointer_are_newer.rs`：8 条全过（1.3 原样行） | 一致 |

### 1.2 与 kb 的一处冲突（并排整行抄，条款还没写回）

- kb：「现算不成不隔离、不拒（主 agent 2026-10-08 定）」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）
- 代码：「if !one_pointer_has_both_copies_overwritten_by_a_later_instance(」（`crates/singlefs-core/src/mount.rs:2078`），守卫为真时走「记录不齐」那一支、仍不成拒。
- 判定：冲突。这是已知的：派发材料说第 392 行那一句还没写回；用户 2026-10-15 的定案（上面第 43 行）替换了它。按定案，代码是对的，kb 那句要改；这一格不是代码问题。

### 1.3 复跑与变异（原样）

命令（在冻结副本的拷贝上；`rerun.sh` 是同一批的整套复跑）：`grep -rnF journal_instance_in_the_system_configuration crates | wc -l`，输出 `0`。

入库用例 8 条全过：`name=` 行见 `research/prompts/m3-b44-code-r1-sonnet-model/out/summary.txt` 第 2 行「baseline harness-newer_unreadable_root_passes_only_when_both_copies_of_one_pointer_are_newer: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out」。

入库的三行 C834 变异（`crates/mutations.tsv` 第 3821、3822、3823 行）我各改一次、跑点名的目标，都红在点名的测试上：

- 第 3821 行「同一指针只要一份更新就放过」（`.all` 改 `.any`）：8 条里红 2 条，点名的 `newer_root_whose_tree_table_has_one_copy_overwritten_and_the_rest_reading_back_zero_refuses_the_writable_mount_in_both_device_orders` 在内；
- 第 3822 行「一律放过」：8 条里红 5 条，点名的 `newer_root_after_an_acquisition_that_landed_on_one_device_only_refuses_the_writable_mount_in_both_device_orders` 在内；
- 第 3823 行「一律拒」：8 条里红 3 条（三条 urJ 放过用例），点名的 `newer_root_whose_pointer_slots_all_hold_units_of_the_mount_that_could_not_see_it_is_passed_in_both_device_orders` 在内。

完整行（红的测试全名与汇总）在 `out/summary.txt` 的 `row=3821`、`row=3822`、`row=3823` 行，cargo 原始输出在 `out/logs/mut-3821.log` 等。

### 1.4 「有没有 R 走得完而它放过并写掉的历史」「有没有入库 urJ 那一形它拒的」

入库 urJ 那一形：三条用例（没有读故障、映射根两槽读回全零、树表两槽读回全零）在代码下都做成、落到 P，两种交盘次序各一次；第 3823 行变异（一律拒）把这三条打红。**没有 urJ 那一形被拒。**

「R 走得完而被放过」要同时满足：守卫为假（有一个指针两份都读出自证的更新实例单元头），而 R 其实还走得完。按代码推（读码，推的，没造镜像）：

1. 树表那一支：树表是根记录指向的第一个单元，R 两份都没了，任何一棵树都找不到。走不完。
2. 映射根那一支：重建这一版时「mapping: central_mapping_tree.into_tree()?」（`crates/singlefs-core/src/recovery.rs:5398`）读不出中央映射树就返回错，R 不可重建。走不完。入库用例里这一支只在「映射根被盖、树表读回全零」时出现（树表那一对同时读不出），**没有一条用例单独钉「映射根两份被盖、树表完好」这一形**；按代码它放过，且 R 此时走不完，所以不构成「放过一条走得完的 R」，但是这一句靠读码，不靠用例。
3. 「单元头实例代号 > R 的实例代号」会不会出现在 R 自己引用的单元上：R 引用的单元是 R 那次发布或更早的发布写的，写序实例代号 ≤ R 的实例代号；只有被后一次实例回收再分配、盖写过的槽才会出现 > R。所以「两份都 > R」等于「这个单元两份都被盖」。推的（前提是实例代号随取号单调增，我没去核那条条款的原文）。
4. 位置条目是提示（D19 已定项 5）：R 的这两个指针是 R 自己那次发布写的，提示就是真位置，R 的映射里没有第二个位置可退，所以「提示被盖」不会有「单元其实在别处、R 还走得完」的退路。推的。

什么现象会推翻 K1 判「一致」：(a) 造出一个镜像，守卫为真而 R 走得完（上面 2 或 3 被打破，例如映射根两份被盖而 R 仍能重建）；(b) 入库 urJ 用例在冻结副本上红（我跑的是绿）；(c) 条款第 43 行里的某个量（16 KiB、32 KiB、实例代号比 R 大）与代码不是同一个量。

### 1.5 问题 2 的回答（K1 逐句）

读哪几个槽：树表与映射根两个指针各两条位置条目（第 2162、2166 行）。按什么长度读：索引节点 16 KiB，数据单元与打包记录单元 32 KiB（第 2188、2199 行）。比的是单元头写序里的实例代号对 R 的实例代号，严格大于（第 2191、2202、2204、2168 行）。空指针：不进判（第 2164 行，条款没写）。读不出：交 `None`，落到「记录不齐」那一支（第 2078、2082 行）。逐句对，没有对不上的；与 kb 第 392 行的冲突见 1.2，是待写回，不是代码错。

## 二、K2 A1、A2 两条用例与 A3 变异行（问题 3）

判决要钉的（`research/prompts/code-backlog-b2-code-r1-main-verification.md` 第 32 至 34 行）：A1 豁免先于判重；A2「cur 那一版引用的单元不进逐盘验」那一道；A3 整行替换变异表里「步 4：影子账不豁免候选根引用的槽（保守读法）」的点名测试。我的做法：在副本里按冻结副本 `crates/mutations.tsv` 的行把代码改坏，跑点名的目标，看红的是哪条断言，再还原（工具 `research/prompts/m3-b44-code-r1-sonnet-model/tools/mutrow.py`，每行锚点命中恰好 1 次才改）。

### 2.1 A1

| 要钉的 | 代码行 | 判定 |
|---|---|---|
| 「改前（次序互换）红、改后绿；`prove-red.sh` 抓到新行」（`research/prompts/code-backlog-b2-code-r1-main-verification.md:32`） | 被钉的一行：「.filter(|covered| !exempt.contains(covered) && isolated.insert(*covered))」（`crates/singlefs-core/src/mount.rs:1752`）。变异行 `crates/mutations.tsv:3824` 把它改成 `isolated.insert(*covered) && !exempt.contains(covered)`（先判重、后看豁免）。两条新用例：`mount.rs:10251`、`mount.rs:10282`（函数名带 `..._is_still_isolated_for_the_unapplied_journal_records`） | 一致 |
| 红在哪一条断言 | 变异下过滤 `shadow_ledger_exempts_and_isolates_slot_by_slot_tests` 的 8 条里红 2 条，正是上面两条：`mount.rs:10273:9` 与 `mount.rs:10301:9` 的 `assert_eq!`，`left: (false, true)`、`right: (true, false)`（含义：槽 s 变成没隔离、可发；应当隔离着、发不出去）。输出在 `out/logs/mut-3824.log` | 一致：红的就是判决要钉的那一道，不是别的断言顺带红 |
| 改后绿 | 冻结副本上 `cargo test -p singlefs-core --lib` 269 条全过（`out/summary.txt` 第 1 行） | 一致 |

`prove-red.sh` 是否抓到新行：**复核不了**。我没起它（它在变异整表那一档，派发写了不跑重型）；我用自己的 `mutrow.py` 做了它做的那一步（改一行、跑点名、看红），只证到「这一行改了会红在这两条上」。

### 2.2 A2

| 要钉的 | 代码行 | 判定 |
|---|---|---|
| 「那一行变异红在新用例上；两条位置项不同槽的映射条目怎么认条款没写（K2.4），用例只取同槽」（`research/prompts/code-backlog-b2-code-r1-main-verification.md:33`） | 被钉的一行：「} else if !is_referenced_by_the_current_version {」（`crates/singlefs-core/src/mount.rs:7881`，判决写的第 7821 行是旧行号）。变异行 `crates/mutations.tsv:3825` 改成 `true || !is_referenced_by_the_current_version`。用例 `mount.rs:10493`；同槽前提：「data_locations[0].slot, data_locations[1].slot,」（`crates/singlefs-core/src/mount.rs:10503`） | 一致 |
| 红在哪一条断言 | 变异下用例红在最后那条 `assert!(second_rollback.is_ok(), ...)`（断言行 `mount.rs:10551:9`，`second_rollback.is_ok(),` 在第 10552 行），消息「现行那一版与目标共用的单元不进逐盘验，盘 1 那一份坏着回退照做」，值是 `Some(IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50180) })`（`out/logs/mut-3825.log`）。前面两处 `.expect`（第一次回退）在变异下仍过，说明红的是「第二次回退时现行版本与目标共用的坏份单元被拉进逐盘验」这一道 | 一致 |

### 2.3 A3

| 要钉的 | 代码行 | 判定 |
|---|---|---|
| 「整行替换第 1 行」与「的点名测试换成 core 的 `a_candidate_roots_two_slot_placement_exempts_its_second_slot_from_an_abandoned_roots_placement`」（都在 `research/prompts/code-backlog-b2-code-r1-main-verification.md:34`；原句中间夹着变异行名，名字里的括号不能进引号） | 变异行 `crates/mutations.tsv:40`：名字与改法（把 `&referenced_by_candidates` 换成 `&BTreeSet::new()`）不变，点名换成 `-p singlefs-core --lib -- a_candidate_roots_two_slot_placement_exempts_its_second_slot_from_an_abandoned_roots_placement`、红的测试名同。旧行在 HEAD 是第 41 行（点名 `rollback_by_a_forward_publish` 里的 harness 用例，判决 K4.4 说它不红）；判决写「第 1 行」与行号对不上（HEAD 里 1 至 5 行是注释），按名字找得到、内容对 | 一致（行号表述对不上，内容对上） |
| 「门禁 checker-tier-crates-mutation-replay 复跑那一行红」（`research/prompts/code-backlog-b2-code-r1-main-verification.md:34`） | 变异下该测试红在 `mount.rs:10241:9`，消息「起点 + 1 被候选根（mkfs 那一版）的实例表 (起点, 跨 2) 罩着：不隔离」（`out/logs/mut-40.log`）；基线绿 | 我用自己的 `mutrow.py` 证到「改了会红」；门禁那一道**复核不了**（登记为重型/变异档，写进「要主 agent 跑的命令」） |

推翻 K2 判「一致」的现象：(a) 把 `mount.rs:1752` 或 `mount.rs:7881` 改回原样之外的写法（例如两个条件都留着、只换名字）而三条用例仍全绿；(b) 变异下红的是别的断言（例如构造用例的 `.expect` 先 panic），而不是上面写的那几条；(c) `mutations.tsv` 对应行的锚点命中数不是 1（我这一次每行都是 1，见 `out/summary.txt` 的 `anchor_hits=1`）。

## 三、K3 `system_configuration.rs` 的 `parse_slot` 判节点大小

| 条款句 | 代码行 | 判定 |
|---|---|---|
| 「读者判这 4 字节等于 `NODE_BYTES`，不等就判这一槽不可择」（`.claude/kb/checks-owed.md:559`） | 偏移：「const NODE_BYTES_OFFSET: u64 = 301;」（`crates/singlefs-core/src/system_configuration.rs:45`），字段表同值：「节点大小 301」（`.claude/kb/layout/01-first-txn.md:158`）；判：「if u64::from(recorded_node_bytes) != NODE_BYTES {」（`crates/singlefs-core/src/system_configuration.rs:579`） | 一致 |
| 「节点大小那 4 字节不等于 `NODE_BYTES` 就当这一槽自证不过、不可择，别的槽照旧可择，拒因单列一个成员点名节点大小字段」（`records/2026-10-13-代码欠账批.md:103`）：自证不过、不可择 | 读者里与整槽校验和不过并成同一臂：「) => Ok(SystemConfigurationSlotReading::NotSelfDescribing),」（`crates/singlefs-core/src/recovery.rs:964`），前一行（第 963 行）是 `NodeBytesNotTheFormatConstant { .. }`。槽 0、槽 1 与逐档槽距试读（`recovery.rs:1012` 起）同走 `mountable_slot_or_refusal`（第 942 行），分流一样 | 一致 |
| 同上：别的槽照旧可择 | 用例 `the_slot_with_a_foreign_node_size_is_not_chosen_and_the_other_slot_of_the_device_is`（`crates/singlefs-harness/tests/system_configuration_slot_with_a_foreign_node_size_is_not_chosen.rs`）基线绿；`crates/mutations.tsv:3810`（读者改成照收）改坏后红在它身上（`out/summary.txt` 的 `row=3810`） | 一致 |
| 同上：拒因单列一个成员点名节点大小字段 | 「NodeBytesNotTheFormatConstant { recorded_node_bytes: u32 },」（`crates/singlefs-core/src/system_configuration.rs:680`），带盘上读到的值 | 一致 |
| 「坏镜像用例只改这 4 字节（重算整槽校验和），择系统配置必须拒、只读挂载同样拒」（`.claude/kb/checks-owed.md:559`） | 用例 `every_slot_with_a_foreign_node_size_refuses_choosing_the_system_configuration_and_the_read_only_mount`，同一个测试目标，4 条全过 | 一致 |
| 会不会误拒合法的槽（题目要问） | 写者只有 `to_slot` 一个入口：mkfs（`crates/singlefs-core/src/make_filesystem.rs:651`）与取号、抬 F 重写系统配置槽（`crates/singlefs-core/src/transaction.rs:571`）都经它；它写前断言位置：「writer.assert_position(NODE_BYTES_OFFSET, "节点大小");」（`crates/singlefs-core/src/system_configuration.rs:478`），值取 `NODE_BYTES`（第 274 至 276 行）。本仓写出的槽都是 16384，不会被拒 | 一致（读码）；会被拒的只有别的实现写的、节点大小不是 16384 的镜像，D22 已定项 26 第二档说第一版没有改节点大小的路（该句在代码注释第 263 行转述） |

条款没写的两格（不判错，交主 agent 看要不要写条款）：

1. 同一槽既节点大小不对、每区槽数 S 又越界时，代码先判节点大小（第 579 行在第 586 行之前）：这一槽按「自证不过」被跳过，而不是「整池拒绝挂载」。S 越界的条款（`RootRingSlotsPerRegionOutOfRange`，整池拒）与这一格的先后条款没写。后果只在「另一槽完好」时可见：改前拒整池，改后挂上另一槽。
2. 池级 checker 一侧也加了同名判（`crates/singlefs-checker/src/lib.rs:287`，`Verdict::NodeBytesNotTheFormatConstant`）。它不在这一轮射程（只 core 与 format），我没核。

推翻 K3 判「一致」的现象：(a) 一份本仓写出的系统配置槽被 `parse_slot` 报成 `NodeBytesNotTheFormatConstant`；(b) 节点大小错的槽在槽 0、槽 1、逐档试读三条入口里有一条被当成可择或整池拒（我只看到用例过了、读码三条入口同走一个函数）。

## 四、K4 `pointer.rs` 写者断言

条款：I-2.5（`crates/singlefs-checker/invariants/04-校验和.md:10`）。

| 条款句 | 代码行 | 判定 |
|---|---|---|
| 「任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序」（`crates/singlefs-checker/invariants/04-校验和.md:10`） | 码 2 / 码 3 指针：「self.location_entries_ascend_by_device().is_ok(),」（`crates/singlefs-core/src/pointer.rs:326`）；码 1 数据指针：「location_entries_ascend_by_device(&self.locations).is_ok(),」（`crates/singlefs-core/src/pointer.rs:401`）。改前两处是 `locations[0].device <= locations[1].device`（diff 里的 `-` 行） | 一致：写者与读者（`recovery.rs:5096` 的「位置条目不按设备身份严格升序」）同一个严格判据。中央映射条目与 journal 点名项的位置数组不经这两个 `write_to`（`crates/singlefs-core/src/records.rs:389` 的 `build_mapping_entry` 不断言），不在这一格 |
| 「非全零的指针不享这条豁免」（同上） | 「if *self == Self::empty_root() {」（`crates/singlefs-core/src/pointer.rs:316`）：整个结构等于 `empty_root()`（头、两条位置条目、实例代号、出生序号全零）才豁免；写者第 326 行调的是同一个方法，所以写者与读者的豁免是同一个。数据指针 88 字节、条款说的是「整条 86 字节全零」，数据指针不豁免与条款一致 | 一致：全零那一例外认得对 |
| 题目问：有没有合法写路径会写出同盘两条位置条目的非空指针 | 所有本仓造指针的入口都经 `location_entries`：「sorted.sort();」（`crates/singlefs-core/src/make_filesystem.rs:274`）后取 `sorted[0]`、「device: sorted[1],」（第 282 行）；写入口那一层：「location_entries(&identities, slot, unit)」（`crates/singlefs-core/src/transaction.rs:658`）。盘表里的身份各不相同（推的：盘身份是盘表的键；`sorted[1]` 取第二块盘，盘表只有一块时在写者之前就越界，改前也是） | 一致：本仓自己造的指针不会同盘。**没有合法写路径会写出同盘非空指针** |
| 盘上读来、照抄写回的指针（读侧是不是都判过） | 重建这一版时判一遍：「location_entries_ascend_by_device(&data_pointer.locations)」（`crates/singlefs-core/src/recovery.rs:5164`）等，清单在 `pointers_of_the_rebuilt_version_ascend_by_device`（第 5121 行起）：根记录四条、树表条目根、extent 上下段节点、inode 叶容器、分配记录树节点、记账树、中央映射树、数据指针 | **对不上，探针打中**（4.1） |

### 4.1 探针：盘上读来的容器树子指针同盘，被重写的根照抄时撞新断言

上面那张清单里没有墓碑树、谱系树、退役树三棵容器树的指针：`recovery.rs:3204` 到 `3245` 把它们按盘上读来的值装进重建出的这一版（`pointer: *pointer`、`ContainerTreeRead::index_nodes_version`），而 `pointers_of_the_rebuilt_version_ascend_by_device` 不枚举 `tombstone_containers`、`tombstone_tree_index_nodes`、`lineage_*`、`retirement_*`（`crates/singlefs-core/src/transaction.rs:2884` 至 `2896` 的字段）。容器树的内部条目与 inode 树同形，用 `build_inode_internal_entry` 写（`crates/singlefs-core/src/records.rs:259` 起，内含 `child.write_to`），所以下一次重写根时，盘上读来的子指针原样进 `NodePointer::write_to`。

我造的坏镜像（探针 `research/prompts/m3-b44-code-r1-sonnet-model/probe/probe_equal_device_pointer_in_a_copied_container_tree_entry.rs`，从入库的 C841 用例改来）：新池两个 inode，2 号 unlink 到底（墓碑树一片容器），把墓碑树根条目里指向这片容器的子指针的第二条位置条目的设备身份改成与第一条相同（槽、校验和不动，两份本来内容相同，读得通），根单元的校验和沿引用链补好；挂载后把墓碑容器压到一片 1 条，发 3 号的 unlink 意图（要开第二片、重写根、抄第一片的子指针）。

结局（原样，`out/summary-probe-new.txt`、`out/summary-probe-old.txt`，cargo 输出在 `out/logs/probe-new-rerun.log`、`out/logs/probe-old-rerun.log`）：

- 冻结副本的代码：`probe mount=ok`，`probe publish=panic 位置条目按设备身份严格升序（I-2.5），整条全零的空指针（empty_root）豁免：盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过`，panic 位置 `crates/singlefs-core/src/pointer.rs:325:9`。
- 把写者断言临时改回改前的 `<=`（只在副本里改）：`probe mount=ok`，`probe publish=ok`。

判定：写者本身与 I-2.5 一致（上一张表前三行），但读侧漏判容器树指针，而写者断言的消息「盘上读来、要照抄写回的指针在 recovery::rebuild_version 判过」对这三棵树不成立。改前同盘指针被悄悄收下、改后在发布里 panic：这是 C476（盘上内容可控时走得到的 panic）那一类。逆序（设备 1 在前）改前也 panic，所以缺口不是这一批带进来的，这一批把「同盘」也并进了 panic 面。I-2.5 条款说「一律判损坏」，对应的处置是读侧返回 `InvariantViolated`，不是 panic。

候选改法（交主 agent 判，我没改 `crates/`）：把三棵容器树的容器指针与 `pointers_below_the_root` 并进 `pointers_of_the_rebuilt_version_ascend_by_device` 的清单（`crates/singlefs-core/src/recovery.rs:5121` 起）；用上面的探针当用例，改后应在挂载处拒、不 panic。探针只造了墓碑树一支，谱系树、退役树同构，我没各造一份。

推翻 4.1 的现象：探针在冻结副本上不 panic，或挂载处已经拒（我跑出来是 `mount=ok` 加 `publish=panic`，两次一致：手跑一次、`rerun.sh` 一次）。

## 五、K5 `recovery.rs` 的 `allocation_records_are_one_per_device` 与 `unit.rs` 的 `index_node_entry_capacity` 前提断言

条款：外部审核 ①②（`records/2026-10-13-代码欠账批.md:50`）。

| 条款句 | 代码行 | 判定 |
|---|---|---|
| ①「`recovery.rs:4453` `allocation_records_are_one_per_device` 记录与盘列表都空时第 4487 行 `expect` 会崩——属实」（`records/2026-10-13-代码欠账批.md:50`） | 开头断言：「!device_identities.is_empty(),」（`crates/singlefs-core/src/recovery.rs:4468`）；原 `expect` 的消息换成「开头断言过盘表非空，上面又核过盘表里每块盘都有记录：至少有一块盘的落点集合」（`crates/singlefs-core/src/recovery.rs:4502`）；文档里补了 `# Panics`。用例 `an_empty_device_table_with_no_records_panics_on_the_non_empty_device_table_precondition`；变异 `crates/mutations.tsv:3812` 红在它身上 | 一致 |
| ① 同句后半「两个产品调用方…走到这里之前先经读者读实例表单元…零块盘时那一步已报错返回，产品路径走不到」 | 两个产品调用方都从读者取盘表：「let device_identities = reader.device_identities();」（`crates/singlefs-core/src/recovery.rs:2894`、`:5400`），之后进包装函数 `allocation_records_are_one_placement_per_device_on_every_device`（第 4731 行；调用在第 2895、5404 行）。读到调用处之前，分配记录树已经读回（第 2872 行起的 `read_allocation_record_tree(...)?`）；盘表为空的读者（`Vec<(盘, 设备)>` 实现）每次 `read` 都是 `None`，树读不出就在 `?` 处返回 | 一致（读码）。这是我对「产品调用方走得到新断言吗」的回答：走不到。只有「`device_identities()` 空、`read` 却读得出」的读者才走得到；`crates/` 里这样的实现只有测试替身，且它们列 2 块盘 |
| 题目问：「有记录、盘表空」改前 `false`、改后 panic，射程 | 改前先走 `placements_per_device.len() != device_identities.len()` 那一判（`recovery.rs:4492`）返回 `false`；改后断言排在最前。`pub fn allocation_records_are_one_per_device` 在 `crates/` 里的调用方：上面包装函数的两处调用（第 4753、4777 行）与 core 内的测试，别处零 | 射程：只有「盘表空」的公开 API 直调。产品路径走不到；这个变化已经写进文档的 `# Panics`，不是静默改。条款（①）要的是「契约缺口」补上：补了 |
| ②「`unit.rs:151` `index_node_entry_capacity` 条目宽 0 除零、key 宽过大下溢——式子属实，调用方全传格式常量」（`records/2026-10-13-代码欠账批.md:50`） | 断言：「entry_width > 0,」（`crates/singlefs-core/src/unit.rs:158`）与「key_width <= node_bytes && index_node_header_bytes(key_width) <= node_bytes,」（`crates/singlefs-core/src/unit.rs:162`），式子「(node_bytes - entries_start) / entry_width」（`crates/singlefs-core/src/unit.rs:166`）。调用方：`container_tree.rs:59`、`deleted_inodes.rs:20`、`inode_tree.rs:127`、`code_two_tree.rs:107`、`:108`，宽度都来自常量（容器树 key 宽、`deleted_inodes` key 宽、`INODE_KEY_WIDTH_IN_BYTES`、`MultiLevelCodeTwoTree::key_field_widths()`，`transaction.rs:2408` 起）；`parse_index_node` 不调它，盘上的宽进不来 | 一致 |
| ② 边界：头宽恰好装得下一个节点时 | 头宽 = 86 + 2 × key 宽 + 29（`unit.rs:145` 的 `index_node_header_bytes`）；key 宽 8134 时 16383 不 panic、交 0（用例 `index_node_entry_capacity_of_the_widest_key_whose_header_fits_the_node_is_zero_when_no_entry_fits`），8135 时 16385 panic；`mutations.tsv:3813`、`:3814`、`:3815` 三行各红在点名的用例上（`out/summary.txt`） | 一致；头宽 = 16384 在 key 宽为整数时不可能出现（2k + 115 是奇数），所以「不宽过节点」与「留得下一条」之间没有第三种取值 |

推翻 K5 判「一致」的现象：(a) 一个产品入口给 `allocation_records_are_one_per_device` 传空盘表而前面没有返回（需要一个列不出盘却读得出内容的读者）；(b) 任一 `index_node_entry_capacity` 调用方的宽度来自盘上读来的值。

## 六、K6 `container_tree.rs` 开新容器 checked 加法与 `PublishError::NextContainerNumberPastTheTopOfItsRange`

条款：C841 用户定案，「开新容器那一次发布用 checked 加法，溢出返回类型化错误、不 panic，读盘不变」（`records/2026-10-13-代码欠账批.md:103`）。

| 条款句 | 代码行 | 判定 |
|---|---|---|
| 「开新容器那一次发布用 checked 加法，溢出返回类型化错误、不 panic」（`records/2026-10-13-代码欠账批.md:103`） | 「rightmost_container.checked_add(1).ok_or(」（`crates/singlefs-core/src/container_tree.rs:552`）；类型化错误 `NextContainerNumberPastTheTopOfItsRange { tree, rightmost_container }`（`container_tree.rs:461`）；两个公开追加函数改交 `Result`：「) -> Result<ContainerTreeAfterThisPublish<Record>, NextContainerNumberPastTheTopOfItsRange> {」（`crates/singlefs-core/src/container_tree.rs:483`、`:521`）。全仓 `grep -n 'container *+ *1'` 在 core 零命中，别处没有同类加法 | 一致 |
| 同上：只在「开新容器」那一支加，装得下时不看号 | 加法在 `else` 支里（`container_tree.rs:545` 起），追加进开放容器那一支不碰；没有记录要追加时整棵照抄、不进循环。用例 `opening_a_container_after_one_numbered_at_the_top_of_its_range_returns_an_error` 两种都钉了（号为 `u64::MAX`，装得下照常追加、装满要开新的才报错） | 一致 |
| 「读盘不变」/ 错误「在发写之前交回、盘不变」（题目问） | 三棵树的追加都在计划阶段：墓碑树「.map_err(PublishError::NextContainerNumberPastTheTopOfItsRange)?」（`crates/singlefs-core/src/transaction.rs:5304`）与退役树（第 5340 行）在 `ordinary_allocation_part`（第 5189 行），谱系树（第 5516 行）在 `lineage_tree_after_this_publish`（第 5485 行，由 `resolve` 的第 5377 行调）；`prepare_the_version_publish`（第 6583 行）在动分配器与任何写之前先调「.ordinary_allocation_part(previous, &trees, deletion_and_lineage)?」（第 6605 行），`resolve` 在 `settle_the_allocation_record_tree`（第 6794 行）里的第 6827 行，还在分配器副本上。读侧一个字节不加判（`records/2026-10-13-代码欠账批.md:127`：「读者读盘不判容器号上界（用户定的是写路径报错，读侧不加）」） | 一致。证据强度不一：墓碑树一支有端到端用例 `an_unlink_intent_that_opens_a_tombstone_container_after_one_numbered_at_the_top_of_its_range_is_refused_before_any_write`（两块盘逐字节不变、分配器记录不变），变异 `mutations.tsv:3858` 红在它身上；谱系树、退役树两支只有 core 层追加函数的单测（W6 交回自己也说了：「C841 谱系树、退役树两条路只有 core 单测罩着」，`records/2026-10-13-代码欠账批.md:127`），发布层这两支靠读码（上一格） |
| 三处穷举 `match` 的处置对不对 | `refusal_is_short_of_space`：「| PublishError::NextContainerNumberPastTheTopOfItsRange(_)」（`crates/singlefs-core/src/mounted_session.rs:934`）归 `false`，注释「坏镜像，推抬 F 不会让号变小」；模型比对：同名成员归 `Unexplained`（`crates/singlefs-harness/src/model_comparison.rs:725`）；历史记录里的名字串（`crates/singlefs-harness/src/history.rs:2906`） | 一致：归 `false` 是对的，归 `true` 会让会话把它当空间不够去推抬 F 再重判、而号永远不会变小；模型里没有它的理由（健康的内存盘上容器号从 1 逐片加一） |

推翻 K6 判「一致」的现象：(a) 谱系树或退役树的坏镜像上，发布在写之前不返回错误而 panic 或已写盘（现在只有读码，没有端到端用例，这是最弱的一支）；(b) 别处还有一个对盘上读来的容器号加一的未检查加法（我 grep 的是 core 的 `container + 1` 一类写法）。

## 七、K7 删死代码与只动注释的几十处

条款：`.claude/kb/milestone/03-third-txn.md` 五·10（第 114 行）：「代码审阅 D 级没做完的：死代码、恒真测试、`let _ =` 压警告、约 60 处注释与代码不一致」；C842：「删掉这个函数，连同 e139 那句注释改掉」（`.claude/kb/checks-owed.md:561`）。回扫报告（`~/.local/share/singlefs/work/m3-item5-10-sweep/report.md`，1435 行）**我没读**，下面都是对着删除项自己查的。

### 7.1 删掉的是不是真的零调用

diff 里删掉的 `pub` 项，在冻结副本 `crates/` 与主仓 `scripts/` 的 `.rs`、`.wgsl` 里字面串命中数（工具 `tools/k7-zero-callers.sh` 的原样输出，排除 `e165`、`e168` 两个二进制里同名的自家 `PublishShape` 结构）：

```
JournalSequenceNumber::COUNTER_LIMIT 命中 0
.origin_of( 命中 0
.rightmost_path( 命中 0
.child_holding_data_unit( 命中 0
.upper_height() 命中 0
.lower_height() 命中 0
.previous_index_of_a_carried_container( 命中 0
device_offset_of_the_head 命中 0
EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES 命中 0
.data_unit_pointer() 命中 0
.data_unit_records() 命中 0
txg_is_inside_the_one_revolution_window 命中 0
tombstone_containers_one_root_node_holds 命中 0
PublishShape:: 命中 0
every_unlink_step_from 命中 5
NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS 命中 0
```

`every_unlink_step_from` 的 5 处都在 `unlink.rs` 的测试模块里（`crates/singlefs-core/src/unlink.rs:256`、`:323`、`:359`、`:371`、`:381`）：函数从 `pub` 挪进了测试模块，是搬不是删。`rightmost_path` 另有一处文字命中是测试名里的 `the_rightmost_path`（`container_tree.rs:669`），不是调用。`ExtentTreePlan::roles`、`PublishShape` 的 `rewritten_roles`、`ResolvedPublish::shape` 这几个方法/类型名太常见，字面串查不干净，靠编译：我在副本里编过 `singlefs-core` 库测试与四个 harness 目标，全绿（整仓 213 个测试二进制是主 agent 观测的 `merge-and-verify` 结果，我复核不了）。

### 7.2 不改行为

- `comment-only-change.py` 判 `admission.rs`、`block_device.rs`、`journal.rs`、`make_filesystem.rs` 四份只动了注释。
- 别的文件里的代码改动我逐份看过，除删除与 K1 至 K6、K8 以外只有：测试模块的搬移与加强（`mount.rs` 把 `SparseDeviceInMemory` 改 `pub(super)` 并加 `blank()`，`code_two_tree.rs` 的 `the_level_byte_bounds_the_height` 把空叶换成各装一把 key 的叶并去掉恒真的 `u8::MAX.checked_add(1)`，`extent_tree.rs` 的 `upper_root_level_for` 加 `# Panics` 与 `expect` 消息，`extent_tree.rs` 测试补内联数据指针一格），产品语义没动。
- C842：`txg_is_inside_the_one_revolution_window` 删净（上面 0 命中），e139 那句注释已改（`scripts/experiments/e139_tightened_floor.rs` 里现在写「核心层后来退回整个环，那个判法函数已删，见 C842」，`git diff HEAD` 可见）。

### 7.3 改过的注释说对了没有（抽查，不是全查）

注释行共约 190 行（`+` 行里以 `//` 起头的，mount.rs 55、transaction.rs 24、system_configuration.rs 16…）。我对着代码核了这几处，都说对了：`block_device.rs` 的「七个方法」（`BlockDevice` trait 里正好 `read_at`、`write_at`、`write_zeroes_at`、`barrier`、`probe_physical_block_size`、`size_in_bytes`、`journal_record_offsets_hint` 七个）；`allocator.rs` 的「全空聚簇段数不是增量维护」（`empty_segments` 每次读都 `filter` 数一遍，`allocator.rs:732`）；`extent_tree.rs` 的 `upper_root_level_for` `# Panics`（跨度从第 8 层起饱和在 `u64::MAX`，`extent_tree.rs:76`、`:95`）；`format/lib.rs` 的槽距说法（`slot_spacing_for` 是 `4096.div_ceil(io_min) * io_min`，`system_configuration.rs:206`，io_min = 3072 得 6144）；`admission.rs` 里点名的 `committed_reservation_for_tombstones_and_retirement_records`、`mount::publish_empty_after`、`mount::establish_instance`、`transaction::prepare_the_row_publish_on_a_version_without_file` 都存在；`mount.rs` 里关于 `raise_rollback_floor` 的新句「盘表、分配器与现行那一版只借给它（`&mut`）」与签名一致（`mount.rs:3900`）。其余约 180 行没逐句核。

推翻 K7 判「一致」的现象：任何一个删掉的名字在主仓别处（包括没编进我副本的 `scripts/experiments` 工作区、`.wgsl`）还有调用；或一份「只动注释」的文件，注释里点名的东西与代码不符。

## 八、K8 `mounted_read.rs` 的 `leaves` 与 inode key 宽合成

条款：W4、W6 交回两条（`records/2026-10-13-代码欠账批.md:118`、`:127`）。

| 条款句 | 代码行 | 判定 |
|---|---|---|
| 「`ExtentTreeReadsAtOpen.leaves` 上段停在内部节点时照记 1，要改得改代码加用例」（`records/2026-10-13-代码欠账批.md:118`）：计数改法 | 「leaves: upper_leaves + lower_leaves,」（`crates/singlefs-core/src/mounted_read.rs:771`）；上段那一份：「let upper_leaves = upper_leaves_read_at_open(level_of_the_node_read_last.get());」（`crates/singlefs-core/src/mounted_read.rs:732`），函数里 `Some(0) => 1`（第 304 行）、`Some(_) | None => 0`（第 305 行）。最后读的那个节点的层级由读节点的闭包记（第 722 行，`Cell`，第 711 行） | 一致。我追了三个边：(1) 上段走到叶：最后读的是层级 0 ⇒ 1；(2) 上段停在内部节点（中途没有孩子罩着这个 inode）：最后读的是内部节点 ⇒ 0，用例 `opening_a_file_whose_upper_segment_stops_at_an_internal_node_counts_no_leaf` 钉住（核 `node_reads: 1, height: 1, leaves: 0`）；(3) 根自己是叶、inode 不在它的范围里：`find_upper_leaf_entry` 第一次读根就返回（`crates/singlefs-core/src/extent_tree.rs:1066`，根只读一次、以 `already_read` 传下去），最后读的是层级 0 ⇒ 1，对（读到的就是一片叶） |
| 同上：求值时机 | `upper_leaves` 在 `find_upper_leaf_entry`（第 726 行）返回后立刻算（第 732 行），下段读节点还没开始，所以闭包里被下段覆盖的层级不会混进上段那一份 | 一致 |
| 同上：读错误 | 节点读不出（`?`）时闭包不更新层级，但此时 `find_upper_leaf_entry` 整体返回错，`open_file` 失败，`leaves` 不被用；节点读到但解不开时记 `None`，随后 `find_upper_leaf_entry` 自己也按解不开报错 | 一致 |
| 「inode 树 key 宽 8 在 `mounted_read.rs` 与 `recovery.rs` 各写一份字面量，合成一处跨 W2、W3 的文件」（`records/2026-10-13-代码欠账批.md:118`）与「inode key 宽合成 `inode_tree.rs` 的 `INODE_KEY_WIDTH_IN_BYTES`（顺带换了 `inode_tree` 扇出与 `transaction` 装 inode 根两处字面量）」（`records/2026-10-13-代码欠账批.md:127`） | 唯一定义：「pub const INODE_KEY_WIDTH_IN_BYTES: usize = 8;」（`crates/singlefs-core/src/inode_tree.rs:122`）；使用处：`inode_tree.rs:128`（扇出）、`mounted_read.rs:454`、「TREE_KIND_INODE => Some(INODE_KEY_WIDTH_IN_BYTES),」（`crates/singlefs-core/src/recovery.rs:3840`）、`transaction.rs:7547`（装 inode 根）。`mounted_read.rs` 里原来的两个私有常量删了 | 一致 |
| 有没有漏掉的字面量 | 我在 core 里找仍等于 8 的「key 宽」：`DELETED_INODES_KEY_WIDTH_IN_BYTES`（`deleted_inodes.rs:15`）与 `CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES`（容器树）是别的树的 key 宽，各有定义，不是 inode 树那一份漏换；`recovery.rs:6359` 用绝对值断言钉住 8。`crates/singlefs-checker` 的 `walk.rs` 独立实现、不引 core 的常量（设计如此） | 一致；`deleted_inodes.rs:14` 的文档说它「与 inode 树的 key 同一个量」，也可以引这一个常量，条款没要求，不算漏 |

变异：`mutations.tsv:3859`（`leaves` 多数 1）红在 `mounted_read::tests::opening_a_file_whose_upper_segment_stops_at_an_internal_node_counts_no_leaf`；`:3860`（常量改 16）与 `:3861`（读者核根另写字面量）都红在 `recovery::tree_kind_key_width_tests::deleted_inodes_tombstone_and_lineage_trees_have_eight_byte_keys_and_logged_operations_none`（`out/summary.txt`）。

推翻 K8 判「一致」的现象：(a) 打开一个读得通的文件，`leaves` 与「读到的层级 0 节点个数」对不上（含上段停在内部节点、下段多层、内联三种）；(b) core 里找到一处按 inode 树 key 宽取 8 的字面量不引 `INODE_KEY_WIDTH_IN_BYTES`。

## 九、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 | 一致（逐句对上），另有一处待写回的冲突 | 读哪几个槽、读多长、比哪个实例代号、空指针、读不出走哪一支，十句都对上用户 2026-10-15 的定案；入库 urJ 三条放过用例绿、三行 C834 变异各红在点名用例；与 `23-journal的角色与格式.md:392` 并排冲突是已知的待写回，不是代码错；「映射根两份被盖、树表完好」这一形没有单独用例，靠读码推 R 走不完 |
| K1 之 R 走得完的历史 | 没造出，但映射根那一支只有读码 | 1.4 |
| K2 A1 | 一致 | 次序互换（`mutations.tsv:3824`）红在两条新用例的 `assert_eq!` 上（`mount.rs:10273`、`:10301`） |
| K2 A2 | 一致 | `true \|\|` 变异（`:3825`）红在最后那条 `second_rollback.is_ok()`（`mount.rs:10551`），用例只取同槽 |
| K2 A3 | 一致（行号表述对不上，内容对上） | 第 40 行点名换成 core 用例，改坏红在 `mount.rs:10241`；门禁复跑与 `prove-red.sh` 复核不了 |
| K3 | 一致，两格条款没写 | 偏移 301、读者判、与校验和不过同一臂、别的槽照择、拒因单列；节点大小与 S 越界的先后、checker 侧没核 |
| K4 | 写者与条款一致；读侧漏判，探针打中 | 没有合法写路径写出同盘指针；但三棵容器树的盘上指针不在重建时的升序清单里，同盘指针被重写时改后 panic、改前放过（探针 `mount=ok`、`publish=panic` 对 `publish=ok`），属 C476 类，缺口老的、这一批把同盘并进 panic 面 |
| K5 | 一致 | 断言在公开 API 的前提上，产品调用方走不到；「有记录、盘表空」false 变 panic 的射程只有直调 |
| K6 | 一致，证据强度不一 | checked 加法、类型化错误、计划阶段交回、三处 `match` 处置都对；谱系树、退役树的发布路径没有端到端用例 |
| K7 | 一致（抽查） | 删掉的 16 个名字零调用（`every_unlink_step_from` 是搬进测试）；4 份文件只动注释；注释核了 6 处，约 180 行没逐句核；回扫报告没读 |
| K8 | 一致 | `leaves` 三条边都对；inode key 宽合成一处，没有漏换的同树字面量 |

## 十、要主 agent 跑的命令

1. 整套复跑（量法的 22 行汇总，约 12 分钟以上，机器忙时更久；跑完看末行 `name=rerun_compare same`）：
   `bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/m3-b44-code-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-b44-code-r1/repo --work <新目录>`，环境变量 `RERUN_THREADS=31`（默认 31）。
2. K4 探针两个变体（各约 2 到 4 分钟编译加 10 秒到 1 分钟跑）：同一条命令前面加 `RERUN_PROBE=new RERUN_SKIP_BASELINE=1 RERUN_ROWS=""` 与 `RERUN_PROBE=old RERUN_SKIP_BASELINE=1 RERUN_ROWS=""`，输出里 `probe-new:`、`probe-old:` 两行与 `out/summary-probe-new.txt`、`out/summary-probe-old.txt` 比（末行 `rerun_compare` 会报 differ，因为参考汇总是整套那一份，只看 `probe-` 行）。
3. K2 A1、A3 与 K2 A2 在门禁变异复跑上红不红（我复核不了的那一半），轻型起法，三行各点名：`bash scripts/hosts/run-with-memory-cap.sh <上限> bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick --item 3824 --item 3825 --item 40 <根>`；`prove-red.sh`（`scripts/mutation/prove-red.sh`）抓不抓得到这三行，我也没起。
4. 本轮没有留下需要等的后台任务。

## 十一、没做什么

- 不判别的腿：K 格的「哪一格会错」是攻方腿的活。我给出的探针是读码时撞到的一条线索，已经复现了两次，判决与是否并进修红批归主 agent；我没改 `crates/`。
- 没读：五·10 回扫报告全文（`~/.local/share/singlefs/work/m3-item5-10-sweep/report.md`）；六份实现员报告（`m3-b44-w1` 至 `w6`）；岔路单 `c834-r3-forks.md`。引的条款一律来自 `c834-r3-main-verification.md`、`code-backlog-b2-code-r1-main-verification.md`、`records/2026-10-13-代码欠账批.md`、kb 与 `checks-owed.md`。
- 没验：池级 checker 一侧（`crates/singlefs-checker`）的 C840 判法；谱系树、退役树的发布路径端到端；K7 的约 180 行注释逐句；`mapping_root` 单独被盖、树表完好那一形的镜像（没造）。
- 没跑：全量 `cargo test`、checker 档测试、门禁阶段、`prove-red.sh`、变异整表；崩溃状态全量（没造任何超过 10⁶ 个状态的历史）。
- 量法的两条教训（写进来是因为会影响别人复跑）：(1) 编译目录共用时，用 `rsync -a` 保留冻结副本的旧修改时间会让 cargo 把别的树留下的产物当成新鲜的，我量到过基线红在别的变异留下的产物上；`rerun.sh` 里已改成 `--no-times`。(2) 机器 loadavg 一百以上时，内存包装排队加编译常超过前台 4 分钟，我外层的 `timeout` 截断过几条，截断时变异脚本里被改的文件可能没还原（我每次截断后核过副本与冻结副本 `diff -rq` 一致，才继续）。
- 草稿与副本：我的仓副本 `copy/`、编译目录 `target/`（2.6G）与七个复跑目录 `rerun-*` 在 `~/.local/share/singlefs/work/m3-b44-code-r1-sonnet/` 下，交回前已删（量完、日志与汇总已拷进模型目录 `out/`，没有再用）；该目录里只剩几十 KB 的日志与 `tools/` 脚本，没有仓副本和编译目录。
