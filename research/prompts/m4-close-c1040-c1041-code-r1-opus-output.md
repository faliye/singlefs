# m4-close-c1040-c1041-code-r1 攻方腿报告

日期：2026-11-18。轮：代码轮第一轮，前几轮判决：无。被判对象按开工快照 `research/prompts/m4-close-c1040-c1041-code-r1-snapshot/` 读（主工作区 17 份被判文件与 `crates-sha256.txt` 逐个 `OK`，kb 五份与 `kb-sha256.txt` 逐个 `OK`，开工时现核）。禁读清单里的两处没读。

## 草稿与复跑

没写模型目录（没有计数模型、没有调枚举函数的原型）。草稿是仓副本里的三处测试改动，副本 `/home/fy5090/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-attack/copy`（`minimal-copy.py --with-test-inputs` 建，57M，`.deliverables` 写成「交接」），只改测试、不改 `crates/*/src`：

| 副本里的文件 | 做什么 | sha256 |
|---|---|---|
| `crates/singlefs-verification/tests/read_only_mount_does_not_compare_the_probed_physical_block_size.rs`（新） | 关 1 与 D12 那一格：同一份字节换一块探测值不同的盘，可写挂载与只读挂载各怎么判 | `113335e73ef8de2b36c64b5a92c131e2e962eddc3ff6c6926d06dc9b44fb4550` |
| `crates/singlefs-verification/tests/core_and_the_pool_checker_name_different_cells_for_a_slot_carrying_two_refused_values.rs`（新） | 关 1：一槽两格同错时 core 与池级 checker 点名哪一格 | `a7f4143c243e1fd0bdfa1142905e185c7dd01749cf6ee590ca5d94cd7961ae9c` |
| `crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`（改） | 关 3：探测计数装置每探一次往 stderr 打调用链里 `singlefs_core` 的帧；那两条探测次数断言换成本报告提的判法 | `2bd48d773c0611b67e272d2835e10551a456458bef145790b1f9a546b28f2371` |

改前原样留在草稿目录 `raise-floor-harness.orig.rs`。三份都只过了 `rustfmt --check`（语法），没编、没跑；命令在文末「要主 agent 跑的命令」。下面凡写「待主 agent 跑」的数都是推的。

## 各格判定一览

| 格 | 判定 | 一句依据 | 量的 / 推的 |
|---|---|---|---|
| 关 1·core 与池级 checker 对四格的整池判决 | 没打中 | 单格、两格同错两边都整池拒；可择 / 不可择 / 自证不过三类的分流两边同序 | 推的（读码） |
| 关 1·一槽两格同错时点名哪一格 | 打中，不分辨判决 | S 越界加单元大小错：core 在解槽那一步先报 S，池级 checker 先报单元大小 | 推的，草稿测试待主 agent 跑 |
| 关 1·mkfs 写了一字节才拒 | 没打中 | 几何判定与探测比对都在第一个写之前 | 推的（读码） |
| 关 1·可写挂载三个入口绕过比对 | 没打中 | 三处都经 `device_table_disagreeing_with` | grep 量过调用点 |
| 关 1·挂着之后的会话入口不比探测值 | 打中 | `publish_user_change` 进循环前只比身份与「三样」，探测只在推抬 F 那一支里 | grep 量过探测调用点；行为推的 |
| 内部矛盾·D12 已定项 4、5 与只读挂载 | 打中 | kb 两处写只读挂载也拒，只读挂载与恢复里零个探测调用 | grep 量过；行为草稿测试待跑 |
| 内部矛盾·代码里引 D12 的两种说法 | 打中 | mount.rs 与测试文件头引成「拒绝可写挂载」，make_filesystem.rs 又写只读也拒 | grep 量过 |
| 内部矛盾·D22 已定项 2「max」「未达到」与代码「不等就拒」 | 推不出 | 条款按池内最大值与单向「未达到」写，代码按两盘同构取等号 | 推的 |
| 关 2·验证包两处直接调池级 checker 函数 | 不成立（不算违例） | 两处都在审计一侧，被审计的 core 没共用；代价是两个 CPU 审计者是同一个算法 | 推的（读码） |
| 关 3·已知红测试那两次探测 | 来路：推抬 F 入口核盘表；判「测试判法要换」 | 调用链 mounted_session.rs:6328 → mount.rs:5008 → 7760 → 7710 → 7603 → 7627 | log 19999 量到 `(2, 0)`；来路推的，带调用链的副本命令待跑 |
| 关 4·后置条件空真 | 没打中 | 两个执行函数每一臂都可达，新加的长度条款与逐字节等价相容 | 推的 |
| 关 4·组合引理用不到的前提 | 没打中 | 两条引理的前提各两半，每半边单删都有反例 | 算术（推的） |
| 关 4·8 行必红被别的错误顶替 | 没打中 | 判的是落点函数集合，8 行各只改一处，落点都在被改的那个函数 | 推的 |
| 关 4·必红表漏行 | 打中（覆盖缺口，不是错） | 两个 None 臂新加的 `range_intersection_length(...) == 0` 没有自己的必红行 | 推的 |
| 外推·写四格的只有 `to_slot` | 成立（产品代码范围） | 产品代码里写系统配置槽的只有它；测试里手搭槽的有池级 checker 库单测夹具（已补三格） | grep 量过 |
| 外推·各条流探测值取参数 | 推不出（只在 grep 范围里成立） | 重放崩溃镜像那几处按参数造盘，内存盘判表那两处写死 512 且参数也是 512 | grep 量过 |
| 外推·已有流钉值不变 | 不成立一处 | 探测次数这一钉（关 3 那条红测试）随 C1040 变了；崩溃流判定集合推不出 | log 19999 量过 |
| 外推·验证包读系统配置的模型与 core「两槽各读一次」 | 打中（现有流走不到） | 四格错的本池槽：core 那一读收进自证集，验证包那一份模型丢掉 | 推的 |
| 八格检查单 | 见第四节 | 成立 5 格；不成立 1 格（类型与名字）；推不出 2 格（带 --item 的项要等 list-items 跑出来、新崩溃流只核到 grep 范围），推不出照派发算打中 | — |

## 一、过度外推与越位（实现员报告里「推的，没跑」那几句）

### 1.1 「写系统配置槽的只有 `SystemConfiguration::to_slot`」

成立，范围缩到产品代码。`to_slot` 写四格之前各断言一次位置（「writer.assert_position(UNIT_BYTES_OFFSET, "单元大小");」（`crates/singlefs-core/src/system_configuration.rs:677`）），值取格式常量。测试一侧手搭槽字节的有池级 checker 库单测夹具（`image.rs` 的 `tests` 模块，这一批补写了 305、309、313），别的测试都是读 `to_slot` 写出的槽再改个别字节。没找到第二个产品写者。

### 1.2 「崩溃枚举与放量各条流造盘时，探测值取的就是参数」

推不出全量，只在我 grep 到的范围里成立：重放崩溃镜像造盘的三处（`crash_injection.rs` 两处、`history.rs` 一处）都写 `PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size)`；`crash_judge_tables.rs` 那两处写死 512，同一函数里 mkfs 参数也是 512。`physical_block_size:` 取非 512 字面量的只有 `core_review_geometry_back_chain_and_empty_inode.rs`（已改成取参数）与 e197（不做 mkfs）。运行时才算出参数的调用点我没逐个追，与实现员同一个缺口。

### 1.3 「所以钉值不变」

不成立一处。实现员说的钉值是崩溃流的判定集合；但仓里另有一类钉：探测次数。已知红测试那条（第三节 3.3）就是拿「发布路径别处不探」当判别子数探测次数（「发布路径别处不探：数探的次数」（`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3770`）），C1040 让推抬 F 那条路每块盘多探一次，这一钉变了。用 grep 查过别的测试：数探测次数的只有这一份文件（`probes_of_the_physical_block_size` 别处零命中）。崩溃流本身：录制装置不录探测，流的步数不变（推的）。

### 1.4 验证包那一份「两槽各读一次」模型与 core 不再同判（打中，现有流走不到）

- 输入：本池（fsid 相同）一块盘上某一槽四格之一不是格式常量、整槽校验和重封，而这一槽恰好不是择系统配置时读的那一槽（例：两块盘槽 0 记的槽距不同，择到的那份槽距是 4096，另一块盘按自己槽 0 的 8192 读槽 1，偏移 4096 那里放着这一槽）。
- core：「Some(bytes) => match SystemConfiguration::parse_slot(&bytes) {」（`crates/singlefs-core/src/recovery.rs:5276`），`parse_slot` 不判四格，这一槽进 `self_verified`。
- 验证包：「Some(bytes) => match crate::verification_read::parse_system_configuration(&bytes) {」（`crates/singlefs-verification/src/mount_transparency.rs:2423`），这一批让它判四格、交 `Err(())`；`Err` 那一支只在自证不过或 fsid 是别的池时计数，本池的这一槽两样都不算，从模型里消失。
- 四句：分不分辨改法——分辨（改 `parse_system_configuration` 的调用方，还是给 core 那一读也加四格判，结局不同）；系统看得到判别子——看得到（字节都在）；满足的分句：`verification_read.rs` 新文档串写「与 core 择系统配置、池级 checker 读槽同判」，对择系统配置成立，而 `mount_transparency.rs` 拿它模型的是 core 的「两槽各读一次」，那一读不判四格；改法——批 2 报告「什么现象会推翻」第 3 条自己点过 `mount_transparency.rs`，推测「没有」，那一句没有被别的条款收住。
- 走不到的理由：今天所有流里四格都是常量（1.1），所以不影响任何现有判定。推的，没建用例。

## 二、内部矛盾

### 2.1 core、池级 checker、验证包三处的判法与次序

整池判决层面三处一致（没打中）：单格错时 core 报 `SystemConfigurationValueRefused`，池级 checker 把每块盘每槽读成 `CarriesAValueThisReaderRefuses`，验证包两处都不认这一槽。分流次序也对得上：magic、校验和、incompat 不认得（不可择）、节点大小（当自证不过）在四格之前，core（`parse_slot`）与池级 checker（`check_system_configuration_slot`）都这样。fsid 是别的池的槽：可写挂载按本池 fsid 筛掉，池级 checker 不筛、照 I-7.13 红——这是 C707 那一套的老差别，不是这一批新加的，任何不收之值都一样，这里不算打中。

点名哪一格层面打中（不分辨判决）：

- 输入：mkfs 之后四个系统配置槽都改成单元大小（305）= 65536、每区槽数 S（362）= 17，整槽校验和重封。
- core：S 在解槽那一步判，「.map_err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange)?;」（`crates/singlefs-core/src/system_configuration.rs:791`），早于「unit_geometry_fields_of_the_format(slot)?;」（`crates/singlefs-core/src/recovery.rs:1441`）⇒ 报 `RecoveryFailure::RootRingSlotsPerRegionOutOfRange`。
- 池级 checker：四格在「if let Some(disagreeing_field) = unit_geometry_field_that_disagrees(&bytes) {」（`crates/singlefs-checker/src/image.rs:1258`）先判，S 在 `geometry_of` 里「let slots_per_region = u64::from(slot[362]);」（`crates/singlefs-checker/src/image.rs:861`）才判 ⇒ 报 `UnitGeometryFieldDisagrees(UnitBytesNotTheFormatConstant)`。
- 草稿测试 `core_and_the_pool_checker_name_different_cells_for_a_slot_carrying_two_refused_values.rs` 钉这两个预测，待主 agent 跑。
- 四句：分不分辨改法——只分辨「点名格」，不分辨整池判决；系统看得到判别子——看得到；满足的是池级 checker 新成员文档串说的「实现那张「读者不收之值」表里排在加密类型之后的同四格」与新测试文件头的「拒在同一格」：单格时成立，两格时次序不同；改法——把 checker 的四格挪到 S 之后、或把 core 的 S 挪进那张表，两种都修这一格；R、槽距这几格两边次序早就不同（core 槽距先、checker R 先），只改四格不会让全表同序。

### 2.2 D12 已定项 4、5 与只读挂载（打中）

kb 两处都说只读挂载也拒：

- 「| `physical_block_size` | **拒绝挂载**（可写与只读都拒）」（`.claude/kb/decisions/12-目标介质.md:75`），HEAD 里同一行就是这样写的（`git show HEAD:` 那份第 75 行）。
- 「代码已做这一比（可写与只读挂载都拒；mkfs 时参数与探测不同也拒）」（`.claude/kb/decisions/12-目标介质.md:103`），这一句是工作区里没提交的改动（`git diff HEAD --stat` 那份文件 17 增 5 删），与开工快照 `kb-sha256.txt` 一致。
- D22 也不分可写只读：「挂载时逐设备复算比对，未达到拒绝挂载」（`.claude/kb/decisions/22-单元原子性怎么合成.md:63`，原文带加粗，比对时去掉）。

代码：`grep -n 'probe_physical_block_size' crates/singlefs-core/src/mounted_read.rs crates/singlefs-core/src/recovery.rs` 零命中、退 1（开工时现跑）。core 里探测的调用点一共 21 行，与「比对」有关的只有「let probed_physical_block_size = device.probe_physical_block_size();」（`crates/singlefs-core/src/mount.rs:7627`）与 mkfs 的 `make_filesystem.rs:614` 两处，别的是块设备自己的对齐判据与预演盘的构造。

代码里引条款的说法自相矛盾：

- 「对不上 ⇒ 拒绝可写挂载」（`crates/singlefs-core/src/mount.rs:764`），新测试文件头同一句（`crates/singlefs-verification/tests/writable_mount_refuses_a_device_whose_probed_physical_block_size_differs_from_the_system_configuration.rs:2`）。kb 里没有这句：`grep -rnF '对不上 ⇒ 拒绝可写挂载' .claude/kb/` 零命中，D12 那一行写的是「拒绝挂载（可写与只读都拒）」。这是把条款摘成了更窄的版本。
- 「照参数写出的池，可写与只读挂载逐盘比探测值都会拒」（`crates/singlefs-core/src/make_filesystem.rs:216`）：说只读挂载会拒，与 mount.rs 那一句和代码都对不上。
- 实现员 `result.md` 第 211 行（转述）：把 D12 已定项 4 那一行读成「拒绝可写挂载」，据此判只读挂载不用比——读错了表，kb 那一格明写可写与只读都拒。批 2 报告第 238 行起把只读挂载停下交主 agent，那一条的理由是接口卡住，不是条款不要求。

反例输入（草稿测试 `read_only_mount_does_not_compare_the_probed_physical_block_size.rs`，待主 agent 跑）：

- 记录 4096 的池（两块探测 4096 的盘上 mkfs、取号、暖机、第一个文件），盘 0 换成字节相同、探测 512 的盘。预测：`mount_writable` 拒成 `PhysicalBlockSizeProbedOnTheDeviceDiffersFromTheSystemConfiguration { identity_handed_in: 0, probed: 512, recorded: 4096 }`（C1040 在，阳性对照）；`mount_read_only` 照常挂上（4096 宽的根槽在 512 盘上对齐，读得通）。这一形就是 E72 判 `refuse_rw` 的方向（4096 → 512）。
- 记录 512 的池，盘 1 换成探测 4096 的盘。预测：只读挂载不成，但报的是读不出（512 宽的根槽在 4096 盘上不对齐），不是探测值与记录值不等；原样打 `ATTACK_READ_ONLY … result=` 一行。

四句：

1. 分不分辨改法：分辨。批 2 报告给的两种加法里，乙（`PoolReader` 新方法带默认体交 `None`）在包装读者上照旧不判，这一格对它还中；甲（必须实现）修这一格，但要动 `judging_units_entries.rs` 的登记；第三条路是改 kb（把 75、103 两行改成只拒可写），那是条款改动、要走三方。
2. 系统做决定那一刻看不看得到判别子：**看不到**。只读挂载只拿 `&dyn PoolReader`（`mounted_read.rs:651`），`PoolReader` 没有探测；判据要它分「探测等于记录」与「不等」两种情形，而它手里的东西两种情形逐条相同。这是 `.claude/rules/evidence-discipline.md` 那张表的「判别子观测不到」：要么改接口（让它看得到），要么改判据（kb），不能只在实现里补一判。
3. 满足的分句：D12 已定项 4 表 `physical_block_size` 一行「拒绝挂载（可写与只读都拒）」的「只读」那半句；已定项 5 第 103 行「代码已做这一比（可写与只读挂载都拒…）」是一句现状陈述，此刻为假。
4. 改法在打中的格上还中不中：只改注释（把 make_filesystem.rs:216 改成只说可写）不修这一格，只是让代码注释不再自相矛盾；103 行的现状句要么随实现补上，要么改成「只读挂载还没做这一比（C1040）」。

### 2.3 关 2：验证包两处直接调池级 checker 的函数

不成立（不算违例）。条款：「被禁止的是两件事：需要答案的那一刻还没有答案；审计与被审计用同一段代码。」（`.claude/rules/fs-design.md:22`）。被审计的是 core；这两处都在审计一侧：

- `verification_read.rs` 文件头：「验证侧的盘面读法。解析与校验走 checker 的独立实现」（`crates/singlefs-verification/src/verification_read.rs:1`），它是 `mount_transparency` 拿来对照 core 挂载结果的那一份读法。
- `memory_pool.rs:758` 那一处给池级 checker 算扫描候选（`ImageReader::candidate_journal_slots`），喂的是 checker 自己。
- core 那一侧是 `system_configuration.rs` 的 `unit_geometry_fields_recorded_in_the_slot` 加 `recovery.rs` 的 `unit_geometry_fields_of_the_format`，不引 checker；checker 自己切字节、自己写 N 的常量，两边共用的只有 `singlefs_format` 的三个格式常量（D13 已定项 12 准共享格式常量，checker 的 `lib.rs` 那段注释引的就是它）。

代价（不是违例，但要知道）：验证包这两处与池级 checker 是同一个算法，`unit_geometry_field_that_disagrees` 写错一处，三处一起错；新测试里「验证侧与池级 checker 同判」那几条断言等于拿 checker 比 checker。对 core 真正独立的只有 core 自己那一份，GPU 判器这四格还没判（I-7.13 记在 C1041）。

### 2.4 D22 已定项 2 的「max」「未达到」与代码的「不等就拒」（推不出）

D22 那一格写「mkfs 按池内 `max(physical_block_size)` 划」「挂载时逐设备复算比对，未达到拒绝挂载」（同上 `.claude/kb/decisions/22-单元原子性怎么合成.md:63`），读成字面是：异构池按最大值划根槽、挂载只拒根槽宽够不着探测值的盘（单向）。代码是双向等号：mkfs 要每块盘都等于参数（两盘 512、4096 时参数取 4096 也拒），可写挂载记录 4096、探测 512 也拒（变异行「C1040：可写挂载只拒探测值大于记录值的盘」钉着双向）。D12 已定项 5「第一条线两盘同构」与 E72 判 4096 → 512 `refuse_rw` 支持等号；两条条款之间的张力不由这一批定，记推不出、交主 agent。

## 三、哪一条最脆弱

### 3.1 mkfs 判与第一次写的先后（没打中）

`make_filesystem` 里「let unit_area_start = check_geometry(parameters, &sizes)?;」（`crates/singlefs-core/src/make_filesystem.rs:647`）之后紧跟「first_device_probing_a_physical_block_size_other_than_the_parameter(parameters, devices)」（`crates/singlefs-core/src/make_filesystem.rs:649`），两样之前只有读盘宽（`size_in_bytes`），清零与写都在后面。对外入口先读随机源取 fsid（不碰盘）再进内层入口。试过的形状：两块盘探测值相同而与参数不同、只有一块不同、几何本身也不收（报几何那一格，同样不写）——都在写之前拒。新测试核「录制流 0 步、两盘逐字节不变」罩住了这一点。

### 3.2 可写挂载有没有绕过比对的入口

挂载那一层没打中：比对只住在 `device_table_disagreeing_with` 里，它的调用点只有两处——`of_a_pool_on_disk`（`crates/singlefs-core/src/mount.rs:981`）与参数核那一处（`crates/singlefs-core/src/mount.rs:7710`）；后者被可写挂载（「parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table(」（`crates/singlefs-core/src/mount.rs:9922`））与挂着之后收盘表的入口（`caller_inputs_agreeing_with_the_disk`，`crates/singlefs-core/src/mount.rs:7760`）调。可写挂载里这一判在读根环与第一个写之前。

挂着之后的会话入口打中：

- 会话发布用户改动先比身份——「if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {」（`crates/singlefs-core/src/mounted_session.rs:2964`），身份只比号、不读盘（`crates/singlefs-core/src/mount.rs:1050` 的 `is_the_device_table_of_the_mount`）；再自己核「可见」、本盘设备号、落后支三样（`mounted_session.rs:2984` 起的三个函数），这三个函数都不调 `device_table_disagreeing_with`，也不探测（core 里探测调用点 grep 全列过，见 2.2）。之后就进循环发布、写盘。
- 只有同一次调用里被空间拒、推抬 F 时才走「match push_one_floor_raise_within_the_admission_budget(」（`crates/singlefs-core/src/mounted_session.rs:6328`）→ `caller_inputs_agreeing_with_the_disk` → 探测。
- 它自己的文档串说两边是同一套核，末句「所以这次调用发出的每一个写之前都核过」（`crates/singlefs-core/src/mounted_session.rs:2930`）；C1040 把探测加进 `caller_inputs_agreeing_with_the_disk` 共用的那一份之后，推抬 F 那一支核四样、用户改动那一支核三样，这句对探测那一样不成立。`run_compaction`（2624 行那一处比身份是读文件的入口，3213、3400 两处是整理）、`resend_the_frozen_publish` 同一情形（推的，没逐个读完）。
- 输入：两块探测 4096、记录 4096 的盘上可写挂载做成；之后把同身份、同字节、而盘 1 探测 512 的盘表交给 `MountedSession::publish_user_change`。推：比身份过、三样过，发布照常写（4096 对齐的写在 512 盘上也对齐）；反方向（记录 512、换上探测 4096）则在写 512 宽的根槽时块设备报 `Unaligned`，那之前同一次发布的数据单元可能已写下——会话里「写了才拒」。
- 没建用例：会话要先有一版带文件的现行版本，新池挂上之后现行那一版是 `WithoutFile`，`publish_user_change` 在三样之前就拒成 `NoFileVersionToChange`；要造得借 `common` 那一套建池（镜像走文件），这一轮没做。所以这一格的行为是推的，调用点是 grep 量过的。
- 四句：分不分辨改法——分辨（改法一：用户改动入口也走 `device_table_disagreeing_with`；改法二：认定会话里盘表借着、不换盘，推抬 F 那一支也不探；两种在 3.3 那条红测试上结局不同）；系统看得到判别子——看得到（盘对象在手里，能探）；满足的分句——不是 D12 已定项 4（那是挂载时），是上面那句文档串与 D18 已定项 11「挂着之后收盘表的每个入口……核」的同一套之说；改法在打中格上——改法二修文档串的不一致、不修热换盘。

### 3.3 关 3：已知红测试那两次探测

量到的：任务 19999（别的会话交的，主 agent 在派发里点了号）`task-server/task log 19999` 原样三行：

```
assertion `left == right` failed: 报「被宽护卡住」那一次不预演：一块只进内存的盘都没建、一个改盘调用都没有
  left: (2, 0)
 right: (0, 0)
```

调用链（推的，读码；带调用链打印的副本命令待主 agent 跑，见文末第 1 条）：

1. 再试同一个写，会话第一次发布被空间拒，进「match push_one_floor_raise_within_the_admission_budget(」（`crates/singlefs-core/src/mounted_session.rs:6328`）。
2. 它第一句就核调用方输入：「caller_inputs_agreeing_with_the_disk(parameters, devices, allocator, current)?;」（`crates/singlefs-core/src/mount.rs:5008`），在判预算之前。
3. → 参数核（`crates/singlefs-core/src/mount.rs:7760`）→「let disagreeing_device_table = device_table_disagreeing_with(devices, system_configuration);」（`crates/singlefs-core/src/mount.rs:7710`）→「physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration(」（`crates/singlefs-core/src/mount.rs:7603`）→ 每块盘一次「let probed_physical_block_size = device.probe_physical_block_size();」（`crates/singlefs-core/src/mount.rs:7627`）。两块盘 = 2 次。
4. 之后 F 已在上限，交回 `Stopped`，报「被宽护卡住」；没有预演。预演只在要出路时建只进内存的盘：「let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(」（`crates/singlefs-core/src/mounted_session.rs:3978`），盘在 1862 行建，构造里「physical_block_size: disk.probe_physical_block_size(),」（`crates/singlefs-core/src/mounted_session.rs:924`）——也是每块盘 1 次。`rehearse_the_push_then_the_remainder_of_the_change(` 在 core 里只有这一个调用点（grep 量过）。

所以 `(2, 0)` 里的 2 与「预演了一次」的 2 在这个计数器上分不出来：测试拿探测次数当「建没建只进内存的盘」的判别子，前提是测试旁那句「发布路径别处不探」，C1040 让推抬 F 入口每块盘探一次，前提没了。

判：**测试判法要换**，不是「那条路不该探测」。依据：`caller_inputs_agreeing_with_the_disk` 的文档串定它「照可写挂载同一套核」，可写挂载要比探测值（D12 已定项 4），推抬 F 入口跟着比是这套设计的直接结果；推抬 F 的公开入口（`raise_rollback_floor`、`unmount`、管理员回退）收的是调用方现交的盘，那里不比就是热换盘的口子。附条件：3.2 的改法二（会话里不再探）若被采纳，结论反过来，那条路就不该探、测试原判法对。

换法（只在副本上改了，待主 agent 跑）：报「被宽护卡住」那一次期望 `(盘数, 0)`（入口核盘表那一遍），要出路那一次期望 `(2 × 盘数, 0)`（再加预演建盘那一遍，计数器是累计的——原测试第二条断言 `(盘数, 0)` 在 C1040 之后同样会红，19999 只是先红在第一条）。若 3.2 改法一被采纳，用户改动入口也探，两个期望各再加一个盘数。更稳的判法是让计数装置分得出「谁在探」，这一轮没做。

四句：分不分辨改法——分辨（改测试 / 拆开推抬 F 入口的核，两种改法在这一格上结局相反）；系统看得到判别子——测试看不到（只数总次数），这正是要换判法的原因；满足的分句——测试旁「发布路径别处不探：数探的次数」那句前提；改法在打中格上——只改第一条断言的期望不够，第二条（3816 行那条，`mounted_session` 预演之后）同样要改。

### 3.4 关 4：交集长度证明（`crates/singlefs-replay-prefix/src/file_range_proofs.rs`）

**后置条件空真：没打中。** 两个执行函数没有 `requires`，每一臂都可达（`bytes_in_unit` 相交时进 `Some`、不相交或区间倒置时进 `None`；`checked_bytes_in_indexed_unit` 外层 `None` 只在乘加溢出时）。`Some` 臂新加的 `length == range_intersection_length(...)` 与原有的逐字节等价条款相容：逐字节等价说交集正好是 `[unit_start + in_unit, + length)`，长度大于 0 时那段的长度就是规格算的最大起点到最小终点之差。它在逻辑上是原有条款的推论，不是空真，也不矛盾。`checked_bytes_in_indexed_unit` 用数学乘加写单元端点，与执行体里 `checked_mul` / `checked_add` 成功分支的值相等（`checked_unit_byte_range` 的 ensures 交代了）。

**组合引理用不到的前提：没打中。** 两条引理的前提各是两半，每半边单删都有反例（算术，推的；`range_intersection_length` 记作 L）：

| 引理 | 只留的那半 | 反例（写区间 / 单元区间 / 分界） | 左边 | 右边 |
|---|---|---|---|---|
| `lemma_adjacent_unit_intersection_lengths_add_up` | `boundary <= unit_end` | 写 [0, 10)，单元 [5, 8)，分界 3 | L = 3 | 0 + L(0,10,3,8) = 5 |
| 同上 | `unit_start <= boundary` | 写 [0, 10)，单元 [2, 5)，分界 9 | L = 3 | L(0,10,2,9) + 0 = 7 |
| `lemma_split_write_intersection_lengths_add_up` | `split <= write_end` | 写 [5, 9)，分界 2，单元 [0, 10) | L = 4 | 0 + L(2,9,0,10) = 7 |
| 同上 | `write_start <= split` | 写 [0, 5)，分界 8，单元 [0, 10) | L = 5 | L(0,8,0,10) + 0 = 8 |

（第三行只留的是 `split <= write_end`，删的是 `write_start <= split`；第四行反过来。）引理文档说的「写区间可以为空或倒置」「单元区间可以为空或倒置」也成立：倒置时 L 的最大起点不小于最小终点，两边都是 0。

**8 行新必红被别的错误顶替：没打中。** 必红表判的是落点函数集合、不是条数：「判的是第六段落点，不是条数」（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:3`）。8 行各只改一处，逐行推它红在哪个函数：

| 行 | 改的是 | 推的落点 | 能不能落到别处 |
|---|---|---|---|
| 三行 `intersection-length-contract-*`（ensures 前插 `false`） | 两条引理与见证的结论 | 各自那个 proof fn | 见证调两条引理时拿到 `false` 当前提，只会更好证，不多出别的错 |
| `intersection-length-unit-split-needs-valid-boundary` | 删引理 1 的 `requires` | 引理 1 | 见证那几处调用不再要证前提，少报、不多报 |
| `intersection-length-write-split-needs-valid-boundary` | 删引理 2 的 `requires` | 引理 2 | 同上 |
| `write-intersection-exact-length`（`+ 1`） | `bytes_in_unit` 的 `Some` 臂 | `bytes_in_unit` | 调用方 `checked_bytes_in_indexed_unit` 拿到的后置条件自相矛盾（逐字节等价推出 = L，又说 = L + 1），在它那里成了假前提，不报错；所以只落一处，与登记的 `proof_errors=1` 对得上 |
| `indexed-intersection-exact-length`（`+ 1`） | `checked_bytes_in_indexed_unit` 的 `Some` 臂 | 它自己 | `verus!` 里另一个调用方是见证 `witness_file_range_boundaries`（`file_range_proofs.rs:282` 那一行起），它拿这条后置条件证 `Some(Some((0, 1, 3)))`；改坏之后 `Some` 臂自相矛盾，在见证里成了假前提，同样不报错（core 在 `verus!` 外调） |
| `intersection-length-witness-sum`（4 改 5） | 见证第一条 | 见证 | 只此一处 |

**必红表漏行（打中，覆盖缺口）：** 新加的四条长度条款里，两个 `Some` 臂各有一行必红，两个 `None` 臂没有：「&&& range_intersection_length(write_start as int, write_end as int, unit_start as int, unit_end as int) == 0」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:171`）与「&&& range_intersection_length(write_start as int, write_end as int, unit_index * capacity, unit_index * capacity + capacity) == 0」（`crates/singlefs-replay-prefix/src/file_range_proofs.rs:221`）。表头说「每一条 ensures、每一条循环不变式（含主循环的出口）、每个引理的结论各一行，改坏了证明就断」（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv:13`）。

- 变异：`171` 行的 `== 0` 改成 `== 1`。推：`bytes_in_unit` 的 `None` 臂在不相交输入上走得到、L 是 0，证不出 = 1，红在 `bytes_in_unit`；表里没有这一行。
- 变异：`221` 行同样改。推：红在 `checked_bytes_in_indexed_unit`。旧行 `indexed-intersection-does-not-drop-written-bytes`（把求交换成 `None`）条数由 1 变 2，第二条多半就是这一臂的长度条款证不出——但落点仍是同一个函数，删掉这一条款它照样红在那里，所以那一行钉不住这一条款。
- 这两处不是错，是「改坏了证明就断」的行没写全。只在我的推导上成立，被攻过零轮；要加的两行交主 agent 定、经 formal-verification 跑。

**旧三行 `proof_errors` 1 → 2：** 不影响判定（条数只报不判，表头第 3 行）。其中两行在 `partial_write_proofs.rs`（这一批没改那份文件），条数变化不能由这一批的改动解释；推测是求解环境或无关 spec 的漂移（表头括注 C654 那句说的就是这一类），没核。

**射程：** README 新段写「规格与组合引理只涉及字节区间和返回长度，不代表缓冲区拷贝、设备 I/O 或持久化顺序已经精化到这些性质」，与 `write_request_split.rs` 只把元组第三项放进 `length_in_bytes` 对得上；core 拿到 `length` 之后怎么拷贝没被证，射程句已经说了。

## 四、八格检查单（逐格回）

| 格 | 回 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立（附注一条） | 「没有新加 `assert!`」按宏字面成立，按实质有 4 处新 panic 点：`to_slot` 里新加的 4 个 `writer.assert_position`（例「writer.assert_position(UNIT_BYTES_OFFSET, "单元大小");」（`crates/singlefs-core/src/system_configuration.rs:677`）），`assert_position` 里面就是「cursor == expected,」（`crates/singlefs-core/src/bytes.rs:557`）那条 `assert!`；`system_configuration.rs` 不在 `verus!` 里，它的 `requires` 不被静态证。触发条件：写到该字段时游标不等于字段表偏移；它之前全是定宽字段，任何输入都不触发，只有改代码（在 305 之前增删字段）才触发。新读函数的 panic（槽短于 4096 / 325）：core 只在 `parse_slot` 判过槽宽之后调，池级 checker 只在 `check_system_configuration_slot` 过了之后调，验证包两处（`verification_read.rs:94`、`memory_pool.rs:758`）同样排在它之后（`memory_pool` 那一处是 `.ok().filter(...)`，`Err` 时不进 filter） |
| 类型与名字 | 不成立（一处） | 同一个量两种类型：`probed_physical_block_size: PhysicalBlockSizeInBytes` 对 `physical_block_size_in_the_parameters: u32,`（`crates/singlefs-core/src/make_filesystem.rs:221`）与 `physical_block_size_in_the_system_configuration: u32,`（`crates/singlefs-core/src/mount.rs:769`），比的时候拆 newtype：「(probed_physical_block_size.0 != physical_block_size_in_the_parameters).then_some(」（`crates/singlefs-core/src/make_filesystem.rs:615`），`crates/singlefs-core/src/mount.rs:7628` 同形。根子是 `SystemImmutableSizes::physical_block_size` 本来就是 `u32`（不是这一批加的），但这一批新加的两个错误成员可以两边都用 newtype 而没有。四格是裸 `u32`、与 `u64` 常量比时 `u64::from`，与系统配置别的定宽字段同一种写法。补丁里新加的行没有 `_ =>`（`grep -n '^+.*_ =>' changes.patch` 零命中、退 1）；新 `expect` 在产品代码里只有偏移常量转 `usize` 那一种，写了依据 |
| 不兼容旧代码与旧格式 | 成立 | 没有认旧版本、旧布局号的分支。`memory_pool.rs:758` 的 `.filter` 让四格错的槽 0 不被信、退到罩住任何合法几何的候选，是收紧不是放宽；`verification_read.rs:94` 交 `Err(())` 不专为别的池放宽，但它进 `mount_transparency` 之后的处置与 core 那一读不同（1.4），那是同判问题、不是兼容问题 |
| 不新建判定库 | 成立 | `grep -nE '^\+.*(std::fs\|File::\|CARGO_MANIFEST_DIR\|env::var\|hostname\|thread::\|available_parallelism)' changes.patch` 零命中、退 1；新测试只用 `SparseBlockDevice`、`RecordingBlockDevice`、`MemoryPool` |
| 所有任务多机可用 | 成立 | 同上那条 grep；证明只依赖 Verus 与登记表 |
| 测试优先多线程 | 成立 | 每条用例一个场景、内存稀疏盘、毫秒到秒级（推的，没量），不写线程数；不值得切片 |
| 带 --item 的命令与项 | 推不出（等命令） | 两份报告点名了全部新目标：`system_configuration_unit_geometry_fields_are_refused_by_core_and_the_pool_checker`、`writable_mount_refuses_…`、`make_filesystem_refuses_…`、`memory_pool_scan_candidates_…`、验证包 `--lib` 那一条与池级 checker `--lib` 四条。`--list-items` 列不列得出 17 行变异与 3 个新 proof 项要跑命令（文末第 4、5 条）。已知红测试的目标为什么没跑到：合入后验证按补丁改到的路径挑目标，「src/ 下别的（不论是不是 .rs）→ 库目标」（`scripts/gate/merge-and-verify.sh:80`），`mount.rs` 的改动只挑 core 的 `--lib`，那份测试文件与它的目标根文件补丁都没碰，所以挑不到 |
| 新崩溃流多罩了什么 | 推不出 | 第二个写者：产品代码里没有（1.1）。在探测值与参数不同的盘上 mkfs 或可写挂载的流：grep 范围里没有（1.2），全量没追。另：探测次数那一钉变了（1.3），不是崩溃流 |

## 改法与它修哪一格（都只在我的推导上成立、被攻过零轮；没有一格「量过」）

| 改法 | 修 2.1 点名格 | 修 2.2 只读挂载 | 修 3.2 会话入口 | 修 3.3 红测试 | 标 |
|---|---|---|---|---|---|
| 池级 checker 四格挪到 `geometry_of` 的 S 之后（或 core 把 S 挪进那张表） | 修 | 不碰 | 不碰 | 不碰 | 推的 |
| `PoolReader` 加必须实现的探测方法，只读挂载择完系统配置逐盘比 | 不碰 | 修 | 不碰 | 不碰 | 推的（批 2 报告的加法甲） |
| 同上但带默认体交 `None`、`None` 跳过 | 不碰 | 包装读者上不修 | 不碰 | 不碰 | 推的（加法乙） |
| 改 D12 第 75、103 行为「只拒可写」 | 不碰 | 改判据（新一轮） | 不碰 | 不碰 | 推的，要走三方 |
| `publish_user_change` 等会话入口也走 `device_table_disagreeing_with` | 不碰 | 不碰 | 修 | 测试期望各再加一个盘数 | 推的 |
| 会话里认定盘表不换、推抬 F 那一支不探 | 不碰 | 不碰 | 只修文档串不一致 | 原判法对，测试不用改 | 推的 |
| 测试两条期望改成 `(盘数, 0)`、`(2 × 盘数, 0)` | 不碰 | 不碰 | 不碰 | 修（副本里已改） | 推的，待主 agent 跑 |
| 必红表补两个 `None` 臂的 `== 0` 改 `== 1` 两行 | 不碰 | 不碰 | 不碰 | 不碰 | 推的（修 3.4 漏行） |

## 要主 agent 跑的命令

在哪起：第 1–3 条在副本 `/home/fy5090/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-attack/copy`（`.deliverables` 写成交接），第 4、5 条在仓根。每条先看末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`，再用 `task-server/task log <号> --all` 看下面写的行。第 1 条的目标 19999 照不缩域交过；范围声明闸若判它会走崩溃枚举，照拒绝说明二选一改。第 4、5 条若被范围声明闸要求带声明，照它的说明补。

1. 已知红测试，带调用链打印、换过两条期望（副本）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_given_states_of_the_images_harness_scenarios_hand_out 里的 1 条测试; 范围=1 个测试; 为什么=攻方腿副本：探测计数装置每探一次打调用链，判已知红测试那两次探测从哪条路来，并试换过的两条期望' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-attack/copy -- cargo test -p singlefs-verification --release --test crash_given_states_of_the_images_harness_scenarios_hand_out -- write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push --exact --nocapture
```

   看：`ATTACK_PROBE probe_number=1` 与 `=2` 两段里有 `singlefs_core::mount::physical_block_sizes_probed_on_the_devices_differing_from_the_system_configuration` 与 `push_one_floor_raise_within_the_admission_budget`（来路坐实）；`=3`、`=4` 两段里有 `rehearse_the_push_then_the_remainder_of_the_change`（预演建盘）。前两段里出现的是预演那几帧，3.3 的来路判断就被推翻。`test result: ok. 1 passed` 说明换过的期望一路走通；`FAILED` 抄出 `panicked at` 那一行与 `left` / `right`。

2. 只读挂载比不比探测值（副本）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 read_only_mount_does_not_compare_the_probed_physical_block_size 里的 2 条测试; 范围=2 个测试; 为什么=攻方腿副本草稿：同一份字节换一块探测值不同的盘，核只读挂载比不比探测值，D12 已定项 4 写可写与只读都拒' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-attack/copy -- cargo test -p singlefs-verification --release --test read_only_mount_does_not_compare_the_probed_physical_block_size -- read_only_mount_takes_a_pool_recording_4096_on_a_device_probing_512_that_writable_mount_refuses read_only_mount_on_a_pool_recording_512_with_a_device_probing_4096_reports_something_other_than_the_probe --exact --nocapture
```

   看：`2 passed` 坐实 2.2（`ATTACK_READ_ONLY recorded=4096 … result_is_ok=true`）；`ATTACK_READ_ONLY recorded=512 … result=` 那一行原样抄进判决。第一条红在「预测只读挂载照常挂上」，就是只读挂载另有别的机制拒了，2.2 的行为判断推翻（条款与注释的矛盾仍在）。红在 `mkfs`、取号、暖机那几个 `expect` 是草稿装置的问题，不判。

3. 两格同错时点名哪一格（副本）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 core_and_the_pool_checker_name_different_cells_for_a_slot_carrying_two_refused_values 里的 1 条测试; 范围=1 个测试; 为什么=攻方腿副本草稿：一槽两格同错时 core 与池级 checker 各点名哪一格' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-attack/copy -- cargo test -p singlefs-verification --release --test core_and_the_pool_checker_name_different_cells_for_a_slot_carrying_two_refused_values -- core_names_the_slots_per_region_and_the_pool_checker_names_the_unit_size_on_the_same_slot --exact --nocapture
```

   看：`1 passed` 坐实 2.1；`ATTACK_TWO_REFUSED core=` 与 `pool_checker=` 两行原样抄。

4. 变异门禁能不能点到这一批 17 行（仓根）：

```
task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items
```

   看：log 里以 `C1040：`、`C1041：` 起头的项数是 17（`grep -c` 数）。

5. 形式化门禁能不能点到新 proof 项（仓根）：

```
task-server/task submit -- bash .claude/gate.d/formal-verification.sh --list-items
```

   看：有 `proof:lemma_adjacent_unit_intersection_lengths_add_up`、`proof:lemma_split_write_intersection_lengths_add_up`、`proof:witness_intersection_length_partitions` 三项。

## 没打中的形状

- core 对池级 checker（整池判决）：四格各单错一格；四格错再叠 R 错、加密类型错、格式版本错、incompat 不认得、节点大小错（这五种两边对四格的相对次序一样：incompat 与节点大小先于四格、两边都当不可择或自证不过；格式版本、加密类型先于四格；R 后于四格）；四格错的槽只在逐档找槽 1 时读到（两边都先判值、不核偏移，整池拒）；某块盘槽 0 带四格错而别的盘槽 0 好（core 当场整池拒，checker 把那一槽记成不收之值、同样不作保）。取样只在字段组合上，没有按字节扫。
- mkfs：参数与两块盘探测都不同、只与一块不同、几何同时不收；对外入口取 fsid 那一步。
- 可写挂载：三个入口、按 fsid 筛完一槽都没有时不筛再择那一步（参数核照样拒错 fsid）、盘数与 `devs` 不同、身份重复。
- 证明：两条引理每半边前提单删（都有反例，见 3.4 那张表）；空与倒置区间；u64 上界；8 行必红逐行推落点。
- 关 2：找过 core 那一侧有没有引 checker 的四格函数（没有）。

## 这条腿自己的限度

- 一个任务都没交、一行项目代码都没跑；除了 grep 结果与任务 19999 的日志三行，文中的行为判断都是读码推的。三份草稿只过了 `rustfmt --check`，编不编得过、断言写得对不对都待主 agent 跑。
- 3.2 会话入口那一格没建用例（理由在那一节），行为是推的。
- 1.4 `mount_transparency` 那一格没建用例，且现有流走不到。
- GPU 判器两个内核没读；运行时才算出参数的调用点没逐个追；Verus 没跑，必红表的落点是推的。
- 只攻了分到的三个面；正推那几格（条款逐条对照）没做。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数（这一轮一个也没量出）不算入库装置上的数；没往统一任务服务交任务，判决要引的数没复跑，命令在「要主 agent 跑的命令」一节。
- 没改 `crates/`（主工作区），没写模型目录与 `rerun.sh`（没有模型，只有副本里的测试草稿）。

## 清理

草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-c1040-c1041-code-r1-attack/` 里：副本 `copy`（57M，`du -sh`）留着、`.deliverables` 写成交接，主 agent 照上面第 1–3 条在里面跑，用完删；`raise-floor-harness.orig.rs` 是副本里那份装置改前的原样（`.deliverables` 写成备份）；`log-19999.txt`、`minimal-copy.txt`、两份 `rustfmt` 输出是我读的原始输出。没有编译目录（没编过）。
