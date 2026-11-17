# m4-gpugc-floor-raise-precheck-code-r3 攻方腿（Opus）报告（2026-11-10）

立场：假设撤回之后的先算判法是对的这一倾向错了，找前两轮没列的先算缺口（T2 主攻），兼 T1、T3 与检查单八格。
这一腿一条项目代码都没跑（子 agent 不交任务）：「量的」只有两类——我在冻结副本与仓里做的文本比对（python 读文件、grep），与第二轮主 agent 复跑留下的产物（任务 11964，`~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/opus-run/out/`，判决第一节引过它的 sha256 前缀）里我数出来的行；别的都是读码推的。新探针没编、没跑，等主 agent 跑「要主 agent 跑的命令」。
没读禁读清单里的文件（`research/prompts/m4-gpugc-floor-raise-precheck-code-r3-sonnet-output.md` 与 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-sonnet/`）。
被引的代码文件（`mounted_session.rs`、`admission.rs`、`transaction.rs`、`allocator.rs`、`mount.rs`、`compaction_plan.rs`、`crates/mutations.tsv`）在冻结副本与仓里此刻那一份 sha256 前 12 位逐个比过、相同，引文写仓里的相对路径，行号就是冻结副本的行号。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/`（从第二轮攻方模型拷来改：`device.rs` 原样，`pool.rs` 原样加一个查队列的方法，其余重写）。

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-opus/probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model
```

| 文件 | sha256 |
|---|---|
| `probe/Cargo.toml` | `2a82557e282dd5a0fa0db81988b1ee1de231e05173180e991f75319c7da9126e` |
| `probe/core_probe_module.rs` | `b091dc4c6a052f76a18ac629a61ea86272801019c21ec2b4b5b1ceca5e8855d1` |
| `probe/src/device.rs` | `81a8ae9934428a5a09ec1f59cc0f0567e49eb41aa7135ab584d9f956338959a3` |
| `probe/src/main.rs` | `0062c811588d7b6327055a99806fffcad55aeb2042dc9d80ec2cb1943df4d1e3` |
| `probe/src/pool.rs` | `e4125643126be91539e6a445988eb0719c53d82f89c8429260d2dc3ddeb7b4ac` |
| `probe/src/shapes.rs` | `1abff8d19c0b4c28d3eeea18c013e8992d99bf62a716e73c522c5d8784e92876` |
| `probe/src/tally.rs` | `ae517ba0855f588ba5935b00e8fbd27a58bbc51d8ad5e1797dfe9bdfb13a33a0` |
| `tools/patch_the_copy.py` | `bc019cb6b4be556c4972aa812aeea90cbe2959b937362e46b190e9c25fc8ca39` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `tools/rerun.sh` | `c5885b948ffe8ea24b77deee7b82bc522859712dc51ebe15a100bc7e151f1063` |

`out/` 是空的：探针一次没编、没跑（只用 `rustfmt --check` 核过语法：六份源文件都只报排版差、没有解析错误；`bash -n` 核过两份 shell，`python3 -m py_compile` 核过拼探针的脚本）。补丁的两处原文与工作区成员那一行在冻结副本里各命中 1 次（python `str.count` 量的）。

探针做什么（各文件头有一段）：
- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps`（与模板逐行比过：那一段之前、之后都相同）。冻结副本拷进 `$WORK/copy`，`tools/patch_the_copy.py` 拼探针，`cargo build --offline --release -p floor-raise-precheck-r3-probe`，跑两条自证（各 62 个配置）与全量（372 个配置），自证一条不合或全量自洽不合就退 1。
- `tools/rerun-from-a-clean-work-directory.sh`：服务撞内存上限重排同一条命令时，`rerun.sh` 拒非空的 `--work`；这一层先删掉运行目录（只认工作根下、名字以 `-run` 收尾的）再交给 `rerun.sh`。
- `tools/patch_the_copy.py`：只改拷贝里的 `mounted_session.rs` 两处——先算每进一次记下那一次拒绝的成员名；读一块被点名的盘那一处改调探针的 `judged`：照代码判一次、记下读数与上界，再按开关交结局（照代码 / 一律放得下 / 一律放不下），判法本身不动。文件末尾拼上 `probe/core_probe_module.rs`。
- 一个配置（`probe/src/shapes.rs`）：前缀与第二轮攻方探针同一套抽法（同一个种子前缀相同：两块盘各加 0 / 64 / 256 槽，40 / 120 / 300 步，一半崩在一次写的第一条记录之后再挂、C 那一圈罩那一槽）。探测点上挑目标之后另发 0 / 4 / 8 次小覆盖写（每次一个改用户可见状态的发布，抬 F 的上限往上走，让「推了写得进」的格有底）。g 从段外槽对最少那块盘的数 − 2 扫到全部算上再加 3（至多 160 个 g），每个 g 在两份拷贝上发：C 照冻结副本；O 关掉先算（被拒就推）。
- 每格记：C 白推（推了至少一串、没写进）按头一次被拒是哪一判与最后报什么分类，`Publish` 记拒绝成员名，探测点上合取 7 已不成立没有；推完之后按分配器给出缘由（槽对那一判推完仍短而推前上界够 = 上界松；式子那一判上界减掉推完还剩的 defer 就不够 = 上限够不到的 defer）；C 判出放不下而 O 写进 = 误拒；C 头一次就判出而已发成过版 = 调用开头的走空写过盘。
- 出路：被宽护卡住、C 那一圈罩着而头一次判出、推过一串之后判出、白推不报出路，四类在每个探测点各有 5 次份额，按这一类第 1、2、4、8、16 格取（第二轮那一份 10 次份额先到先得，见 T3-1）；被宽护卡住的先调 `ways_out_of_the_wide_protection` 要推环预演，再卸载、可写挂载、重发同一个写，记重挂之后 C 那一圈还罩不罩着。
- 崩溃状态：这一题判的是推不推、报什么、出路走不走得通；探针只用一个崩溃点（第一条记录之后）造 C 那一圈当前提，不当崩溃一致性的样本，不调枚举函数，没有要算的状态数（这一腿与接手的腿合计 0 个状态）。

## 各格判定一览

| 格 | 判定 | 一句话 | 分量 |
|---|---|---|---|
| T2-1 落得下那一判遮住合取 7（与记录条数上限） | 打中（读码；第二轮产物里 B 臂 `Publish+pushed` 1624 格是线索，拒绝成员没记） | 落得下那一判排在合取 7 之前，先算只看被拒那一判、放过，推一串；推完落得下过了，合取 7 拒成 `Publish`。抬 F 改不了全空段数，代码自己的注释写着「推了只多发几次空发布」 | 读码 + 第二轮产物计数；探针 `c_white_publish_refusal`、`c_white_conjunct7_short_before_the_series` |
| T2-2 槽对那一判每个放得开的槽各补一对（「defer 每槽算一对」） | 打中（第二轮产物量的：撤回之后那一臂推了没写进 20483 / 50198 格，推了写进 3 格；按判分类待探针） | defer 里一个数据单元两槽算两对、聚簇段里的槽也算；需求落在松出来的那一截上就白推一串，F 抬到上限；C 那一圈罩着时还报一条卸载重挂不一定走得通的出路。要单列一行欠账 | 第二轮产物 + 读码；探针 `c_white_first_slot_pairs`、`c_white_slot_pairs_bound_loose` |
| T2-3 式子那一判的 defer 一项含这一串上限够不到的槽 | 条款许（kb 第 275 行「可用加 defer 加隔离」），不是代码缺口；交用户 | 最近 3 次非空发布释放的槽在上限之上，抬 F 放不出，先算照算 | 读码；探针 `c_white_formula_unreachable_defer` |
| T2-4 调用开头先走空发成几步、这次改动头一次被拒就判出 | 字面不成立（读码），可达性推不出 | 条款与代码文档都写「头一次被拒就判出时盘上逐字节不变」，括号里只放过「推过一串之后、或 unlink 意图发成之后走空」 | 读码；探针 `fired_first_with_versions_published` |
| T2-5 上界算小、误拒的新机制 | 没打中（读码） | ckpt_cost、已承诺预留、扣住的槽、聚簇段集合都推不出推完比上界多 | 读码；探针 `misrefused`（底见 T3-4） |
| T1-1 撤回与第 119 批那一版逐处同不同 | 相同（量的，文本比对） | 第三轮改动全文与第二轮改动全文后 12 段按行互为逆（删 46 行、回 4 行），第一段「九种」留着；第一轮改动全文加的 417 行全在 | python 比对 |
| T1-2 残留 | 没有（量的，grep） | 字段、参数、`expect`、用例名、变异行都没留下 | grep |
| T1-3 D16 已定项 10 射程逐句 | 对得上，除 T2-4 那一句 | 射程九句逐句对到代码行 | 读码 |
| T3-1 `outlet_lost_first_round=0`、`outlet_lost_after_a_series=0` | 装置走不到（量的：两项一次都没试） | 每个探测点 10 次卸载重挂份额全被更小 g 上的被宽护卡住格先用光：610 = 61 × 10、90 = 9 × 10 | 第二轮产物 + 模型源码 |
| T3-2 判决把全量读数安在「读成 0 的一臂」上 | 口径错位（量的） | 全量里 `stuck_reports`、`outlet_lost_first_round` 都数在 A（带那一截）上；读成 0 那一臂自己的数在自证一：`stuck_reports=1079 c_ring_holding_at_probe=9` | 第二轮产物 |
| T3-3 被宽护卡住之后照出路卸载重挂 | 0 / 700 写进（量的） | 宽护那一报出现过，列的卸载重挂一次都没让那个写写进 | 第二轮产物 |
| T3-4 `published_where_the_old_bound_fired=0` 的底 | 底只有 3 格（量的） | 全量 50198 格里推了写得进的只有 3 格；自证二（那一截读成大数）一格都没有 | 第二轮产物 |
| T3-5 `vain_series_by_the_inside_term` 数的是什么 | 数的是格（g 值），独立单位是探测点（读码 + 量的） | 18105 格出自 282 个探测点里的 217 个；同一点上的格共用一份前缀 | 模型源码 + 产物 |
| T3-6 两份探针共用的前提 | 四样（读码） | 同一份核心代码、同一条出路判法（卸载重挂再发）、都没造读故障的宽护、都只比那一截有没有、不数撤回之后自己白推几格 | 读码 |
| 检查单：走不到的理由二次查实 | 成立 | 先算里还剩两处 `expect`，调用点各一处，没变 | grep |
| 检查单：类型与名字 | 成立 | 结构体只剩三个槽数字段 | 读码 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 只删 | 读改动全文 |
| 检查单：不新建判定库 | 成立 | 只删 | 读改动全文 |
| 检查单：所有任务多机可用 | 成立 | 没有新用例 | 读改动全文 |
| 检查单：测试优先多线程 | 成立 | 没有新用例；判法模块 10 条都是纯函数 | 读码 |
| 检查单：带 --item 的命令与项 | 成立 | 删掉的变异行与用例名零命中；余下 21 行「抬 F 先算」原文各命中 1 次 | python + grep |
| 检查单：新崩溃流多罩了什么 | 成立（不要新流） | 撤回只少推，不改推的写什么；T2-1、T2-2 的白推走已有的抬 F 那一串 | 读码 |

## T2　未定项清单的完整性（主攻）

前两轮已判的格（第一轮 T1-1 至 T1-4、T3-3、T3-4，第二轮 T1-2、T1-3、T1-4）不再攻。下面每一形都在撤回之后的代码上，与聚簇段那一截无关。

### T2-1　落得下那一判遮住合取 7：先算放过、推一串、再拒成推抬 F 改不了的那一种

打中（读码；第二轮产物里有线索，拒绝成员没记）。历史（每一步许可它的那一句）：

1. 同一次挂载里盘写得很满，有一块盘记录口径的全空聚簇段数掉到 `N₀`（第一版 1）以下：段里只要还有一条未释放的记录就不算全空（「一段里只剩 defer 槽时这里算它全空」（`crates/singlefs-core/src/allocator.rs:906`）反过来说）。不要故障、不要崩溃。
2. 用户发一次带数据单元的写，段外槽对少于这次的数据单元：发布路径先判落得下那一判（「admit_the_units_landing_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13317`）），拒成空间准入（「.map_err(PublishError::SpaceAdmissionRefused)?;」（`crates/singlefs-core/src/transaction.rs:13328`））。合取 7（「admit_the_empty_cluster_segments_floor_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13373`））与记录条数上限（「refuse_a_publish_of_more_journal_records_than_the_limit(」（`crates/singlefs-core/src/transaction.rs:13350`））都排在它后面，这一次轮不到。
3. 先算只认空间准入那一种拒绝（「let PublishError::SpaceAdmissionRefused(refused) = refusal else {」（`crates/singlefs-core/src/mounted_session.rs:6015`）），读被点名那块盘：槽对那一判上界不小于需求（defer、隔离或扣住的槽够补，见 T2-2），不判放不下；拒绝是空间不够，推一串。
4. 推完重发：落得下那一判这次过了，合取 7 照判。全空段数按记录口径数，抬 F 回收 defer 不改它——代码自己的注释：「抬 F 把 defer 里的槽回收不改这个数，推了只多发几次空发布；原样交回这个成员」（`crates/singlefs-core/src/mounted_session.rs:2094`）；推的空发布自己开段还可能再减一。拒成「.map_err(PublishError::EmptyClusterSegmentsBelowTheFloor)?;」（`crates/singlefs-core/src/transaction.rs:13381`），它不是空间不够（`mounted_session.rs` 第 2093–2095 行那一臂），交回 `Publish`，`floor_raises` 里带着刚推的那一串。

结局：一串 2 或 3 次空发布、F 抬到上限、回退候选集缩到下限，换来的是一个推抬 F 本来就改不了的拒绝。记录条数上限是同一形（与 g 定死，抬 F 改不了），只是排得更前。

线索（量的，第二轮产物 `probe.out` sha256 前缀 `8e72cdeed7477443`，我按 `COUNT … A=… B=…` 行数的）：读成 0 的那一臂（就是撤回之后的代码）结局 `Publish+pushed` 1624 格（50198 格里；其中崩后再挂那一族 1609 格），自证一（两臂都读成 0，62 个配置）172 格；推之前就拒成 `Publish` 的 212 格。第二轮探针没记 `Publish` 里是哪一种拒绝，也没记探测点上合取 7 成不成立，所以这一形「是被遮住、还是推串自己把全空段推下去」分不出——新探针两样都记。

四句：
1. 分不分辨臂：分辨「先算只看被拒那一判」（冻结副本）与「先算另判合取 7 这一刻成不成立」（下面改法甲）；第二轮那一截的有无不分辨（B 臂就是撤回之后）。
2. 系统那一刻看得到吗：合取 7 看得到。它读的是分配器增量维护的全空段数，现成的判是「pub fn foreground_writes_are_below_the_empty_cluster_segments_floor(」（`crates/singlefs-core/src/compaction_plan.rs:1882`），会话文件第 55 行已经引进来；抬 F 不改这个数（第 2094 行注释）。记录条数上限要走完固定点才知道，先算那一刻只有下界（数据单元数），看不全。
3. 字面分句：D16 已定项 10 射程「推抬 F 放得出的全放出来也放不下就一串都不推」（`.claude/kb/decisions/16-发布语义.md:275`）——合取 7 不是空间准入那几判，它在不在「放得下」的口径里条款没写；更贴的是代码自己定的「推了只多发几次空发布；原样交回这个成员」（第 2094 行）：设计是合取 7 不推，被落得下那一判遮住时推了。射程那一句「短缺种类混着来时可能推过一串才判出」不罩它：合取 7 不是先算的短缺种类，最后报的也不是 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`。
4. 改法碰不碰得到：第一轮的改法（加那一截）与第二轮攻方改法一（判出放不下时带出路）都碰不到；碰得到的只有改法甲（零轮，没实现）。

翻面条件（跑前写下）：探针全量 `c_white_publish_refusal` 为 0 → T2-1 改「推的，取样没撞上」；大于 0 而 `c_white_conjunct7_short_before_the_series` 也大于 0 → 坐实被遮那一形；大于 0 而后者为 0、`COUNT c-white detail … cause=EmptyClusterSegmentsBelowTheFloor` 大于 0 → 是推串自己把全空段推下去的那一形（系统那一刻只有「全空段等于 `N₀`」这一个下界可判）；`cause=JournalRecordsOfThePublishExceedTheLimit` 大于 0 → 记录条数那一形。

### T2-2　槽对那一判「每个放得开的槽各补一对」：白推一串，还可能列一条走不通的出路

打中（第二轮产物量的是现象，归因待探针）。这就是第二轮攻方 T1-4 说的「defer 每槽算一对」的松；第二轮判决只登了它的一个后果（推过一串之后判出放不下、丢出路），头一次就白推这一后果没登。

上界：段外槽对加「slots.deferred + slots.isolated + slots.free_but_blocked;」（`crates/singlefs-core/src/mounted_session.rs:5982`）每槽一对（「+ bytes_of_slots(slots_that_could_complete_a_pair * slots_of_a_data_unit),」（`crates/singlefs-core/src/mounted_session.rs:5987`））。实际推抬 F 放得出的：defer 里一个数据单元两槽回收出来是一对，按上界算两对；聚簇段里回收出来的元数据单槽仍在聚簇段里，一对都不补；隔离的槽要写掉 R 那一槽才回来（一串里写不写得到看环位置）。

历史：
1. 同一次挂载里几次整份重写把旧数据单元放进 defer，之后又有至少 4 次改用户可见状态的发布（上限够得到它们）。
2. 用户写 g 个数据单元，段外槽对 P < g，落得下那一判拒（「if room_on_the_device.slot_pairs_for_data_units < units.data_units {」（`crates/singlefs-core/src/admission.rs:1075`））。
3. 先算：上界按 P + D 对算（D 是 defer 槽数），真能放出的是 P + D / 2 对左右；g 落在两者之间，照推（「(upper_bound.0 < i128::from(short.demand.0)).then_some(」（`crates/singlefs-core/src/mounted_session.rs:6000`）不成立）。
4. 推完重发仍拒；再先算：上界随 defer 减少一起掉，掉到需求以下就报 `NoSpaceEvenWith…` 带着刚推的串（第二轮 T1-4 那一形），没掉下去就再推一次，上限已不高于 F（「if ceiling <= current.root.rollback_floor {」（`crates/singlefs-core/src/mount.rs:4853`）），报推满仍不够；C 那一圈罩着时报被宽护卡住、列卸载重挂，而重挂之后同一个写照样放不下（T3-3：0 / 700）。

量的（第二轮产物，我按 `COUNT … A=… B=…` 行逐格加的；B 臂把那一截读成 0，就是撤回之后的代码）：

| 产物 | 格 | B 推了没写进 | 其中不报被宽护卡住 | B 推了写进 |
|---|---|---|---|---|
| `probe.out`（372 个配置，sha256 前缀 `8e72cdeed7477443`） | 50198 | 20483 | 15209 | 3 |
| `self-check-arm-a-zero.out`（62 个配置，sha256 前缀 `96a3d96e6a143e51`） | 4326 | 3167 | 2369 | 0 |

拆法：`probe.out` 里 B 的 15209 格 = `NoSpaceAfterRaisingTheFloor+pushed` 7634 + `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease+pushed` 5951 + `Publish+pushed` 1624；另 5274 格是 `+pushed+stuck`。第二轮判决撤回那一截时引的是「A 白推而 B 头一次就判出」的 18105 格（其中不报出路 17870）；同一份产物里撤回之后的代码自己推了没写进的不报出路格 15209，与它同一个量级。哪几格是槽对那一判、哪几格是式子那一判（T2-3）、哪几格是 T2-1，第二轮探针没记，新探针按头一次被拒那一判分（`c_white_first_*`），并按推完之后的分配器给缘由。这些都是扫边界的格（g 从段外槽对最少数 − 2 起扫），格数随扫法变，不是自然负载下的频率；一个探测点是一个独立单位（T3-5）。

四句：
1. 分不分辨臂：分辨「每个放得开的槽补一对」（冻结副本）与「按回收出来成几对算」（第二轮攻方改法表第 4 行：数据单元两槽一对、聚簇段里的不算，要按段另维护 defer 计数）；第二轮那一截的有无不分辨。
2. 系统那一刻看得到吗：一半。defer 槽在哪一段、成不成对，分配器有位图，没有增量计数；准入路径不许扫（`.claude/rules/fs-design.md`「记账是事务的副产品」），要新维护一个按段的 defer 计数才看得到。粗一点的「defer ÷ 2」看得到，但它在回落那一形上会少算（提交内生块回落到最低空槽时元数据单槽落在聚簇段外，另一半空着时放开它就多一对），宁大勿小不许 ÷ 2。
3. 字面分句：射程只写「短在段外槽对那一判同样先算」（`.claude/kb/decisions/16-发布语义.md:275`），没给这一判的上界；代码文档那一句「另一半本来就空着时放开一个槽就多一对」（第 5967 行）只在另一半空着时成立。违的是同一行开头「推抬 F 放得出的全放出来也放不下就一串都不推」——第二轮判决撤回那一截引的就是这一句。
4. 改法碰不碰得到：撤回（第二轮判决的改法）碰不到；攻方改法一（判出放不下时带出路）碰不到头一次的白推；碰得到的只有改法乙（零轮）。

要不要单列一行欠账：要。理由三条——前两轮判决只把它当「推过一串之后丢出路」的成因登了（第二轮判决第二节第 3 行），头一次就白推、F 抬到上限这一后果没登；它是撤回之后白推的主要来源之一（待探针按判分出比例）；它让被宽护卡住那一报列出一条照做也写不进的卸载重挂（T3-3）。

翻面条件：探针全量 `c_white_first_slot_pairs` 为 0 → 改「推的，取样没撞上」；大于 0 而 `c_white_slot_pairs_bound_loose` 占它大半 → 坐实「松」是成因；`c_white_slot_pairs_bound_loose` 很小而槽对那一判白推很多 → 白推另有成因（例如推串自己的提交内生块在开着那一段 bump 掉几对），回看。

### T2-3　式子那一判的 defer 一项含这一串上限够不到的槽

条款许，不算代码缺口（读码）。准入抬 F 的上限是第 4 新的非空根，推的空发布不算状态，所以最近三次非空发布释放的槽在这次挂载里推多少串都放不出；代码明写「defer 窗口里全部已释放的槽：只算槽数、不问这一串的上限够不够得到它们。」（`crates/singlefs-core/src/mounted_session.rs:5944`），条款也写「式子那一判的上界是可用加 defer 加隔离」（`.claude/kb/decisions/16-发布语义.md:275`）。

历史：同一次挂载里连着三次大重写（每次把上一份的数据单元放进 defer），紧接着一个写，需求落在「可用 + 上限够得到的 defer」与「可用 + 全部 defer」之间：先算放过，推一串到上限，回收够得到的那一截，重发仍拒；再先算上界不变（回收的挪进可用），再推一次，上限不高于 F，推满仍不够，不报出路。上限已在 F 时这一形一个字节都不写（只换了报的成员）。

四句：分辨「全部 defer」（冻结副本、条款）与「释放代不高于上限的 defer」；系统那一刻看不到——上限要读根环与现行那一版的实例表（`push_one_floor_raise_within_the_admission_budget` 里算上限那一步读盘），分配器也没有按释放代的 defer 计数，先算是只读内存的；字面合条款；前两轮改法都碰不到。所以写进交用户表：射程同一行里「放得出的全放出来」与「上界是可用加 defer 加隔离」两句在这一形上互相顶，要么条款认这一截白推，要么加按释放代的计数与上限读数。探针 `c_white_formula_unreachable_defer` 给它的份量。

### T2-4　调用开头先走空、这次改动头一次被拒就判出：盘上不是逐字节不变

字面不成立（读码），可达性推不出。射程写「头一次被拒就判出时盘上逐字节不变（推过一串之后、或 unlink 意图发成之后走空才判出的，盘上有这次调用写的东西）」（`.claude/kb/decisions/16-发布语义.md:275`），代码文档写「头一次被拒就判到这里时它们是空的，盘上逐字节不变」（`crates/singlefs-core/src/mounted_session.rs:645`）。可是每一次改动先走空：「if !current_version_of_trees.queue_of_committed_intents_is_empty() {」（`crates/singlefs-core/src/mounted_session.rs:3682`）——队列不空时走空的那几步先发成、记进 `versions_published`，之后这次改动头一次被拒就判出放不下，报里 `floor_raises` 空而 `versions_published` 不空，盘上有这次调用写的东西，括号里两种都不是。

可达性：要这一次调用开头队列不空、而且走空不推就走得动。上一次调用的走空停在空间不够上时，这一次开头状态没变、照样停，走不动；停在不是空间不够的错上（写失败冻结、抬 F 那一串读写出错）、之后错过去了，才走得动。所以要一次瞬时的读写错，推的，没造。

四句：不分辨臂（只是条款与文档的一句）；系统看得到（报里 `versions_published`）；字面分句是上面两句；改法：条款与第 645 行文档补「这次调用开头没走空发成任何一步」，行为不用改（走空本来就该走）。探针 `fired_first_with_versions_published` 只是看取样里有没有（探针不注故障，多半是 0）。

### T2-5　上界算小、误拒：试过的新机制

没打中（读码）。逐个看推一串之后可用会不会比上界多出一截：checkpoint 保留池与切换预留按 ckpt_cost 算，分配记录树那一项按几何整棵树（`admission.rs` 第 1186–1188 行），空发布不改；已承诺预留按这次写自己现算；扣住的槽式子本来算空闲、槽对那一判已算进上界；聚簇段集合一次挂载里只增不减，段外槽对只会变少；`PlacementRefused` 不先算、照推。探针的 `misrefused` 给取样，底有多大看 `oracle_published_after_a_series`（T3-4）。

## T1　「已定」项撑不撑得住

### T1-1　撤回之后与第 119 批那一版逐处同不同

相同（量的，文本比对，没跑项目代码）：
- 第三轮改动全文（`research/prompts/_m4-gpugc-floor-raise-precheck-code-r3-diff.md`）的 `+` / `-` 行，与第二轮改动全文（`_…-r2-diff.md`）除第一段（「八种」改「九种」）之外的 12 段按行多重集互为逆：第三轮删 46 行、回 4 行，正是第二轮那 12 段加的 46 行、删的 4 行（python `Counter` 比，两个方向都相等）。
- 第一轮改动全文给 `mounted_session.rs` 加的 417 行，在冻结副本里逐行都找得到；先算那一段（第 5934–6145 行）与判法模块（第 8184–8430 行）里每一个非空行，要么在第一轮改动全文里（加的或上下文），要么是第一轮之前就在、第一轮没碰的推串那一段。判法模块 10 条 `#[test]`，与第一轮改动全文里那一段的条数相同。
- 冻结副本的 `mounted_session.rs` sha256 `4d58410438a9…`，与开工快照 `crates-sha256.txt` 那一行、与仓里此刻那一份相同。

这一法比的是行，不比行的位置；位置由上面第一条（逆补丁按段对上）兜住。

### T1-2　残留

没有（量的）：`grep` `unblocked_slot_pairs_inside_the_cluster_segments`、`an_unblocked_slot_pair_inside_the_cluster_segments`、`聚簇段里没挡`、`卸载重挂放得出`、`slot_pairs_a_remount_frees`，冻结副本的 `crates/` 与 `scripts/` 零命中；仓里 `crates/ scripts/ .claude/gate.d .claude/kb` 零命中；`git grep --untracked` 删掉的用例名（`research/` 之外）零命中。`of_the_device` 回到一个参数（「fn of_the_device(device_map: &DeviceFreeMap) -> Self {」（`crates/singlefs-core/src/mounted_session.rs:5954`）），唯一调用点在第 6029 行。

### T1-3　D16 已定项 10 射程与撤回之后的代码逐句

射程在 `.claude/kb/decisions/16-发布语义.md` 第 275 行（kb 文件 sha256 与开工快照 `kb-sha256.txt` 相同）。

| 射程里的一句 | 代码 | 对不对得上 |
|---|---|---|
| 推抬 F 放得出的全放出来也放不下就一串都不推，报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` | 第 6080–6088 行判出就交回、不推 | 对得上；「放得出的」由上界近似，上界比放得出的大（T2-2、T2-3），比放得出的小的没找到（T2-5） |
| 头一次被拒就判出时盘上逐字节不变 | 第 645 行文档同一句 | 调用开头先走空那一形不成立（T2-4） |
| 推过一串之后、或 unlink 意图发成之后走空才判出的，盘上有这次调用写的东西 | 报里带 `floor_raises`、`versions_published` | 对得上 |
| 先算罩 `accept`、`publish_user_change` 与走空三处 | `push_one_floor_raise_for_a_refusal_for_space` 三处调用（第 3795、6195、7030 行） | 对得上 |
| 不罩改动发成之后另发的搬迁批 | 搬迁批不经这个函数 | 对得上（第一轮判过） |
| 也不判提交内生块那一判（照推） | 「JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,」（`crates/singlefs-core/src/mounted_session.rs:5990`） | 对得上 |
| 短缺种类混着来时可能推过一串才判出 | 只读被拒那一次点名的盘 | 对得上；合取 7 不是短缺种类，被遮那一形另算（T2-1） |
| 式子那一判的上界是可用加 defer 加隔离 | 「short.available.0 + bytes_of_slots(slots.deferred) + bytes_of_slots(slots.isolated),」（`crates/singlefs-core/src/mounted_session.rs:5977`） | 对得上 |
| 短在段外槽对那一判同样先算，上界恰好等于需求照推 | 第 5980–5989 行；严格小于才判出（第 6000 行） | 对得上；这一判的上界条款没写（T2-2） |

## T3　第二轮报出的数能不能核

读的是第二轮主 agent 复跑留下的产物（任务 11964）：`~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/opus-run/out/`，`probe.out` sha256 `8e72cdeed7477443cb9ae038c88d0c979bace1e66c51de7cbdb192a1ebfeebe5`，`self-check-arm-a-zero.out` sha256 `96a3d96e6a143e513f8586c0cf81bb5fc3e55608c0ae078b7edc7b54eda8095c`；模型源码读的是仓里那一份 `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/`（与主 agent 拷去跑的那一份 `diff -r` 相同）。下面整行抄的都是这两份产物里的原样行。

### T3-1　`outlet_lost_first_round=0`、`outlet_lost_after_a_series=0`：装置走不到

两项一次都没试过（量的）。走卸载重挂（D）的份额每个探测点 10 次（「const REMOUNTS_PER_PROBE_POINT: u32 = 10;」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/probe/src/shapes.rs:55`）），先到先用（「if follow_the_way_out && !a.published() && remounts_left > 0 {」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/probe/src/shapes.rs:508`））。g 从小往大扫，同一个探测点上被宽护卡住的格（上界够、推到上限）在 g 小的那头，头一次就判出放不下的格（「let lost_with_the_c_ring = a.kind == NO_SPACE_EVEN && holding > 0;」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/probe/src/shapes.rs:468`））在 g 大的那头。产物：

```text
probe COUNT c-ring-holding-at-probe 61
probe COUNT stuck remount-retry refused 610
probe COUNT stuck-reports 7730
self-check-arm-a-zero COUNT c-ring-holding-at-probe 9
self-check-arm-a-zero COUNT stuck remount-retry refused 90
self-check-arm-a-zero COUNT stuck-reports 1079
```

610 = 61 × 10、90 = 9 × 10：C 那一圈罩着的每个探测点，10 次份额全用在被宽护卡住的格上；两份产物里 `outlet-lost` 起头的计数行都是 0 行（`grep -c` 量的），也就是 `outlet-lost-first-round remount-retry …` 与 `outlet-lost-after-a-series remount-retry …` 一次都没记过。所以判决第二节第 2、3 行「`outlet_lost_first_round=0`」「`outlet_lost_after_a_series=0`」是「没试」，不是「试了没撞上」。新探针四类出路各有份额、按 1、2、4、8、16 格取。

### T3-2　判决把全量的读数安在「读成 0 的一臂」上

口径错位（量的）。判决第二节第 2 行写「那一截读成 0 的一臂（第一轮修补之前的判法）上 `outlet_lost_first_round=0`，`stuck_reports=7730`、`c_ring_holding_at_probe=61`」。全量那一趟 A 照冻结副本（带那一截），这几项都数在 A 上（「let a = publish_observed(&mut arm_a, change, options.arm_a);」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/probe/src/shapes.rs:452`），`stuck-reports` 看的是 `a.stuck`）；读成 0 那一臂自己的数在自证一（62 个配置）：`stuck_reports=1079`、`c_ring_holding_at_probe=9`、`outlet_lost_first_round=0`（同样没试过，见 T3-1）。结论方向不变（读成 0 那一臂一样报过被宽护卡住），引的数要换。

### T3-3　宽护那一报出现过，不等于它列的出路走得通

量的：被宽护卡住之后照出路正常卸载、再可写挂载、重发同一个写，`probe.out` 610 次、自证一 90 次，全部被拒（只有 `refused` 一行，没有 `published` 行）。这 700 次取的是每个探测点上 g 最小的那几格被宽护卡住的格，按理最容易放得下。产物没记重挂之后 C 那一圈还罩不罩着，所以分不出是「重挂没解开 C 那一圈」（这一族是崩在第一条记录之后、不带读故障的窄的那一圈，代码文档写的是「报这一形不说明写掉 R 那一槽之后放得下」（`crates/singlefs-core/src/mounted_session.rs:721` 那一段））还是「上界比重挂放得出的大」（T2-2）。判决用 `stuck_reports=7730` 说「宽护那一报在取样里出现过」是对的；但第一轮采纳那一截要保的「出路」，在两份取样里没有一次让写写进。新探针记重挂之后 C 那一圈、推环预演的结局。

### T3-4　`published_where_the_old_bound_fired=0` 的底只有 3 格

量的。这一项要 A 推了写进而 B 头一次判出（「let published_where_the_old_bound_fired = a.published() && b.fired_first_round();」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/probe/src/shapes.rs:467`））；全量里 A 推了写进的格一共 3 格（`probe COUNT one-long-mount A=published+pushed B=published+pushed 3`），自证二（那一截读成大数）一格都没有（`self-check-arm-a-huge` 的 `A=` 行里没有 `published`）。g 从段外槽对最少数 − 2 起扫（第 439 行），推抬 F 帮得上的那一窄条多半被步长跨过。所以判决第二节第 4 行把它与第一轮任务 11937 的 `hits=0` 并成「两份取样都没撞上，记没打中」，第二份的底是 3 格。新探针在探测点前抬上限，`oracle_published_after_a_series` 报底有多大，自证二（一律判放不下）证误拒那一数会红。

### T3-5　`vain_series_by_the_inside_term` 数的是什么

数的是格，不是串（读码 + 量的）：每个 g 一格，A 推了至少一串、没写进、B 头一次判出就记 1（「let vain = !a.published() && a.series > 0 && b.fired_first_round();」（`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/probe/src/shapes.rs:466`））；每格的两臂从同一份池拷出来。372 个配置里 90 个在造 C 那一圈时就退出（`create-refused` 57、`write-does-not-fit-without-pushing` 33），走到探测点的 282 个；`vain-sizes-per-probe-point` 非 0 的 217 个。所以「白推 18105 串」是 217 个探测点上 18105 个 g，格数随步长与扫法变，独立单位是探测点。判决的结论（那一截白推、换不来写进）不受影响，引数时要写成「格」。

### T3-6　两份探针共用的前提

读码：
1. 同一份冻结副本的核心代码，同一种内存稀疏盘；没有一份是另写的分配器模型。
2. 出路都只用「正常卸载、再可写挂载、重发同一个写」一种判；推环那一条出路（预演过才列）两份都没走。
3. 都没造读故障的宽护本身（攻方用崩在第一条记录之后的窄的那一圈；辩方用验证包已有的场景，`forms` 那一项撞同名镜像 panic）。T3-3 的 0 / 700 因此回答不了「宽护时出路能不能走」。
4. 都只比「那一截有没有」两臂，不把撤回之后的代码自己推了没写进的格当一项报出来（攻方产物 `COUNT` 行里有，T2-2 是我数的；辩方的 `white_push_new` 按它 `side_by_side.py` 文件头的定义只数新上界那一臂）。
5. 都只发单个 inode 的整份写；`accept` 攒批那一路没走。

## 检查单八格

### 走不到的理由二次查实：成立

撤回删掉了第 122 批那一处新 `.expect`；先算那一段（第 5934–6208 行）里还剩两处：「.expect("没挡的槽都没置分配位，分配位没置的都算空闲：没挡的槽数不大于空闲槽数"),」（`crates/singlefs-core/src/mounted_session.rs:5961`）与「.expect("拒绝里的盘来自这个分配器：准入的两份读数都按它的盘建");」（`crates/singlefs-core/src/mounted_session.rs:6026`），就是第一轮追过的两处。调用点：`of_the_device` 只在第 6029 行调，`devices_short_even_with_everything_a_floor_raise_could_release` 只在第 6081 行调（`grep -n` 量的），都没变。

### 类型与名字：成立

`SlotsAFloorRaiseCouldRelease` 只剩 `deferred`、`isolated`、`free_but_blocked` 三个字段，都是槽数（第 5943–5951 行）；第二轮攻方说的「槽对与槽共用裸 u64」随那一截一起没了。

### 不兼容旧代码与旧格式：成立

改动全文只有删与回到第 119 批那一版的行，没有认旧版本、旧布局的分支（T1-1 的比对）。

### 不新建判定库：成立

改动全文里没有新的目录、表、库。

### 所有任务多机可用：成立

没有新用例、新脚本。这一腿的探针只从参数与 `--repo` 下的登记文件取路径与线程数，不写死主机。

### 测试优先多线程：成立

没有新用例；判法模块 10 条都是纯函数，不涉及切片。

### 带 --item 的命令与项：成立

删掉的变异行那一句与它的原文在 `crates/mutations.tsv` 零命中（`grep -c` 量的）；余下 21 行「抬 F 先算」（第 7477–7497 行）的原文按 `\n` 展开之后在 `mounted_session.rs` 里各命中 1 次（python `str.count` 量的），没有一行因撤回失去锚点。删掉的用例名在 `research/` 之外零命中（T1-2）。只读登记，没跑 `--list-items`。

### 新崩溃流多罩了什么：成立（不要新流）

撤回只让先算少放过一截，推的那一串写什么不变；T2-1、T2-2 的白推走的是已有的抬 F 那一串，写形与 `floor-raise-pushed-by-the-session` 那一项罩的同一种（推的）。

## 这一腿提的改法各修哪一格

都是这一腿提的，只在读码上推过，没实现、没在副本上跑，被攻过零轮。每一格都是「推的」；探针跑出来只回答打中的格在不在，回答不了改法本身。

| 改法 | T2-1 被遮的合取 7 | T2-2 槽对上界松 | T2-3 上限够不到的 defer | T2-4 开头走空 | 标 |
|---|---|---|---|---|---|
| 甲：先算在被拒是空间不够时另判一次 `foreground_writes_are_below_the_empty_cluster_segments_floor`，这次发布归前台写入类、带普通分配而合取 7 这一刻已不成立，就不推、原样报合取 7 | 修被遮那一形；推串自己推下去的那一形不修 | 不修 | 不修 | 不修 | 推的 |
| 乙：槽对那一判的上界按「回收出来成几对」算：分配器按段增量维护 defer 里成对的数据单元与段外单槽（回落那一形）的计数 | 不修 | 修（只剩隔离那一项的松） | 不修 | 不修 | 推的 |
| 丙：defer 一项按释放代不高于这一刻上限算（按释放代的 defer 计数加一次上限读数） | 不修 | 收窄 | 修 | 不修 | 推的 |
| 丁：D16 已定项 10 射程与第 645 行文档补「这次调用开头没走空发成任何一步」 | — | — | — | 修（只改文字） | 推的 |
| 戊：射程写明这一判的上界与它许的白推（槽对那一判「每个放得开的槽补一对」、式子那一判「全部 defer」），不改代码 | — | 条款与代码对齐，不改行为 | 同左 | — | 推的 |

## 要主 agent 跑的命令

1. 探针整份复跑（`tools/rerun.sh` 经包装起；包装只多一步：先删掉运行目录，服务撞内存上限重排同一条命令时照样跑得通）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-opus/probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model
```

   - 在哪起：仓根（命令里的路径都是绝对路径）；运行目录 `probe-run` 在这一腿认领的草稿目录下，`.deliverables` 写成交接。在新 shell 里起。第二轮那一条没带范围声明、照样交成了；范围声明闸要它时照 `.claude/hooks/run-scope-guard.sh` 文件头补。
   - 不经包装的等价写法（运行目录要不在或是空的）：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-opus/probe-run`。
   - 跑完看：`TASK_SERVER_RESULT` 那一行 `exit=0` 算跑完、退非 0 的件数为 0。`task-server/task log <号> --all` 里依次有 `PATCH_THE_COPY …`、`name=build done …`、`name=exit_codes never=0 always=0 probe=0`、三份的 `ESTIMATE`、`WALL_SECONDS` 与 `PROBE_RESULT`、可能有一行 `name=misrefusal_base_zero …`（自证二的底是 0：全量的误拒读数按「装置走不到」读）、`name=self_checks …`、`name=rerun_done …`。
   - 退 1 而日志尾巴是 `cargo build` 的报错：这一腿没编过探针，编不过是它的错，交承接的腿照 `$WORK/out/build.log` 修。退 1 而写「自证一不合」「自证二没红」「全量自洽不合」：探针判不出它要判的，全量的数作废。`probe=3`（或自证那两趟是 3）是估时超过 40 分钟，`.out` 里有 `ESTIMATE_EXCEEDS_FORTY_MINUTES` 与先跑那一批的计数，交承接的腿缩配置数。
   - 内存与时间（推的，没量）：编译要编验证包的库，占大头；第二轮那一份探针全量 `WALL_SECONDS 267.5`、两条自证 33.5 与 68.7 秒（量的，第二轮产物），峰值约 7 GiB（正文第五节）。这一份每格同样两臂，O 关掉先算会多推，出路份额从每点 10 次变成至多 4 类 × 5 次、外加推环预演（拷分配器、在只进内存的盘上推至多一环），估挂钟是第二轮的 2 到 3 倍（全量 10–15 分钟），峰值 7–10 GiB。
   - 承接：新派同立场的腿，派发提示写「承接：research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-output.md」与任务号，它读 `task-server/task log <号> --all` 与 `probe-run/out/` 下三份 `.out`，按 T2-1、T2-2、T2-3、T2-4 写下的翻面条件改判定，再把 T3-1、T3-3、T3-4 三处换成新探针的读数（`lost_first_followed`、`lost_first_remount_published`、`stuck_remount_published`、`stuck_push_ring_admitted`、`COUNT outlet … holding-after=…`、`oracle_published_after_a_series`、`misrefused`）。

2. 引文核对（文本工具，交回前这一腿自己跑过，末行贴在报告末尾）：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-output.md`。

## 交用户的条目（第三轮之后不再开轮，这一轮新冒出来的都标零轮）

| 条目 | 是什么 | 标 |
|---|---|---|
| T2-1 被遮的合取 7 | 落得下那一判拒、合取 7 这一刻已不成立时，先算照推一串，推完拒成 `Publish`（合取 7）；改法甲 | 零轮；形是读码推的，第二轮产物里撤回之后那一臂 `Publish+pushed` 1624 格是线索，成员待探针 |
| T2-2 槽对上界每槽补一对 | 单列一行欠账：头一次就白推一串、F 抬到上限，被宽护卡住时还列一条重挂之后放不下的出路；改法乙或戊 | 现象量的（第二轮产物），归因待探针；改法零轮 |
| T2-3 式子上界含够不到的 defer | 射程同一行里「放得出的全放出来」与「上界是可用加 defer 加隔离」在这一形上互相顶；要么条款认这一截白推（戊），要么加计数与上限读数（丙） | 读码；零轮 |
| T2-4 开头走空 | 射程与第 645 行文档补一句（丁） | 读码；可达性要一次瞬时读写错，没造 |
| 出路那一前提没被量过 | 第一轮采纳那一截、第二轮撤回它，都以「被宽护卡住那条出路」为准；两份探针都没造读故障的宽护，攻方那一份走过的 700 次卸载重挂没有一次让写写进，`outlet_lost_*` 是没试（T3-1、T3-3） | 量的（第二轮产物）；要造宽护得另立一题 |

## 没打中的形状

读码试过、判不出毛病的（都是推的）：
- T2-5 那几样误拒的机制。
- 走空里的删除类发布：没有数据单元，落得下那一判的数据单元那一样永远够；式子那一判不判；提交内生块那一判先算交 `None`、照推——条款许的，不是缺口。
- 取不到落点（`PlacementRefused`）：先算不判、照推，与条款一致（第 6010 行文档）。
- 上限卡在读坏或读回全零的槽上：推一次不抬 F 的空发布，是条款定的推法，先算放不放过与它无关。
- `accept` 那一路：与另两处共用同一个函数，没有只在一处出现的形。
- 上限不高于 F 时先算放过：推的那一步一个字节都不写，只换了报的成员（T2-3 末句）。

探针的取样范围（等主 agent 跑）：372 个配置、种子 1–372，前缀与第二轮同一套抽法；探测点前 0 / 4 / 8 次小写；g 至多 160 个；两条自证各 62 个配置、种子 900001 起。这一份取样只算一次观测。

## 这条腿自己的限度

- 一条项目代码都没跑：T2 的形都是读码推的；「量的」只有文本比对与第二轮主 agent 复跑留下的产物。探针连编都没编过（子 agent 不起 cargo），只核过语法；被引的 API 照第二轮攻方那一份编过、跑过的探针的用法写，新用到的几样（`ways_out_of_the_wide_protection`、`RootRingPushRehearsal`、`FloorRaiseStop`、`empty_segments`、合取 7 的 `N₀`、`queue_of_committed_intents_is_empty`、`Option::is_none_or`）在冻结副本里逐个查过签名与可见性，编不过的可能仍不小。
- 没造读故障的宽护（宽护本身），T3-3 与「出路被吃掉」那一问这一腿也答不了；探针的出路四类都落在崩在第一条记录之后的窄的那一圈上。
- 白推的「缘由」是按推完之后那一刻的分配器给的粗判：式子那一判把推串自己释放进 defer 的旧元数据也算成「够不到的 defer」，会多给；槽对那一判只看推完之后段外槽对够不够。只作分份量用，不作判臂的判据。
- 扫边界的格数随扫法变（T3-5）；独立单位是探测点。
- `probe/src/main.rs` 一次写进 162 行，超过「每次写进不超过 150 行」那一条；内容没受影响，照实记。

## 草稿与清理

- 认领目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-opus/`：这一腿没建仓副本、没建编译目录。`scratch/`（48K，`du -sh` 量的：`rustfmt --check` 的输出与生成 `rerun.sh` 用的那一段文字）交回前删掉。`.deliverables` 一行：`probe-run` 交接，给主 agent 的复跑用（目录还没建）。
- 模型目录在仓里 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/`，`out/` 是空的。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」）。
- 副本上的数：这一腿一个都没有。改法表里每一格都是推的，没在副本上实现。
- 没跑门禁、没跑 `--list-items`、没跑 `--selftest`。
- 第一轮攻方探针（任务 11937 那一份）的「误拒」底有多大没去数，T3-4 只说了第二轮那一份。
- 引文核对的输出一度写进了会话临时目录根下的一个文件（不在写范围里），核完当场删了，照实记。

引文核对：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-output.md` 退 0，末行「✓ 核了 33 处引文、0 处裸行号，对上 33 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）」。
