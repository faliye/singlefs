# m4-capacity-grow-r2 辩方腿报告（Sonnet）

日期：2026-11-18。背景 `research/prompts/_m4-capacity-grow-r2-background.md`，被复核的判决 `research/prompts/m4-capacity-grow-r1-main-verification.md`。
方案按 `crates/` 今天的实现谈：没有建模型、没有跑任何命令（没有模型文件，也就没有 sha256 与复跑脚本）；全部结论是读码加手推，读数一律「推的，待主 agent 跑」。
行号取 2026-11-18 工作区此刻的样子，与第一轮判决里引的行号有偏移（工作区在第一轮之后改过 `mount.rs`、`mounted_read.rs`、`recovery.rs`），每个行号都是这一轮现 `grep -nF` 过的。

范围先说清：逐盘容量表今天在 `crates/` 里没有（`grep -rn 'capacity_table\|容量表' crates/*/src | wc -l` 本次输出 0）。所以「落后盘怎么走」读的是今天的路径（只读挂载、恢复、可写挂载的落后支与几何判），「乙、丙、丁在这条路径上各在哪一步分叉」是把规则放进今天的路径推出来的。

## 一、一览

| 判决条目 | 辩方判定 | 证据（细节在后面各节） |
|---|---|---|
| 第 1 问：J3 靠「只读挂载报告没有点名落后盘的栏」坐实，这一步够不够 | 够得着，但只够到只读那一半；恢复与可写挂载那两半够不着 | 只读挂载与恢复今天对落后盘没有别的拒因、没有别的报告栏（第二节）。恢复与可写挂载对 J3 盘面各有一道已有的拒（第二节 B、第四节） |
| 判决写的证据：`mounted_read.rs:635`–`:637` | 站得住但不够：它只证明「这一栏没有点名落后盘」，不证明「没有别的兜」；要证后者另需四处（第二节） | 第二节 A 到 C |
| 判决表第一行：成链取最小「J3 中，漏放行」 | 站得住（只读那一半）；但同一形状也落在丙的五步构造形上，只是判决表把它记成「没打中（剩一个五步构造形）」 | 第三节 |
| 判决表把「成链取最小」与「生效取最小、点名按最大、挂起的变大可放弃（丁）」分成两行 | 补一条「点名按最大比探测值」之后是同一个东西：正推自己的纯函数已经带「放弃挂起的变大」；补「落后盘的槽不参与成链与取小」则是另一条规则，并且要付一笔代价 | 第三节 |
| 判决表丁「没推完」那一格（旧镜像盘面上可写挂载按旧容量接着跑） | 够得着：可写挂载在取号之前有两道已有的拒挡在写之前（几何判、落后支），J3 盘面走不到「接着跑」；这一格对成链取最小同样成立，不是丁独有 | 第四节 |
| G5（正推自报，判决写「只有一条腿报、没核」） | 真红，不是已有判法兜住：I-5.2、I-3.20、I-3.16 今天都无条件按「容量」判，今天只有 F 那一格有同形的豁免（只管 I-3.16、判的是 F 不是容量） | 第五节 |

「同样打中别的臂」的总判：J3 里分两件事。一件是「落后盘的内容旧」，只读挂载对它今天按设计不报（`.claude/kb/decisions/18-块里携带什么信息.md:322`），这一件对乙、丙、丁、成链取最小没有区别。另一件是「记录的容量比盘实际容量小」，只有「生效表取最小而点名也只比生效表」的规则漏；点名取最大的规则（乙、丁，丙在带意图标记时除外）不漏。判决表把第二件当成成链取最小独有的缺陷，补一行点名规则就消掉。

## 二、第 1 问：只读挂载与恢复对落后盘今天实际做什么

盘面（J3 族）：盘 0 两槽新（新系统配置、根环里有新根、实例号新），盘 1 整盘换回旧镜像（系统配置槽、它那一片根环、journal 尾与实例号都旧），盘 1 容量等于旧表那一项。

### A. 只读挂载

只读挂载依次：择系统配置、择根、扫 journal 并重放、判退、打开。逐步读到的是：

- 择系统配置不比落后：每盘取世代号大的那一槽，第一块出得了配置的盘那一份留下，后面的盘只比 fsid 与设备数。`crates/singlefs-core/src/recovery.rs:1897`：「None => chosen = Some(best_on_device),」；`crates/singlefs-core/src/recovery.rs:1901`：「|| previous.immutable.device_count != best_on_device.immutable.device_count」。跨盘不比实例号、journal 尾、F。
- 择根取全部根环槽里 (checkpoint_txg, 实例代号) 最大的（`crates/singlefs-core/src/recovery.rs:2048` 起的 `choose_root`），落后盘上的旧根只是候选里的小项，选不上。`crates/singlefs-core/src/mounted_read.rs:685`：「let chosen_root = choose_root(reader, &system_configuration)」。
- 判退里「系统配置见证到哪一版」取全池各盘各槽 journal 尾的最大值，落后盘拉不低它。`crates/singlefs-core/src/recovery.rs:8248`：「let witnessed_journal_counter = slots_of_each_device」，下面接 `.map(|slot| slot.quantities.journal_tail).max()`。
- 只读打开沿所选根读 inode 树、extent 树与中央映射；不读分配记录树，也就不走几何判。`grep -c 'allocation_records_fit_the_pool_geometry\|read_allocation_record_tree\|AllocationRecordTreeGeometry' crates/singlefs-core/src/mounted_read.rs` 本次输出 0。
- 落后支（缺单元、缺根）在只读挂载里不存在：`grep -n 'devices_without_the_selected_version\|devices_behind_the_current_version' crates/singlefs-core/src/mounted_read.rs crates/singlefs-core/src/recovery.rs | wc -l` 本次输出 0；两个函数的调用点只在可写挂载取号之前（`crates/singlefs-core/src/mount.rs:8217`：「devices_without_the_selected_version(」）与挂着之后收盘表的入口（`crates/singlefs-core/src/mounted_session.rs:2658`：「devices_behind_the_current_version(reader, parameters, current_version_of_trees)」）。
- 这是条款定的，不是漏写：`.claude/kb/decisions/18-块里携带什么信息.md:322`：「只读挂载不做这道核、逐条试位置条目读所选那一版」。
- 有一条用例把它钉成「照旧」：`crates/singlefs-verification/tests/fsync_drop_and_devices_without_the_selected_version.rs:589`：「fn read_only_mount_reads_the_selected_version_from_the_device_that_carries_it() {」，里面「盘 1 停在旧状态」一格 `mount_read_only(&image)` 做成、读回所选那一版（`crates/singlefs-verification/tests/fsync_drop_and_devices_without_the_selected_version.rs:624`：「let mounted = mount_read_only(&image)」）。

报告栏：只读挂载的报告里与盘有关的只有一栏，`crates/singlefs-core/src/mounted_read.rs:637`：「pub devices_without_a_mountable_system_configuration_slot: Vec<DeviceIdentity>,」，点名的是两槽都不可择的盘；落后盘的两槽都自证得过，不进这一栏。另有一个运行期计数 `crates/singlefs-core/src/mounted_read.rs:92`：「pub data_unit_reads_with_one_copy_failing: u64,」，读单元时前一份坏、后一份对才加一，不指名哪块盘、挂载那一刻不报，不算「点名落后盘」。

### B. 恢复

`recover` 与只读挂载同一条前半段（`crates/singlefs-core/src/recovery.rs:8393` 起：择系统配置、`choose_root`、扫环、重放、判退），后半段多走一遍冷走读 `walk_to_file_in_the_unit_area_starting_at`（`crates/singlefs-core/src/recovery.rs:7388`）。冷走读里读分配记录树并判几何：`crates/singlefs-core/src/recovery.rs:7633`：「allocation_records_fit_the_pool_geometry(reader, unit_area_start, &roots.allocation.records)?;」；几何判里按盘末判落点：`crates/singlefs-core/src/recovery.rs:2866`：「if slot.0 + span_slots > unit_area_end_slot {」，拒因 `crates/singlefs-core/src/recovery.rs:2868`：「what: "分配记录的跨度越过单元区末尾",」。盘末今天取 `PoolReader::device_size_in_bytes`（探测值）（`crates/singlefs-core/src/recovery.rs:2847` 一带；R3 之后的出口换成记录值，那是判决 R3 一段已定、今天还没落地的事）。恢复报告里的盘栏同样只有 `devices_without_a_mountable_system_configuration_slot`（`crates/singlefs-core/src/recovery.rs:8401`）。

所以 J3 盘面上（盘 1 容量 X，根与记录引着 X 之后的落点）：

- 只读挂载：做成，不点名，不报别的栏（与判决一致）。
- 恢复（冷走读）：在几何判上拒，报 `AllocationRecordOutsideThePoolGeometry`。这个成员在判退里归内容拒收一类（`crates/singlefs-core/src/recovery.rs:8040`：「| RecoveryFailure::AllocationRecordOutsideThePoolGeometry { .. }」），三样读者并集里有内容拒收就不退（`crates/singlefs-core/src/recovery.rs:8117`：「if failures.contains(&Some(FailureKindForTheFallBack::ContentRefused)) {」），所以恢复不会退到上一版把它绕过去，直接判失败。
- 读分配记录树时用的几何是 `of_reader`，按读者报的盘字节数算（`crates/singlefs-core/src/allocation_record_tree.rs:148`：「pub fn of_reader<Reader: PoolReader + ?Sized>(reader: &Reader) -> Self {」）。盘上那棵树是按 2X 写的、几何按 X 算，先撞树读那一步的位置判、还是先撞 2866 那一句，这一轮没逐步追（推的，待主 agent 跑）；两者都是内容拒收，结局相同。

### C. 判定

够得着：判决说的「只读挂载放行、不点名」读码上成立。但判决只引了一个「这一栏没有」的行（`mounted_read.rs:635`–`:637`），要成立还要排除别的兜，上面 A 的六条加 B 才排除完。判决这一步的措辞「J3 坐实打中正推的成链取最小」没有限定到只读挂载，读者容易读成恢复与可写挂载也漏；那两条路 J3 盘面今天各有一道拒（B 与第四节），不漏。

推翻它的现象：只读挂载在某条路径上读了分配记录树或调了落后支（`mounted_read.rs` 里出现 `allocation_records_fit_the_pool_geometry` 或 `devices_without_the_selected_version` 的调用），或「盘 1 停在旧状态」那条用例的只读一格红。

## 三、第 2 问：替「成链取最小」辩护，补一条规则它在 J3 上就不漏

### A. 它漏的是哪一件

正推第一轮的规则里，点名只比生效表：`research/prompts/m4-capacity-grow-r1-sonnet-output.md:136`：「只有「盘探测值小于生效表某项」才在只读报告里点名」。J3 盘面上生效表取最小得旧表，盘 1 探测值等于旧表那一项，所以不点名。漏的是「点名拿什么比」，不是「成链」也不是「取最小」。

### B. 补法一：点名按盘上全部自证槽的表的逐项最大比探测值

只改点名的比较对象；生效表、成链判、写者、取号写的次序一个字不动。

- J3 盘面：盘 0 两槽新表、盘 1 两槽旧表，逐项最大是新表，盘 1 探测值 X 小于新表那一项 2X，只读挂载放行并点名盘 1。
- 不用世代号，所以不吃 J4（世代号逐盘计、跨盘不可比）。
- 对「取号写到一半」的盘面（盘 0 新、盘 1 旧、盘都没缩）：探测值都不小于最大表，不点名，与原规则同。
- 对「取号写到一半又缩回」（J5 那一形）：只读挂载点名、放行，无损；可写挂载若把「点名」当拒，就把 J5 的永久拒带回来。所以可写那一侧不能是「点名即拒」，要么是「挂起的变大且探测值小于挂起那一项就放弃（写回生效表）」。正推的纯函数已经是后一种：`research/prompts/m4-capacity-grow-r1-sonnet-output.md:39`：「候选不逐项大于等于「盘上最大表」（盘上有挂起的、某盘探测值又小于挂起那一项）⇒ `to_write` = 生效表，放弃挂起的变大，点名那块盘。」

补完之后它与判决表里的丁是不是同一个东西：丁的三句是「生效表取逐项最小；只读与可写的点名按逐项最大比探测值；全部槽未写齐新表的变大可以写回旧表放弃」（`research/prompts/_m4-capacity-grow-r2-body.md:11`）。成链取最小的三句是「生效表取成链里最小那张」「放弃挂起的变大」（上面 39 行那句，已在正推的规则里）与补上的「点名按最大」。逐句对得上，是同一条规则。我能找到的唯一出入是放弃的触发写法：正推按「候选不逐项不小于盘上最大表」，丁按「全部槽未写齐新表」，前者看探测值与表、后者看盘面；两种写法在 J3 与 J5 两个盘面上都触发，是否在所有盘面上等价这一轮没枚举（推的，没核到不同）。

对判决的含义：判决表把成链取最小记成「J3 中」、把丁记成「没推完」，两行其实是同一条规则补不补一句点名；J3 在两行上是同一个洞，补一句就都补上。丁「没推完」那一格对成链取最小同样存在（第四节），判决表只把它记在丁一行。

### C. 补法二：落后盘的槽不参与成链与取小

这是派发提示举的例子。它与丁不是同一个东西：生效表在 J3 上取新表（2X），不取旧表；点名按生效表比，盘 1 探测值 X 小于 2X，照 R3 点名（只读放行、可写拒）。要付的代价：

- 怎么认「落后」。便宜的判法（只看槽里的 (journal_instance, journal_tail) 低于全池最大，全池最大的取法同 `crates/singlefs-core/src/recovery.rs:8248`：「let witnessed_journal_counter = slots_of_each_device」）会误排「取号写到一半」里的盘：取号写带的 tail 是取号那一刻读到的见证值（`crates/singlefs-core/src/transaction.rs:773` 一带注释：「回卷写带的 tail 是取号那一写带的同一个见证值」），见证值是全池最大；若盘 1 此前只是错过一次轮换（尾低一格），盘 0 先轮写进新表时 tail 是最大值，盘 1 旧槽 tail 更低，被判落后排除，生效表在盘 1 还带着旧表时就变成新表，等于乙的「一有新表就生效」，J5（记到一半、缩回、可写永久拒）带回来。
- 精确的判法是落后支（`crates/singlefs-core/src/mount.rs:7247` 的 `what_a_device_behind_the_version_lacks`：先看 tail，再看缺单元、缺根）。它要「所选那一版」，而所选版在择根之后才有；择系统配置的调用点里有不带版本的（`crates/singlefs-core/src/recovery.rs:7962`、`:7981`、`:8000` 三处 `choose_system_configuration(reader)?`，另有 `:1601`、`:5112`），表要在版本之前就定下来给几何用，循环依赖。要拆成两段（先不带表择一遍、定版本、再定表），改动面比补法一大得多。
- 只读挂载要新读落后盘上所选版的单元落点（落后支今天只在可写挂载里读），多一笔只读挂载的读量。

所以补法一是便宜的、补法二是另一条规则、带代价；两者不要混成「补一条规则」。我的建议只到这里：J3 要补的洞，补法一够，且让成链取最小与丁合并；补法二没有被攻过，也没有量过，不推荐顶替补法一。

### D. J3 打中哪几条臂

| 规则 | J3 盘面（盘 1 旧镜像、容量 X，另一块盘带新表）只读挂载 | 依据 |
|---|---|---|
| 乙（逐项取大） | 点名：生效表取最大 2X，盘 1 探测值 X 小于它 | 判决表「逐项取大」一行 J3 没打中；R3 只读点名的规则 `research/prompts/m4-capacity-r3-main-verification.md:28` |
| 丙，镜像里没有意图标记 | 点名：没有标记取逐项最大 | 同上 |
| 丙，镜像里带意图标记（判决说的「五步构造形」） | 不点名：有标记取旧表，旧表等于探测值。这是 J3 在丙上的同一个洞，判决表记成「没打中（剩一个五步构造形）」 | `research/prompts/m4-capacity-grow-r1-opus-output.md:105`：「构造形（推的，五步，含一次断电）」 |
| 丁 | 点名：点名按最大 | `research/prompts/_m4-capacity-grow-r2-body.md:11` |
| 成链取最小（原样） | 不点名 | 判决表 |
| 成链取最小 + 补法一 | 点名 | 第三节 B |

「落后盘的内容旧」这一件对四条规则没有区别：只读挂载今天都不报（第二节 A）。

推翻本节的现象：找到一个盘面，补法一（点名按最大）在可写或只读上把一个没有落点越界的盘面误拒；或丁的放弃触发与「候选不逐项不小于盘上最大表」在某个盘面上给出不同的写。

## 四、第 3 问：J3 那一族盘面在乙、丙、丁下可写挂载各走到哪一步

可写挂载的次序（`crates/singlefs-core/src/mount.rs:9856` 的 `mount_writable_with_every_test_only_switch`，后面的行号都是这一个函数与它调的函数）：

1. 择系统配置（`crates/singlefs-core/src/mount.rs:9869` 起，择法在 `crates/singlefs-core/src/recovery.rs:1835`）。
2. 参数核、`every_device_reaches_the_unit_area_start`（`crates/singlefs-core/src/mount.rs:9885`：「every_device_reaches_the_unit_area_start(&*devices, unit_area_start)?;」）。R3 的比较（探测值小于记录项则可写拒）按正推的写法排在这一步之后、读阶段之前：`research/prompts/m4-capacity-grow-r1-sonnet-output.md:37`：「R3 的「每块盘探测值不小于它那一项，否则可写拒」判完」。这一步今天的代码里没有（容量表还没有）。
3. 读阶段（`crates/singlefs-core/src/mount.rs:9898` 起）、判退、建上一版：`crates/singlefs-core/src/mount.rs:9993`：「let previous = rebuild_previous_version(」，错误用 `?` 直接返回。它调 `rebuild_version_in_the_unit_area_starting_at`（`crates/singlefs-core/src/recovery.rs:3512`），其中读分配记录树并判几何：`crates/singlefs-core/src/recovery.rs:3622`：「let allocation_tree_read = allocation_record_tree_read_and_judged(」，里面 `recovery.rs:4470` 起调 `allocation_records_fit_the_pool_geometry`，落点判在 `recovery.rs:2866`。
4. 重建分配器（`crates/singlefs-core/src/mount.rs:10036` 起）、`establish_instance`（`crates/singlefs-core/src/mount.rs:7795`，调用点 `:10070`）。其中取号之前逐盘核：`crates/singlefs-core/src/mount.rs:7922`：「} = devices_passing_the_check_before_the_acquisition(」；有盘不带所选那一版就拒：`crates/singlefs-core/src/mount.rs:8237`：「if !devices_without.is_empty() {」，拒因 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`。落后的判法 `crates/singlefs-core/src/mount.rs:7094`：「if newest_system_configuration_journal_tail >= version_journal_position {」之后才读单元。
5. 取号写（第一个往盘写字节的动作，两轮系统配置槽）。

「逐盘核只读盘、不写」，这四步在取号写之前都不写盘：已有用例钉着，`crates/singlefs-verification/tests/fsync_drop_and_devices_without_the_selected_version.rs:582`：「"拒在取号之前，两块盘逐字节不变"」（盘 1 停在旧状态那条，`:540`）。

### 盘面 J3-小：盘 1 换回旧镜像，盘小到等于旧表那一项（X），根与分配记录引着 X 之后的落点

| 步 | 乙（取大） | 丙，镜像无标记（取大） | 丙，镜像带标记（取旧表） | 丁（取小，点名取大，挂起的变大可放弃） | 成链取最小（原样） |
|---|---|---|---|---|---|
| 1 择配置 | 过（成链） | 过 | 过 | 过 | 过 |
| 2 R3 比较 | 拒：盘 1 探测值 X 小于最大表项 2X，盘上逐字节不变 | 同乙 | 过：生效表取旧表 X，盘 1 探测值等于它 | 点名盘 1；是拒还是放行继续，看丁在可写侧怎么写「点名」（读法一：拒，同乙，这一格结束；读法二：挂起的变大且探测值小于挂起项，放弃，继续） | 过：生效表 X，不点名，继续 |
| 3 建上一版 | 没走到 | 没走到 | 拒：记录引着 X 之后的落点，按生效表 X 的几何判拒（`recovery.rs:2866`），内容拒收、不退上一版（`recovery.rs:8117`），盘上逐字节不变 | 读法二：同丙带标记一格，拒 | 拒，同丙带标记 |
| 4 逐盘核 | 没走到 | 没走到 | 只有第 3 步的版本里没有 X 之后的记录时才走到：盘 1 的 tail 小于所选版、又缺所选版里被 tail 之后记录点名的单元，拒（`crates/singlefs-core/src/mount.rs:8237`） | 同左 | 同左 |
| 5 取号写 | 没走到 | 没走到 | 走到的前提：第 3、4 步都过，即所选版里没有越过 X 的记录、盘 1 也不缺单元。这时越过旧容量的东西不在所选版里，写回旧表不丢账 | 同左 | 同左 |

读法的出处：判决表丁一行写「生效取最小、点名按最大、挂起的变大可放弃」（`research/prompts/m4-capacity-grow-r1-main-verification.md:36`）、正文写「已知没推完的一格：旧镜像盘面上可写挂载按旧容量接着跑时，越过旧容量的单元已在另一块盘上，结局取决于落后盘那一支」（`research/prompts/_m4-capacity-grow-r2-body.md:11`）；「接着跑」是读法二。读法一的话丁在这个盘面上等于乙，那一格不存在。

结论（读码推的，没跑）：

- 丁「没推完」那一格可以收：J3-小盘面上落后盘那一支（第 4 步）排在几何判（第 3 步）之后，而第 3 步按生效表（取小的 X）判记录，越过 X 的记录一条就拒，拒发生在第一个写之前。落后盘那一支只在「所选版里没有越过旧容量的记录」时才轮到，那时越过旧容量的东西不在所选版里，没有「按旧容量接着跑却压到别的盘上的单元」的历史。
- 这一条对成链取最小原样成立，不是丁独有；与第三节 B「它们是同一条规则」一致。
- 这道拒不是为容量表新加的：今天恢复与可写挂载就按探测值判这个几何（`recovery.rs:2847` 一带取 `device_size_in_bytes`），R3 把出口换成记录值之后，取小的生效表正好让它按旧容量判。

### 盘面 J3-大：同样的旧内容，但物理盘仍是 2X（用户只还原内容、没缩盘）

| 步 | 乙、丙（取大） | 丁、成链取最小（取小） |
|---|---|---|
| 2 R3 比较 | 过：探测值 2X 不小于最大表项 | 过（生效表 X） |
| 3 建上一版 | 过：生效表 2X，记录在盘内 | 拒：生效表取 X，记录引着 X 之后的落点，几何判拒；而盘 1 物理上放得下 |
| 4 逐盘核 | 拒：盘 1 落后且缺所选版里被 tail 之后记录点名的单元（`crates/singlefs-core/src/mount.rs:8237`），盘上逐字节不变 | 没走到 |

可写挂载两边都拒，只是拒因不同，盘上都没写。附带发现（推的，没跑）：取小的生效表让恢复的冷走读（`crates/singlefs-core/src/recovery.rs:7633`）在这个盘面上拒，而盘 1 的容量其实够；乙、丙取大不拒。只读挂载不读分配记录树（第二节 A），两种规则下都做成。这是取小规则（丁、成链取最小）对取大规则的一个代价，判决表没有列：取小把「落后盘的旧表」变成了几何判的基准。

推翻本节的现象：用一条用例造 J3-小盘面（盘 1 的字节数改成旧表那一项、盘上记录引着它之后的槽）喂可写挂载，结局不是拒在 `crates/singlefs-core/src/mount.rs:9993` 那一步建上一版的几何判里，或拒之后盘上有字节变；或 `fall_back_judged_on_the_three_readers` 把几何拒当「字节丢了」退到上一版（那样几何拒就绕过去了）。

## 五、第 4 问：G5 是真红还是已有判法兜住

G5 的写法（正推第一轮）：取号写完成之后、写行那次发布的根落盘之前，生效表已是新容量，最新有效根的记账统计（空闲、已分配、不可回收、全空聚簇段数、碎片度）还是旧容量；崩溃点重放的每个崩溃状态都跑 checker（`crates/singlefs-checker/invariants/05-空间记账.md:8`：「崩溃点重放每个崩溃状态都判」），这一段崩溃态上 I-5.2、I-3.20、I-3.16 红不红。

### 今天 checker 对这一段怎么判

- 容量的来源是读者报的盘字节数，不是系统配置：`crates/singlefs-checker/src/walk.rs:22288`：「let capacity = reader.device_bytes(*device).and_then(|bytes| {」。今天没有容量表，这一段崩溃态上盘字节数没变，等式成立，不红（这是今天不红的原因，不是有判法兜住）。
- I-5.2 无条件比这个容量：`crates/singlefs-checker/src/walk.rs:22294`：「judgements.judge("I-5.2", matches!((free, allocated, capacity)…」；I-3.20 (a) 紧接着用同一个 `capacity`（`crates/singlefs-checker/src/walk.rs:22298`：「judge_free_allocated_and_unreclaimable_of_a_device_sum_to_the_capacity(」）；I-3.16 重算全空段时自己再读一次盘字节数（`crates/singlefs-checker/src/walk.rs:20722`：「.device_bytes(device)」）。三处都没有「取号已写、根没落」的条件。
- 唯一同形的豁免是 F：`crates/singlefs-checker/src/walk.rs:22060`：「roots[newest_index].2.rollback_floor == effective_rollback_floor;」，`crates/singlefs-checker/src/walk.rs:22359`：「if accounting_was_written_under_the_effective_floor {」。它只管 I-3.16，比的是最新根带的 F 与 F 生效值，不比容量；G5 这一段两边的 F 相等，豁免不触发，I-3.16 照判。所以它是同形的先例（系统配置先抬、根后落、checker 按「根带的值是否等于生效值」豁免），不是已有的兜。
- 「必红」今天就有一份：`crates/singlefs-verification/tests/fragmentation_statistics_follow_the_probed_device_capacity.rs:125`：「fn the_same_pool_bytes_on_devices_probed_three_cluster_segments_larger_redden_the_empty_segment_statistics(」，文件头说明「盘上一个字节不改，探测出来的容量加长三个整段」时红的恰是 I-3.16、I-5.2、I-3.20 三条（`crates/singlefs-verification/tests/fragmentation_statistics_follow_the_probed_device_capacity.rs:5`：「（I-5.2（空闲统计对得上） 跟着红：空闲 + 已分配 不再等于单元区」）。记账容量与 checker 容量不等时 checker 的红集合就是这三条，与 G5 声称的三条一致。R3 判决把 checker 的容量出口改成取记录值（`research/prompts/m4-capacity-r3-main-verification.md:30`：「读数出口：全部取记录值」）、那条必红用例随之反转；反转之后，同一个不等（记录容量大、记账容量小）从「盘字节数加长」搬到「取号写把表写成新值、根还没落」，红集合不变。

### 判定

真红（推的，待主 agent 跑）：R3 的容量出口落地之后，取号写完成到写行根落盘之间的崩溃态上 I-5.2、I-3.20、I-3.16 红，没有已有判法兜住；要收，要新加一道豁免，F 那一道（`crates/singlefs-checker/src/walk.rs:22060`、`crates/singlefs-checker/src/walk.rs:22359`）是它的形状：正推给的写法（盘上某个自证槽的实例代号大于最新有效根的实例代号时改判「统计容量不大于生效表容量」，不等那一半记部分没判）与它同形；checker 里算「每块盘最大实例代号」的读法已有（`crates/singlefs-checker/src/walk.rs:5874`：「fn judge_instance_carriers(」）。写法本身这一轮没验。

两点限定：

- 这一红今天在崩溃点重放里造不出来：重放的内存池每块盘一样大（`crates/singlefs-verification/src/memory_pool.rs:418`：「/// 崩溃点重放的内存池每块盘一样大，字节数住 `device_size_in_bytes` 那个字段；池里没有这块盘就 `None`。」），没有「表变大而盘字节数已变」的世界。上面的三条红是 checker 对「容量不等」的已量读数（今天的必红用例），不是在 G5 崩溃态上量出来的读数。
- G5 不分辨乙、丙、丁：任何规则只要表在取号写里记、统计在后面的发布里记，就有这一段。窗口的起点随规则不同（推的）：乙取大，先轮写第一个新槽生效表就变（窗口含先轮与后轮的全部崩溃态）；丁与成链取最小取小，最后一个旧槽被覆写才变（窗口从后轮屏障前的那几个态起）。所以它能比窗口大小，分不出规则出局，与判决「G5 第二轮攻」的位置一致。

推翻判定的现象：R3 的容量出口落地之后，用一条造「取号写完成、根未落」盘面的用例（表新、记账旧）喂 checker，I-5.2 不红；或 checker 里已有一道按「最新根的实例代号小于某槽的实例代号」豁免记账判法的条件（`walk.rs` 里按实例代号判记账的分支，这一轮 `grep` 没找到：`grep -n '取号' crates/singlefs-checker/src/walk.rs` 的 17 处命中里没有一处改判记账统计）。

## 六、要主 agent 跑的命令

没有模型，没有复跑脚本。下面两条是量已有用例在今天代码上的读数，用来把第二节与第五节里「读码推的」两句换成量的；判决要引的数没有复跑。

1. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 fragmentation_statistics_follow_the_probed_device_capacity 里的 1 条测试; 范围=1 个测试; 为什么=量记账容量与 checker 容量不等时红哪几条不变量，印证第一轮自报的 G5 红集合' task-server/task submit -- cargo test -p singlefs-verification --test fragmentation_statistics_follow_the_probed_device_capacity -- the_same_pool_bytes_on_devices_probed_three_cluster_segments_larger_redden_the_empty_segment_statistics --exact`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 行 `exit=0`；`task-server/task log <号>` 里「test result: ok. 1 passed」。绿说明红集合恰是 I-3.16、I-5.2、I-3.20 三条（用例自己断言）。

2. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 fsync_drop_and_devices_without_the_selected_version 里的 2 条测试; 范围=2 个测试; 为什么=量落后盘在只读挂载做成、可写挂载取号之前拒这两个今天的结局，支撑第二节与第四节' task-server/task submit -- cargo test -p singlefs-verification --test fsync_drop_and_devices_without_the_selected_version -- read_only_mount_reads_the_selected_version_from_the_device_that_carries_it stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write --exact`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 行 `exit=0`；日志里「test result: ok. 2 passed」。

## 没做什么

- 没建模型、没造 J3-小 或 J3-大 的盘面去跑可写挂载与恢复：第四节里「拒在几何判、不在落后支」与第二节「恢复在几何判上拒」都是读码推的，没有现成用例喂这个盘面（`grep -rln '跨度越过单元区末尾' crates/singlefs-verification/tests` 只命中 `bad_disk_input_campaign.rs` 一份，它造的是单盘坏记录，不是盘字节数变小）。要量，交实验设计员造「盘字节数改成旧表那一项、盘上记录引着它之后的槽」的内存盘。
- 没追盘上那棵分配记录树按 2X 写、读者几何按 X 算时，先撞树读的位置判还是先撞 `recovery.rs:2866`（第二节 B）。
- 没核丁的放弃触发写法与正推的写法在所有盘面上是否等价（第三节 B），也没回答「放弃写回与记录只升不降冲突不冲突」（第二轮第 2 问，不在这一轮分给辩方腿的四条里）。
- 没验正推给的 G5 豁免写法本身，只判了 G5 是真红。
- 没读禁读清单里的任何文件；没往统一任务服务交任务；判决要引的数没复跑（命令在上一节）。
- 草稿目录里没建仓副本、编译目录或别的文件，没有要删的。
