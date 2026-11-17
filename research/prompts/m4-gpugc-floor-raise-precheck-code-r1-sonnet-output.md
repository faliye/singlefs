# m4-gpugc-floor-raise-precheck-code-r1：正推腿报告（Sonnet，2026-11-10）

被判对象：第 119 批补丁里 `mounted_session.rs`（冻结副本与主工作区 sha256 同为 `080bd6bc4e3d18f5697e7ca968c09c7babef8f0fe948242930e2f55ec5625ced`，量的，`sha256sum`）。
代码引用的路径与行号都取冻结副本 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1/repo/`（不是主工作区，别的文件主工作区另有别人的改动）。
kb 引用写 kb 文件自己的行号（现查）；kb 条款整行抄在文末「附录 A」，由脚本逐行取出，不手抄。
没写模型。写了一个探针（两份测试文件，一份核上界、一份「弄坏上界」证它会红），**一行都没编过、没跑过**，命令在「要主 agent 跑的命令」。凡标「待主 agent 跑」的格，我只给了读码的推。
主 agent 量的数只引了任务日志（`task-server/task log 11701 --all` 存在草稿目录 `log11701.txt`）。

## 0. 先说结论

- 这一判做的事与判决第三节第 1 条、D16 已定项 10 射程那句一致。比用户原话宽的两处（上界多算「隔离」、槽对那一判也先算）kb 已写，方向都是少拒。
- 「上界」我推下来是真上界（宁大勿小），**推的，没量过**：式子那一臂每一项、槽对那一臂「一个放开的槽至多补一对」都落到文件:行（T1-a、T1-b）。没找到「推了本来写得进」的历史（T1-d），也没找到一个会把上界打穿的机制；探针把这个推理交给真推一串前后的读数去核。
- 最脆弱的三处：
  1. 判法只看「这一次被拒的那一判」。落点那一判排在式子那一判前面，提交内生块那一判（②）永远照推，所以「落点能救、式子救不了」的写第一轮不拦、白推一串，第二轮才拦；② 单独短而什么都放不出来时，每次照推（T3-b）。
  2. 第二轮才判到放不下时，kb 写的「盘上逐字节不变」不成立：前面推成的串已落盘（T2-c，两句并排抄在 T2-c）。
  3. 「上界恰好等于需求照推」整池上没钉；推一串之后上界会漂，恰好相等那一格推完就翻面（T3-a，探针量 drift）。
- 检查单八格：成立 7，不成立 1（测试优先多线程，理由没写），推不出 0。另有一处文档过期：`UserChangeRefused` 现在九个成员，文档写「八种」（T2-c）。

## 1. 实现今天的样子（正推，读码）

- 判法在 `push_one_floor_raise_for_a_refusal_for_space` 开头一处：「devices_short_even_with_everything_a_floor_raise_could_release(allocator, &refusal);」（`crates/singlefs-core/src/mounted_session.rs:5955`），紧接「if !devices_short.is_empty() {」（`crates/singlefs-core/src/mounted_session.rs:5956`），在「if !refusal_is_short_of_space(&refusal) {」（`crates/singlefs-core/src/mounted_session.rs:5968`）与第一次「match push_one_floor_raise_within_the_admission_budget(」（`crates/singlefs-core/src/mounted_session.rs:5975`）之前，所以预算和上限都不先问。
- 三处调用共用：`crates/singlefs-core/src/mounted_session.rs:3780`（发布路径）、`:6069`（走空）、`:6904`（accept）；`grep -n 'push_one_floor_raise_for_a_refusal_for_space('` 现查，调用点只这三处加定义。
- 只认 `SpaceAdmissionRefused`：「let PublishError::SpaceAdmissionRefused(refused) = refusal else {」（`crates/singlefs-core/src/mounted_session.rs:5889`）。`PlacementRefused` 没有 `short_devices` 可读，类型上走不进判法。
- 三个成员各一臂，没有 `_ =>`：式子「short.available.0 + bytes_of_slots(slots.deferred) + bytes_of_slots(slots.isolated),」（`crates/singlefs-core/src/mounted_session.rs:5851`）；槽对「slots.deferred + slots.isolated + slots.free_but_blocked;」（`crates/singlefs-core/src/mounted_session.rs:5856`）再乘一个数据单元的槽数；提交内生块「JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,」（`crates/singlefs-core/src/mounted_session.rs:5864`）。
- 严格小于才判放不下：「(upper_bound.0 < i128::from(short.demand.0)).then_some(」（`crates/singlefs-core/src/mounted_session.rs:5874`）。准入拒的条件是「(available.0 < i128::from(demand.0)).then_some(DeviceShortOfDemand {」（`crates/singlefs-core/src/admission.rs:899`），同向，所以上界 = 需求 = 放得下。
- 任一块盘判出放不下就整个不推；只有每块被判短的盘都救得回来才推。

## T1　过度外推与越位

### T1-a　式子那一臂的上界，逐项落到文件:行

式子九项见 D28 已定项 1（附录 A，`28-挂载期承诺量.md:21`）。推一串抬 F 的空发布之后各项动不动：

| 项 | 推串之后 | 补丁怎么算 | 依据（冻结副本） |
|---|---|---|---|
| 已分配 | 回收 defer 里的槽才降；空发布自己写新单元（升） | 加 `deferred`，不扣空发布自己写的，上界只会偏大 | 「占着的槽数：仍分配的加上已释放、还在 defer 窗口里的」（`crates/singlefs-core/src/allocator.rs:327`）；回收「self.allocated_slots -= span;」（`crates/singlefs-core/src/allocator.rs:852`） |
| 被抛弃根独占量（隔离） | 被抛弃根离开根环才降 | 全部隔离槽都加，不问这一串写不写得到那几个根环槽 | 「隔离着的槽（式子第九项）」（`crates/singlefs-core/src/mounted_session.rs:5820`）；清隔离的入口 `crates/singlefs-core/src/allocator.rs:779` |
| 抬 F 扣住的槽 | 计数上本来就空闲，位上挡着，F 生效后放开 | 式子不加（式子没扣它们）；槽对那一臂加 | 「计数上空闲而位上挡着的槽」（`crates/singlefs-core/src/mounted_session.rs:5822`）；放开「allocator.release_reclaim_holds();」（`crates/singlefs-core/src/mount.rs:5649`） |
| 挂载期承诺量 | 挂载期常量 | 不加 | 「挂载期间常量」（`crates/singlefs-core/src/admission.rs:1701`） |
| 环窗口 | 按根环几何现算，空发布不改几何 | 不加 | 「RingWindowOfTheReading::DeductedOnEachDevice {」（`crates/singlefs-core/src/admission.rs:1751`）；退役树层数读 `version_to_build_on`，空发布不碰退役树（推的） |
| checkpoint 保留池 | 分配记录树取几何整棵、中映射树与记账树随条目数；空发布只多一级台阶行，不降 | 不加 | 「拿得准的上界只有几何上整棵树」（`crates/singlefs-core/src/admission.rs:1188`）；「行数因此至多是这一版的行数加 1」（`crates/singlefs-core/src/admission.rs:1308`）。保留池推串之间不降（推的） |

两个会让上界偏小的机制，我都推了一遍，没推出能成立的：

- 空发布自己换下的旧单元进 defer，会不会在同一串里又被回收，从而「可用」涨过 `可用+defer+隔离`？这一串回收的下界在串开始前就算定：「let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);」（`crates/singlefs-core/src/mount.rs:5500`），准入这一格的回收是「RaiseFloorEntry::Admission => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,」（`crates/singlefs-core/src/mount.rs:4399`）。这一串自己换下的单元释放代是这一串里某次发布的 txg，大于串开始前的候选下界，回不来（推的，没量过）。
- 只有测试开关 `ReuseWindow::ForcedToZero` 会在释放那一刻当场回收（「ReuseWindow::ForcedToZero => {」`crates/singlefs-core/src/allocator.rs:2431`），那一档 defer 恒为 0，也没有「推一串」可言（推的）。

补一条宽的方向：上界不问这一串的上限够不够得到 defer 里的槽（「只算槽数、不问这一串的上限够不够得到它们」，`crates/singlefs-core/src/mounted_session.rs:5818`），也把隔离全算上，而一串抬 F 只放得出隔离里的一部分，放全要显式推环。所以判法是故意偏松：该拦的拦不全（T3-c），不会拦错。

### T1-b　槽对那一臂的上界

- 一个槽属于且只属于一个对：「let first_index_of_the_pair = index - index % 2;」（`crates/singlefs-core/src/allocator.rs:526`）。一对要变成「没挡」，对里至少有一个槽的挡位变了（已分配位清掉、隔离清掉、扣住放开），所以新增的对数 ≤ 变了挡位的槽数 ≤ `deferred + isolated + free_but_blocked`。补丁每个槽补一整对（乘数据单元的槽数）：「PlacementRule::UserData => 2,」（`crates/singlefs-core/src/transaction.rs:2896`）。两个槽合补一对时多算，只偏大。
- 段外槽对只减不增的方向：聚簇段集合只有 `insert`，没有 remove / clear：「self.cluster_segments.insert(segment_start);」（`crates/singlefs-core/src/allocator.rs:2625`）；`grep -n 'cluster_segments\.\(remove\|clear\|retain\|pop\)\|cluster_segments = '` 在 `allocator.rs`、`mount.rs`、`transaction.rs`、`mounted_session.rs` 零命中（量的，我跑了这条 grep）。所以推串不会把某个聚簇段里的对「放回」段外。
- 单位对得上：①的 `available` 与 `demand` 都按槽对 ×2 槽的字节记：「available: bytes_of_slots(slots_of_the_pairs).0,」（`crates/singlefs-core/src/admission.rs:1084`），补丁加的也是（槽数 ×2 槽）的字节。

### T1-c　有没有越位到不该拦的拒绝

- `PlacementRefused`：类型上走不进（见第 1 节 `:5889`），照旧推。
- 提交内生块那一判（②）：`=> None`（`:5864`），照旧推。理由是补丁文档写的「这一判数的槽抬 F 那一串自己也要占」（`crates/singlefs-core/src/mounted_session.rs:5842`）；但上界是上界，抬 F 那一串自己占不占不影响「放得出的全放出来也不够」这个判断，②也能算上界，见 T3-b。这是漏拦，不是错拦。
- 合取 7（全空聚簇段少于 N₀）：它不是 `SpaceAdmissionRefused`，走 `Publish` 原样交回：「PublishError::EmptyClusterSegmentsBelowTheFloor(_)」（`crates/singlefs-core/src/mounted_session.rs:2081`，在 `refusal_is_short_of_space` 的「不是空间不够」那一组里），判法不碰。
- 删除类发布：式子那一判不判，只有落点那一判（数据单元 0 个，①走不到，只可能是②，照推），所以 unlink / 截断走空里的拒绝都不被这一判拦。
- **没罩到的推串入口**（不在判决点名的三处里，所以不算越位也不算漏）：搬迁批「push_one_floor_raise_within_the_admission_budget(」（`crates/singlefs-core/src/mounted_session.rs:2447`）、挂载续做删除类发布（`crates/singlefs-core/src/mount.rs:8419`）、可写挂载自己的准入（`crates/singlefs-core/src/mount.rs:8963`，需求逐盘记 0）。判决第三节第 1 条点名的是「`accept` 与 `publish_user_change` 同一处」，这三处不在内，kb 射程那句也只说 `accept`。

### T1-d　找「推了本来写得进、被判成放不下」的历史

反推问法：假设上界算小了，本该看到某条历史上「判法交回新成员、同一状态下不判法时推一串后写得进」。

- 现成的正例历史：`crash_enumeration_floor_raise_pushed_by_admission.rs` 的 384 槽小盘覆盖写：直接发布到准入先拒（「段外成对的空槽 0 对」`crates/singlefs-verification/tests/crash_enumeration_floor_raise_pushed_by_admission.rs:84`，注释「写这条用例时拒在第 62 次」`:85`），再推一串（`push_one_floor_raise_within_the_admission_budget(` 在 `:132`），推完「推过抬 F 之后覆盖写做成」（`crates/singlefs-verification/tests/crash_enumeration_floor_raise_pushed_by_admission.rs:163`）。这条历史段外槽对 = 0、需求 1 对、推完够：上界至少是 1，槽对臂在这一格上不会误拒（推的：要 defer / 隔离 / 扣住至少一个，推通了就说明有）。但这条录制流**直接**调 `push_one_floor_raise_within_the_admission_budget`，不经判法，所以它不是判法的对拍，只是历史。
- 判法的对拍：验证包 18 个目标，任务 11701 日志数出来 `test … ok` 1215 条、`FAILED` 0 条、18 个 `test result`（量的，我数的）；其中点名「被宽护卡住」的 17 条全绿，含「a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through」（`crates/singlefs-verification/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:1586`）：推环推不通、只有卸载再挂能放过，这一格判法放行（上界够），所以没把出路吃掉。
- 主 agent 的探针读数（实现员报告引的）`available_after=54984704`、`demand=268435456`、`deferred=103`：上界 `54984704 + 103×2×16384 = 58359808` 远小于需求（来自实现员报告，我没复核读数文件）。这是「怎么推都不够」的正例，不是误拒。
- 我没找到一条历史是误拒。**这不等于没有**：整池只在 384 槽小盘与 320 MiB 小池上摆过，没有一条在「恰好只差一个放不出的槽」的边上摆过（实现员自己写了摆不出，`result.md`「没做什么」）。探针第一份（`probe_the_bound_of_what_a_floor_raise_could_release.rs`）在三条历史上每推一串前后读一遍，核「可用_后 ≤ 上界」「段外槽对_后 ≤ 上界」，并打印每串的 drift；第二份把上界里的 defer 拿掉，必须红。待主 agent 跑。

推翻 T1-a / T1-b 的现象：探针第一份有任一行 `a.available > bound_of_the_formula` 或 `a.pairs_outside_the_cluster_segments > bound_of_the_pairs`（违例行会列在 panic 信息里）；或第二份（弄坏上界）反而绿（说明探针没牙，T1 的反推观测无效）。

## T2　内部矛盾与循环依赖

### T2-a　判法读的分配器与后面推串用的是不是同一份

是同一份，三处各一：

- 发布路径：被拒来自「let refusal = match publish_the_change_once(」（`crates/singlefs-core/src/mounted_session.rs:3719`，第 3 个参数 `allocator`，`:3722`），推串用的是同一个形参 `allocator`（`:3780` 那次调用的参数表里）。
- 走空：「} = publish_the_queue_until_it_is_empty(」（`crates/singlefs-core/src/mounted_session.rs:6050`）的 `allocator`（`:6053`）与推串的 `allocator`（`:6072`）。
- accept：被拒的判在 `&self.session.allocator`（`crates/singlefs-core/src/mounted_session.rs:7005`、`:7015`），推串的 `allocator` 来自「} = &mut self.session;」（`crates/singlefs-core/src/mounted_session.rs:6897`）那次解构，同一个字段。
- 要预演的出路（`rehearse_the_push_then_the_remainder_of_the_change`）：拷分配器与盘，判法与推串都在拷贝上，同一份，不串到会话上（`crates/singlefs-core/src/mounted_session.rs:1819` 起）。

判法读的是「拒绝当时」的 `short.available`，加「此刻」分配器的 defer / 隔离 / 扣住。两者之间没有分配器改动：被拒都发生在「动分配器与任何写之前」（D28 接线「拒了在任何写之前返回」，附录 A，`28-挂载期承诺量.md:37`）。`expect("拒绝里的盘来自这个分配器…")` 走不到见检查单第 1 格。

### T2-b　三层记账一致吗：预算、备忘、冻结种类、上一条完成的意图

| 记账 | 判法交回时 | 与旧路径比 |
|---|---|---|
| 预算（`publishes_pushed_in_this_admission`） | 判法在预算判之前就返回，调用方没有 `+= publishes_of_this_raise` | 旧路径也是 `Err` 就不加；一致 |
| 备忘（`floor_raises` 与 `refusals_that_pushed_the_floor_raises` 一一对应） | 前面各串各自的被拒都在；这一次的被拒放 `last_refusal`，没推所以不进备忘 | 与 `NoSpaceAfterRaisingTheFloor` 同一形：「last_refusal: refusal,」（`crates/singlefs-core/src/mounted_session.rs:5998`）之外的两个 Vec 照搬；一致 |
| 冻结种类 | 判法返回前一个写都没发，不会新冻结 | 发布路径上 `note_the_kind_of_a_publish_frozen_by_this_call` 在被拒那一臂就调过（「note_the_kind_of_a_publish_frozen_by_this_call(」`crates/singlefs-core/src/mounted_session.rs:3770`）；它只在「这一趟新冻结」时动手（「if was_frozen_before_this_call {」`:2153` 提前返回），判法返回不改它；一致 |
| 上一条完成的意图 | 被拒、没发成的改动在 `?` 交回，不清 | 「let mut published = published?;」（`crates/singlefs-core/src/mounted_session.rs:2984`）在清之前交回，清的那句「compaction.last_completed_intent = None;」（`:3014`）在后；新成员同旧成员；一致 |
| `current` 的版本 | `accept` 里「*current = PoolVersion::of_a_shared_version_of_trees(version_pushed);」（`crates/singlefs-core/src/mounted_session.rs:6915`）在 `pushed.map_err` 之前赋值，判法返回时 `version_pushed` 还是 `Arc::clone` 的现行版，赋回是原样 | 一致 |
| `AcceptRefused::floor_raises_pushed_by_this_accept` / 运行库观测 | 补丁加了一臂（`crates/singlefs-core/src/mounted_session.rs:6510`）；运行库「Err(refused) => refused.floor_raises_pushed_by_this_accept(),」（`crates/singlefs-mount-runtime/src/lib.rs:902`）读它 | 一致；漏了这一臂编不过（无 `_ =>`） |

### T2-c　两处对不上（并排抄，整行）

**对子 1　kb 与代码文档：第二轮才判到放不下时，盘上还是不是「逐字节不变」。**

kb 一侧（`16-发布语义.md:275` 的一句，整行在附录 A，这里抄这句所在的整句）：

> 抬 F 之前先算（用户 2026-11-09 定）：推抬 F 放得出的全放出来也放不下就一串都不推，报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，盘上逐字节不变；式子那一判的上界是可用加 defer 加隔离，短在段外槽对那一判同样先算，上界恰好等于需求照推。

代码一侧两处（整行）：

> `crates/singlefs-core/src/mounted_session.rs:631`：`    /// 这一次判之前已经推成的串、发成的版在里面，头一次被拒就判到这里时它们是空的，盘上逐字节不变。装箱同 `NoSpaceAfterRaisingTheFloor`。`
>
> `crates/singlefs-core/src/mounted_session.rs:5932`：`/// 推抬 F 放得出的全放出来也放不下（[`devices_short_even_with_everything_a_floor_raise_could_release`]）交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，一个写都不发；`

对不上在哪：`:631` 自己带条件（头一次被拒），kb 那句没带；`:5932` 的「一个写都不发」也没带条件。判法每一轮都判（`:5955` 在 `push_...` 开头，调用方每轮被拒都进这个函数），第一轮落点那一判拒（式子那一判不在第一轮的读数里，T3-b）、推一串、第二轮式子那一判拒时，前面推成的串已落盘，`floor_raises` 非空，盘上变过。代码文档 `:631` 对，kb 与 `:5932` 说宽了。判决：条款要加条件（只在头一次被拒就判到时逐字节不变），不是代码要改。推翻条件：找到一条第二轮才判到的历史，`floor_raises` 为空但盘上变了——那是代码错。

**对子 2　文档：成员数。**

> `crates/singlefs-core/src/mounted_session.rs:2899`：`    /// [`UserChangeRefused`] 的八种。`

成员（`awk 'NR>=571 && NR<=645 && /^    [A-Z][A-Za-z]*[ ({,]/ {print NR": "$0}'`，我这一次跑的原样输出，行首空白已去）：

```
576: DeviceTableOtherThanTheOneOfTheMount {
586: NoFileVersionToChange {
599: DevicesWithoutASelfVerifiedSystemConfiguration { devices: Vec<DeviceIdentity> },
605: OwnDeviceNumbersDifferFromTheDeviceTable {
616: DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
621: Publish {
629: NoSpaceAfterRaisingTheFloor(Box<NoSpaceAfterRaisingTheFloor>),
632: NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(
638: FloorRaiseFailedWhilePushingForSpace(Box<FloorRaiseFailedWhilePushingForSpace>),
```

九个，文档写八种，补丁加成员时没改这一句。只是文档，不影响行为；同函数文档里别的数我没逐个核。

### T2-d　D16 已定项 10「过不了就在任何读写之前拒」与 accept 抬 F、与这一判

三层关系（kb 行号见附录 A）：

1. 已定项 10 定案一句：`accept` 进门先做合起来的准入与条数预判，「过不了就在任何读写之前拒」（`16-发布语义.md:273`）。
2. 同条射程（`:275`）改了范围：`accept` 被空间拒时自己抬 F；「条数预判过不了与冻结照旧在任何读写之前拒」——空间不够不在「照旧」里，是先推再判。
3. 这一判又收回一块：推了也写不进的写，在**头一次被拒**时仍是「任何读写之前」拒；推过一串之后才判到的，前面的串是写盘。

代码的对应：条数预判与冻结在 `judge_a_data_change_before_any_read_or_write`（`crates/singlefs-core/src/mounted_session.rs:6865` 一带的「if !refusal_is_short_of_space(&cause)」前，非空间的拒绝没推过串时原样拒），空间拒走 `judge_a_data_change_raising_the_floor_for_space` 里的推串（`:6904`），推串入口第一步就是这一判。所以三层不矛盾；矛盾只在对子 1 说的「第二轮」。

### T2-e　循环依赖

- 判法的上界里含「隔离」，而被判守住的这一串不会放全隔离（放全要显式推环，`UnmountThenMountWritableAgain` / `PushTheRootRing…` 两条出路，`crates/singlefs-core/src/mounted_session.rs:839`、`:855`）。所以「上界够」不等于「这一串推得通」，需要宽护那条出路的写会照推：推完停在 `FloorAlreadyAtTheCeiling`（「FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}」，`crates/singlefs-core/src/mounted_session.rs:1960`），照旧报「被宽护卡住」。这是故意的（保出路），代价是白推一串（T3-c）。不是依赖环：判法读的量（defer、隔离、扣住）推串只会让它们往同一个方向走，判法本身不读推串的结果。
- 出路预演里再走一遍这一判（预演在拷贝上重发这次改动，拒绝再进 `push_one_...`）：预演的结局只分「放过了 / 还被拒」两类（`mounted_session.rs:844`、`:864` 的 `match`，「RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }」），新成员落进「还被拒」，不丢出路也不多列出路。

## T3　哪一条最脆弱

### T3-a　上界恰好等于需求照推

- 纯判法上钉住了：测试 `a_formula_demand_exactly_covered_by_available_deferred_and_isolated_bytes_is_still_pushed_for`（冻结副本 `crates/singlefs-core/src/mounted_session.rs:8059` 起的 `everything_a_floor_raise_could_release_tests` 模块里）与变异「抬 F 先算：上界恰好等于需求也判放不下」（`crates/mutations.tsv:7480` 那行，`<` 改 `<=`）。实现员自己写了整池上摆不出恰好相等（`result.md`「没做什么」）。
- **脆弱处在判法每一轮重判**：上界 = 可用 + defer + 隔离。推一串之后：回收把 defer 搬进可用（和不变）；空发布写新单元 W 个（可用 −W）；换下旧单元 R 个进 defer（defer +R）。上界变化 = R − W。R = W 时不变；W > R（记账树台阶行多一级、分配记录树多一片叶……）时下降。所以头一轮上界 = 需求恰好照推，推完第二轮上界 < 需求，翻面，交回新成员——这时头一轮的那一串已落盘白推了。这不是错（真放不下），但「恰好相等照推」在整池上只能走到「推一串再拒」。
- 我没法读码定 R − W 的符号和大小（推的）。探针第一份每串打印 `drift`（推串之后的 可用+defer+隔离 减推串之前同一量，单位字节）：`drift` 全 0 则这一格在这三条历史上不翻面；负的则翻面，负多少。待主 agent 跑。

### T3-b　短缺种类混着来

- 落点那一判（①②）排在式子那一判之前：「admit_the_units_landing_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13315`），式子那一判在后：「admit_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13415`）。一次被拒只带先拒的那一判。
- 第一轮被落点那一判拒、可救（①够）、而式子那一判会在推完之后拒、救不了（第二轮才判到）：第一轮必推一串，白写一串，第二轮才拦。这就是 T2-c 对子 1 的来路。
- ② 永远照推（`=> None`）：当 defer = 0、隔离 = 0、没有扣住的槽、而没挡的槽不够提交内生块时，②短缺什么都放不出来，仍推：每次 `accept` 白推。补丁文档说因为抬 F 那一串自己也要占槽，但上界不扣这一串自己的占用，②可以算：没挡的槽 + defer + 隔离 + 扣住的 ≥ 提交内生块要的槽。**判决点名的是「①式子 + ①槽对」两种，②照旧推是判决的范围，不是实现错**；我记作漏拦，交主 agent 定要不要补。
- 同一次被拒里各盘可以混着是①或②（`admission.rs:1075` 的 ① 臂是 `return Some`，`:1092` 的 ② 臂是 `then(…)`，逐盘各走各的）。判法逐盘判：任一盘 ① 判放不下就拒；② 的盘不挡拒。

### T3-c　多块盘一块短一块够

- 逐盘判：`devices_short_even_with_everything_a_floor_raise_could_release` 对 `refused.short_devices` 逐个 `filter_map`（`crates/singlefs-core/src/mounted_session.rs:5889` 起），只有被拒里列出的盘才判，没列的盘是够的。任何一块判出放不下，整个拒，不拿别的盘的富余补，与准入的逐设备合取同向（D28 已定项 1，附录 A，`28-挂载期承诺量.md:26`）。
- 上界逐盘取自己的 defer / 隔离，不跨盘。落点要两盘同槽，补丁没算这个（上界更偏大，宁大勿小）。
- 一块短、一块够、短的那块上界够：照推，推完两块各自的上界在第二轮各判。
- 没有整池多盘的新用例钉「一块够一块短」，核心层 10 条是纯函数、盘数 1 条；验证包 4 条里「an_accept_larger_than_everything_a_floor_raise_could_release_is_refused_without_a_series」是验证包 `common_admission_pool` 的 `PoolUnderTest` 两块盘的池（`pool.devices`，`accept_raises_the_floor_for_space.rs:726` 附近），没分「哪块短」。推不出的格：两块盘上界相差悬殊时是否按预期（推的：按逐盘 `filter_map` 应该对）。

### T3-d　被宽护卡住那条出路有没有被这一判吃掉

- 出路怎么列：`ways_out_after` 先无条件放 `UnmountThenMountWritableAgain`（「let mut ways_out = vec![WayOutOfTheWideProtection::UnmountThenMountWritableAgain {」`crates/singlefs-core/src/mounted_session.rs:839`），推环只在预演放得过时列（`:844` 的 `ChangeAdmittedAfterThePush`）。
- 能放得过的写：推环 / 卸载再挂 能放出的量不超过「可用 + defer + 隔离」（这两条出路放不出比 defer、隔离更多的东西；它们还要自己占槽）。所以走得通的出路对应的写，上界 ≥ 需求，判法放行，不被吃（推的）。
- 对拍：任务 11701 日志里点名「被宽护卡住」的 17 条全绿（上面 T1-d），含「列两条出路」「只列卸载」「推环后仍被拒、只列卸载、卸载放过」三种形。这些写上界够，所以不被拦（量的，日志）。
- 被判吃掉的只有：上界 < 需求的写，原来在被宽护卡住时**无条件**列了「卸载再挂」，其实卸载也放不过；现在这类写交回新成员，不列这一条假出路。我没找到一条「卸载再挂本来放得过、上界却 < 需求」的形（卸载抬 F 到现行那一版回收的是 defer 里的，隔离靠覆写根环槽解开，都在上界里；推的）。
- 代价：「被宽护卡住」那一报只在 `FloorAlreadyAtTheCeiling` 时出（`:1960`），新成员不带这一报；第二轮才翻面的写（T3-a）原来会带「被宽护卡住」，现在不带。那类写推环出路本来也放不过（上界 < 需求）。

## 检查单八格

### 1. 走不到的理由二次查实 —— 成立

两处 `expect`（补丁正文 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r1-diff.md` 第 1–265 行是产品代码段，第 266 行起是 `#[cfg(test)]` 模块；产品代码段 `grep -cE 'todo!|unreachable!|assert!|panic!'` = 0，`grep -nE '\.expect\('` 只有第 125、190 行两处，量的）：

- `SlotsAFloorRaiseCouldRelease::of_the_device`：「.expect("没挡的槽都没置分配位，分配位没置的都算空闲：没挡的槽数不大于空闲槽数")」（`crates/singlefs-core/src/mounted_session.rs:5835`）。要成立需要「分配位清着的槽数 = 空闲计数」：空闲计数初值「free_slots: unit_area_slots,」（`crates/singlefs-core/src/allocator.rs:477`，位图初值全 false），分配位只经 `set_blocking_bit(.., Allocated, ..)` 改，`grep -n 'SlotBlockingBit::Allocated'` 命中 537（分支）、847（清，配 `free_slots += span` 在 `:854`）、941（置，配 `free_slots -= span` 在 `:951`）三处（量的，我跑的 grep），没有别的写入。「没挡」要求分配位清着，所以没挡的槽 ≤ 分配位清着的槽 = `free_slots`，`checked_sub` 不会下溢。
- `devices_short_…` 的 `find … expect`：「.expect("拒绝里的盘来自这个分配器：准入的两份读数都按它的盘建");」（`crates/singlefs-core/src/mounted_session.rs:5900`）。函数私有，调用点只有 `push_one_floor_raise_for_a_refusal_for_space` 一处，它的三处调用拒绝与分配器是同一份（T2-a）；两份读数都按 `allocator.devices` 建：式子读数 `AdmissionReading::of_allocator`（「.map(|device_map| DeviceAdmissionTerms {」`crates/singlefs-core/src/admission.rs:763`，逐盘一条），落点读数「pub fn of_every_device_of_the_allocator(allocator: &PoolAllocator) -> Vec<Self> {」（`crates/singlefs-core/src/admission.rs:987`）。读数里盘身份两两不同由 `AdmissionReading::new` 断言。

这两处没有测试钉「走不到」；变异 7497「读分配器时扣住的槽读成 0」把第一处的减数换成 `free_slots`，只证 `free_but_blocked` 读对，不证 `expect` 走不到。结论靠读码（推的，没量过）。

### 2. 类型与名字 —— 成立（一处文档过期，见 T2-c 对子 2）

- 新名字全表（我 `grep -rn` 数出来，文件数）：`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 14 个文件（核心层 1、运行库测试 1、验证包 11、变异表 `crates/mutations.tsv` 1，`grep -rn … | awk -F: '{print $1}' | sort | uniq -c` 数的）；`SlotsAFloorRaiseCouldRelease` 只在 `mounted_session.rs`；`DeviceShortEvenWithEverythingAFloorRaiseCouldRelease`、`most_a_floor_raise_could_make_available`、`device_short_even_with_everything_a_floor_raise_could_release`、`no_space_even_with_everything_a_floor_raise_could_release` 都在 `mounted_session.rs`（后者与 `most_a_floor_raise_could_make_available` 各另在验证包 / 变异表出现一处，是引用）。没有名字撞已有成员。
- 字节与槽：槽数在补丁里一律是裸 `u64`（`SlotsAFloorRaiseCouldRelease` 三个字段），字节是 `BytesOnOneDevice(u64)` / `AvailableBytesOnOneDevice(i128)` 两个新类型。槽数裸 `u64` 与仓里惯例一致：`DeviceFreeMap::deferred_slots() -> u64`（`crates/singlefs-core/src/allocator.rs:633` 起）、`BytesOnOneDevice::of_slots(slots: u64)`（`crates/singlefs-core/src/admission.rs:75`）。补丁里槽→字节的唯一换算是闭包 `bytes_of_slots`，出口立刻包进 `AvailableBytesOnOneDevice`；槽对那臂的「槽数 × 数据单元槽数」是槽 × 槽/单元 = 对数 × 2 槽，乘完再 `bytes_of_slots`，单位对得上（T1-b）。字节与槽没有共用同一个新类型。
- `_ =>`：补丁新增非测试代码里零命中（对补丁正文 `grep -c '_ =>'` = 0，量的）。三个 `match` 都穷举：`most_a_floor_raise_could_make_available` 三臂，`floor_raises_pushed_by_this_accept` 补一臂，clippy 带 `-D clippy::wildcard_enum_match_arm`（实现员命令，任务 11699 退 0）。

### 3. 不兼容旧代码与旧格式 —— 成立

补丁正文 `grep -c '^diff --git'` = 3，文件是 `compaction_plan.rs`、`mounted_session.rs`、`singlefs-mount-runtime/src/lib.rs`（量的）。没有格式 crate 的文件，没有盘上字节的改动；`grep -cE '兼容|旧格式|legacy|deprecat|cfg\(feature'` 在补丁正文零命中（量的）。唯一的接口变化是 `UserChangeRefused` 加了一个成员（进程内类型，非盘上格式）；仓内所有 `match` 都已加臂（验证包、`scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs:253` 已加，主工作区现查）；`e206_fixed_point_cost_histories.rs` 的 `refusal_cause_name` 是 `other =>`，新成员按名字落进去、不丢数（实现员报告「交主 agent」，读码）。

### 4. 不新建判定库 —— 成立

补丁正文 `grep -cEi 'rocksdb|sqlite|File::create|std::fs|\.tsv|OpenOptions'` = 0（量的）。补丁带的 `crates/mutations.tsv` 21 行是现有的变异表（`awk -F'\t' '$1 ~ /^抬 F 先算：/ {c++}'` = 21，量的），不是新的判定存放处。

### 5. 所有任务多机可用 —— 成立

新用例所在的两个文件 `grep -nE 'std::env|env!|/home|/tmp|/proc|/sys|hostname|HOME|available_parallelism|std::fs|File::|tempfile|thread::|spawn'` 在 `accept_raises_the_floor_for_space.rs` 零命中，在 `mount_runtime_accept_raises_the_floor_for_space.rs` 去掉枚举名 `SizeOfTheUnlinkedFile::…`（含 `File::` 子串）之后也零命中（量的，我跑的，`grep -vc` 得 0）；核心层 10 条是纯函数。池全在内存盘上现造。内存：三条新运行库用例各 `mounted_copy_of(filled)` 拷一份填满的池，并行起时再多 3 份（每份约 0.7 GiB，来自正文，推的，没量过）；正文已记 15 GiB 量级。我的探针只用 384 槽小盘（两块各几 MiB 稀疏镜像），不碰本机状态。

### 6. 测试优先多线程 —— 不成立（理由没写）

两条崩溃循环用例（「fn a_write_that_never_fits_pushes_no_series_on_any_mount_of_a_crash_loop(shape: ShapeOfTheFill) {」`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1056`）一条历史内顺序崩溃 / 再挂 / 再试（每一步读上一步留下的盘），本质不能切片并行；两条形状各一个 `#[test]`，cargo 在用例之间并行。规则要求「不能并行的写明为什么」（`.claude/rules/implementation-workflow.md:79`）：这个文件里与线程有关的两句是「挂上（不起维护线程，用例一拍一拍推）」（`:123`），讲的是运行库不起维护线程，不是讲这个循环为什么单线程；函数文档只写了判别力，没写顺序依赖。理由没写，格打中（轻）。核心层纯函数用例不涉及并行。

### 7. 带 --item 的命令与项 —— 成立（读登记，没跑）

- 崩溃点重放三项都在 `.claude/gate.d/crash-point-replay-items.tsv`：`floor-raise-pushed-by-the-session`（第 18 行）、`truncate-to-zero-on-a-nearly-full-pool`（第 70 行）、`last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool`（第 76 行），首列逐字相同（`sed -n 18p/70p/76p | cut` 取出，量的）。
- crates-mutation-replay 五项在 `crates/mutations.tsv`，首列逐字匹配各恰好 1 行（`awk -F'\t' -v n=… '$1==n'`，量的）：「抬 F 先算：式子那一判的上界不算 defer」7481、「…不算隔离」7482、「抬 F 先算：槽对那一判一个槽只换一个槽的量不换一对」7488、「抬 F 先算：拒绝里丢掉推串的那几次被拒」7492、「accept 抬 F：拒绝里不带推成的串」7361。
- 一处读码提醒：`floor-raise-pushed-by-the-session` 这条流的录制**不经**判法（直接调 `push_one_floor_raise_within_the_admission_budget`，`crates/singlefs-verification/tests/crash_enumeration_floor_raise_pushed_by_admission.rs:132`），所以不管判法对不对，它的状态数不变；跑它不能证判法。

### 8. 新崩溃流多罩了什么 —— 成立（不用加新流）

- 这一判让被判放不下的写在推串之前就拒：盘上没有新写，崩在这之后的状态 = 拒绝之前的那个已落盘状态，不是新状态。已有的「会话推串」流录的是推串中间的崩溃状态（它自己直接推，不经判法）、贴边两条流的探测改成同时认新旧成员（`crash_enumeration_truncate_to_zero_on_a_nearly_full_pool_stream.rs:234` 一带）。
- 「被判放不下时录制流一个写都没有」已经被三条运行库用例钉住：录制流长度 0（「"{label}：一个写都不发（录制流记每个写、写零、discard 与屏障）"」`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1040`）加盘上扇区前后逐字节相等（「written_sectors_of_the_runtime(&mounted_pool) == sectors_before,」`:1080`、`:1135`）。再加小用例没有新信息。
- 没钉的是第二轮才判到的形（T2-c 对子 1）：前面推成的串后面接「放不下」。这一形的盘上状态 = 一串正常推串的状态，由「会话推串」流罩；缺的只是这一形的会话级用例，不是崩溃流。

## 要主 agent 跑的命令

探针不是模型：不写模型目录、没有 `rerun.sh`。探针源码在草稿目录，副本是冻结副本的整份拷贝（52 MiB）加这两份测试文件，已在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-sonnet/.deliverables` 写成「交接」。副本里的 `mounted_session.rs` 与冻结副本 sha256 相同（`080bd6bc…c2ced`，量的）。

探针文件 sha256（量的，`sha256sum`）：

```text
f4c4e3d892055f899d0718325187f4bf23ec0915ea87652feafff4e72850a00e  probe/probe_the_bound_of_what_a_floor_raise_could_release.rs
6c2c6f0bc199702788c04470132317fd995db71b61e6d0b26c867df6f5b75d33  probe/probe_the_bound_broken_on_purpose.rs
```

两份在副本里的位置：`probe-copy/crates/singlefs-verification/tests/` 下同名（`cp` 过去的，内容逐字同上）。第二份与第一份只差三处（diff 在草稿目录可复现：`diff probe/probe_the_bound_of_what_a_floor_raise_could_release.rs probe/probe_the_bound_broken_on_purpose.rs`）：上界里去掉 `deferred`（式子臂、槽对臂各一处）与测试函数名。

**命令 1　探针（两份一起编一次，第 2 份必须红）**

```text
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=探针两份测试文件 probe_the_bound_of_what_a_floor_raise_could_release 与 probe_the_bound_broken_on_purpose; 范围=2 个测试目标; 为什么=核抬 F 先算的上界在真推一串前后站不站得住，并证弄坏上界时探针会红' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-sonnet/probe-copy -- cargo test -p singlefs-verification --no-fail-fast --test probe_the_bound_of_what_a_floor_raise_could_release --test probe_the_bound_broken_on_purpose -- --nocapture
```

- 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-sonnet/probe-copy`（上面 `--cwd`）。
- 看哪几行：`task-server/task log <号> --all` 里
  - 第 1 份：`test the_bound_of_what_a_floor_raise_could_release_holds_across_every_series_pushed_on_three_histories ... ok`，且有 `BOUND_PROBE series_in_total=<n> violations=0` 一行、`n` 大于 0；每串每盘一行 `BOUND_PROBE label=… round=… device=… available_before=… available_after=… … drift=… pairs_before=… pairs_after=… bound_of_the_pairs=…`，`drift` 一列读出来贴回（T3-a）。
  - 第 2 份：`test broken_on_purpose_the_bound_without_the_deferred_slots_must_be_pierced_on_three_histories ... FAILED`，panic 信息里「上界被推串打穿」后面列出至少一条违例。**这份红是探针有牙的证据**：若它也绿（`... ok`），第 1 份的绿作废，T1 的反推观测算没做。
  - 整条命令的 `TASK_SERVER_RESULT` 会是 `type=failed`（第 2 份红）：以上面两行为准，不以 `exit=` 为准。
- 没编过：编不过是我的错，新派一条同立场的腿「承接：本报告」修；`FloorRaiseStop` 的 `Debug`、`PlacementRoomOnOneDevice`、`admission_reading_before_a_publish` 的签名我对着冻结副本读过，但没过编译器。
- 资源（推的，没量过）：编译验证包为主（先例：任务 11701 编 18 个目标连跑 1417 秒、内存峰值 10.75 GiB；这里只 2 个目标，编译共用同一份依赖）。运行本身：384 槽小盘三条历史，每条覆盖写约 60 次加最多 8 串，秒级、内存几十 MiB。别和别的重活并行起。

**待主 agent 跑的格**：T1-a / T1-b 的「上界站不站得住」（第 1 份）、T1 反推观测「探针有牙」（第 2 份）、T3-a 的 `drift`。其余格是读码与引日志，不等命令。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1-a 式子臂上界 = 可用+defer+隔离 | 一致（推的，待探针） | 每项逐项落到行；隔离全算比一串抬 F 放得出的多，偏松；用户原话只写 defer，kb 射程已写隔离 |
| T1-b 槽对臂上界 = 段外槽对 + 每个放得开的槽各补一对 | 一致（推的，待探针） | 一个槽只属一对；聚簇段集合只增不减；单位对得上 |
| T1-c 越位 | 一致 | PlacementRefused 类型上进不来，② 与合取 7 不碰；搬迁批、挂载续做、挂载准入三处推串不在这一判里（范围，不是错） |
| T1-d 误拒历史 | 没找到 | 18 个目标 1215 条全绿含 17 条「被宽护卡住」；整池没摆过恰好差一槽的边，不等于没有 |
| T2-a 同一份分配器 | 一致 | 三处调用拒绝与推串用同一个字段 |
| T2-b 三层记账 | 一致 | 预算、备忘、冻结种类、上一条完成的意图、current 逐项同旧成员 |
| T2-c 对子 1：第二轮才判到时「盘上逐字节不变」 | 冲突 | kb 与 `:5932` 说宽了，`:631` 带了条件；要改的是条款 |
| T2-c 对子 2：文档「八种」 | 冲突 | 现九个成员，文档没改（只是文档） |
| T2-d D16 已定项 10 的关系 | 一致 | 头一次被拒就判到的仍在任何读写之前拒，推过之后才判到的不是 |
| T2-e 循环依赖 | 没有环 | 上界含隔离而这一串放不全隔离，是故意保出路，代价白推一串 |
| T3-a 上界恰好等于需求 | 推的，待探针 | 纯函数钉住；整池没钉；推一串后上界漂 R−W，恰好相等那一格推完翻面 |
| T3-b 短缺种类混着来 | 漏拦 | 落点那一判在式子前，第一轮看不见式子短；② 永远照推（判决范围内） |
| T3-c 多块盘 | 一致（推的） | 逐盘判、任一盘放不下就拒；没有整池多盘「一短一够」用例 |
| T3-d 宽护那条出路 | 没被吃 | 走得通的出路对应的写上界够；被拦的是原本无条件列的假出路「卸载再挂」 |
| 检查单 1 走不到的理由 | 成立 | 两处 `expect` 靠分配位与空闲计数成对维护、单一调用点；无测试钉，靠读码 |
| 检查单 2 类型与名字 | 成立 | 槽裸 u64 同仓里惯例，字节新类型；无 `_ =>`；文档「八种」过期另记 |
| 检查单 3 不兼容旧代码与旧格式 | 成立 | 3 个文件、无格式 crate、零兼容词 |
| 检查单 4 不新建判定库 | 成立 | 零命中；变异表是现有表 |
| 检查单 5 多机可用 | 成立 | 零本机状态；新增 3 条运行库用例并行多 3 份约 0.7 GiB（推的） |
| 检查单 6 测试优先多线程 | 不成立 | 崩溃循环顺序依赖，但理由没写 |
| 检查单 7 --item 与项 | 成立 | 三个崩溃点项与五个变异项都在登记里，各恰好一行；没跑 |
| 检查单 8 新崩溃流 | 成立 | 不产生新盘上状态；「一个写都没有」已被三条运行库用例钉；缺第二轮形的会话级用例 |

## 推翻条件汇总

- T1-a / T1-b：探针第 1 份出现违例行；或第 2 份（弄坏）绿。
- T2-c 对子 1：找到一条第二轮才判到的历史，`floor_raises` 为空但盘上变了（那是代码错）。
- T3-a：探针 `drift` 全为 0（则恰好相等不翻面，至少在这三条历史上）。
- T3-b：造出一条历史，落点那一判可救、式子那一判不可救，第一轮没推串（则我读错了次序，`transaction.rs:13315` / `:13415`）。
- T3-d：造出一条「卸载再挂本来放得过、上界 < 需求」的写（则隔离 / defer 之外还有释放来源，上界算小了）。
- 检查单 1 的第一处 `expect`：找到一条路径改了分配位而没动 `free_slots`（`allocator.rs` 里 `SlotBlockingBit::Allocated` 的写入点多于 847 / 941）。

## 没做什么

- 不判别的腿（攻方）的格，不替主 agent 采纳或出判决；没往统一任务服务交任务、判决要引的数没复跑（命令在上面「要主 agent 跑的命令」一节）。
- 没编译、没跑任何测试或探针，没起 cargo。探针没过编译器。
- 没读禁读清单里的两项（Opus 本轮的产出与模型目录）。
- 没核 `readings.out`（实现员报告引的探针读数文件），那几个数是实现员报告里的，标了来源。
- 没逐个核运行库三条新用例是否在主 agent 的合入后副本上绿：任务 11701 只跑验证包，运行库那一族的绿是任务 11776 的证红日志里「抓到 3 条」间接给出，未单列基线绿。
- 实现员报告列为「串变少、钉绝对数的会动」的那几处（`random_histories` 的 `(32, 0, 0, 1141)` 等），我只核到：`random_histories` 是任务 11701 的 18 个目标之一（`grep -c 'Running tests/random_histories'` = 1），18 个 `test result` 全是 `0 failed`，所以没动或随补丁一起改过；动过哪个数我没逐个读。
- 草稿目录里的 `probe-copy`（52 MiB）留给主 agent 用完删（`.deliverables` 已写交接）；`log11701.txt` 248 KiB、`probe/` 28 KiB 是文本。

## 附录 A　kb 条款整行（脚本 sed -n 逐行取出，不手抄）

每条先一行出处，再一行原文。

出处：16-发布语义.md:38

| 抬 F 的上限 | **准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根；「有效根」「非空」在环里全部的根上取；挂载那一刻读不出或自证不过、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它（用户 2026-10-04 定，零轮：先定只并读不出的槽，代码轮第三轮之后重定为读不出或自证不过两类都并；与「环里最旧有效根」那一行把读不出的槽并进回收门槛同一个思路）。读坏槽与读回全零的槽都并进上限：挂载那一刻读回全零、之后这个进程没写过的槽，同读坏的槽一样当作可能是一条非空有效根，F 不越过它，能住的最大 txg 的取法同读坏的槽（「环里最旧有效根」那一行「挂载时读不出的槽」那一句；用户 2026-11-02 定「并」，原话在变更史）；上限卡在这两类槽上（并进它们之后上限不高于现行 F，不并它们时上限高于现行 F）时，不在这次挂载里报 ENOSPC，推一次不抬 F 的空发布（带现行 F、txg 加一，算进这次准入的已推次数，受「准入」那一行的 B 限）再判；预算放不下这一次，或这一次取不到落点，就拒这次请求，下一次请求从零预算接着推，推到这个进程写掉那一槽，上限不再被它卡住（用户 2026-11-02 定「推空」，原话在变更史；F 不越过读坏槽或读回全零的槽里的非空根）。一次处置的目标 = min(这次释放的释放代, 第 4 新的非空根)。**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小） |

出处：16-发布语义.md:44

| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)，保留池 = ckpt_cost 个元数据块（这次发布要接在后面的那一版现算，同一个数也是 c_max，D28（挂载期承诺量） 已定项 4；实现 `crates/singlefs-core/src/admission.rs` 的 `checkpoint_cost_of_the_version_to_build_on`、`checkpoint_reserve_pool`、`admission_reading_with_the_switch_reserve_of`；用户 2026-11-02 定，原话在变更史）；准入不够、或准入放行而落点取不到时，先推空发布抬 F 再判（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC）；**可写挂载的准入不够时，写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判**；一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8；上限卡在读坏槽或读回全零的槽上推的空发布也算在内，「抬 F 的上限」那一行），做满仍不够才报 ENOSPC；挂载处推满仍不够时挂载照样做成可写，之后的发布照发布准入判、不够报 ENOSPC，正常卸载照常（D2（RAID 条带策略） 已定项 13 那一格的例外）（用户 2026-09-26 定，被攻过零轮）；挂载续做的删除类发布被落点那一判以空间拒时同样推抬 F 再试，推满仍放不下挂载照样做成可写（D8（核心索引结构） 已定项 16 第 6 条，用户 2026-10-03 定） |

出处：16-发布语义.md:273

**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。会话的入口：`accept` 把一个改动收进待发；`publish_when_due(now)` 由调用方按周期调，到触发就发；`publish_pending_now` 立即排空；原 `publish_user_change` 语义不变，先排空待发再做它自己。数据写类可并：同一 inode 的写折叠、建 inode 求和；unlink、截断、建删快照、整理、回退、卸载是屏障，进门先排空待发、自己单发。`accept` 进门先做这一批合起来的准入与条数预判，过不了就在任何读写之前拒，不在返回成功之后拒；发布写失败冻结之后 `accept` 在任何读写之前拒。

出处：16-发布语义.md:275

**射程**：定的是频率，不定一次发布事件内部的顺序——那在 `litmus/commit-publish.litmus`，它钉的是写块内容 → `smp_wmb` → 写 generation，对频率一字未提。这一版不并行（用户 2026-10-31 定）：两个 inode 的写不并进一次发布（`accept` 拒成 `WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion`），并发读写与并行提交归里程碑六（[06-sixth-txn.md](../milestone/06-sixth-txn.md)）。`accept` 被空间拒时自己抬 F（用户 2026-11-09 定）：与 `publish_user_change` 同一个推法、同一个上限，推满仍放不下才拒；条数预判过不了与冻结照旧在任何读写之前拒。抬 F 之前先算（用户 2026-11-09 定）：推抬 F 放得出的全放出来也放不下就一串都不推，报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，盘上逐字节不变；式子那一判的上界是可用加 defer 加隔离，短在段外槽对那一判同样先算，上界恰好等于需求照推。先算之前量到的连续被拒推串的代价：每次 `series=2 empties=6`，一次白写 `writes=40 barriers=16 written_bytes=500736`；状态数掉下去不是推串磨掉的（`write=Nothing` 同样在第 4 次崩溃掉到 1）。两样已知边角：

出处：28-挂载期承诺量.md:21

可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数 − 环窗口(d)

出处：28-挂载期承诺量.md:26

- **逐设备算，每块盘各自满足**（逐设备合取）：式子对每块盘 d 各算一份，准入要每一块盘都够，一块盘不够就拒，不拿别的盘的富余补。前五项（含挂载期承诺量里的切换预留、被抛弃根独占量）取这块盘自己的值——两份副本各落一块盘，一块盘满了不能靠别的盘补；后三项——待删占用、已承诺预留、checkpoint 保留池——是全池的承诺量，**按全部副本之和记字节**（物理字节，与 D5（快照 / 空间记账机制） 已定项 7 的「已分配」同一口径），摊到每块盘各扣「副本之和 ÷ 副本数」。这个分界决定 D5（快照 / 空间记账机制） 已定项 4 里哪几个统计量要带设备维。「需求」按盘字节计、每盘相同（与 D5（快照 / 空间记账机制） 已定项 7 的「已分配」逐盘物理字节同口径）。

出处：28-挂载期承诺量.md:37

- 拒了在任何写之前返回：发布路径报 `PublishError::SpaceAdmissionRefused`，可写挂载报 `MountError::SpaceAdmissionRefusedBeforeAcquisition`。

出处：28-挂载期承诺量.md:40

- 发布经挂着的会话调（`crates/singlefs-core/src/mounted_session.rs`，实五）时，准入拒或落点被拒由会话照 D16（发布语义） 已定项 1「准入」那一行推抬 F 的空发布、再重发；可写挂载在 `mount::establish_instance` 写行与暖机之后推。
