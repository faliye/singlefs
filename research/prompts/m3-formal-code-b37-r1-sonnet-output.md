# m3-formal-code-b37-r1 云端正推腿（Sonnet）报告（2026-10-09）

腿：云端正推；格：B1–B4；立场：从条款与驱动正推每处改动做的是不是它们说的。
代码引文读冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree/`，写 `crates/…` 相对路径加那棵树里的行号（核引文时 `--root` 先给这棵树）。
条款与驱动引文是仓里的文件与行号。模型与脚本在 `research/prompts/m3-formal-code-b37-r1-sonnet-model/`（`tools/rerun.sh`，输出 `rerun-output.txt`）。

## 先说结论

| 格 | 判定 |
|---|---|
| B1 | 一致。三档并起来恰好罩住 `Criterion::ALL` 的 35 行，没有一行三档都不判；每行读数只来自判它的那一档。旁注三条见 B1 第 6 节 |
| B2 | 一致。25 与 harness 那份是同一个量；三种写的名字与次序逐个等于 `IN_REPORT_ORDER`。旁注一条：同一单测的文档注释还写着 23 |
| B3 | 一致。改后的名字存在，说的两种不适用与 `walk.rs` 的判法对得上；它只列了四种不适用里的两种，不算错 |
| B4 | 一致加缺口。两行变异锚点各命中一次、测试筛得到；第一档（r3 Direct）被改掉没有变异罩着，也造不出能罩着的变异；r1、r2 两段无变异；nff 两处钉值无专属变异 |

## B1　E165 全表对照

### 1. 条款说的

- 原登记 5.4：「对照那段循环遍历判据表的每一行（判据表是装置里的一个数组，加判据就多一行；对照按这张表迭代，不是逐条手写调用）」（`research/prompts/e165-preregistration.md:508`）。
- 同一行：「有一条判据在它的对照上违例数为 0，这一段作废」（`research/prompts/e165-preregistration.md:508`）；作废条款 V2：「有一条判据在它的阳性对照上没红，或对照的」（`research/prompts/e165-preregistration.md:897`）。
- r2 登记 5.5：「原登记 5.4 那张表的每一行在 r2 档照跑（对照循环遍历判据表，新判据加进同一张表）。新加的：」（`research/prompts/e165-r2-prereg.md:450`）；T-表一行「扫盘建表之后删掉表里计数器最大的那一条」（`research/prompts/e165-r2-prereg.md:462`），T-行一行「扫盘建表那一遍的实现结局换成 Panicked，表不动」（`research/prompts/e165-r2-prereg.md:463`）。
- r5 登记 5.4：「对照循环遍历判据表，J8 五格各占一条」（`research/prompts/e165-r5-prereg.md:462`）。
- 条款没写 T 对照该配哪个 `--profile`：r2 登记只说「`--table direct|scan|both`（不给取 `direct`）」属于 r2 档（`research/prompts/e165-r2-prereg.md:409`），r5 登记说 `--table` 在 r5 档「照 r2 档的意思认」（`research/prompts/e165-r5-prereg.md:420`）。这一项条款没说，不算冲突。

### 2. 三档并起来是不是恰好罩住 `Criterion::ALL`

改动后的三趟（冻结副本）：

- 「let all_families = [Family::LegalHistory, Family::Adversarial, Family::MultiDisk];」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18295`）。
- 第一趟 `let mut readings = positive_control_readings(`（同文件第 18296 行）取 `RunProfile::r3(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),`（第 18298 行）；
  第二趟 `readings.extend(positive_control_readings(`（第 18301 行）取 `RunProfile::r5(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),`（第 18303 行）；
  第三趟 `readings.extend(positive_control_readings(`（第 18306 行）取 r3 加「.with_table_source(RecordTableSource::Both),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18309`）。
- 覆盖断言与红断言：「.any(|reading| reading.criterion == *criterion),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18316`）、
  「"判据 {} 没有对照的读数",」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18317`）、
  「assert!(reading.is_red_as_required, "对照没红：{reading:?}");」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18322`）。
- 判据表：`labelled_enumeration!` 展开出「const ALL: &'static [Self] = &[$(Self::$variant,)+];」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:2143`），`enum Criterion` 在第 6121–6157 行，我用 `awk 'NR>=6121 && NR<=6157' … | grep -c '=>'` 数出 35 行。

我把 `RunProfile` 的四个判档谓词（第 340–373 行）和 `is_judged_under`（第 6211–6250 行）从源码里原样解析出来，对三趟算每行被谁判（`model/tools/coverage.py`，输出 `model/rerun-output.txt`）。原样输出里的三行：

```
Criterion 行数: 35
三趟都不判的行: []
pass1 判的行数: 29  pass3 判的行数: 31  pass1 是 pass3 的子集: True  pass3 比 pass1 多判: ['TableComparisonSeesADifferentRow', 'TableComparisonSeesADifferentTable']
```

分趟独有的行（同一份输出末尾三行）：只有 r5 趟判 J8a、J8a_unmount_marker、J8b、J8c 四行；只有 r3 加 Both 趟判 T_table、T_row 两行；r3 Direct 趟没有独有的行。
旧测试只有 r3 一趟，我的模型推演它缺 6 行，第一条是 J8a——与根环线日志 `checker-quick.log` 第 407–419 行报的「判据 J8a 没有对照的读数」同一条（`~/.local/share/singlefs/work/m3-r1/merge-10z-i/checker-quick.log`，主 agent 观测，我读过）。

### 3. 每一行的对照是不是在判它的那一档里读出来、红

- 循环先按族过滤，再按档过滤：「if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7758`），
  「if !criterion.is_judged_under(run_profile) {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7761`）。
  某一趟里不判的行，`control_readings_of` 根本不被叫，所以不会有「A 档有读数、B 档才判、并起来顶替」的读数：并集里一行的每条读数都来自一趟判它的档。
- `is_judged_under` 和真跑时出行用的是同一批谓词：J8 行「| Self::J8cNothingAppliedReturnsTheChosenRoot => run_profile.judges_j8(),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6225`）对出行处第 7953 行 `if run_profile.judges_j8() {`；
  J7 行「| Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.judges_j7(),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6221`）对出行处第 7941 行；
  拆行「Self::J6QnSplitByI812 => run_profile.splits_j6qn_by_i812(),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6213`）对第 8257 行；
  T 行「run_profile.compares_the_two_record_tables()」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6215`）对出 `name=table_comparison` 的第 8040 行。
- 红断言对并集里每条读数都下（第 18321–18323 行），不是只对「每行第一条」。
- T 的两条对照自己带档：`run_profile_of_a_control` 把 T 的两个假结局强制到 Both，别的假结局强制到 Direct：「without_the_record_table_knobs.with_table_source(RecordTableSource::Both)」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6308`）。
  所以 T 的读数不会因为跑在 Direct 趟里而少一条，也不会让 Direct 趟里的 T 读数冒充「判过」——Direct 趟里 T 本来就不判，被第 7761 行滤掉。

### 4. 三档与 E165 正式跑用的档、`--table` 取法对不对得上

读驱动 `research/scripts/replay.sh`：

| 测试里的趟 | 正式跑里的对应 | 对得上吗 |
|---|---|---|
| r3 Direct | `--profile r3`：「--bounds segment1 --family adversarial --profile r3」（`research/scripts/replay.sh:763`），不带 `--table` | 对得上（正式跑只跑对抗一族，测试三族都跑，是并集更宽） |
| r5 Direct | `--profile r5`：「--bounds segment1 --family legal --profile r5」（`research/scripts/replay.sh:855`）、对抗（第 858 行）、多盘（第 861 行），都不带 `--table` | 对得上 |
| r3 加 Both | 正式跑里 `--table both` 全配 `--profile r2`：「--family legal --profile r2 --table both」（`research/scripts/replay.sh:741`）、对抗（第 744 行）、多盘（第 747 行） | 对不上档名：正式跑没有 `--profile r3 --table both` 这一条 |

对不上的那一格不影响判据，三点依据：

1. 命令行认这个组合：「"r3" | "r4" => Ok((」下面接 `.with_table_source(table_source)`（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17529`、`:17531`），所以 `--profile r3 --table both` 是一条能跑的命令，只是正式跑没用过。
2. T_table、T_row 判不判，只看建表法，不看判据集：「run_profile.compares_the_two_record_tables()」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6215`）；r2 档与 r3 档在这一格上没有差别。
3. 正式跑那个配法（r2 加 Both）的 T 对照另有一条单测钉着读数：「let run_profile = RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence)」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18198`）接「.with_table_source(RecordTableSource::Both);」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18199`）。

真跑时每次只选一族，T 的两条对照随 `--table both` 进每一族：「positive_control_readings(&selection.families, run_profile, thread_count);」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17769`）加上 T 行的 `is_judged_in_every_family`（第 7758 行已引）。测试一次选三族，是把各次真跑的对照并在一起。

### 5. r1、r2 两段原样，r2 那条断言还成不成立

- 改动全文 `research/prompts/_m3-formal-code-b37-r1-diff.md` 里这条单测的两个块：第一块只改文档注释，第二块从 r2 断言的末几行（上下文）起、在其后加减。r1 段（「let r1_readings = positive_control_readings(&families, RunProfile::R1, thread_count);」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18277`）和 r2 段的调用部分（第 18282–18286 行）两个块都没碰到。
- r2 段还成立：r2 读数是 `RunProfile::r2(…)` 档（第 18284 行）跑出来的，该档不拆 J6Qn：「CriteriaSet::RegisteredBeforeR2 | CriteriaSet::WithJ6QnAndJ7 => false,」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:370`），所以 `is_judged_under` 把 `J6QnSplitByI812` 滤掉，「"r2 档不判 J6Qn 的 I-8.12 拆行"」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18291`）的 `.all(…)` 在非空的读数上成立。
- 新增的 `readings` 变量在 r2 段之后才声明，r2 段不受影响；r2 段里的 `[Family::LegalHistory, Family::Adversarial, Family::MultiDisk]` 仍是内联的数组（第 18283 行），没被改成 `all_families`。

### 6. 三档都不判的行，以及我看到的旁注

三档都不判的行：没有（第 2 节输出）。旁注三条，都不改变 B1 的判定：

1. 第一趟（r3 Direct）是第三趟（r3 加 Both）的子集：第 2 节输出「pass1 是 pass3 的子集: True」。第三趟里非 T 的对照一律被强制回 Direct（第 6302 行 `.with_table_source(RecordTableSource::Direct)`），读数与第一趟逐条相同（读码推的），只多 T 两行。去掉第一趟，覆盖和红断言一样成立；它只是多算一遍。
2. r2 段的读数只用来验「不拆行」那一条，没有逐条验红（第 18282–18292 行），改动前就是这样。r2 档（正式跑 `--table both` 所在的档）的对照红不红，靠 r3 趟顶；两档只差拆行一行。
3. 这条单测验的是「每一行都有读数、每条读数都红」，不验「读数确实出自判它的那一档」。后者靠第 3 节说的共用谓词保证。若有人把 `is_judged_under` 的 T 臂改成永远判，T 的读数会在 Direct 趟里出现（被强制回 Both，照样红），单测不会红。这是我读码推的，没跑变异，也没查有没有别的测试管这一臂。

另有一处条款字面与实现的差：r5 登记写「J8 五格各占一条」（`research/prompts/e165-r5-prereg.md:462`），实现是 4 个 `Criterion` 行（J8a 两个单元格合一行）。五个格各有钉值的对照：C8a 钉实例表指针、分配记录树根、卸载记号三格（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7571`、`:7575`、`:7580`），J8b 在 C8c（第 7634 行），J8c 在 C8d（第 7661 行），五格都有对照。不构成缺口。

### 7. 什么现象会推翻 B1

- 任何一行 `Criterion::ALL` 在 `is_judged_under` 下对 r3 Direct、r5 Direct、r3 加 Both 三档都为假：主 agent 可在 `model/tools/rerun.sh` 的 `coverage.py` 重跑，输出「三趟都不判的行」非空即推翻。
- 某条读数的 `criterion` 字段标错（如 J5b 的读数标成 J5a）：我只读了 `control_readings_of` 里各臂把 `criterion` 传下去的写法，没逐条核每个读数的标签；这一点在「没做什么」里列出。
- `cargo test` 在主工作区该单测不是 `1 passed`（主 agent 已跑过一次，日志在 `~/.local/share/singlefs/work/e165-control-test-fix/fixed-run-2.log`，我读过）。

## B2　nff 钉值

### 1. 25 与 harness 那份是不是同一个量

判定：同一个量、同一条路、同一段切法。

- harness 那一份：「(25, "14+1+1+1+4".to_string(), 16402)」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:356`），量的是 `mkfs_operations`，即「let mkfs_operations = &operations[..pool.mkfs_operation_count];」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:332`）。`mkfs_operation_count` 取在 `make_filesystem` 刚返回时：「let mkfs_operation_count = stream.operations().len();」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:165`）。
- nff 那一份：五段由 `new_pool_file_creation_paths` 切，第一段「("mkfs", &operations[..mkfs_operation_count]),」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1355`）；单测里喂它的是「let paths = new_pool_file_creation_paths(&operations, run.mkfs_operation_count, boundary);」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2472`）。`run.mkfs_operation_count` 在 harness 的场景函数里同样取在 `make_filesystem` 之后：「let mkfs_operation_count = stream.operations().len();」（`crates/singlefs-harness/src/scenario.rs:173`）。
- 参数一致：nff 用 `e142_parameters(512, 512)`（第 2451 行），槽距由 `slot_spacing_for(512)` 向上取整到 4096（`crates/singlefs-core/src/system_configuration.rs:208`），与 harness 自己写的 `fixed_structure_slot_spacing: 4096`（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:97`）相同；两处的区域设备 `[0, 1, 0]`、日志环字节数都是 `JOURNAL_RING_DEFAULT_BYTES`、根环槽数都是 `AT_MAKE_FILESYSTEM`（`scenario.rs:58–66` 对 `new_pool_file_creation_publish.rs:93–99`）。
- 另外四段 2、20、35、57 在两边逐个相同（harness 第 368、380、392、404 行；nff 第 2476 行同一个数组）。
- 命令验证：harness 档测试我在冻结副本上跑了，原样末段（日志在 `~/.local/share/singlefs/work/m3b37-sonnet/harness-run.log`）：

```
running 9 tests
test recorded_paths_match_the_registered_segment_sequences ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

  命令是 `CARGO_TARGET_DIR=<草稿目录>/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 15 cargo test --offline --locked -p singlefs-harness --test new_pool_file_creation_publish`，退出码 0。nff 那条是 checker 档，我没跑；主 agent 的日志 `~/.local/share/singlefs/work/e165-control-test-fix/nff-run.log` 末行「test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.81s」（我读过，没复跑）。
- 来源：harness 第 347–349 行的注释写明为什么 23 变 25：「mkfs 23 → 25 步，第一段 12 → 14 个写」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:348`）。nff 的改动只是把这个数跟上。

### 2. 三种写的名字与次序

判定：逐个相同。

- `describe_publish_writes` 经 `describe_writes_by_kind` 按 `IN_REPORT_ORDER` 逐种列、没写过的列 0：「let per_kind: String = WrittenStructureKind::IN_REPORT_ORDER」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:180`）。
- 报告次序：「pub const IN_REPORT_ORDER: [WrittenStructureKind; 15] = [」（`crates/singlefs-core/src/write_accounting.rs:38`），其中第 45–47 行依次是 `DeletedInodesTreeNode`、`TombstoneTreeNode`、`LineageTreeNode`，夹在第 44 行 `AccountingTreeNode` 与第 48 行 `CentralMappingTreeNode` 之间。
- 名字：「WrittenStructureKind::DeletedInodesTreeNode => "deleted_inodes_tree_node",」（`crates/singlefs-core/src/write_accounting.rs:104`）、「WrittenStructureKind::TombstoneTreeNode => "tombstone_tree_node",」（`crates/singlefs-core/src/write_accounting.rs:105`）、「WrittenStructureKind::LineageTreeNode => "lineage_tree_node",」（`crates/singlefs-core/src/write_accounting.rs:106`）。
- 单测期望串：「deleted_inodes_tree_node_write_calls=0 deleted_inodes_tree_node_written_bytes=0」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1980`）、第 1981 行墓碑树、「lineage_tree_node_write_calls=0 lineage_tree_node_written_bytes=0」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1982`），在第 1979 行记账树与第 1983 行中央映射树之间。
- 机器核（`model/tools/kinds.py`，原样输出末行）：

```
逐个相同、次序相同： True
```

  同一次输出还列了期望串里的 15 个种类与 `IN_REPORT_ORDER` 经 `report_name` 的 15 个名字，两串相等。合计那一段没动：`write_calls=3 written_bytes=8704`（2 × 4096 + 512）与新加三种各 0 一致。

### 3. 这两处之外，同一文件的别的内联单测有没有对着旧值

判定：没有对着旧值的断言；有一处文档注释还写着旧值。

- 该文件共 17 条 `#[test]`（我用 `grep -n '#\[test\]'` 数出 17 处）。我细读了 7 条（第 1796、1966、1993、2450、2487、2577、3066 行起的那几条），其余 10 条用 `awk` 加 `grep` 扫了带数字的断言行（扫出的数是 txg、窗口条数、`matches=true`、设备一层计数、冷重开之后整池第 3、53 次写（第 2570 行注释写明从重开起数，与 mkfs 无关）之类），没有一条出现 mkfs 步数或这三种写的字样。对着这两样东西的只有 `publish_writes_line_…`（种类串）和 `the_boundary_marked_…`（五段数）。已读的几条里，「28+2+1+2」段形（第 1934 行）与窗口计数都从新池新建文件之后算起，不含 mkfs。17 条全绿靠主 agent 的日志（上面第 1 节），我没复跑。
- 旧值残留：钉值单测的文档注释还写「钉住该文件那五处断言的操作数：23、2、20、35、57」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2446`），后面括号里解释 18 → 20、33 → 35、57，没提 mkfs 23 → 25。断言本身是对的（第 2476 行 `vec![25, 2, 20, 35, 57],`），注释落后一拍。同一段注释还写「（该文件第 187–188 行）」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2444`），`warm_up_operation_count` 在 harness 文件里今天是第 189、201 行，这一处落后是改动之前就有的。
- 变异表里点名这个单测的行 1126 名字写「又靠流尾往回数 23 步」（`crates/mutations.tsv:1126`）：23 是那条变异的替换文里的数字（取流尾 23 步当事务），不是 mkfs 的数；钉到 25 之后这条变异仍然红（切出来 [25, 2, 32, 23, 57]，与 [25, 2, 20, 35, 57] 不等，我读码推的，没跑）。

### 4. 什么现象会推翻 B2

- `new_pool_file_creation_on_device` 的 17 条内联单测在主工作区有一条红，或 harness 的 `new_pool_file_creation_publish` 有一条红。
- `kinds.py` 的最后一行不是 True（换了 `IN_REPORT_ORDER` 次序或名字之后）。
- 有第三个对着 mkfs 步数的断言，没跟着改（我只在该文件和 harness 那份里找过，见「没做什么」）。

## B3　image.rs 注释

判定：一致。

- 改后的两行：「/// I-4.1：所选根（重放之后那一版）指向的树全可遍历；checker 自己按 D23（journal 的角色与格式） 已定项 14 的规则只读重放（`walk` 的」（`crates/singlefs-checker/src/image.rs:211`）、
  「/// `replay_the_next_mount_would_do`），一条记录都不施加时报不适用，前缀停在点名单元验证不过（起点含介质损坏）时报不适用。」（`crates/singlefs-checker/src/image.rs:212`）。
- 名字存在，且正是 I-4.1 用的那一个：「fn replay_the_next_mount_would_do<」（`crates/singlefs-checker/src/walk.rs:8878`），I-4.1 的判法直接调它：「match replay_the_next_mount_would_do(」（`crates/singlefs-checker/src/walk.rs:9011`，另一处调用在第 6141 行，是 `version_the_next_mount_applies_first` 里）。旧名 `version_the_next_mount_applies` 在 `crates/` 的 `.rs` 与 `.wgsl` 里整词命中 0 次（我用 `grep -rnw` 数的）。`walk.rs` 里确有 `fn version_the_next_mount_applies_first(`（`crates/singlefs-checker/src/walk.rs:6134`），它包着前者，但 I-4.1 的判法没经过它，所以 I-4.1 的注释指向 `replay_…` 才对。
- 说的行为对得上判法：
  - 一条记录都不施加：「None => ReplayTheNextMountWouldDo::NothingApplied,」（`crates/singlefs-checker/src/walk.rs:8984`），I-4.1 判法对它报不适用：「ReplayTheNextMountWouldDo::NothingApplied => judgements.not_applicable(」（`crates/singlefs-checker/src/walk.rs:9033`）。
  - 前缀停在点名单元验证不过：「if !named_units_of_the_record_verify_on_both_copies(reader, record) {」（`crates/singlefs-checker/src/walk.rs:8962`）返回 `StoppedAtANamedUnitThatDoesNotVerify`，I-4.1 判法对它报不适用，理由字串「"重放前缀停在一条点名单元两份没都验过的记录上：起点含介质损坏，不在 I-4.1 判的合法镜像里",」（`crates/singlefs-checker/src/walk.rs:9030`）。
- kb 对得上：I-4.1 一行写「起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用」（`.claude/kb/invariants.md:177`）、「所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））」（`.claude/kb/invariants.md:177`），实现栏里用的函数名也是「R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`）」（`.claude/kb/invariants.md:177`）。
- 不完整但不错：I-4.1 判法有四种不适用，判法处自己的文档注释列全了「一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过（起点含介质损坏）都报不适用。」（`crates/singlefs-checker/src/walk.rs:9001`）。image.rs 的注释只写了其中两种。改前也只写这两种，这一批没有让它变错，也没有补全。
- 这一批改的是文档注释，没有 `[`…`]` 形式的 rustdoc 链接，改名不会让 `cargo doc` 多出断链（我读码推的，没跑 `cargo doc`）。

推翻 B3 的现象：`walk.rs` 里 I-4.1 判法不再直接调 `replay_the_next_mount_would_do`（比如改成经 `version_the_next_mount_applies_first`），或 `NothingApplied`、`StoppedAtANamedUnitThatDoesNotVerify` 两支不再报不适用。

## B4　变异行

判定：两行本身一致（锚点、测试名、改回去红）；罩不住的地方有三处，列在后面，能补的给了候选行。

### 1. 两行锚点各恰好命中一次

- 行 3383：「E165 全表对照只取 r3 一档（J8 五行无对照）」（`crates/mutations.tsv:3383`）；行 3407：「E165 全表对照不取两份表都取的那一档（T_table、T_row 无对照）」（`crates/mutations.tsv:3407`）。两行都是六段（`awk -F'\t'` 数 `NF` 得 6）。
- 把第三段的 `\n` 还原成换行后，在 e165 装置源文件里数命中次数（`model/tools/anchors.py`，原样输出，冻结副本）：

```
test fn lines 18274 18324
row 3383 anchor count 1 lines 18301 test every_criterion_has_a_positive_control_and_every_control_is_red args -p singlefs-checker-tier --bin e165_replay_bounded_enumeration
row 3407 anchor count 1 lines 18308 test every_criterion_has_a_positive_control_and_every_control_is_red args -p singlefs-checker-tier --bin e165_replay_bounded_enumeration
```

  同一个脚本对仓根（主工作区）跑，两行也各命中一次、行号相同。两个锚点都落在这条单测函数体里（第 18301–18303、18308–18311 行），分别改的是第二趟的档（`r5` → `r3`）和第三趟的建表法（`Both` → `Direct`）。
- 主 agent 报「`crates-mutation-rows.py static-check` 整表 `CHECKED 3405 322 10`」我没复跑（那是门禁 code-source-discipline 的一格，子 agent 不跑）。
- 这两行在 3495 行的表里的位置是第 3383 与 3407 行，夹在别的线的第 10z 批各块之间，不在表尾。判法按行读，位置不影响。

### 2. 点名的测试筛得到

- 第六段 `every_criterion_has_a_positive_control_and_every_control_is_red` 在装置里只有一处定义：「fn every_criterion_has_a_positive_control_and_every_control_is_red() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18274`）。已有的两行 1462、1625 也用同一个名字与同一串参数「E165 阳性对照循环只跑判据表第一行」（`crates/mutations.tsv:1462`）、「E165 r2 阳性对照表里删掉 J7c 那一行」（`crates/mutations.tsv:1625`），是同一个筛法。
- 主 agent 的 `prove-red.sh` 日志：「✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了」（`~/.local/share/singlefs/work/e165-control-test-fix/prove-red.log:3`，我读过，没复跑）。它引的每条逐次日志目录 `copy/prove-red-logs/` 现在已不在，所以「红在哪句断言上」我没法从日志核。

### 3. 改回去红在断言上（推演，不是复跑）

`model/tools/coverage.py` 末尾的推演（原样输出）：

```
旧测试 [r3]                                         没有读数的行数=6 第一条=['J8aFieldsFromTheNewRootSegment'] 全部=[…]
行 3383 [r3, r3, r3+Both]（r5 那趟改回 r3）             没有读数的行数=4 第一条=['J8aFieldsFromTheNewRootSegment'] 全部=['J8aFieldsFromTheNewRootSegment', 'J8aUnmountMarkerFromTheNewRootSegment', 'J8bFilesystemIdentifierOfTheChosenRoot', 'J8cNothingAppliedReturnsTheChosenRoot']
行 3407 [r3, r5, r3]（Both 改回 Direct）              没有读数的行数=2 第一条=['TableComparisonSeesADifferentTable'] 全部=['TableComparisonSeesADifferentTable', 'TableComparisonSeesADifferentRow']
```

即行 3383 改回去后断言「"判据 {} 没有对照的读数",」（第 18317 行）报 J8a；行 3407 改回去后同一条断言报 T_table。两处推演各有一份独立的实际观测对着：根环线日志「判据 J8a 没有对照的读数」（`~/.local/share/singlefs/work/m3-r1/merge-10z-i/checker-quick.log:415`，旧测试的真红）；主 agent 的中间态日志「判据 T_table 没有对照的读数」（`~/.local/share/singlefs/work/e165-control-test-fix/fixed-run.log:31`，当时 J8 已加、第三趟还没加，与行 3407 改回去后的状态相同）。

已有的 1462、1625 在新测试下仍红：1462 把 `Criterion::ALL` 的循环截成第一行，每一趟都只剩 J0，第 18316 行的覆盖断言红；1625 把 J7c 从循环里滤掉，J7c 只被 r3 趟判，三趟里都没有它的读数。读码推的，没跑。

### 4. 三档里的第一档（r3 Direct）被改掉，r1、r2 两段被改坏

- 第一档：没有变异罩着，也造不出能罩着的变异。B1 第 2 节的输出「pass1 是 pass3 的子集: True」，`coverage.py` 推演里「第一趟换成 r2」「第一趟换成 r5」「第一趟整个删掉」三行都是「没有读数的行数=0」。改它的参数，单测仍绿——等价变异，不是变异表该收的东西。
- r2 段：没有变异罩着。四条点名这条单测的行（1462、1625、3383、3407）里，锚点落在 r1、r2 段的一条都没有（`anchors.py` 只报 3383、3407 两行落在函数体里）。把 r2 段的档改成 r3，第 18291 行的「"r2 档不判 J6Qn 的 I-8.12 拆行"」会红（r3 判拆行，读数里出现 `J6QnSplitByI812`），是一条能罩着的变异。候选行（锚点我数过，冻结副本里恰好 1 次，在第 18284 行；没追加进表，`crates/` 不归我写）：

  - 变异名：`E165 r2 档那一段改用 r3 档（该档不判 J6Qn 的 I-8.12 拆行这一条断言红）`
  - 文件：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`
  - 原文：`            RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),\n            thread_count,\n        );\n        assert!(\n            r2_readings`
  - 替换文：同上，把 `r2(` 换成 `r3(`
  - 参数：`-p singlefs-checker-tier --bin e165_replay_bounded_enumeration`；测试名：`every_criterion_has_a_positive_control_and_every_control_is_red`
- r1 段：没有变异罩着，也罩不住。r1 段只断言「每条读数都红」（第 18279 行 `assert!(reading.is_red_as_required, "r1 档对照没红：{reading:?}");`），把 `RunProfile::R1` 换成 r3，读数照样全红，单测仍绿。要罩住它得在单测里先加一句 r1 档读数的内容断言（例如 r1 档读数里没有 J6Qn、J7 行），这是改单测，不是补变异；列在这里供主 agent 取舍。

### 5. nff 两处钉值有没有变异罩着、要不要补

- 25：没有专属变异。点名 nff 这条钉值单测的只有行 1126（「new_pool_file_creation_paths 的暖机 / 事务分界又靠流尾往回数 23 步」，`crates/mutations.tsv:1126`），它改的是切分点，不是 mkfs 步数；钉到 25 之后它仍红（见 B2 第 3 节）。harness 侧有一条罩 mkfs 步数数法的变异：「实审 B3a-2 第 1 条：录制器把连着的屏障不分设备并成一道（mkfs 的操作数退回 21）」点名 `recorded_stream_matches_the_registered_mkfs_segment_sequence`（`crates/mutations.tsv:928`），改的是录制器按设备记屏障；C651 那两步整段清零（23 → 25 多出来的）有没有变异罩着，我没查到（行 239、242、243 改的是清环范围，点名别的测试）。nff 里的 25 与 harness 的 25 是同一个量（B2），我的看法是补一条 nff 专属变异价值低，取舍归主 agent。
- 三种写：没有变异罩着新加的三种。现有的行 71、72 都改 `IN_REPORT_ORDER` 里的 `RootSlot`（「增补 1：报数清单把根槽换成重复的 journal 记录（真设备二进制的结果行跟着错）」，`crates/mutations.tsv:72`）。新三种的次序只有这条 nff 单测在管：核心层那条测试只验每种恰好一笔与和等于合计（`crates/singlefs-core/src/write_accounting.rs` 第 301–332 行，我读过），harness 两个用到 `IN_REPORT_ORDER` 的测试按种类取值或求和（`write_accounting_per_structure.rs` 第 84–93 行、`deletion_scenario_writes.rs` 第 101–112 行，读过）。所以补一条是值的：墓碑树与谱系树在清单里对调，编译不变、只有 nff 这条单测红。候选行（锚点数过，`write_accounting.rs` 第 46 行起恰好 1 次）：

  - 变异名：`增补 1：报数清单把墓碑树与谱系树对调（真设备二进制的结果行次序跟着错）`
  - 文件：`crates/singlefs-core/src/write_accounting.rs`
  - 原文：`        WrittenStructureKind::TombstoneTreeNode,\n        WrittenStructureKind::LineageTreeNode,`；替换文：两行对调
  - 参数：`-p singlefs-checker-tier --bin new_pool_file_creation_on_device -- publish_writes_line`（同行 72）；测试名：`publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros`
  - 这条行我只推演了它会红（nff 期望串里墓碑树在谱系树之前），没跑；`e158_root_choice_repair.rs` 第 4865 行也按这个次序出行，它的单测我没看，所以「只有 nff 这条单测红」是下限不是全称。

### 6. 什么现象会推翻 B4

- 两行里任一行的锚点在主工作区命中次数不是 1（`anchors.py` 对仓根跑）。
- `prove-red.sh` 对这两行不是「抓到」（主 agent 报的是抓到）。
- 把候选的 r2 变异（第 4 节）加进表后 `prove-red.sh` 不抓到：说明我对第 18291 行断言的推演错了。

## 要主 agent 跑的命令

没有必须由主 agent 跑才能下判定的命令。下面三条是复核用的，按需要：

1. 复跑我的三个只读脚本加 harness 档那一条（仓根，不是重型测试；cargo 经内存包装 8G、线程 15）：
   `bash research/prompts/m3-formal-code-b37-r1-sonnet-model/tools/rerun.sh ~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree harness`
   看三处：`三趟都不判的行: []`、`逐个相同、次序相同： True`、末段 `test result: ok. 9 passed; 0 failed`。
2. 若采纳 B4 第 4、5 节的两条候选行：追加进 `crates/mutations.tsv` 后照 `~/.local/share/singlefs/work/e165-control-test-fix/` 那次的做法跑 `prove-red.sh`（checker 档，我不跑），看是不是「抓到」。
3. B2 第 3 节那处落后的文档注释（nff 文件第 2444–2448 行）改不改，归主 agent。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| B1 恰好罩住 | 一致 | 三趟并起来判 `Criterion::ALL` 35 行，没有一行三趟都不判；旧测试缺 6 行（J8 四行、T 两行） |
| B1 在判它的档里红 | 一致 | 循环先按族、再按 `is_judged_under` 滤，读数只来自判它的那一趟；红断言对并集里每条读数下 |
| B1 档与 `--table` 取法 | 部分对得上 | r3 Direct、r5 Direct 对得上驱动；r3 加 Both 驱动里没有（正式跑的 `--table both` 全配 `--profile r2`），T 的判法只看建表法，读数不受影响；条款没写配哪个档 |
| B1 r1、r2 两段 | 一致 | 两段代码行改动的两个块都没碰到；r2 的不拆行断言仍成立 |
| B1 三档都不判的行 | 没有 | 旁注：第一趟是第三趟的子集；r2 读数不逐条验红；单测不验「读数出自判它的档」，靠共用谓词 |
| B2 25 | 一致 | 同一个量、同一条路、同一段切法；harness 档 9 条测试我跑了，全绿 |
| B2 三种写 | 一致 | 名字、次序与 `IN_REPORT_ORDER` 逐个相同（脚本核） |
| B2 别的内联单测 | 没有对着旧值的断言 | 钉值单测的文档注释还写 23，落后一拍 |
| B3 | 一致 | 新名字存在，I-4.1 的判法直接调它；两种不适用与 `walk.rs`、kb 对得上；只列了四种里的两种 |
| B4 锚点与测试名 | 一致 | 两行各命中一次，落在单测函数体里；测试名与已有两行同一个 |
| B4 改回去红在断言上 | 推演一致 | 3383 报 J8a、3407 报 T_table；逐次日志已不在，靠旧测试真红与中间态日志对着 |
| B4 第一档被改 | 没有变异罩着，也造不出 | 等价变异 |
| B4 r1、r2 两段 | 没有变异罩着 | r2 能罩（候选行给了）；r1 只断言全红，换档不红，罩不住 |
| B4 nff 钉值 | 25 无专属变异、三种写无变异 | 三种写的次序只有这条 nff 单测管，候选对调变异给了 |

## 没做什么

- 没跑 checker 档任何测试（E165 那条单测、nff 17 条、`crates-mutation-rows.py static-check`、`prove-red.sh`）；这些只引了主 agent 的日志。harness 档 `new_pool_file_creation_publish` 跑了（9 通过）。
- 没逐条核每个阳性对照读数的 `criterion` 标签；只读了各臂把 `criterion` 传下去的写法（`any_copy…` 等四条多盘臂各用自己的行，`c8*`、`j7a`、`unmount_marker`、`i812` 用传入的或固定的行）。
- 没实跑任何变异（含候选行）；B4 的「红在断言上」是推演。
- 没读 nff 17 条里的另外 10 条的完整函数体，只扫了带数字的断言行；没查冻结副本以外的地方（`crates/` 其余文件、kb 其他条）还有没有对着 mkfs 23 步的断言。
- 没追加或改 `crates/mutations.tsv`、`crates/` 下任何文件、kb；候选变异行只写在本报告里。
- 没判别 B1–B4 以外的格；同一文件里别的线的块（例：`image.rs` 第 282 行、根环线与删除线的改动）不判。
- 编译目录：harness 档那次用的 `CARGO_TARGET_DIR=~/.local/share/singlefs/work/m3b37-sonnet/target` 已删（`rm -rf`），草稿目录里留 `build1.log`、`harness-run.log`、`anchors.py` 与 `model/`；仓里只有本报告与 `research/prompts/m3-formal-code-b37-r1-sonnet-model/`。
- 没有开线程以外的并发；全程 `nice -n 19`，编译与测试线程上限 15。启动时 `ps` 看到别的会话在跑 `cargo test --release -p singlefs-checker-tier --lib --tests`（31 线程），没有性能测量进程（qemu-system、vm-bench、e152、fio 都没有）；我没等它，照规矩加 `nice -n 19` 起，harness 编译 10.7 秒。

## 交回前的 cite-check

命令：`python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b37-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree --root /home/fy5090/code/singlefs`，末行原样：

```
  ✓ 核了 65 处引文，对上 65 处，没判 0 处（逐处列在上面）
```
