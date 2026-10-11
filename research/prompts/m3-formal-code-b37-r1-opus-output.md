# 第 37 批代码轮第一轮：云端攻方腿（Opus）报告（2026-10-09）

轮名 m3-formal-code-b37-r1；攻 B1–B4，重点 B1。代码引文读冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree/`，写 `crates/…` 相对路径与那棵树里的行号（开工时这几份文件与主工作区逐字相同，sha256 前 12 位两边一致）。没读禁读清单里的文件。

## 复跑

模型只建「哪一档跑哪几条对照」这一层的逻辑，纯 Python，不编 Rust、不起 cargo、不跑任何 checker 档测试：

```text
cd /home/fy5090/code/singlefs/research/prompts/m3-formal-code-b37-r1-opus-model
bash tools/rerun.sh            # 默认根 ~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree
242a2471e852cdf4aabb911b41f0eea5cf036fc98333189597ab807871e114b5  b1_profile_union.py
ab1abddeb5dd4bc5b2d219dd9cd0fdd09659c6ac57529954e5abe242ac678716  tools/rerun.sh
```

模型先核源码里它依赖的 12 行原文还在（对不上退 3、模型作废），再从单测正文里抽出每一处 `positive_control_readings` 调用（族、档、并进哪一份），所以单测或变异行改了，模型跟着变。限度：对照读数一律当「在判它的那一档里算就红」，只管覆盖、不管数值。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| B1 并集会不会让没判的读数顶替 | **打中**（B1-甲） | 并集只问「某一档里有这条判据的读数」，不问「判它的每一档里都有」；`is_judged_under` 只管对照、不管判定，删掉它在某一档的一条臂，判定照判、对照没了，另两档的读数把空缺顶上，单测照过。模型扫 230 个「只删不加」的写法：单测红 36，放过且某条正式跑出现「判了没对照」177 |
| B1 r1、r2 两段原样 | **打中**（B1-乙，原有、不是这一批引入的） | r1 段只逐个断言读数红，读数为空照过；r2 段的读数一个都不判红，只判「没有拆行」 |
| B1 三档与正式跑对不对得上 | **打中**（B1-丙，口径） | 正式跑带 `--table both` 的五条驱动全是 r2；单测的第三档 r3+both 没有一条正式跑用它；r3 直接建表那一档有的读数 r3+both 全有，第一档多余 |
| B1 r2「不判拆行」那条断言 | 没打中 | 仍成立，也仍会红 |
| B1 有没有一行三档都不判 | 没打中 | 三档并起来 35/35；只要 r5 与 r3+both 两档就罩全 |
| B1 交叉标签顶替（A 条判据的对照函数给 B 条记读数） | 没打中 | 每条对照函数记的判据都是它自己那一臂的 |
| B2 25 是不是同一个量 | 没打中 | 同一个 `make_filesystem`、同一组几何、切在同一处；清零步数与盘大小无关 |
| B2 同一文件别处的旧值 | **打中**（文字） | 被改的那条单测自己的文档注释还写「23、2、20、35、57」，两处「该文件第 187–188 行」也已不对 |
| B2 三种写的名字与次序 | 没打中 | 与 `IN_REPORT_ORDER`、`report_name` 逐个相同 |
| B3 改后的函数名 | 没打中 | 名字对；I-4.1 那份只读重放就是它 |
| B3 那一句说的行为 | **打中**（说法不全，原有） | 判法报不适用的有四种，注释只写两种 |
| B4 两行锚点 / 筛得到 / 红在断言上 | 没打中 | 各命中 1 次，施加后缺的正是 J8 四条、T 两条，红在「判据 {} 没有对照的读数」 |
| B4 第一档、r1、r2 段被改坏有没有变异罩着 | **打中** | 第一档换成 r5 或 r3+both 单测照过（第一档多余，任何变异都罩不着）；r1、r2 段的弱点是单测自己的写法，变异表这一类罩不住；B1-甲那 177 种写法变异表里一行都没有 |
| B4 nff 两处钉值有没有变异罩着 | **打中**（次序那一处） | 报数次序只有 nff 那条单测钉着，core 自己那条单测换次序照绿，变异表里没有换次序的行 |

## B1　E165 全表对照

### B1-甲（打中）：并集把「判它的那一档没有对照」藏起来

**放过它的那几行。** 单测把三档的读数并成一份，只问每条判据在并集里有没有一个读数：

- 「let mut readings = positive_control_readings(」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18296`），之后两次 `readings.extend`，第三档带「.with_table_source(RecordTableSource::Both),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18309`）；
- 覆盖只判「.any(|reading| reading.criterion == *criterion),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18316`），`ControlReading` 不带档，并起来以后分不出读数来自哪一档。

**为什么这会漏。** 一条判据「有没有对照」由 `is_judged_under` 定（「if !criterion.is_judged_under(run_profile) {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7761`），它只在 `positive_control_readings` 里用；正式跑里「判不判」不经它，域的判定行直接按族与 judges_* 打：基础那 18 条无条件判（「j0_violations={} failed_states={} j1_violations={}」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8139`），J6Qn、J7、J8、T 各看「if run_profile.judges_j6qn() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8207`）、「if run_profile.judges_j7() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8218`）、「if run_profile.judges_j8() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8237`）、「if run_profile.compares_the_two_record_tables() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8260`）。两处各管一半：`is_judged_under` 在某一档上少了一条臂，那一档照判、不跑对照；运行时 V2 只对「跑了而没红」的读数报作废（「if !reading.is_red_as_required {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17779`），没跑的对照不报。单测这时该红，但另两档里那条判据的读数把并集补齐了。

**具体坏法（一行改动，单测放过）。** 把基础那一臂「| Self::ALimit => true,」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6248`）改成 `=> !run_profile.judges_j8()`：

1. r5 档不再给 J0…A_limit 这 18 条跑对照；`main` 里照样判它们（第 8139 行那一行与对抗一族那一行都不看档）。
2. 单测 r1 段：r1 档 judges_j8 为假，18 条照跑、照红，过。r2 段：没有拆行，过。
3. 并集：r3、r3+both 两档里这 18 条都有读数，`any` 成立；三档读数全红。单测过。
4. 正式跑 `driver_e165_r5_segment3_legal` 等 4 条驱动（replay.sh 第 854 行起）判了 J0…IA2、A0…A_limit，一条对照都没跑，V2 不触发，`controls_not_red=0`。

同形的还有（模型逐个跑过）：J6Qn/多盘那一臂「| Self::MultiDiskAppliedSetAndLandedVersion => run_profile.judges_j6qn(),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6230`）改成 `judges_j8()`，r2、r3 档的 J6Qn 与多盘四条判了没对照（6 条驱动）；T 那一臂加 `&& splits_j6qn_by_i812()`，r2+both 的 T_table、T_row 判了没对照（5 条驱动，正式跑带 `--table both` 的全部），而单测里唯一的 T 读数来自 r3+both。T 两条另有一条单测直接调 `control_readings_of`（「fn the_table_comparison_controls_read_2989_on_the_tables_and_3110_on_the_rows() {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18197`），绕过 `is_judged_under`，这一坏法它也看不见。

模型输出（`bash tools/rerun.sh` 原样，节选第 17–19 行与「点名的几种写法」五行）：

```text
== r5 ∪ r3+both 罩全 35/35；r3 直接建表那一档有、而 r3+both 没有的 = []
== 只删不加的变异：共 230 个；单测红 36；单测放过且某条正式跑有判据没对照 177；单测放过、正式跑也不缺 17
== 单测放过且正式跑有缺口的 177 个里，被「每档各自对照集钉死」的改法判红的 177 个（正式跑有缺口的变异全体里判红 210 个）
== 点名的几种写法
  基础臂 `=> true` → `=> !run_profile.judges_j8()`：单测 过；正式跑缺口 4 条驱动，第一条 driver_e165_r5_segment3_legal 缺 ['J0', 'J1', 'J2', 'J3', 'J4', 'J5a', 'J5b', 'J5a_vs_J5b', 'J6', 'IA1', 'IA2']
  基础臂 `=> true` → `=> !run_profile.judges_j7()`：单测 过；正式跑缺口 5 条驱动，第一条 driver_e165_segment2 (r2, segment2, L+A) 缺 ['J0', 'J1', 'J2', 'J3', 'J4', 'J5a', 'J5b', 'J5a_vs_J5b', 'J6', 'IA1', 'IA2', 'A0', 'AS_whole_table', 'AS_consecutive_gap_continues', 'AS_consecutive_gap_stops', 'A_mutual', 'A_structure', 'A_limit']
  J6Qn/多盘臂 `judges_j6qn()` → `judges_j8()`：单测 过；正式跑缺口 6 条驱动，第一条 driver_e165_segment2 (r2, segment2, L+A) 缺 ['J6Qn']
  T 臂 `compares…()` → `compares…() && splits_j6qn_by_i812()`：单测 过；正式跑缺口 5 条驱动，第一条 driver_e165_r2_segment4_table_both_legal 缺 ['T_table', 'T_row']
  J7 臂 `judges_j7()` → `judges_j7() && compares…()`：单测 过；正式跑缺口 1 条驱动，第一条 driver_e165_segment2 (r2, segment2, L+A) 缺 ['J7a_instance_table', 'J7a_allocation_record_tree_root', 'J7a_filesystem_identifier', 'J7b', 'J7c']
```

「只删不加」= 六条臂里某一条在 7 个可达档类（r1、r2、r2+both、r3、r3+both、r5、r5+both；r1 不认 `--table`，「"r1" if table_source_label.is_some() || disk_order_label.is_some() => {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17519`）的某个非空子集上从判变成不判，逐个子集枚举。正式跑取 replay.sh 里起 e165 的 16 条驱动（root_choice 两族不在这张判据表里）。只建删、不建加：加出来的对照在不判它的档里算出什么数，模型说不了。

**谁还能抓到。** 这条单测以外没有别的地方调 `positive_control_readings`（`grep -n 'positive_control_readings('` 只命中定义、`main` 第 17769 行与这条单测）；启动自检不调它。上面这些坏法要到 replay.sh 整份逐字比正式产物（`name=control` 行少了）才红，那是重型的实验复跑，不是这条单测与它的变异门。

**四问。**

1. 分不分辨臂：分辨「并集覆盖」与「每档各自覆盖」——177 个放过的写法，每档各自钉死的判法全部判红（上面第三行输出）。不分辨这一批改前与改后：改前只取 r3 一档，同样的写法照样放过（改前那条单测本来就红着）。
2. 被判的东西当时看不看得到：看得到。三档的读数在 `extend` 之前是三份各自的 `Vec`，档就在手边；被并掉了。期望的「每档该有哪几条」要取登记的表、写死成字面，不能再从 `is_judged_under` 现算（那是被改坏的东西）。
3. 满足的是判据字面的哪一句：正文第三节 B1 那一格「每一行的对照都在**判它的那一档**里红（一行在 A 档有读数、在 B 档才判，并起来会不会让一条没判的读数顶替）」——这里 B 档判、只有 A 档有读数，并集顶替。条款一侧：原登记 5.4「对照那段循环遍历判据表的每一行」、r5 登记 5.4 标题「每条判据都跑」；V2 只管跑了不红，管不到没跑。
4. 跑前条款给的改法：正文第五节没给改法，不适用。我提的改法见本报告「改法与各修哪一格」。

**推翻条件。** 有一条别的测试（不经 `is_judged_under`、按档写死期望集）在上面任一写法下判红；或 `main` 的判定其实也经 `is_judged_under`（那样少一条臂会同时少判，变成另一种错，不再是「判了没对照」）。

### B1-乙（打中，原有）：r1 段空集照过，r2 段读数不判红

- r1 段：「let r1_readings = positive_control_readings(&families, RunProfile::R1, thread_count);」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18277`）之后只有「assert!(reading.is_red_as_required, "r1 档对照没红：{reading:?}");」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18279`），没有「r1 档该有哪 18 条」。坏法：基础那一臂改成 `=> run_profile.judges_j6qn()`，r1 档一条对照都不跑，`r1_readings` 为空，循环不进、照过；并集里 r3/r5/r3+both 照有那 18 条，照过。正式跑 r1 三条驱动（`driver_e165`、`driver_e165_light`、`driver_e165_adversarial`）判了基础 18 条没对照。模型「例：臂 base 在 ['r1'] 上不判 ⇒ 单测过」那一行是它。
- r2 段：r2 档的读数只拿来判「.all(|reading| reading.criterion != Criterion::J6QnSplitByI812),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18290`），红不红不判、缺不缺不判。r2 是正式跑的档：replay.sh 第 699–880 行里直接起装置带 `--profile r2` 的有 6 条（`awk` 截段后 `grep -c` 数的：多盘、`--table both` 三族、反盘次序两条），另有 `driver_e165_segment2` 与几条杀伤界驱动经 `e165-replay-mutation-kill-bounds.sh` 以 r2 档起基线。今天 r2 与 r3 的对照数值只在拆行那一条上可能不同（`splits_j6qn_by_i812` 只影响对抗一族的 I-8.12 计数，「CriteriaSet::RegisteredBeforeR2 | CriteriaSet::WithJ6QnAndJ7 => false,」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:370`），所以「只在 r2 上不红」的坏法今天要先加一个看档的分支才造得出——这一半是推的、没造出具体的一行。缺的那一半（r2 上少跑）就是 B1-甲 的 r2 例子，造得出。
- r2 段的读数是整份算出来的（三族全跑），算了不判，花的挂钟与一档 r3 相当。

### B1-丙（打中，口径）：三档与正式跑的档对不上

- 正式跑带 `--table both` 的全是 r2：replay.sh 第 740 行起的 `driver_e165_r2_segment4_table_both_legal` 等三条与反盘次序两条（例：「--bounds segment1 --family adversarial --profile r2 --table both」，`scripts/replay.sh:744`）。没有一条正式跑用 r3+both。改动的注释说 T 两条「正式跑带 `--table both`」，没说正式跑是 r2。
- 正式跑的 J7 档是 r2（第二段基线 L+A、`--table both` 合法一族），r3 档只有对抗一族一条正式跑（`driver_e165_r3_segment1_adversarial`，replay.sh 第 762 行），J7 是合法一族的判据——单测里 J7 的读数全来自没有正式跑的 r3/L、r3+both/L。
- 第一档多余：非 T 的对照在跑之前一律退回直接建表（「.with_table_source(RecordTableSource::Direct)」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6302`），所以 r3+both 的读数 = r3 的读数再加 T 两条。模型：「r3 直接建表那一档有、而 r3+both 没有的 = []」，「r5 ∪ r3+both 罩全 35/35」。
- 今天数值上没有害处：T 两条在 r2+both 上的读数另有一条单测钉着（第 18197 行那条，取 r2+both），r3+both 与 r2+both 在 L 域上的 T 读数按代码读是同一个数（推的，没跑）。
- 顺带：`RecordTableSource` 的文档注释还写 `--table` 「只在 r2 档有」（第 205 行），而参数解析里 r3、r4、r5 都认（「"r3" | "r4" => Ok((」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17529`）。这条注释不在这一批的改动里。

### B1 没打中的

- **交叉标签顶替**：逐个读了 `control_readings_of` 的每一条分支与它调的每个对照函数，每条读数记的判据都是进来的那一条（固定写死的几处——`any_copy`、三个 `mirrored_*`、`j6qn`、`i812_split`、`c8a`–`c8d`——写死的也正是 match 进来的那一臂）。一条判据的读数只会出自判它的那一档，「A 条的对照给 B 条记读数」不成立。
- **三档都不判的行**：没有。35 条判据三档并起来都有读数（模型「原样：单测 过；缺 []」）。正文说「J8 五行」，判据表里 J8 是四条（J8a 一条罩实例表指针与分配记录树根两格，加 `J8a_unmount_marker`、`J8b`、`J8c`）；r5 登记 5.4 那张对照表也是 C8a–C8d 四条加单测 C8e，「J8 无出处」只报不判。所以「J8 五行」是变异行与注释的叫法，不是少了一条。
- **反奇偶（`--marker-parity odd`）**：正式跑有 r5 odd 两条，单测三档都取 even，没跑 odd 分支；但 odd 分支另有单测逐条钉（C8a、C8c 两种奇偶循环，C8b、C8d 各取 odd，r2 odd 有 `check_the_odd_marker_parity_turns_the_unmount_marker_controls`），漏不到。
- **反盘次序、`--table scan`**：非 T 对照在跑之前把盘次序退回正常、建表退回直接（第 6300–6312 行那个函数），读数与默认档逐字相同；`--table scan` 没有正式跑，T 两条在 scan 档不判、也不跑对照，两边一致。
- **r1 档带多盘一族**：`main` 拒（「if selection.families.contains(&Family::MultiDisk) && !run_profile.judges_j6qn() {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17696`），造不出 r1 判多盘而没对照。
- **r2「不判拆行」那条断言**：成立；`splits_j6qn_by_i812` 给 r2 加上判，r2 段会红（判据真会动）。

## B2　nff 钉值

### 25 是不是同一个量（没打中）

两边量的是同一段：

- harness：「let mkfs_operation_count = stream.operations().len();」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:165`）紧跟 `make_filesystem` 之后，「let mkfs_operations = &operations[..pool.mkfs_operation_count];」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:332`），钉「(25, "14+1+1+1+4".to_string(), 16402)」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:356`）。
- nff：经 `run_new_pool_file_creation`，「let mkfs_operation_count = stream.operations().len();」（`crates/singlefs-harness/src/scenario.rs:173`）同样紧跟 `make_filesystem`（「let genesis = make_filesystem(parameters, devices).map_err(|error| {」，`crates/singlefs-harness/src/scenario.rs:167`），切段「("mkfs", &operations[..mkfs_operation_count]),」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1355`）。
- 参数：nff 用 `e142_parameters(512, 512)`，与 harness 的 `parameters()` 逐字段相同（fsid 同一个常量值、区域设备 [0, 1, 0]、物理块 512、io_min 512、槽距 `slot_spacing_for(512)` = 4096、journal 环默认、根环每区槽数取 mkfs 时那一档）。
- 23 → 25 那两步是 mkfs 每块盘多一次整段清零（「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」，`crates/singlefs-core/src/make_filesystem.rs:557`）。每个区间只调一次 `write_zeroes_at`，步数与盘大小无关，所以 harness 的 4 GiB 文件镜像与 nff 的稀疏内存盘步数相同；录制器都套在最外层。
- 主 agent 已带 `SINGLEFS_HEAVY_TESTS=user-request` 跑过 nff 17 条全绿，25 在 nff 那条路上是量过的。我没跑。

推翻条件：nff 那条单测在冻结副本上红在第一个数；或 `run_new_pool_file_creation` 与 harness `build_pool` 之间有一处 `make_filesystem` 之前的写（今天两边都没有）。

### 同一文件别处的旧值（打中，文字）

被改的那条单测自己的文档注释没跟：

- 「钉住该文件那五处断言的操作数：23、2、20、35、57」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2446`）——断言已是「vec![25, 2, 20, 35, 57],」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2476`）。这段注释还逐段解释了 18 → 20、33 → 35，没有 mkfs 23 → 25 那一句。
- 「（该文件第 187–188 行）」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2444`）与第 1460 行同一句：harness 那份 `warm_up_operation_count` 今天在「let warm_up_operation_count;」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:189`）与「warm_up_operation_count = stream.operations().len();」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:201`）。这两处不是这一批改的，但正文这一格问的就是同一文件里对着旧值的地方。

断言里对着旧值的：主 agent 那次 17 条全绿就排除了（断言对着旧值会红）。我另 grep 了 nff 文件里 21、23、4114、16402、`zero_fill` 这几个 mkfs 相关的数，剩下的 21 一处是 `pool_writes_between` 单测里手造的差（第 2039 行），其余几处是暖机第一次空发布的写数，都与 mkfs 无关。

### 三种写的名字与次序（没打中）

- 次序由「pub const IN_REPORT_ORDER: [WrittenStructureKind; 15] = [」（`crates/singlefs-core/src/write_accounting.rs:38`）定：…、AccountingTreeNode、「WrittenStructureKind::DeletedInodesTreeNode,」（`crates/singlefs-core/src/write_accounting.rs:45`）、TombstoneTreeNode、「WrittenStructureKind::LineageTreeNode,」（`crates/singlefs-core/src/write_accounting.rs:47`）、「WrittenStructureKind::CentralMappingTreeNode,」（`crates/singlefs-core/src/write_accounting.rs:48`）、…。nff 的期望串逐个是「deleted_inodes_tree_node_write_calls=0 deleted_inodes_tree_node_written_bytes=0」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1980`）、tombstone、「lineage_tree_node_write_calls=0 lineage_tree_node_written_bytes=0」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1982`），夹在记账树与中央映射树之间，对得上。
- 名字取 `report_name`（「WrittenStructureKind::DeletedInodesTreeNode => "deleted_inodes_tree_node",」，`crates/singlefs-core/src/write_accounting.rs:104`），对得上。
- 正文引的「`write_accounting.rs` 第 103–107 行 `WrittenStructureKind` 的报告次序」指的是 `report_name` 那几条 match 臂；定次序的是第 38–54 行 `IN_REPORT_ORDER`。两处今天次序相同，引的位置不准、结论不受影响。

## B3　image.rs 注释

- 名字对（没打中）：「fn replay_the_next_mount_would_do<'records>(」（`crates/singlefs-checker/src/walk.rs:8878`）就是 I-4.1 判法 `judge_trees_of_the_replayed_version_are_traversable`（第 9002 行）调的那份只读重放。旧名 `version_the_next_mount_applies` 仓里没有；名字最像的「fn version_the_next_mount_applies_first(」（`crates/singlefs-checker/src/walk.rs:6134`）是 I-3.10 那一版的取法，失败时交 `None`、不报不适用，不是 I-4.1 的，所以没选它是对的。
- 行为那一句说得不全（打中，原有）：注释写「`replay_the_next_mount_would_do`），一条记录都不施加时报不适用，前缀停在点名单元验证不过（起点含介质损坏）时报不适用。」（`crates/singlefs-checker/src/image.rs:212`），只列两种；判法报不适用的有四种，walk.rs 自己的注释写全了：「一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过（起点含介质损坏）都报不适用。」（`crates/singlefs-checker/src/walk.rs:9001`），代码见「ReplayTheNextMountWouldDo::AnchorUndecided => judgements.not_applicable(」（`crates/singlefs-checker/src/walk.rs:9017`）与「ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor => judgements.not_applicable(」（`crates/singlefs-checker/src/walk.rs:9021`）。注释说的两种没错，漏了两种；读注释的人会以为锚点认不定时 I-4.1 判红。这一句不在这一批的 diff 里（diff 只换了函数名）。
- 「报不适用」是判法函数做的，重放函数自己只交结局（`NothingApplied`、`StoppedAtANamedUnitThatDoesNotVerify` 等）。注释的括号挂在「只读重放」上，读成「重放由它做、不适用由 I-4.1 报」是通的；要更直，括号里可以并列点判法函数名。

推翻条件：`judge_trees_of_the_replayed_version_are_traversable` 对 `AnchorUndecided` 或 `ChosenRootTxgHasNoSuccessor` 不报不适用（今天第 9017、9021 行都报）。

## B4　变异行

### 两行新变异（没打中）

模型把变异表里点名这条单测的四行逐条施加在冻结副本的单测正文上，再抽一遍档（`bash tools/rerun.sh` 原样）：

```text
== crates/mutations.tsv 里点名这条单测的行
  第 3383 行「E165 全表对照只取 r3 一档（J8 五行无对照）」：原文命中 1 次；施加后单测 红；缺 ['J8a', 'J8a_unmount_marker', 'J8b', 'J8c']
  第 3407 行「E165 全表对照不取两份表都取的那一档（T_table、T_row 无对照）」：原文命中 1 次；施加后单测 红；缺 ['T_table', 'T_row']
```

（同一段里第 1462、1625 两行旧变异在新单测上同样红，缺全表 / 缺 J7c。）施加后的代码照样编得过（换的只是档的构造式），红在「"判据 {} 没有对照的读数",」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18317`）那一条断言上。筛法列 `-p singlefs-checker-tier --bin e165_replay_bounded_enumeration` 不带 `--` 过滤，跑整个 bin 的单测，与第 1462、1625 行同法。模型只说「覆盖缺了哪几条」，真实二进制红不红要主 agent 跑（见文末命令）；主 agent 已在副本上 `prove-red.sh` 两行都「抓到」。

### 第一档、r1、r2 两段（打中）

- 第一档 r3 直接建表多余，任何变异都罩不着它（`bash tools/rerun.sh` 原样）：

```text
== 单测正文自己改坏
  第一档 r3 直接建表换成 r5：原文命中 1 次；单测 过；缺 []
  第一档 r3 直接建表换成 r3+both：原文命中 1 次；单测 过；缺 []
```

- r1、r2 两段：弱点在单测自己的写法（B1-乙），变异表是「改坏被测代码、要求单测红」，罩不住单测本身少断言。能写成变异行的是 B1-甲 那一类改坏 `is_judged_under` 的写法；变异表里点名这个 bin、锚点落在 `is_judged_under` 六条臂上的行一条都没有（按第 3 列的原文筛，只有第 1625 行锚在调用处，删 J7c 是各档一起删，会红）。按今天的单测，这类行加进去是活的；要等改法落地才杀得掉。

### nff 两处钉值（打中一处）

- 25：没有点名 nff 那条单测的变异；「mkfs 不清 [0, 根环基址)（C651 第一轮判决用户定案 W1 撤回）」（`crates/mutations.tsv:1825`）罩着同一处代码，点名的是 harness 那条单测。nff 那条是 harness 那份的镜像，不补也有人抓；补一行点 nff 那条（23 ≠ 25 会红，推的）可以，但它是 checker 档。
- 报数次序：只有 nff 那条单测钉着。core 自己那条只判「每种一笔」「各种之和等于合计」（「"报数清单漏了一种或重复了一种：各种长度互不相同（2 的幂），和对不上"」，`crates/singlefs-core/src/write_accounting.rs:316`）与名字互不相同（「assert_eq!(names.len(), 15, "结果行的字段名前缀互不相同");」，`crates/singlefs-core/src/write_accounting.rs:332`），把 `IN_REPORT_ORDER` 里两种对调它照绿；全仓钉字段次序的串只在 nff 这一份（`grep -rln tombstone_tree_node crates` 只有 write_accounting.rs 与 nff 两个文件）。变异表里 write_accounting.rs 的行是「把根槽换成重复的 journal 记录」一类（「增补 1：报数清单把根槽换成重复的 journal 记录（真设备二进制的结果行跟着错）」，`crates/mutations.tsv:72`），重复一种 core 也会红；没有一行是换次序。这一批改坏的正是这条串，补一行换次序的变异只能由它杀。建议的行（原文在 write_accounting.rs 命中 1 次，模型外用 python `str.count` 数的；红不红推的、没跑）：

```text
增补：报数次序里已删 inode 树与墓碑树对调（各种之和不变，core 的自检照绿）	crates/singlefs-core/src/write_accounting.rs	        WrittenStructureKind::DeletedInodesTreeNode,\n        WrittenStructureKind::TombstoneTreeNode,\n        WrittenStructureKind::LineageTreeNode,	        WrittenStructureKind::TombstoneTreeNode,\n        WrittenStructureKind::DeletedInodesTreeNode,\n        WrittenStructureKind::LineageTreeNode,	-p singlefs-checker-tier --bin new_pool_file_creation_on_device -- publish_writes_line	publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros
```

## 改法与各修哪一格

都只在我的模型上量过、被攻过零轮。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲：三档读数不并，每档各自断言「对照集 = 按登记写死的那一份」（字面表，不从 `is_judged_under` 现算） | B1-甲 | 量过（模型）：只钉单测今天跑的 r1、r2、r3、r5、r3+both，177 个里判红 170；改钉正式跑用的 r1、r2、r2+both、r3、r5，177 个全红。原样输出见下 |
| 乙：r2 段读数也逐个断言红 | B1-乙 后半 | 推的；今天造不出只在 r2 上不红的具体坏法 |
| 丙：r1 段加「r1 档对照集 = 基础 18 条」 | B1-乙 前半 | 量过（甲 的钉法含 r1，`base 在 ['r1'] 上不判` 那一类判红） |
| 丁：第三档换成 r2+both（正式跑真用的档），去掉第一档 r3 直接建表 | B1-丙 | 推的；并集照样罩全：按模型「每一档」那张表合，r2+both 缺的 J8 四条与拆行 r5 都有，r5 缺的 J7 五条与 T 两条 r2+both 都有 |
| 戊：nff 那条单测的文档注释 23 → 25、两处「第 187–188 行」改成今天的行号 | B2 文字 | 推的（只改注释） |
| 己：image.rs 那一句补上「锚点认不定、所选根 txg 没有后继」两种 | B3 | 推的（只改注释） |
| 庚：补上一节那行换次序的变异 | B4 次序 | 推的；原文命中 1 次是数过的 |
| 辛：B1-甲 那一类改坏 `is_judged_under` 臂的写法补一两行变异（例：「| Self::ALimit => true,」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6248`）换成 `=> !run_profile.judges_j8(),`，原文命中 1 次） | B4 / B1-甲 | 推的；今天的单测下它是活的，甲 落地之后才杀得掉 |

```text
   钉单测今天跑的 r1、r2、r3、r5、r3+both：放过且有缺口的 177 个里判红 170 个
   钉正式跑用的 r1、r2、r2+both、r3、r5：放过且有缺口的 177 个里判红 177 个
```

## 要主 agent 跑的命令

都是 checker 档，这条腿不跑。工作目录取改法所在的副本（不是冻结副本、不是主工作区）；内存上限取 `scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值，线程 15。

1. 坐实 B1-甲 的一个例子（改坏之后单测仍过）：在副本里把 e165 装置第 6248 行「| Self::ALimit => true,」换成 `| Self::ALimit => !run_profile.judges_j8(),`，再跑
   `SINGLEFS_HEAVY_TESTS=user-request bash scripts/run-with-memory-cap.sh <上限> bash scripts/capped.sh 15 cargo test --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- every_criterion_has_a_positive_control_and_every_control_is_red`
   看 `test result:` 一行：`1 passed` 即坐实「单测放过」。同一副本上 `cargo run --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds light --family legal --profile r5`（经同一包装）看 `name=verdict scope=run` 行：`controls_not_red=0`、`void_conditions=0`，而 `name=control criterion=J0` 一行都没有，即坐实「正式跑判了没对照、V2 不报」。
2. 改法 甲 落地后，同一条单测在未改坏的副本上 `1 passed`、在第 1 条那份改坏的副本上 `1 failed`。
3. B4 建议的换次序变异（若采纳）：`bash scripts/prove-red.sh` 照变异表那一行跑，看是不是「抓到」、红在 `publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros`；同时 `-p singlefs-core --lib report_order` 应当仍绿（证明只有 nff 那条杀得掉它）。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| B1 | 交叉标签（一条判据的对照函数给另一条记读数） | `control_readings_of` 全部分支与它调的对照函数，逐个读 |
| B1 | 三档都不判的行 | 35 条判据 × 三档，模型 |
| B1 | 反奇偶、反盘次序、`--table scan`、r1 带多盘 | 读码；反奇偶另找到四条单测与一条 r2 的 check 函数钉着 |
| B1 | 「只加不删」的 `is_judged_under` 写法（某档多跑一条对照） | 没扫：多出来的对照在不判它的档里算出什么数，模型说不了；读码看，r1 档多跑 J7b/J7c 或 J8 会撞 `unreachable!`（第 7743、7472 行），多跑 J7a 读数是 0 不等于 762，单测都会红 |
| B2 | 25 与 harness 不是同一个量 | 两条路的 mkfs 调用、切段处、参数逐字段、清零步数与盘大小的关系 |
| B2 | 断言里还有对着旧值的 | 交主 agent 那次 17 条全绿排除；另 grep 了 mkfs 相关的数 |
| B3 | 改后的名字指错函数 | walk.rs 里名字相近的 `version_the_next_mount_applies_first` 对照过 |
| B4 | 两行新变异锚点多命中、编不过、红在别处 | 原文命中数用 `str.count` 数；施加后的档构造式与原来同型 |

## 这条腿自己的限度

- 一次抽样；「没打中」各格不支撑「没有」（`.claude/rules/three-way-inference.md` 那一节「没打中」一次不算）。
- 模型只管覆盖，不管数值；「正式跑判不判」取的是今天 `main` 的判定行按 judges_* 打这一读法（第 8118–8273 行读出来的），没跑装置。正式跑的驱动列表抄自 replay.sh 第 331–355 行与第 699–880 行，没核杀伤界驱动里每个模式起装置的参数。
- B1-乙 后半（只在 r2 上不红）没造出具体的一行。
- 引文行号全取自冻结副本；开工时主工作区这几份文件与冻结副本逐字相同，之后主工作区再改，行号可能对不上。

## 没做什么

- 没编 Rust、没跑任何 cargo 与 checker 档测试；harness 那条 `recorded_paths_match_the_registered_segment_sequences` 也没跑（主 agent 已有 nff 全绿的读数，同机另有会话在跑 checker 档整编，没去抢锁）。
- 没判正推那几格；没替主 agent 采纳；没碰 `crates/`。
- 草稿目录 `~/.local/share/singlefs/work/m3b37-opus/` 里只有模型三次输出（`model-run1.txt`–`model-run3.txt`），没建仓副本、没建编译目录。
