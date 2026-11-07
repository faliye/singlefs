# 逐写独立判第二轮攻方腿（opus，2026-11-06）

轮名 per-write-judging-r2；批次 records/2026-11-05-删轻型并入重型.md#并包。攻击面（第二轮分工「第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方」）：① 第一轮原型「写数超过 19 就整段跳过」这一方法缺陷本身，改成按子集取样真判；② 在反例 A、B 那几条臂上量逐写判与全组合判的差；③ 两腿都没看过的地方（正推腿 8 类敏感取样点里挑 3 类以上）。方案按 `crates/` 今天的实现谈。

**本报告里没有一个数是量的。** 原型写完了、`cargo fmt` 过了，没编、没跑（子 agent 不交任务）；下面的「预测」全写在跑之前，跑出来之后不改、只往下另记。命令在「要主 agent 跑的命令」一节。引的第一轮读数（任务 8321、8686）是主 agent 跑的，标「量的（主 agent）」。

## 各格判定一览

| 格 | 判定 | 依据（细节在各节） |
|---|---|---|
| ① 第一轮原型的方法缺陷 | 改了（待跑） | 每段不再按写数跳过：写数多的段按八族取样（空集、单写、成对、只差一个、组格、单盘格、随机、目标态）逐态真判；另把单元段按副本组切成 ≤12 个写的小段逐段全量判。改法与它对派发定义第 3b 步的出入见第一节 |
| ①′ 产生结论的方法：「代表法」没定合成规则 | 打中（推的，演绎；`late_unit` 臂待跑核） | 正文只说「判 S=∅ 与每个单写」「组内全组合、组间各判一次」，没说怎么由这几态合成 S 的判定。正推腿用的是并（red(∅) ∪ …）。前缀本身红、S 把它补绿时，任何「red(∅) ∪ …」形式的代表都红，与全组合判相反；只有「增减」合成能变绿。候选乙要成立，合成规则必须写成增减，不能写成并（第二节） |
| ② 反例 A 一对（`late_unit`）| 预测：甲两种合成、乙按并都打中，乙按增减不中 | 第二节预测 R3；第一轮 8686 一态没判，这一轮的取样族里组格全取（13 组），含 {x0,x1} 的态 4095 个 |
| ② 反例 A 两对（`late_two_units`）| 预测：四种合成都打中 | 预测 R4：乙按增减在「两对四份都落」的态上合成出 R_X ∩ R_Y ≠ ∅（推的） |
| ② 反例 B（`record_before_units`）| 预测（把握低）：覆盖写那一种 C2 上构造不出；顺序写两个数据单元那一种上有 | 预测 R6、R7：覆盖写的记录可能点名 C2 全部单元（N = W2）；顺序写每条记录只点名自己的数据单元，恢复按记录逐条施加（D23 已定项那一句，第二节） |
| 用户动作没放开扫（第一轮限度）| 放开了一步 | C2 取覆盖写、顺序写两个数据单元、建一个 inode 三种，前缀与故障固定（段落 `user_actions`） |
| 漏掉的 5 类破坏条件与判据循环 | 逐条给了臂，4 类写成命令 | L1 `late_unit`、L2 `record_before_units`、L3 `relocation_copies` 与 `relocation_copies_over_a_missing_source`、L4 `rotation_with_units` 写成命令；L5、L6、L7 只给规格（第三节）。transaction.rs:10564、walk.rs:17931、17966 三处今天的原文已核（第三节） |
| 正推腿 8 类敏感取样点 | 挑 4 类写成命令 | 混合种类段（`rotation_with_units`、`units_records_root`）、多组同时被盖且有引用关系（`planted_two_groups`）、小段全量对照（段落 `small_segments_exhaustive`）、整理/搬迁那一类单元段的一半（`relocation_copies*`，只有搬迁副本，没有复用槽）（第四节） |

## 复跑命令与模型文件

模型目录 `research/prompts/per-write-judging-r2-opus-model/`（原型没编过、没跑过；`out/` 还没有）：

| 文件 | sha256 |
|---|---|
| `proto/Cargo.toml.in` | `66bb2fe3b614b719a99e672fdf9da4df870045b340f438de3df7c843520793c2` |
| `proto/src/main.rs` | `53d83e0432d83ce64be1ba69f208e1a34b10b612aeb484d44d85cb2c7815c257` |
| `tools/rerun.sh` | `ca26e2944d01589b5d5576abb94e83bec6bd704baf8693fd4061b8e15296798c` |

整份复跑（交主 agent，见「要主 agent 跑的命令」第 1 条）：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/copy --work /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/rerun-work`。

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`。服务撞内存上限重排时整条重起，第一轮的版本因为 `--work` 非空被模板挡下、两次都没跑成；这一版不碰模板的参数检查，把要跑的东西全放在 `--work` 旁边的 `<work>.partial` 里：每个段落的输出先写 `.tmp`、跑完才改名，重起时末行是 `PROTO_FINISHED` 的段落跳过；四个段落都完了才把 `out/` 挪进 `--work`、删掉旁边那一份。模板自证里那一格「`name=rerun_roots` 行」对这一版不成立（`rerun_steps` 换了），模板的 `--selftest` 不适用于这份拷贝。

原型做什么：两块内存稀疏盘（`SparseBlockDevice`），每种 C2 的流 mkfs 只跑一次、写全录进一条流（取号 → 暖机 → 新池新建文件 A → 覆盖写 C1 → C2）；流是原型自己调 core 的公开入口造的，没从崩溃枚举用例拷。臂改录下来的写表（等于一份把某处写早、写晚、少一道屏障或多写几份的实现）。每个被判的状态调 `judge_crash_image`（两遍恢复、两份 oracle、池级 checker、记录核对器），与四种代表法逐态比：

| 代表法 | 是哪一个候选 | 合成 |
|---|---|---|
| `per_write_union` | 甲 | red(∅) ∪ ⋃_{w∈S} red({w}) |
| `per_write_delta` | 甲 | red(∅) 减去各单写比空集少掉的项、加上各单写比空集多出的项 |
| `copy_group_union` | 乙 | red(∅) ∪ ⋃_{G∩S≠∅} red(S∩G)（同偏移的几份算一组，原地覆写取三态） |
| `copy_group_delta` | 乙 | 同上的增减合成 |

每段每族每种代表法打 `PROTO_COMPARE … compared= red_or_not_differs= red_items_differ= unrepresentable=`，另打 `PROTO_FAMILY`（红的态数、整份与空集不同的态数）、`PROTO_SINGLE_DIFFERS`（单写与空集整份不同的那几个写）、`PROTO_EXAMPLE`（每种代表法前 3 个红项对不上的态）、`PROTO_TARGET`（臂点名的具体状态的真判与两种并合成）。退出码：0 跑完；2 用法错或段形状与预想不同；3 状态数超 10⁶ 或估时超 40 分钟，没跑。

四个段落（一个段落一次跑，各自估时、各自过 10⁶ 上限）：

| 段落 | C2 | 臂 | 判法 |
|---|---|---|---|
| `counterexamples` | 覆盖写 | `unmodified`、`late_unit`、`late_two_units`、`record_before_units` | 取样 |
| `user_actions` | 顺序写两个数据单元、建一个 inode | `unmodified`、`late_unit`、`record_before_units` | 取样 |
| `unseen_shapes` | 覆盖写 | `rotation_with_units`、`units_records_root`、`planted_two_groups`、`relocation_copies`、`relocation_copies_over_a_missing_source` | 取样 |
| `small_segments_exhaustive` | 覆盖写 | 上两行里只有单元写的七条臂，被判段切成 ≤12 写的小段 | 全量 |

## 一、攻击面 ①：第一轮原型的方法缺陷与改法

### 缺陷是什么（量的，主 agent）

第一轮原型只展开写数 ≤19 的段，而这条流的单元段写数是 20–28：复跑任务 8686 的产物（`/home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/rerun-work-3/out.txt`）每条臂两段都打 `PROTO_SEGMENT_SKIPPED … reason=写数超过 19`，末尾 `PROTO_PLANNED_STATES total=0 ceiling=1000000`。它同一份产物里量到了的只有段形状：

- `unmodified` 的 C1、C2 单元段（段 12、16）都是 24 个 `UnitWrite`、`over_an_earlier_write=0 over_nonzero_base=0`；
- `late_unit` 的 C2 单元段 26 写、`late_two_units` 28 写，`over_*` 仍是 0——第一轮预测 Q2「两个反例 A 落在判据可读形式收下的段上」**已经被这一份产物坐实**（量的，主 agent 任务 8686），状态层面一个都没判；
- `record_before_units` 的 C2 单元段挪到段 17，24 写，`over_*` 是 0。

缺陷的根子不是阈值取 19，是「一段要么全量、要么不判」：两个反例要的态（{x0,x1}、「只缺一整组」）在 2^24 里占比极小，按全量就超 10⁶，按跳过就一个都不判。

### 改法

1. **写数多的段按族取样真判**（`sample_of_segment`）。每个状态都真跑 `judge_crash_image`，不估、不合成。族（一个状态可同属几族）：
   - `support`：空集、每个写的每一态（单写，原地覆写另取撕裂单写）、每个副本组的组内全部子态——两个候选合成要用的态全在这一族里，所以四种代表法在每个被比的态上都凑得出（`unrepresentable` 应恒为 0，不为 0 说明取样写错）；
   - `pair`：单元写两两成对（24 写 276 个）；`all_but_one`：单元写只差一个（24 个）；
   - `group_lattice`：若干整组两份全落、其余组一份不落。组数 ≤13 全取（2^13 = 8192），超过随机取 8192 个。这一族专门对着「跨组合取」与「点名单元两份都落才施加」：反例 A 的 {x0,x1}∪任意组、反例 B 的「只缺一整组」都在里面；
   - `device_lattice`：只在一块盘的单元写里取任意子集（每组至多一份）。写数 ≤12 全取，超过随机取 4096 个。对着「每组一份就够」那一类合取；
   - `random`：整个取值域均匀随机 1000 个；`target`：臂点名的具体状态。
   - 段里有原地覆写（三态）时，单元写那一部分的空集、单写、成对、只差一个、组格各与原地覆写的全部 3^m 种组合交叉，单盘格只配「原地覆写全不落」。
2. **小段全量对照**（段落 `small_segments_exhaustive`，`chunked_arm`）：被判的单元段按副本组切成 ≤12 写的小段（组不拆、次序不变），每个小段 2^n − 1 个态全判。等于一份在单元写之间多发几道屏障的实现：小段的前缀是更早的小段全落。这是这一轮里唯一「这段历史的崩溃状态一个不落」的判法。
3. **跑前**：每段打 `full_domain_states`（`crash_state_count_with_torn_in_place_overwrites` 对这一段的闭式减 1）、`judged_states`、`judged_is_full_domain`；整个段落打 `PROTO_PLANNED_STATES judged= full_domain_of_the_judged_segments= ceiling=1000000`，`judged` 超 10⁶ 退 3；再在第一段上单线程判空集与各单写、按每态耗时外推，超 40 分钟退 3（`PROTO_ESTIMATE`）。

### 与派发定义第 3b 步的出入（交主 agent 定）

- **崩溃状态取样了**。第 3b 步写「崩溃状态不在缩的范围里」「一段历史自己就超过约 10⁶ 的，那段不跑」；这条流每个单元段的全量都是 2^24 − 1 ≈ 1.68×10⁷ 以上，照第 3b 步一段都不能跑——那正是第一轮的 `total=0`。派发提示的第 1 条要「在写数多的段上按子集取样判」，我照派发提示做；全量那一半由小段全量段落补（每个小段全量，但小段是改过屏障的历史，不是原流）。10⁶ 的上限按「这一次真判的态数」算，不按「被判段的全量」算；两个数每段都打出来。
- **没经 `enumerate_crash_states` / `enumerate_crash_states_versions`**。它们走判定库（「录入段把流（连同每段的展开）写进库、登清单；判段读回、按块判、存结论、重建计数」，原文在 `crates/singlefs-verification/src/crash_state_enumeration.rs:384` 的文档整行：「/// `observers` 逐状态看（没有就空）。录入段把流（连同每段的展开）写进库、登清单；判段读回、按块判、存结论、重建计数。」（`crates/singlefs-verification/src/crash_state_enumeration.rs:384`）），交回的是计数，比不了逐态红项；原型的模型又不许写真服务。原型直接对每个状态调 `judge_crash_image`，状态集合按「更早的段全落、本段按状态、之后一个不落」取，撕裂镜像照引擎的 `torn_image_of_in_place_overwrite` 按「这次写之前的写全落了」时的旧字节算（「pub fn torn_image_of_in_place_overwrite(old: &[u8], new: &WrittenContents) -> Vec<u8> {」（`crates/singlefs-verification/src/crash.rs:2221`））；原地覆写之后同段若有与它重叠的写，原型不接重放、直接 panic（这条流里没有）。
- 只判点名的那几段（C1、C2 的单元段或并出来的段），不判记录、根、系统配置段，不是「每段都展开」。
- `rerun.sh` 由模板经 python 换掉 `rerun_steps` 后一次写进模型目录（196 行），`main.rs` 在草稿副本里 `cargo fmt` 后整份拷回（1672 行）：这两次写入的生成内容不超过 150 行，但落盘一次超过了「每次写不超过 150 行」的字面。

### 状态数与估时（推的，原型跑前自己打 `PROTO_PLANNED_STATES` 与 `PROTO_ESTIMATE`）

按族的大小算：12 组的段约 1.3 万态，13–14 组约 1.8–2.6 万态，并进系统配置槽的那一段约 4.8 万态。四个段落各约 10 万、10–16 万、14 万、9.5 万，合计约 50 万，单个段落都在 10⁶ 以内，四个合起来远在 10⁷ 以内（本腿与接手它的腿合计的上限）。挂钟：任务 8321 在 31 线程上判约 1 万态、连编译 122.8 秒（量的，主 agent，`MULTI_HOST_RUN_STEP step=peer-command event=end … seconds=122.8`），按此一个段落约 5–20 分钟（推的）。批：一个段落一批，批里的段数由段落定，每批开跑前单线程估时、超 40 分钟退 3（这一批不跑，缩取样再交）；没设限时。

## 二、攻击面 ②：反例 A、B 那几条臂上逐写判与全组合判的差

### ②之一：产生结论的方法——两个候选没有定合成规则（推的，演绎；`late_unit` 跑出来核）

第一轮正文的两个候选只写了判哪几态，没写怎么由这几态还原 S 的判定：

- 「- 候选甲（逐写判新旧）：每个写各判一次…只这一个写落了…，再判 S=∅（一个写都没落）。」（`research/prompts/_per-write-judging-r1-body.md:11`；原行里「只这一个写落了」带一对直角引号，引文里用省略号隔开，免得引号嵌套）
- 「- 候选乙（按单元副本组判）：同一个单元的几份副本算一组，组内取全组合，组间各判一次。」（`research/prompts/_per-write-judging-r1-body.md:12`）

正推腿补的合成是并：「- 甲：判 S=∅ 与每个单写，S 的结论由 red(S) = red(∅) ∪ 并_{w∈S} red({w}) 还原（红不红就是这个集合是否非空）。态数 n+1。」（`research/prompts/per-write-judging-r1-sonnet-output.md:99`），乙同形（并_g red(S∩G)）。第一轮我自己的原型另有一种「增减」合成。两种合成在下面这一格上结论相反：

- 前缀 P（本段 S=∅）本身红，而 S 落下之后全组合判绿（「补回」方向，第一轮漏项 L1）。并合成的代表含 red(∅)，非空，**恒红**——不管候选甲、乙，也不管判了多少态。只有增减合成能把 red(∅) 里的项减掉。
- 这一格在今天对的实现上不出现（D16（发布语义） 已定项 7 的持久顺序成立时前缀只指向已落的单元，P 绿）；它出现在崩溃点重放要抓的那类坏实现上（单元写晚了）。整条用例红不红不变（S=∅ 那一态本身就红），变的是逐态的红绿与红项——入库的判定块与 GPU、CPU 对拍比的就是逐态的这一串。

四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：分不分辨臂——分辨合成规则（并的两个候选一起中，增减时甲中、乙在一对副本上不中），所以判甲乙之前先要把合成规则定死；被判的系统看不看得到——看得到，合成只用已判的几态；满足的分句——正文两个候选的定义句都没有合成分句，打中的是「候选的定义缺一个分句」，不是判据的哪一条；改法——把合成写成增减。增减在两对副本上照样中（预测 R4），所以增减修的是一对那一格，修不了跨组合取那一格。

### 臂与反例

前缀固定为 mkfs → 取号 → 暖机 → A → C1（覆盖写 3000 字节）；C2 是用户动作（三种）；故障是写表次序改一处。

- `late_unit`（反例 A 一对；漏项 L1）：C1 单元段里一对 32K 副本 x0、x1（`copy_pairs_in` 数据单元排前、同长按偏移，第一轮产物里是 `UnitWrite@dev0+822181888x32768`、`UnitWrite@dev1+822181888x32768`）挪到 C2 单元段开头。目标态 `moved_copies_only` = {x0,x1}。
- `late_two_units`（反例 A 两对）：挪两对（第一轮产物里另一对是 `+823361536x32768`）。目标态 = 四份。
- `record_before_units`（反例 B；漏项 L2）：C2 的记录段与单元段对调。目标态 `every_group_but_<偏移>`：C2 单元段里只缺那一整组。施加的门是 D23（journal 的角色与格式） 那一句：「施加前逐项验证点名单元（D16（发布语义）已定项 7，E77（发布的持久顺序）证明承重）——**一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加**，有一份读不出或对不上，这条记录与它之后的都不施加。」（`.claude/kb/decisions/23-journal的角色与格式.md:364`）。按这一句，施加是逐条记录的：顺序写两个数据单元那一种 C2 有两条以上记录、前几条各只点名自己的数据单元，「记录 1 点名的那一组两份都落、别的组缺」时记录 1 施加、记录 2 不施加，这是反例 B 在今天的核心代码里构造得出的一条路（推的，`user_actions` 段落答）。覆盖写那一种只有一条记录（第一轮产物 `segment=13 writes=2 kinds=JournalRecord`，两盘各一份），点名是不是 W2 全部，原型的 `PROTO_FAMILY … family=group_lattice red=` 直接答。
- `unmodified`：对照。

### 跑前写死的预测（写在跑之前，跑出来之后不改，只往下另记）

「态数 N(·)」都指这一族在这一段里实际判了的态里、含括号里那几份的个数；`PROTO_COMPARE` 每族一行，原型打 `compared` 与两种差，我只预测差。

| 号 | 预测 | 推翻它的观测 | 推翻之后怎么改 |
|---|---|---|---|
| R1 | 三种 C2 的 `unmodified` 每段：`PROTO_FAMILY … family=all red=0`、四种代表法 `red_or_not_differs=0 red_items_differ=0 unrepresentable=0`、`PROTO_SINGLES … differing_from_empty=0` | 任一不为 0 | 原型或取样写错（探针 8321 在删除流上量过同形），先查原型，其余预测作废 |
| R2 | `late_unit`、`late_two_units` 的 C2 单元段 `readable_criterion_accepts=true` | false | 与 8686 的段形状对不上，原型重排写错 |
| R3 | `overwrite/late_unit` 的 C2 单元段：`PROTO_BASELINE` 红项非空；`PROTO_TARGET … target=moved_copies_only actual={}`（绿）；`per_write_union`、`per_write_delta`、`copy_group_union` 三种在 `group_lattice` 族 `red_or_not_differs=4095`、`pair` 族 =1、`all_but_one` 族 =24、`device_lattice` 族 =0；`copy_group_delta` 各族 `red_or_not_differs=0 red_items_differ=0` | 目标态红；或三种并/甲的差为 0；或 `copy_group_delta` 不为 0 | 目标态红：反例 A 的「补回到绿」不成立，撤回 ②之一 的实例（演绎那半照留）；`copy_group_delta` 不为 0：C2 的在飞组与 X 组之间另有相互作用，候选乙在一对副本上也不够 |
| R4 | `overwrite/late_two_units` 的 C2 单元段：目标态（四份）绿；`copy_group_delta` 在 `group_lattice` 族的 `red_or_not_differs` 等于这一族里含两对四份的态数（约这一族态数的四分之一，>1000），`all_but_one` 族 =24，`pair`、`device_lattice` 族 =0 | `copy_group_delta` 各族都是 0 | 撤回「候选乙两对打中」（第一轮那一格） |
| R5 | 三条 late 臂（含 `user_actions` 的两条）的红项里一次都不出现 `record_checker:claimed_state_missing_unit` | 出现 | 记录核对器按根槽写归发布不是我读的那样，反例 A 的第二条依据改 |
| R6（把握低） | `overwrite/record_before_units` 的 C2 单元段：`PROTO_BASELINE` 红项为空；`family=all red=0`，四种代表法差都是 0（覆盖写的那条记录点名 C2 全部单元，反例 B 构造不出） | `red>0` | 覆盖写上反例 B 也构造得出：看红项是不是 `record_checker:claimed_state_missing_unit`、甲是不是漏红（`per_write_*` 的 `red_or_not_differs` 等于 `red`） |
| R7（把握低） | `sequential_write_two_data_units/record_before_units` 的 C2 单元段：`red>0`，且 `per_write_union`、`per_write_delta` 的 `red_or_not_differs>0`（甲漏红或错红）；至少一族 `copy_group_delta` 的 `red_or_not_differs>0` | `red=0`，或四种都不差 | 顺序写也凑不出反例 B（记录施加不是逐条的，或点名项不是我读的那样）；反例 B 降为「只在某种点名形状上」，写进没打中的形状 |
| R8 | `new_inode/record_before_units`：与 R6 同 | 同 R6 | 同 R6 |
| R9 | `user_actions` 两种 C2 的 `late_unit`：与 R3 同形（数随 C2 的组数变，`copy_group_delta` 仍是 0、三种并/甲仍不为 0） | 任一种 C2 上三种并/甲的差为 0 | 反例 A 的形状依赖 C2 做什么，「同一段历史上中」那一句要按 C2 分开写 |
| R10 | `overwrite/relocation_copies` 的 C2 单元段：`readable_criterion_accepts=true`；`family=all red=0`；四种代表法差都是 0 | `red>0` | 在飞的搬迁副本单写就让判定变红（扫描方向收了它，又有别的判法不放过），候选甲按单写照样合成得出（或），但「在飞单元不进判定」整句作废 |
| R11 | `overwrite/relocation_copies_over_a_missing_source` 的 C2 单元段：`PROTO_BASELINE` 红项含 `pool_checker:I-2.1`；两个搬迁副本各有一行 `PROTO_SINGLE_DIFFERS`，红项比空集多 `pool_checker:I-1.13`；四种代表法 `red_or_not_differs=0`，`per_write_union` 的 `red_items_differ=0` | 没有 `PROTO_SINGLE_DIFFERS`；或 `per_write_union` 红项有差 | 前者：I-1.13 在这一形上报不适用或两支都放过，撤回第一轮 L3 里「单写就翻红」那半句；后者：搬迁副本之间有合取，L3 的「对多写是或」撤回 |

## 三、第一轮漏掉的破坏条件与判据循环：逐条能量的臂

### 今天的原文（现查）

- 搬迁不盖戳：「/// 原样字节写到目的地（头不盖戳、birth 不变）、映射条目换到目的地、源落点写释放代进 defer、目的地记一条分配代 = 诞生代的已分配记录」（`crates/singlefs-core/src/transaction.rs:10564`）。
- I-1.13 子集规则：「if walked_placements.is_subset(rebuilt_placements) {」（`crates/singlefs-checker/src/walk.rs:17931`），它的注释原文起头一行：「// 遍历方向的落点都是扫描重建认领的这一版的副本，扫描重建多出来的也是这一版逐字节相同的副本：搬迁把单元原样写到目的落点、」（`crates/singlefs-checker/src/walk.rs:17927`）。
- 子集不成立时的另一支是相等比较：「if readable_rebuilt != readable_walked {」（`crates/singlefs-checker/src/walk.rs:17966`）。
- 扫描方向的准入（在飞单元靠诞生代号挡掉的那一句）：「return birth_txg <= self.mount_root_checkpoint_txg;」（`crates/singlefs-checker/src/walk.rs:11256`）；I-7.8 那一路「if birth_txg <= newest_published_txg」（`crates/singlefs-checker/src/walk.rs:5577`）。
- I-1.8 成组门：「if members.len() < 2 {」（`crates/singlefs-checker/src/walk.rs:11472`）。
- I-1.13 报不适用的门是一张七项的表：「pub const INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED: [(&str, &str); 7] = [」（`crates/singlefs-checker/src/walk.rs:18033`），I-2.1 不在表里（表前的注释写明只收两个必红世界里都不红的几条，I-2.1 在两个世界里都红），所以前缀缺一份副本时 I-1.13 照判（推的，R11 核）。

三处行号与第一轮引的相同，原文没变。第一轮 ② 之二的推理（搬迁副本诞生代号不被挡，等价靠子集规则与 I-1.8 同组归并）照原文仍成立（推的）；量它的臂是 R10、R11。

### 逐条能量的臂

| 漏项 | 打破什么 | 能量的臂 | 写成命令没有 | 量什么、哪一行答 |
|---|---|---|---|---|
| L1 前缀引用本段要写的槽（「补回」） | c1；并合成恒红（第二节 ②之一） | `late_unit`、`late_two_units`（覆盖写），`late_unit`（顺序写、建 inode） | 是（段落 `counterexamples`、`user_actions`、`small_segments_exhaustive`） | R2–R4、R9；小段全量 X2、X3 |
| L2 记录先于点名单元（施加是逐条记录的合取门） | c1（R(P) 含记录点名项），可读形式查不出 | `record_before_units`（三种 C2） | 是 | R6–R8；小段全量 X4 |
| L3 诞生代号不挡的在飞单元（搬迁副本） | c2 的前提句「在飞单元不进判定」 | `relocation_copies`、`relocation_copies_over_a_missing_source` | 是（段落 `unseen_shapes`、`small_segments_exhaustive`） | R10、R11。**限度**：臂在字节层面造搬迁批的单元段（原样拷贝到空槽、前缀里映射与分配记录仍指源），没走 `publish_relocation_batch`：真搬迁批同段里还有重写的记账树、分配记录树、中央映射树节点，臂里没有；走真入口要先签发整理意图、开会话（仓里的整理前缀装置 1681 行），这一轮没写 |
| L4 系统配置轮换与单元写同段 | 种类检查会挡（D1）；局部组合表里没有这一类 | `rotation_with_units` | 是（段落 `unseen_shapes`） | R12 |
| L5 I-1.8 判过次数对两份副本是合取（在飞副本诞生代号被盖错时） | E3；判据 D4 靠「实现写对头」才成立 | 规格：C2 单元段的每个在飞单元头里诞生代号改成 C1 的 txg、重算头与载荷校验和（要按单元格式重编，`singlefs-format` 的编码入口），两份都落才凑成一组 | 否（推的，待写） | `PROTO_FAMILY … whole_differs_from_empty` 在 `pair` 族里只有同组两份那几态不为 0、单写为 0 |
| L6 读持久集合的观察者（计数） | E3 计数 | 规格：在每个取样态上另算删除停在中间那一类谓词（单元写至少落一个、记录一份没落、根没落），与「S=∅ 与单写」两类代表按类加权的计数比 | 否 | 一族一行「谓词真的态数」对「按类加权」 |
| L7 真跑实现的可写挂载续做观察者 | c5 没证；随恢复报告变 | 规格：在 R7 的红态与 S=∅ 上挂 `observe_the_mount_continuation`（`crates/singlefs-verification/tests/common_crash_point_observers/mod.rs` 里的公开函数），比续做步数与挂完的盘上的 checker | 否 | 两态续做结果不同即 L2 牵动 L7 |

### 判据循环（第一轮 ②）能量到的那一半

- ②之一「可读形式靠持久顺序成立」：8686 已量到两个 late 臂的 C2 单元段 `over_*=0`（量的，主 agent）；这一轮的 R2 再打 `readable_criterion_accepts=true`，R3、R4 给出这一段上 c1 不成立的具体态与逐写判判错的态数。量到 R3 成立，循环就从「推的」变成「在判据收下的段上有判错的态」。
- ②之二「在飞单元靠诞生代号挡在外面」对搬迁副本不成立：R10、R11。R11 成立时说明在飞单元确实进了判定（单写改了红项），只是改法是或、单写合成得出。

## 四、正推腿 8 类敏感取样点里挑的几类

正推腿第一轮报告 5.2 表的 8 类：σ 段（C561 复用链）、已准入的在飞头、混合种类段、多组同时被盖且有引用关系、写长与头声明不符、整理/快照流的单元段、小段全量对照、GPU 判器一侧。写成命令的：

| 类 | 臂 / 段落 | 它推翻什么（正推腿原话的意思） | 预测 |
|---|---|---|---|
| 混合种类段（b8 单元写与系统配置槽同段） | `rotation_with_units`：C1 的系统配置槽轮换段与 C2 单元段之间的屏障去掉、并成一段（2 个原地覆写取三态 × 24 个单元写） | D1；b8 的 3^m·(n+1) | R12 |
| 混合种类段（b6 根与单元同段） | `units_records_root`：C2 的单元、记录、根三段并成一段 | D1 | R13 |
| 多组同时被盖、且有引用关系 | `planted_two_groups`：C2 单元段末尾四次写盖住 C1 的数据单元一对与一个 16K 节点一对；节点挑「单独盖了就让文件读不出」的那个（在文件那条路上，是数据单元的祖先），挑法打 `PROTO_PLANT_CHOICE` | 乙「组间独立」 | R14 |
| 小段全量对照 | 段落 `small_segments_exhaustive`：七条只有单元写的臂，被判段切成 ≤12 写的小段逐段全量 | 第 1.4 节的合成式在小规模上的真检验 | X1–X6 |
| 整理/快照流的单元段（半类） | `relocation_copies*`：只有「搬迁副本落在空槽」那一半；「单元落在释放后复用的槽上、判据该判不收」那一半没造 | D2 的「不收」那一半没量 | R10、R11 |

没写的四类：σ 段（要 C561 那条记录核对器流的写表，自造要重做复用链）、已准入的在飞头（要续做之后挂完的盘，见 L7）、写长与头声明不符（要按头字节造单元，`slots_claimed_by_a_unit_header` 是私有函数）、GPU 一侧（要卡与对拍目标）。

| 号 | 预测 | 推翻它的观测 | 推翻之后怎么改 |
|---|---|---|---|
| R12（把握低） | `rotation_with_units` 并出来的段：`readable_criterion_accepts=false`、`tearable=2`、`full_domain_states=150994943`（9·2^24 − 1）；`PROTO_BASELINE` 红项为空；`family=all red=0`；四种代表法差都是 0（同一个实例里轮换不改实例代号，I-7.7 的合取在这条流上咬不着） | `red>0` 或任一代表法有差 | b8 在今天的流上是实的；看 `PROTO_EXAMPLE` 哪几态，判据的局部组合表要加这一类 |
| R13 | `units_records_root` 并出来的段：`readable_criterion_accepts=false`；`red>0`；四种代表法在 `all_but_one` 族都有 `red_or_not_differs>0`（「只差一份记录副本」那几态全组合判绿，而根单写红，并与增减都留着它） | 某种代表法各族都是 0 | 混合段上那种代表法碰巧够用；不影响结论（D1 本来就把这种段挡在逐写判外），只改 b6 的局部组合范围 |
| R14 | `planted_two_groups`：`readable_criterion_accepts=false`（`over_an_earlier_write=4`）；至少一个 `PROTO_PLANT_CHOICE … file_unreadable=true`；三个目标态都红；`copy_group_union` 在 `group_lattice` 族 `red_items_differ>0`、四种代表法 `red_or_not_differs=0` | `copy_group_union` 各族 `red_items_differ=0` | 祖先组两份都坏时遮住后代组的项，在「项名」这一层看不出（项名是不变量编号，不分哪个单元）；正推腿「乙组间被遮」那一句在红项层不成立，降为整份层 |
| X1 | 七条臂每个小段 `judged_is_full_domain=true`；`overwrite/unmodified/chunked` 每个小段四种代表法差都是 0 | 任一不成立 | 切段或全量写错，X2–X6 作废 |
| X2 | `overwrite/late_unit/chunked`：C2 单元段切出的第一个小段（X 一对 + 5 组，12 写）里，`per_write_union`、`per_write_delta`、`copy_group_union` 的 `every_proper_subset` 族 `red_or_not_differs=1023`，`copy_group_delta` 为 0；C2 的其余小段与 C1 的小段全是 0 | 三种并/甲不是 1023，或 `copy_group_delta` 不为 0 | 同 R3 |
| X3 | `overwrite/late_two_units/chunked`：C2 的第一个小段（两对 + 4 组）`copy_group_delta` 的 `red_or_not_differs=255` | 0 | 同 R4 |
| X4 | `overwrite/record_before_units/chunked`：与 R6 同（全 0） | 不为 0 | 同 R6 |
| X5 | `relocation_copies/chunked`、`relocation_copies_over_a_missing_source/chunked`：与 R10、R11 同 | 不同 | 同 R10、R11 |
| X6 | `planted_two_groups/chunked`：最后一个小段（两组盖写，4 写）15 态里 `copy_group_union` 的 `red_items_differ≥1` | 0 | 同 R14 |

小段全量的数是「多一道屏障的历史」上的数：每个小段的前缀是更早的小段全落，与原流的同一段不是同一组状态。原流上的数看取样段落。

## 五、改法各修哪一格（全是推的，只在我的推理上过过，没实现、没跑，被攻过零轮）

| 改法 | 反例 A 一对 | 反例 A 两对 | 反例 B（施加的那条记录点名一组） | 反例 B（点名多组） | 代价 / 违反什么 |
|---|---|---|---|---|---|
| 甲，并合成 | 中（推的，待主 agent 跑：R3、X2） | 中（推的，待主 agent 跑：R4、X3） | 中（推的，待主 agent 跑：R7） | 中（推的） | — |
| 甲，增减合成 | 中（推的，待主 agent 跑：R3、X2） | 中（推的） | 中（推的） | 中（推的） | — |
| 乙，并合成 | **中**（推的，待主 agent 跑：R3、X2；第一轮报「不中」，按并合成是中） | 中（推的） | 中或不中，视组内子态红不红（推的，待主 agent 跑：R7） | 中（推的） | 态数 1 + 3G |
| 乙，增减合成 | 不中（推的，待主 agent 跑：R3、X2） | 中（推的，待主 agent 跑：R4、X3） | 视组内子态（推的，待主 agent 跑：R7） | 中（推的） | 同上 |
| J(∅) 红就退回全组合 | 不中（推的） | 不中（推的） | 中（P 绿，推的） | 中（推的） | 读了判定结果，违反第一轮正文第四节第 3 条「读段表就能判」 |
| 引擎在选展开之前走一遍前缀镜像算 R(P)（根环里每条读得出的根走到的槽 ∪ 最新根之上已落的记录点名的槽），W ∩ R(P) ≠ ∅ 的段退回全组合 | 不中（推的） | 不中（推的） | 不中（推的） | 不中（推的） | 读镜像不读判定；R(P) 要与池级 checker 的读法同步、另立会红的检查；第一轮提的，被攻过零轮 |

这一轮新加的一条（被攻过零轮）：**候选的定义要写明合成规则**，并合成在「补回」那一格上恒错（第二节 ②之一），所以「乙比甲强」这句话只在增减合成下有内容。

## 没打中的形状

这一轮没有量到任何东西，下面全是读代码推的、构造不出或不打算拿来判甲乙的：

- **今天对的实现上**（`unmodified` 三种 C2、`relocation_copies`）：前缀绿、在飞写都落在新槽且诞生代号被挡或被子集规则放过时，四种代表法都等于 J(∅)（R1、R10）。第一轮探针 8321 在删除流上量过同形（量的，主 agent）。所以「判据在今天对的实现上会不会错」这一句这一轮仍没打穿，打的是「判据收下的段上，逐写判能不能抓住它该抓的坏实现」。
- **系统配置槽与单元同段**（R12）：同一个实例里轮换不改实例代号，I-7.7 那一格的合取在这条流上咬不着（推的）；要咬着得换实例（续做之后），没造。
- **建一个 inode 那一种 C2 上的反例 B**（R8）：一条记录，推的与覆盖写同。
- 「没打中」按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」只抽了这一次，不支撑「没有」。

## 这条腿自己的限度

- 原型没编、没跑。API 签名逐个对着冻结副本 grep 过（`publish_sequential_write`、`publish_new_inodes`、`TearableInPlaceOverwrites::contains`、`torn_image_of_in_place_overwrite`、`MemoryPool::apply_writes`、`DeviceOffsetInBytes(pub u64)`），借用与类型推断几处按经验改过（作用域线程用 `move` 闭包、`BTreeMap::contains_key` 解一层引用），编不编得过要主 agent 编了才知道；编不过请新派攻方腿承接修（派发提示带「承接：research/prompts/per-write-judging-r2-opus-output.md」与任务号）。
- 每个反例都是改写表造的坏实现，不是今天的实现走得出的历史。搬迁臂是字节层面的模型，没走 `publish_relocation_batch`。
- 用户动作只放开了 C2 这一步（三种），C1 固定为覆盖写；放开扫只放在两条反例臂与对照上，`unseen_shapes` 的五条臂只在覆盖写上跑。
- 取样不是全量：组格在 14 组的段上随机取 8192 个，单盘格在超过 12 写的盘上随机取 4096 个；只差两个以上的组合、跨组合取里「每组一份且至少一组两份」这一类只有随机 1000 个在碰。小段全量那一半是改了屏障的历史。
- 没经 `enumerate_crash_states`，没碰判定库；状态集合是我按 D13（验证路线） 已定项 4 的定义自己取的，撕裂镜像照引擎的函数算，但「原地覆写之后同段有重叠写」那种重放没接（遇到就 panic）。
- 别家实现没引。

## 要主 agent 跑的命令

冻结副本是我建的仓副本 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/copy`（`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs <它> --with-test-inputs` 建的，46.6 MiB，草稿目录的 `.deliverables` 里写成「交接」）。副本里另有原型包 `per-write-judging-r2-proto/`（自己一个 workspace，path 依赖指副本的 `crates/`，锁文件与 cargo 配置拷自副本；`src/main.rs` 与模型目录的逐字节相同，sha256 同上表）。副本里没有编译目录：编译由任务服务在它自己的目录里做。

1. 整份复跑（判决要引的数用这一条；编译、四个段落、存输出都在里面）
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/copy --work /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/rerun-work`
   - 在哪起：仓根（脚本只从参数取路径）；要在新 shell 里起。照 `.claude/rules/three-way-inference.md` 复跑时把模型目录拷进自己的草稿目录、`--work` 给一个新的、不存在的目录。撞内存上限被服务重排时整条重起即可：已跑完的段落在 `<work>.partial/out/` 里，重起时跳过。
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=0`；日志里四行 `name=rerun_part part=<段落> lines=<n>`、四行 `name=rerun_compare part=<段落> same=not_compared`（模型目录还没有 `out/`）、一行 `name=rerun_output directory=<work>/out`。输出在 `<work>/out/<段落>.txt`。退 101 一类是编不过；退 3 是某个段落超 10⁶ 态或估时超 40 分钟（看 `<work>.partial/out/<段落>.txt.tmp` 里的 `PROTO_PLANNED_STATES`、`PROTO_ESTIMATE`、`PROTO_REFUSED`）；退 2 是段形状与预想不同（`PROTO_FLOW_SEGMENT` 全表与 `PROTO_REFUSED`）。
   - 跑完：新派攻方腿，派发提示写「承接：research/prompts/per-write-judging-r2-opus-output.md」、任务号与 `<work>/out/` 的路径；承接的腿对着第二节到第四节的 R1–R14、X1–X6 逐条判，把四个输出拷进模型目录的 `out/`（之后再复跑才比 `same=`）。
2. 只在第 1 条编不过、要看编译错误时：编原型
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/copy/per-write-judging-r2-proto -- cargo build --release --manifest-path /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/copy/per-write-judging-r2-proto/Cargo.toml`
   - 看什么：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`，记下末行的 `build=` 目录；编不过把 `task-server/task log <号> --all` 交给承接的腿。
3. 可选，第 2 条过了、想先看各段落的段形状与状态数而不判：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/copy/per-write-judging-r2-proto -- <build= 目录>/target/release/per-write-judging-r2-attack plan_only`（`<build= 目录>` 照第 2 条末行填）。看 `PROTO_SEGMENT … judged_states=` 与 `PROTO_PLANNED_STATES`；它造一次覆盖写的流与全部臂（含 `planted_two_groups` 挑节点要判的十来个态），不判取样态，退 0。

## 没做什么

- 没往统一任务服务交任务；原型没编、没跑；判决要引的数一个都还没有（命令在上一节）。报告里所有「预测」都是跑前写的。
- 不判正推、辩方那几格；不替主 agent 采纳。第五节改法表每一格都是推的，被攻过零轮。
- L5、L6、L7 三类只给了规格；正推腿 8 类里 σ 段、已准入的在飞头、写长与头声明不符、GPU 一侧四类没写；整理/快照流只造了搬迁副本那一半。
- 没读 `publish_overwrite` 的记录到底点名哪些单元（R6 把握低的原因），交给 `group_lattice` 那一族的红态答。
- 模型目录里的 `tools/rerun.sh` 没跑过；模板的 `--selftest` 对这份拷贝不适用（第一节）。
- 引的第一轮产物与任务日志在仓外（`/home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/`），没进 `research/results/`：它们是主 agent 第一轮的复跑产物，归主 agent 处置。

## 草稿目录里留的

- `/home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/copy`：仓副本，46.6 MiB（`minimal-copy.py` 报的），没删：它是第 1 条命令的 `--frozen` 与原型包所在，`.deliverables` 写成「交接」，主 agent 跑完、判决写完后删。里面没有编译目录。
- `minimal-copy.txt`、`fmt.txt`、`fmt2.txt`、`rerun.sh.draft`（拼 `tools/rerun.sh` 用的草稿）、`.deliverables`、`.owner`、`.writer-lock`：普通文件。
