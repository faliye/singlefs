# 逐写独立判 第二轮 辩方腿报告（Sonnet）

批次：records/2026-11-05-删轻型并入重型.md#并包。立场：辩方。复核第一轮判决 `research/prompts/per-write-judging-r1-main-verification.md` 里挂起与不采的每一条够不够得着、是不是同样打中所有替代方案；替候选乙与正推腿的判据 D1–D5 辩护，给收严后的判据。
行号都是今天工作区现查的（`grep -nF`）。标注：「量」= 主 agent 交的任务日志里的行（任务 8321 的日志在 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/log8321.txt`，任务 8322 我 `task-server/task log 8322 --all` 读后存在草稿目录的 `log8322.txt`）；「算」= 我用 `tools/recount.py` 对这两份日志重算；「读」= 我读代码；「推」= 没有现场观测。
草稿目录：`/home/fy5090/.local/share/singlefs/work/per-write-judging-r2-defense`（认领过，里面只有日志拷贝、`tools/`、`recount-out.txt`、`cite.txt`、`fmtcheck/` 这些普通文件，没建仓副本、没建编译目录）。

## 模型、复跑命令与 sha256

模型目录 `research/prompts/per-write-judging-r2-sonnet-model/`。**探针没编过、没跑过**（我不起 cargo）：只用 `rustfmt --check` 看过三份 `.rs.in` 都能解析（没有 `error:`，有排版差异，不影响编译）。类型对不对要主 agent 编了才知道，编不过请新派辩方腿承接修。

| 文件 | sha256 |
|---|---|
| `probe/tightened_classifier.rs.in` | `2fec45005c2e2c105612102dac7f4c71c684ab8bbfb19205d548a9eddac2aec2` |
| `probe/tightened_dynamic_unlink.rs.in` | `8112515408dc7f3bf1f8ce3adbecdb62ef505cb3ae9d69b4f717e7f5da013b72` |
| `probe/tightened_pairing_flows.rs.in` | `b2596432c2b95dc0d13905e1ab6b39ce296b01d4b7ef13509782f557ea71900a` |
| `tools/rerun.sh`（拷自 `scripts/agents/three-way-rerun-template.sh`，只改 `rerun_steps`） | `7cdbcef0d9dee86cbddb0f0bfc34a74aee85400ce8879bb19c5d6bfb7524bc0b` |
| `tools/recount.py`（读两份日志的段表，重算全量态数与甲、乙态数） | `4334be513e7dd235b6c133026b2fa64f5e3f0ca81028ec1acad2c67b9c5c7db9` |
| `out/recount-out.txt`（`recount.py` 这一次的原样输出，11 行） | `0b66d3e1505ebfe5e834d3328f095e3c5cc0e23fa348cd0b829c231b389d4b52` |

整份复跑（交主 agent，`--frozen` 要一棵测试文件里还没有 `per_write_judging_tightened` 的树，例如开工快照倒推出的树；调查员那份副本里有他自己的探针，追加我的探针不冲突，但这一份脚本只认没追加过我的探针）：
`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录>`

探针做什么：三份 `.rs.in` 追加在两份测试文件末尾。分类器（`per_write_judging_tightened_classifier`）只读写表、基镜像与 `publishes_in`，逐段、逐写判收严后的判据 E1 到 E6（第三节），打 `PROBE_TIGHT_SEGMENT`、`PROBE_TIGHT_FLOW` 行；动态部分（`per_write_judging_tightened_dynamic`）真判状态，用乙与甲的合成对每个状态比。四条测试的名字与命令在「要主 agent 跑的命令」一节。

## 总述

1. 判决表 11 行（`research/prompts/per-write-judging-r1-main-verification.md:17` 到 `:27`），判决本身我逐行都判「站得住」，没有一行建议撤销；其中 4 行（攻方①②③与六类漏项）里「打中」的射程要缩小：①只打中证据、打不到正推腿的推导，②之二与反例 A 打不到 D4 与收严判据，六类里 L5 打不到收严判据、L6、L7 打的是观察者；逐行写在第一节「辩方判定」列。
2. 攻方三个反例的真实射程：反例 A（副本写晚）与它的两对副本形，**案例级不漏**（S=∅ 那个状态本身就红，攻方自己写了 `research/prompts/per-write-judging-r1-opus-output.md:84` 「整条用例红不红不变（P 本身就红）」），只在逐态红项那一层才中；反例 B（记录先于单元）才是会漏红的那一个。两个反例都对「判据」打中，不是对乙打中：判据收下它们才有意义，而正推腿的 D4 对 A 已经拒（A 的单元诞生代号不超挂载根），对 B 不够（挂载根只看根槽写、不看前缀里记录点名的槽）。
3. 收严：D1–D5 不放宽一个字，只加三处（第三节）：E3 加「单元写落在单元区、码标签是 1/2/3」，E6 新加「前缀里没有一条 journal 记录点名这一段要写的槽」，整段退回（正推腿「按写退回」收成「按段退回」，因为被拒的写与过判据的写同段时的相互作用没有量过）。D5 保留，当前没有机器检查的实现（库里没有记读包装）。
4. 态数（算的）：判据对一段成立时，删除流段 12、16、20 在乙下 43、46、40 个状态（甲 29、31、27）；整条删除流 471（甲 382）；对拍三条流 355、378、404（甲 292、314、329）。四个总数都已用两份日志里量到的段表重算：全量态数逐位等于日志里量到的总数（`tools/recount.py` 的输出 `same=True` 四行，第五节）。**这些数的前提是 E1 到 E6 与 D5 在那些段上成立：E1、E2 是量的（8321、8322），E3、E4、E6 的字节半句与 E5 没量过（推的；E6 在正常流上由 E2 加「记录只点名写过的单元」推出，只有记录点名了从没写过的槽时它才独立拦），命令在第七节。**
5. 乙对甲：在判据内两者等价（正推腿 1.4 的推导，我复核了它引的 `walk.rs` 各行，判据内的推导没找到破绽；破绽在判据的边界上：D4 看不见前缀里记录点名的槽，所以加 E6），乙多出来的态是保险：判据一旦错放进一个段，最可能的相互作用是「同一个单元的两份副本」这一种（量的，只有一条阳性对照臂：435 对只有 1 对不是并，且这一对正是组内，`log8321.txt:118`、`:119`、`:120`）。乙在整条删除流上比甲多 89 个状态（算的，471 − 382）。
6. 取样点：命令写了正推腿 8 类里的 4 类（混合种类的段、写长与头声明不符、两组同被盖且有引用关系、小宇宙全量对照含阳性对照），另加攻方的两条写错次序的臂（第六节）；跑前预测写在第六节，不改。

## 一、第一轮判决「逐条判」表逐条复核

判决表在 `research/prompts/per-write-judging-r1-main-verification.md:15`–`:27`，共 11 行数据（`:17` 到 `:27`）。下面逐行写：判决本身站不站得住、判决说的那句话打到的对象够不够得着、是不是同样打中所有替代方案（甲、乙、只判 S=∅、「J(∅) 红就退回」），再写什么现象推翻我这一行。

| 判决里的条目 | 辩方判定 | 依据 | 什么现象推翻我这一行 |
|---|---|---|---|
| 攻方①：「对所有满足判据的段等价」外推过头（采） | 判决站得住；**同样打中别的臂**（甲、乙、只判 S=∅ 共用这三段当全部证据）；够不着正推腿 1.4 的推导（它是推导，不是抽样） | 量：阳性对照两个写 `PROBE_WRITE label=planted segment=16 position=0 … over_an_earlier_write=true`（`log8321.txt:121`，`:122` 是另一份副本）；未改动的段没有 `PROBE_WRITE` 行（`grep -c 'PROBE_WRITE label=unmodified' log8321.txt` 得 0），读数全 0：`PROBE_SEGMENT_STATES label=unmodified segment=12 …` 在 `:163`–`:166`，段 16 在 `:170`–`:173`，段 20 在 `:179`–`:182`。取样只在删除流一条流的三段上 | 找到一个满足 E1–E6 与 D5 的段，真判出某个状态与合成不同（第六节命令里 `PROBE_SYNTH` 的 `yi_*_differ` 非零） |
| 攻方②：判据依赖「被选中的根只能是上一版」「在飞单元靠诞生代号挡在外面」（挂起，第二轮核） | 判决站得住（挂起对）；对 D1–D5 的射程要拆：之一的 A 半够不着、B 半够得着；之二够不着 | 之一：A 里单元的诞生代号是上一版的，D4 要「头里的诞生代号超挂载根」；A 的单元诞生代号不超挂载根，`crates/singlefs-checker/src/walk.rs:11256` 「return birth_txg <= self.mount_root_checkpoint_txg;」判成已发布，所以 D4 对 A 拒绝，判据不收 A（读）；B 里记录先落、单元后落，D4 只看根槽写写下的挂载根，B 够得着，所以加 E6。池级 checker 的挂载根取根环最新槽，不取记录施加出来的那一版：`crates/singlefs-checker/src/walk.rs:21364` 「mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,」（读）。之二：搬迁副本的事实成立（下一行），但 D4 对它拒，判据不收 | 搬迁批的单元段过了 E4，或 B 臂上 E6 判成收 |
| ②之二的事实：搬迁副本头不盖戳、birth 不变；子集规则与相等比较（主 agent「没逐行核」的 `crates/singlefs-core/src/transaction.rs:10564`、`crates/singlefs-checker/src/walk.rs:17931`） | 我逐行核了：事实成立；同样打中别的臂，但判据拒绝，不进判据内 | 读：`crates/singlefs-core/src/transaction.rs:10564` 「原样字节写到目的地（头不盖戳、birth 不变）」；`.claude/kb/decisions/19-块指针的结构与宽度预算.md:192` 「被搬单元逐字节不变、birth 不变」；`crates/singlefs-checker/src/walk.rs:17931` 「if walked_placements.is_subset(rebuilt_placements) {」；`crates/singlefs-checker/src/walk.rs:17966` 「if readable_rebuilt != readable_walked {」。17966 那一支只在遍历方向有一份落点扫描没收进来（头读不出）时才进；那时这一份在前缀里已被逐份判的 I-2.1 判红（`crates/singlefs-checker/src/image.rs:2076` 「judgements.judge("I-2.1", matches, || {」），所以攻方说的「从绿翻红」更像「红上多一项 I-1.13」（推：进入条件我只构造得出头读不出这一种，指针带提示读的那一种没追） | 找到 P 绿而搬迁副本一落就红的状态 |
| 攻方③：反例 A、B（核不动） | 判决站得住（核不动）；A 够不着收严判据（E4 拒），B 够得着 D1–D5、够不着收严判据（E6 拒）；对所有臂的分辨力见第二节 | 量：主 agent 复跑 8686 每段都跳过（判决 `:19` 引 `PROTO_PLANNED_STATES total=0`；跳过的行 `…rerun-work-3/out.txt:11`、总数行 `:54`）。攻方自己写了「两个反例都改的是写表次序（造的是坏实现），不是今天的实现走得出的历史」（`research/prompts/per-write-judging-r1-opus-output.md:165`）；A 的 S=∅ 本身红（`research/prompts/per-write-judging-r1-opus-output.md:84`） | 8686 之后的复跑在 `late_unit`、`record_before_units` 臂上判出 A、B 的状态，且收严判据判「收」 |
| 攻方：六类破坏条件漏 5 类（挂起） | 判决站得住（挂起对）；逐类事实我都核了；L1–L4 够得着收严判据（并进 E4、E6、E1），**L5 够不着**，L6、L7 打的是观察者、不是判定 J，同样打中所有臂 | 逐类在第四节表里，文件:行逐条给 | 第四节任一格的「推翻」那一句 |
| 正推：判据 D1–D5，D5 要 S=∅ 那一遍的读迹，省了就有活反例（挂起，第二轮攻） | 判决站得住（挂起对）；D1–D4 只读写表与基镜像，D5 要读迹，没有一条量过 | 正推腿自己写了「若只实现 D1 到 D4 而不做 D5，这是一个活的反例，也就是说 D5 不能省」（`research/prompts/per-write-judging-r1-sonnet-output.md:90`）；库里没有记读包装，只有实验 bin 里的「struct RecordingReader<'image, 'tables> {」（`crates/singlefs-verification/src/bin/e161_crash_state_dedup_and_time_split.rs:1521`）与「struct ReadRecordingPoolReader<'reader, 'image> {」（`crates/singlefs-verification/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9014`），都包恢复的读，不包池级 checker 的读 | 删除流上某写的扇区被 S=∅ 那一遍读过 |
| 正推：判据内甲、乙在「红不红」「红项」两层等价（不采为结论） | 判决站得住；**同样打中别的臂**（量只在三段上） | 同第 1 行的量；正推腿自己写了「判据内选甲；乙的价值全在判据外」（`research/prompts/per-write-judging-r1-sonnet-output.md:109`） | 同第 1 行 |
| 正推：两份副本都盖时红项不等于两个单写红项的并，甲在红项层不够（采） | 判决站得住；它**只打中甲、不打中乙**；限定词：这一格在判据外（两个被盖的写 `over_an_earlier_write=true`），不能拿来当判据内的证据 | 量：`PROBE_PAIRS_NOT_UNION label=planted segment=16 pairs=1`（`log8321.txt:119`），非并的那一对是 `a=0 … dev0+822181888 b=1 … dev1+822181888`（`:118`，同偏移两盘一对，即一组副本）；`PROBE_PAIR_INTERACTIONS label=planted segment=16 pairs=0`（`:120`）。算：435 对里 434 对是并，唯一不是并的那一对就在组内，所以乙在这一臂上二阶全对（推：三阶以上没量，第六节命令量） | 阳性对照臂上乙的合成对真判出现 `yi_items_differ` 非零 |
| 正推：补两类破坏条件（已准入单元的副本组、单元写与系统配置槽同段）（挂起） | 判决站得住（挂起对）；两类都是对判据的，同样打中甲与乙；收严判据对两类都拒（E4、E1） | 已准入那一支在 `crates/singlefs-checker/src/walk.rs:11265` 「let Some(row) = row_of_the_instance else {」之后，实例更小的头看实例表行判；系统配置轮换与单元写同段是真实可能：`crates/singlefs-verification/src/crash.rs:329` 的注释写了「上一次发布的系统配置槽轮换与这一次的单元写」之间没有屏障时并在同一段（读）；删除流里它们各占一段（`log8321.txt:132`、`:133`） | 某条流上出现过 E1 拒而今天被收下的段 |
| 正推：观察者分三类，只记计数的要按谓词类加权（挂起） | 判决站得住（挂起对）；续做观察者今天不在单元段的展开里，判据碰不到它 | 读：删除流测试文件头「只展开写数小于 10 的段、同一趟跑两遍」（`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:17`）；`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:503` 「只展开写数小于 10 的段那一趟：七个单元段不展开，挂三个观察者」。读持久集合的观察者按类加权是推的：类只有 S=∅ 与 S≠∅ 两个，乙的态集里每类都有代表 | 一个观察者的断言读了持久集合以外随 S 变的东西 |
| 正推：取样点只撑得起这三段，缺 8 类敏感点（采） | 判决站得住；与攻方①同一件事，两腿一致 | 量：删除流 36 段里 7 个单元段 8、12、16、20、24、28、32 都是 `kinds={"UnitWrite"}` 且 `over_an_earlier_write=0 over_nonzero_base=0`（`log8321.txt:133`、`:137`、`:141`、`:145`、`:149`、`:153`、`:157`）；对拍三条流 17 个单元段 `over_an_earlier_write=0`（算：`out/recount-out.txt` 的段表 5+6+6）；但逐态比只做过 12、16、20 | 同第 1 行 |
| 正推要的记录核对器四格今天的判法（采） | 判决站得住 | 任务 8593 绿，主 agent 跑的；我没复跑，不引它的数 | 8593 复跑红 |

## 二、攻方反例 A、B：对乙与各替代臂的射程

### 2.1 两个层次要分开

记 F 为引擎枚举的全部状态，R 为候选实际判了的那些状态。甲判的 S=∅ 与单写、乙判的 S=∅ 与组内子集，都是 F 里真实存在的状态（前面的段全落、本段取一个真子集、之后一个不落，`.claude/kb/decisions/13-验证路线.md:141` 「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」），所以 R ⊆ F（读：定义）。

- **案例级（这条流红不红）**：R 里有红 ⇒ F 里有红，不会误报；漏报当且仅当 F 里有红而 R 全绿。
- **逐态级（每个状态红不红、红项）**：靠合成还原没判的状态；入库的判定块与 GPU、CPU 对拍比的是这一层（判决 `research/prompts/per-write-judging-r1-main-verification.md:31` 「要保红项」）。

反例 A 的 S=∅ 本身红（攻方自己写了 `research/prompts/per-write-judging-r1-opus-output.md:84` 「整条用例红不红不变（P 本身就红）」），任何判了 S=∅ 的臂（包括甲、乙、只判 S=∅）在案例级都不漏，A 只打到逐态级。B 是唯一可能让案例级漏红的：P 绿、每个单写都等于 S=∅，红只在「点名单元都落、别的没落」那一类状态上。

### 2.2 反例 × 臂（攻方自己标的格子，推的；最后一列是辩方加的）

| 臂 | A 一对副本 | A 两对副本 | B（点名一个单元） | B（点名多个单元） | 案例级会不会漏 |
|---|---|---|---|---|---|
| 只判 S=∅ | 逐态中 | 逐态中 | 中 | 中 | A 不漏；B 漏 |
| 甲 | 中 | 中 | 中 | 中 | A 不漏；B 漏 |
| 乙 | 不中 | 中 | 不中 | 中 | A 不漏；B（多单元）漏 |
| 「J(∅) 红就退回全量」 | 不中 | 不中 | 中（P 绿，不退） | 中 | A 不漏；B 漏；且读了判定结果，与判据要写成读段表相违 |
| 攻方的「引擎先走前缀算 R(P)」 | 不中 | 不中 | 不中 | 不中 | 不漏（推；回退集与被抛弃根没想全） |
| **收严判据（E1–E6）加乙** | 不中（E4 拒） | 不中（E4 拒） | 不中（E6 拒） | 不中（E6 拒） | 不漏（推，等第六节命令） |

前五行取自攻方改法表（`research/prompts/per-write-judging-r1-opus-output.md` 的「改法各修哪一格」一节）；最后一行是辩方加的，全是推的，要 `late_unit`、`record_before_units` 两臂的静态判定（第六节第二条命令）才能改成量。

### 2.3 是不是同样打中所有替代方案

1. **分辨力**：只有「A 一对」「B 点名一个单元」两格分得开甲与乙（攻方报告第三节「四句」里的第一句写了「分辨臂」）；另外两格甲乙一起中，共用前提是「可读形式 ⇒ 前缀不引用本段的槽」，攻方也写了「这一笔另立账先修」。
2. **打中的是判据旧形态，不是乙**：两个反例都靠「判据收下它们」才成立。正推腿的 D4 已经拒 A（A 的单元诞生代号不超挂载根）；B 是 D1–D5 没拦住的，因为 D4 的挂载根取写表里最后一条根槽写，不看前缀里记录点名的槽（池级 checker 的挂载根也是根环最新槽，`crates/singlefs-checker/src/walk.rs:21364` 「mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,」，而它另走「由记录施加出来、根槽从没落盘的那一版」：`crates/singlefs-checker/src/walk.rs:21245` 「let versions_applied_only_by_records = versions_applied_only_by_records(」）。E6 补的就是这一处。
3. **B 能不能构造，没定**：B 要点名的单元是这一段写集的真子集。记录每条只装 1 到 5 个点名项（`.claude/kb/decisions/23-journal的角色与格式.md:324` 「seq 批 1 是 8 条记录、每条只装 1 到 5 个点名项」），段 12 有 28 个写、14 个单元，段 13 只有 4 个记录写（量：`log8321.txt:138`），点名项合计是不是盖住 14 个单元我没数（推）。E6 不依赖这一点：不管点名几个，只要前缀里有一条记录点名了本段的槽，整段退回。
4. **次序性质不是循环**（回应攻方②之一 `research/prompts/per-write-judging-r1-opus-output.md:53` 「判据用被测性质的结论来决定怎么测它」）：D16（发布语义）已定项 7 的持久顺序是「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」（`.claude/kb/decisions/16-发布语义.md:215`），实现违反它时，写表里「记录先于它点名的单元」是直接读得出的（记录写的字节里有点名位置，单元写的偏移在写表里）。E6 读的是这张写表，不是判定结果；次序坏了 ⇒ E6 拒 ⇒ 整段照旧全量 ⇒ 全量判出红。攻方说的循环只在调查员 c1 的「可读形式」上成立（它假设前缀的根只指向已落的单元），E6 把这个假设改成每次都核的一条。推的，等 `record_before_units` 臂。

什么现象会推翻第二节：`record_before_units` 臂上 E6 的计数为 0（判成收），或 `late_unit` 臂上 E4 的不过数为 0。

## 三、收严后的判据（供候选乙用）

输入：写表 `writes`（种类、盘、偏移、字节）、基镜像、段表、`publishes_in` 按写表次序归的发布。输出：对段 k 判「收」或「退回」；**不读任何判定结果**。记段 k 的写集为 W，W 里最低的写表下标为 i，P = 前 k−1 段全落。退回的意思是照今天：写数 ≤ 30 全量展开，超过不展开或取甲二（`crates/singlefs-verification/src/crash_amplification.rs:270` 「pub fn expansion_of(self, segment: &[usize]) -> CrashPointReplaySegmentExpansion {」只看段的写数），所以**退回不会比今天多一个状态**。

| 号 | 条件 | 与正推腿 D1–D5 的关系（只许收严，写明收严在哪） | 对付哪几条攻击 |
|---|---|---|---|
| E1 | W 里每个写的种类都是 `UnitWrite` | 同 D1；`crates/singlefs-verification/src/memory_pool.rs:1170` 「StepKind::UnitWrite => units.push(index),」是写表里认单元写的那一处 | 攻方 L4、正推 b8（单元写与系统配置槽同段） |
| E2 | W 里每个写 w：同盘写表下标更小的写与 w 的范围不重叠，基镜像在 w 的范围上全零 | 同 D2；与撕裂判据同形的两条现成代码：`crates/singlefs-verification/src/crash.rs:2194` 「let base_holds_nonzero_bytes_there」、`crates/singlefs-verification/src/crash.rs:2200` 「let an_earlier_write_with_bytes_overlaps = writes[..write_index].iter().any(|earlier| {」 | 攻方 L1 的复用一半；σ 段与整理、快照流的复用段 |
| E3 | W 里每个写：偏移按槽（16384）对齐；头能解（magic、码标签 1/2/3）；**头声明的字节数等于写长**（码 1、码 3 是 32768，码 2 是 16384）；写落在单元区内 | D3 的「声明跨度 × 槽宽 ≤ 写长」**收严成「等于」**，并**新加**「码标签 1/2/3」「落在单元区」两半句。依据：I-7.8 认领按头声明的跨度往后跳，`crates/singlefs-checker/src/walk.rs:5569` 「first_slot_not_claimed = slot + claimed_slots;」；声明与写长相等就没有跨写认领；「落在单元区内」那半句管单元写落进环或系统配置槽的位置 | 正推 b3 的改写；单元写落在尚未写过的根环或 journal 环槽上（E2 看不见：那里基镜像是零） |
| E4 | W 里每个头：实例 > 前缀里最后一条根槽写写下的挂载根实例，或实例相等且诞生代号 > 该根的 checkpoint_txg；前缀里没有根槽写时判不了、整段退回；实例更小的一律拒，不看实例表行 | 同 D4，只在「前缀里没有根槽写」这一格收严（D4 取基镜像的根，E4 判不了就退回）；字面是 `crates/singlefs-checker/src/walk.rs:11252` 「if instance > self.mount_root_instance {」整支判未发布、`crates/singlefs-checker/src/walk.rs:11256` 「return birth_txg <= self.mount_root_checkpoint_txg;」。这是拒绝搬迁副本、已准入单元的那一条 | 攻方反例 A 与两对形、攻方 L1、L3，正推 b7 |
| E5 | S=∅ 那一遍判定读过的扇区与每个 B(w) 不相交 | **同 D5，保留**。当前没有机器检查它的实现（第一节第 6 行），在有之前这一条是「未满足」，不能据此降态；下面第 5 点写了 E4、E6、E2 合起来能静态排掉它防的那个例子，但那是推的，不拿它换 D5（换了就是放宽） | 正推试四（悬空指针） |
| E6（新） | 前缀里（写表下标 < i）每条 `JournalRecord` 写点名的（盘身份，槽号）都不与 W 里任何写罩住的范围相交（点名单元按 32768 字节算） | **新加**。D1–D5 没有一条看前缀里的记录点名。点名项字段表照 `crates/singlefs-checker/src/lib.rs` 的 `check_journal_record` 另写一份：头 496 字节，每项 56 字节，两份位置各（盘 4、槽号 6、校验和 4），点名项数在偏移 12 | 攻方反例 B、L2 |
| 退回规则 | 任一条对 W 里任一写不成立 ⇒ **整段**退回 | 正推腿写的是「判据按写算，不按段算」（`research/prompts/per-write-judging-r1-sonnet-output.md:62`）；**收严成按段**：被拒的写与过判据的写同段时，被拒写的局部组合对其余写的合成没有量过 | 正推 b7、b8；攻方 L3 |

乙的组（不是判据的一条，是乙的定义补全）：段内写按两条边连成连通分量，一条是同偏移（记录核对器的分法，`crates/singlefs-verification/src/crash.rs:148` 「.any(|copies| copies.iter().all(|copy| copy_is_missing(*copy)))」那一层按偏移分组），一条是同归并键（I-1.8 的分法，`crates/singlefs-checker/src/walk.rs:11472` 「if members.len() < 2 {」之前按归并键分组）。判据内的段上两条边连出同一个分法；被拒的段本来就退回，不用它。态数 = 1 + Σ 各组（2^组内写数 − 1）。

**几点说明**

1. **E6 的字节半句不依赖「记录点名的单元是不是写集的全部」**：只要点名里有一个槽落在 W 的范围内就拒。今天的流上点名的是更早发布的单元（推，没解析；第六节命令里的 `e6_records_in_prefix_naming_a_slot` 就是量它）。
2. **E4 是写表里的字节读法，与池级 checker 同一份**：checker 的挂载根取根环最新槽（`crates/singlefs-checker/src/walk.rs:21364` 「mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,」），写表里最后一条已落的根槽写就是它（读；写表里一条根槽写都没有时，挂载根只在基镜像里、写表读不出来，E4 判不了、整段退回，宁可多拒；探针 `PROBE_TIGHT_SEGMENT` 里这种段的 `e4_not_beyond_mount_root` 等于写数）。
3. **E3 的「等于」比 D3 严**：写长大于头声明时 D3 收、E3 拒。删除流单元写长只有 32768 与 16384 两种（探针里的 `PROBE_TIGHT_SEGMENT` 会打 `e3_claimed_bytes_differ`，推：预期 0）。
4. **E1 之后的混合段**：单元写与系统配置槽同段（攻方 L4、正推 b8）E1 拒，整段退回。如果将来要降这类段，正推腿给的 3^m·(n+1) 是个起点，但它没量过，本轮不收。
5. **E5 与 E4、E6、E2 的关系（推的）**：正推腿试四的悬空指针例子要「前缀里有一个结构先于单元落盘并指着它」。指着一个出生代号为 T 的单元的结构，只可能出生在 T 之后的发布里：它要么是同一次发布的节点（在 W 里或更晚，不被任何已落的根引用，扫不到），要么是这次发布的记录（E6 拒），要么是更早发布里写错出生代号（那一个 P 本身红，且 S=∅ 照判）。这条论证靠「指针带出生代号、读单元时核对」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md` 一族的指针字段，我没逐行追），所以只写成推的，不拿来取代 E5。

什么现象会推翻第三节：删除流或对拍流的某个单元段在今天的实现上被 E3、E4、E6 拒（说明收严过头、降不下去），或者某个被 E1–E6 收下的段在第六节的小宇宙对照里判出 `yi_items_differ` 非零（说明收严还不够）。

## 四、对攻方各类与两腿补的破坏条件逐类回应

「事实」列是我核的（读）；「拦」列写收严判据哪一条在字面上拦它；「拦不住时」写退回之后的态数（照今天的做法，算的闭式）。

| 类 | 事实（读） | 收严判据怎么拦 | 拦不住时剩多少态 | 什么现象推翻 |
|---|---|---|---|---|
| 攻方 L1 前缀引用本段要写的槽（A） | 写表里一次写属于哪次发布按它后面第一条根槽 FUA 写归：`crates/singlefs-verification/src/crash.rs:239` 「pub(crate) fn checkpoint_txg_of_the_publish_that_made_the_write(」、`crates/singlefs-verification/src/memory_pool.rs:1171` 「StepKind::JournalRecord => records.push(index),」；晚写的单元仍带上一版的诞生代号 | E4 拒（诞生代号不超挂载根） | 今天的 2^n − 1（n ≤ 30），段 12、16、20 是 268,435,455、1,073,741,823、67,108,863（`log8321.txt:137`、`:141`、`:145` 的 `states=`，量的） | `late_unit` 臂上 E4 不过数为 0 |
| 攻方 L2 恢复施加记录前点名单元两份都验过（B） | `.claude/kb/decisions/23-journal的角色与格式.md:364` 「一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加」；池级 checker 另走记录施加出来的版本，`crates/singlefs-checker/src/walk.rs:21245` 「let versions_applied_only_by_records = versions_applied_only_by_records(」 | E6 拒 | 同上 | `record_before_units` 臂上 E6 为 0 |
| 攻方 L3 诞生代号不挡的在飞单元（搬迁副本） | 第一节第 3 行：`crates/singlefs-core/src/transaction.rs:10564`、`crates/singlefs-checker/src/walk.rs:11256` | E4 拒（搬迁副本实例等于挂载根且诞生代号不超 txg，或实例更小，两种都不满足「超」） | 搬迁批的单元段今天的态数；这些段的段表这一轮没量（推：删除流与对拍三条流是删除类发布，没有搬迁批） | 搬迁批单元段过了 E4 |
| 攻方 L4、正推 b8 系统配置轮换与单元写同段 | `crates/singlefs-verification/src/crash.rs:329` 「上一次发布的系统配置槽轮换与这一次的单元写」的注释：它们之间没有屏障时并在同一段；删除流里各占一段（`log8321.txt:132`、`:133`） | E1 拒，整段退回 | 3^m·2^(n−m) − 1，m 是系统配置写数（今天的闭式，`crates/singlefs-verification/src/crash.rs:1932`） | 删除流或对拍流出现混合种类段（探针的 `e1_only_unit_writes=false` 且段写数 > 2 的行） |
| 攻方 L5 I-1.8 判过次数对两份副本是合取 | `crates/singlefs-checker/src/walk.rs:11472` 「if members.len() < 2 {」；E4 收下的段里在飞单元按 `walk.rs:11252` 整支判未发布，不进扫描项，所以判过次数不随 S 变；红项只收违例与记录核对器两项（`crates/singlefs-verification/src/crash.rs:1151` 「pub fn red_items(&self) -> Vec<String> {」），不适用与计数不进红项 | **够不着收严判据**：这一类只在 E4 拒的段里出现，那些段退回全量，次数精确 | 同 L3 | 一个 E4 收下的段上，某个状态的判过次数与 S=∅ 不同 |
| 攻方 L6 读持久集合的观察者 | `crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:110` 「fn is_cut_after_some_unit_and_before_its_record(&self, persisted: &[bool]) -> bool {」 | 不属判据（判的是 J，不是观察者）；乙的态集里有 S=∅ 与每个单写，覆盖 [S=∅]、[S≠∅] 两类，计数按类加权（推） | 不加态 | 观察者的谓词取值类多于两类 |
| 攻方 L7 真跑实现的续做观察者 | 今天不在单元段的展开里：`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:503` 「只展开写数小于 10 的段那一趟：七个单元段不展开，挂三个观察者」；挂完的盘上准入随新实例变：`crates/singlefs-checker/src/walk.rs:11265` 「let Some(row) = row_of_the_instance else {」 | 不属判据；若将来放进单元段，E4 的挂载根要换成挂完的盘上的根，要另判 | 不在本轮 | 续做观察者被挂到单元段的展开上 |
| 正推 b7 已准入单元的副本组 | `crates/singlefs-checker/src/walk.rs:11265` 起实例更小的头看实例表行 | E4 拒，整段退回 | 同 L3 | 同 L3 |
| 正推 b1 镜像双副本（同一被引用单元两份都被盖） | `crates/singlefs-checker/src/image.rs:2076` 「judgements.judge("I-2.1", matches, || {」每份副本各判一次 I-2.1 | E2 拒（被盖就是盖了更早的写） | 今天的 2^n − 1；这一类正是乙的组要管的（阳性对照 `pairs=1`） | E2 收下的段上组内合成不等于真判 |
| 正推 b2 记录核对器全部副本缺 | `crates/singlefs-verification/src/crash.rs:148` 「.any(|copies| copies.iter().all(|copy| copy_is_missing(*copy)))」 | E2 拒 | 同上（C561 那一段：`.claude/kb/checks-owed.md:666` 的 262144 个状态是今天的数） | 同上 |
| 正推 b3 认领跨相邻槽 | `crates/singlefs-checker/src/walk.rs:5569` 「first_slot_not_claimed = slot + claimed_slots;」 | E3 拒（声明字节数等于写长，不跨写） | 今天的 2^n − 1 | 声明等于写长而认领仍跨写 |
| 正推 b4 同段重叠写 | 同 E2：段内更早的写也算 | E2 拒 | 今天的 3^m·2^(n−m) − 1 | 同 E2 |
| 正推 b5 判过 0 次报不适用 | `crates/singlefs-checker/src/walk.rs:21378` 「if judge_merged_version_total_order(reader, &content_units, &mut judgements) == 0 {」 | 同 L5，不进红项 | 不加态 | 同 L5 |
| 正推 b6 根与单元同段 | `crates/singlefs-verification/src/crash.rs:329` 一族：段按设备切，`.claude/kb/decisions/13-验证路线.md:141` 「带 FUA 的写只放行它自己那块盘」 | E1 拒 | 今天的闭式 | 同 L4 |

**同样打中所有替代方案吗**：表里每一类都是对「判据」打中的，甲与乙同样吃到；乙与甲在这些类上只在 E2 拒的那几格有差别（乙管同一单元两份副本的组内相互作用），而那些段在收严判据下本来就退回。所以这张表里没有一格是「只打中乙」的。

什么现象会推翻第四节：任一格的「推翻」那一句。

## 五、收严判据下各剩多少态

**前提（写在数前面）**：下面「乙」「甲」两列是「E1 到 E6 与 D5 在这一段上成立」时的态数；「现在」那一列是量的；段不成立退回，态数回到「现在」那一列。成立与否里，E1、E2 是量的（`log8321.txt` 的 `PROBE_SEGMENT` 行、`log8322.txt` 的 `PROBE_FLOW_SEGMENT` 行），E3、E4、E6 的字节半句与 E5 没量过，是推的，探针与命令在第六、七节。每组两份副本是推的：写数全是偶数是量的（段表），「每个偏移恰两个写」没量，探针打 `group_sizes`。

重算（这一次真跑出来的原样，`python3 -I tools/recount.py log8321.txt log8322.txt`，输出存在 `out/recount-out.txt`，11 行）：

```
RECOUNT flow=unlink segments=36 unit_segments=[(8, 24), (12, 28), (16, 30), (20, 26), (24, 24), (28, 22), (32, 24)]
RECOUNT   full=1463812286 measured=1463812286 same=True jia=382 yi=471
RECOUNT flow=second-deletion-batch-quarantined-data-unit-records-carry-th segments=28 unit_segments=[(8, 24), (12, 30), (16, 26), (20, 24), (24, 22)]
RECOUNT   full=1178599580 measured=1178599580 same=True jia=292 yi=355
RECOUNT flow=three-unit-file-applied-only-by-its-records-beside-the-versi segments=32 unit_segments=[(8, 24), (12, 30), (16, 18), (20, 16), (24, 16), (28, 24)]
RECOUNT   full=1107689646 measured=1107689646 same=True jia=314 yi=378
RECOUNT flow=deletion-through-the-last-batch segments=32 unit_segments=[(8, 24), (12, 30), (16, 26), (20, 24), (24, 22), (28, 24)]
RECOUNT   full=1195376807 measured=1195376807 same=True jia=329 yi=404
RECOUNT unlink segment=12 writes=28 full=268435455 jia=29 yi=43
RECOUNT unlink segment=16 writes=30 full=1073741823 jia=31 yi=46
RECOUNT unlink segment=20 writes=26 full=67108863 jia=27 yi=40
```

`same=True` 四行说的是：我从段表里逐段用闭式（单元段 2^n − 1、系统配置槽段 3^2 − 1、记录段 2^n − 1）重算出的全量态数，逐位等于日志里量到的总数（删除流 `log8321.txt:124` 的 `state_count=1463812286`，对拍三条流 `log8322.txt:159`、`:188`、`:221` 的 `states=`）。这同时核了正推腿「对拍三条流的段表是凑的」那一句：三条流的段表现在是量的；第三条流正推腿推的是 30+24+24+18+16+16，量到的单元段是 24、30、18、16、16、24（次序不同，写数的多重集相同），推对了。

### 5.1 删除流（算的，前提见上）

| 段 | 写数（量，`log8321.txt`） | 现在（量） | 乙 | 甲 | 组数（推） |
|---|---|---|---|---|---|
| 12 | 28（`:137`） | 268,435,455 | **43** | 29 | 14 |
| 16 | 30（`:141`） | 1,073,741,823 | **46** | 31 | 15 |
| 20 | 26（`:145`） | 67,108,863 | **40** | 27 | 13 |
| 整条流（七个单元段 8、12、16、20、24、28、32） | 233 个写（`:124`） | 1,463,812,286 | **471** | 382 | — |

乙比甲整条流多 89 个状态（算的）。某一段退回时，整条流的数 = 471 − 该段乙的态数 + 该段今天的态数；例：只有段 12 退回是 471 − 43 + 268,435,455 = 268,435,883（算的）。

### 5.2 对拍三条流（算的，前提见上；段表是量的）

| 流（`log8322.txt` 行） | 单元段（量） | 现在（量） | 乙 | 甲 |
|---|---|---|---|---|
| second-deletion-batch-quarantined-data-unit-records-carry-the-mark（`:159`） | 24、30、26、24、22 | 1,178,599,580 | **355** | 292 |
| three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first（`:188`） | 24、30、18、16、16、24 | 1,107,689,646 | **378** | 314 |
| deletion-through-the-last-batch（`:221`） | 24、30、26、24、22、24 | 1,195,376,807 | **404** | 329 |

这三个乙数与正推腿 R2 列的 355、378、404 逐位相同（它的段表是凑的，我这里是量的段表重算，两条路径只共用「单元段按组 1 + 3·(n/2)」这一个前提，不共用段表）。第二条流是「记录先落、根没落、靠下一次挂载施加」的那条：单元段 12 与 16 之间隔着没有根槽写的记录段与两段系统配置槽（量：`log8322.txt` 里该流的段 13、14、15，用 `grep -F 'flow=three-unit-file-applied-only' log8322.txt` 看），E4 取「前缀里最后一条根槽写」的挂载根在这条流里是段 10 那一条根，探针打 `PROBE_TIGHT_SEGMENT` 时看这条流的段 16 过不过 E4、E6（推：过）。

### 5.3 判据内「红项」两层的合成式（回应问题 2 的「保哪一层」）

判据成立的段上，乙的合成是 red_items(S) = red_items(∅) ∪ ⋃ 各组 red_items(S ∩ 组)；红不红是这个并是否非空。两层都由同一个并还原（推，正推腿 1.4）。阳性对照里乙的并在二阶上对（第一节第 8 行的量与算）；三阶以上、含 16 个状态的两组同被盖、含 4096 个状态的小宇宙，是第六节命令量的。

什么现象会推翻第五节：`PROBE_TIGHT_FLOW` 的 `reduced_total_group_form` 与上表不等（说明有段被 E3、E4、E6 拒，或 `group_sizes` 里有不是 2 的组），或 `engine_total` 与量到的总数不等（说明探针读的段表与 8321、8322 不是同一张）。

## 六、敏感取样点：挑了哪几类、写成什么命令、判什么

正推腿 5.2 表列的 8 类（`research/prompts/per-write-judging-r1-sonnet-output.md` 的「还缺的敏感取样点」一节）我挑了 4 类写成能跑的测试，另加攻方两条写错次序的臂，共四条测试：删除流上的静态判据一条（含 `late_unit`、`record_before_units`、`short_write`、`mixed_segment` 四臂）、对拍三条流上的静态判据一条、动态两条（小宇宙对照、两组同被盖）；都在 `per-write-judging-r2-sonnet-model/probe/` 里，命令在第七节：

| 正推腿的类 | 我写的 | 判收严判据的什么 |
|---|---|---|
| 混合种类的段（D1 的反面） | `mixed_segment` 臂：第 11 段（系统配置槽轮换）并进第 12 段，再加四条流的 `kinds` | E1 该拒的拒不拒；今天的流上有没有混合段 |
| 写长与头声明不符（D3 的反面） | `short_write` 臂：第 12 段一个 32768 字节的单元写截成 16384 字节 | E3 的「等于」拒不拒 |
| 多组同时被盖、且有引用关系 | `probe_two_planted_groups_over_every_pair_of_units_of_the_earlier_publish`：第 12 段每一对单元（至多 91 对）被第 16 段两组（两盘各一份、长度相同）盖上，宇宙 = 这两组 4 个写、16 个状态逐个真判；哪一对的合成与真判不等，连两个单元的类标签与偏移一起打 | 乙的「组间独立」；祖先后代同被盖会不会遮红项（正推腿 2.2 的 a） |
| 小段全量对照 | `probe_group_synthesis_against_the_judged_states_of_small_universes`：未改动的段 12、16、20 各取 5 组（10 个写，1024 个状态，3 次取样）逐态真判，与乙、甲的合成比；再做阳性对照（第 16 段一组挪到第 12 段一个单元的两份副本上，宇宙含这一组，3 次取样） | 判据内乙与甲对真判的两层（红不红、红项）；判据外乙还对不对 |
| （攻方的类）反例 A、B | `late_unit` 臂、`record_before_units` 臂 | E4、E6 拒不拒 |

没写的 4 类：σ 段（C561 复用链）、已准入的在飞头（续做之后挂完的盘）、整理与快照流的复用段、GPU 判器一侧。原因：前三类的流构造函数在别的测试文件里，我没读到够写探针的程度；已准入那一类要在挂完的盘上取单写与成对，要先把续做观察者放进探针；GPU 一侧要上卡。这几类对应的是「E2、E4 该拒的段」与「GPU 一侧同样成立」，仍是开着的。

### 跑前写死的预测（跑出来之后不改，只往下另记）

「推翻」栏写读到什么数就推翻；这些预测都是推的。

| 号 | 预测 | 依据 | 推翻它的读数 | 推翻之后怎么改 |
|---|---|---|---|---|
| Q1 | `label=unmodified`：7 个单元段 `passes=true`，`e2_*`、`e3_*`、`e4_not_beyond_mount_root`、`e6_records_in_prefix_naming_a_slot` 全 0，`group_sizes={2: N}`（段 12、16、20 的 N 是 14、15、13），`engine_total=1463812286`，`reduced_total_group_form=471` | E1、E2 是量的（8321）；其余是第三节的推 | 任一单元段 `passes=false` | 看哪一列非零：那一段退回，第五节的数把那一段换成「现在」那一列；E3/E4/E6 拒了今天的正常流，说明收严过头，要重新说明收严在哪 |
| Q2 | `label=late_unit`：段 16 `e4_not_beyond_mount_root=2`、`diag_birth_differs_from_publish=2`、`passes=false` | A 的两份副本头里诞生代号是上一版的 | `e4_not_beyond_mount_root=0` | 撤回「E4 拒 A」，A 要靠 D5 拦 |
| Q3 | `label=record_before_units`：段 12 `e6_records_in_prefix_naming_a_slot` ≥ 1，`passes=false`，`e4_not_beyond_mount_root=0` | 被挪的 4 个记录写点名的是段 12 的单元 | `e6_…=0` | 要么点名项解法写错，要么这一发布的记录没点名段 12 的单元，B 在这条流上构造不出；写进「没打中的形状」 |
| Q4 | `label=short_write`：段 12 `e3_claimed_bytes_differ=1`，`passes=false` | 头声明 32768、写长 16384 | 0 | E3 的「等于」没拦住，查 `claimed_bytes` |
| Q5 | `label=mixed_segment`：段 12 的行是 `e1_only_unit_writes=false` | 并进了两个系统配置槽写 | 出现 `e2_over_an_earlier_write` 行 | 并段构造错 |
| Q6 | 对拍三条流：`PROBE_TIGHT_ENGINE states=` 依次 1178599580、1107689646、1195376807；`reduced_total_group_form` 依次 355、378、404 | 第五节的算 | 任一不等 | 看 `passing_unit_segments` 与 `unit_segments` 差几段 |
| Q7 | `unmodified-trial*`（段 12、16、20 各 3 次）：`baseline_red=false`，`yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0` | 8321 里单写、成对、只差一个、随机 2000 个子集与 S=∅ 逐态相同 | 任一非零 | 判据内「J(S)=J(∅)」被推翻，判据整体要重新说 |
| Q8 | `planted-trial*`（段 16 挪过去一组）：`yi_red_differs=0 yi_items_differ=0`，`jia_items_differ` 约 256（两份都落的状态数：2^(10−2)；至少 1） | 8321 里阳性对照二阶上唯一的非并就在组内 | `yi_items_differ` 非零 | 乙在三阶以上不精确，乙作为「组内全组合」的口径不够：要把有相互作用的组并成大组，大组怎么认写进下一轮 |
| Q9 | `PROBE_TWO_GROUPS_SUMMARY`：`pairs_where_yi_items_differ=0`，`pairs_where_jia_items_differ` ≥ 1 | 同 Q8 | `yi` 非零：打出的 `label=two-groups-<左>-<右>-classes-<类标签>-<类标签>` 里有一个是码 2 节点、一个是它下面的单元，就是祖先后代遮红项，乙的「组间独立」被推翻 | 并组的边加「引用关系」，要读前缀结构，写表上没有 |

什么现象会推翻第六节：Q1 到 Q6 里任一条被推翻，第三节、第五节对应的格要重写；Q7 到 Q9 任一非零，乙在判据内（Q7）或判据外（Q8、Q9）的口径要改。

## 七、要主 agent 跑的命令

只有一条整份复跑；它在冻结副本的两份测试文件末尾追加探针、各起一次 `cargo test`（四条测试，每条 cargo test 那一行在脚本里自带 `SINGLEFS_RUN_SCOPE` 声明，种类 target、范围 1 个测试），输出汇进 `<work>/out.txt`。没往统一任务服务交任务；下面是交给主 agent 的整条。

1. 整份复跑
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录>`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（脚本只从参数取路径，随意也行）；`<冻结副本>` 要一棵两份测试文件里还没有 `per_write_judging_tightened` 的树（开工快照倒推出的树，或主工作区的一份拷贝）；`<新的草稿目录>` 在工作根下、不在或为空。要在新 shell 里起。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号>` 里 `name=rerun_output file=… lines=`（行数是 `PROBE_` 行数）与 `name=rerun_compare same=not_compared`（模型目录里没有 `out/out.txt`，这一趟不比）；再读 `<work>/out.txt`：`PROBE_TIGHT_FLOW`（四条流各一行，对 Q1、Q6）、`PROBE_TIGHT_SEGMENT label=late_unit|record_before_units|short_write|mixed_segment`（Q2–Q5）、`PROBE_SYNTH`（Q7、Q8）、`PROBE_TWO_GROUPS_SUMMARY`（Q9）。退 101 是编不过：错误在 `<work>/out-*.txt` 里，请新派辩方腿承接修（派发提示写「承接：research/prompts/per-write-judging-r2-sonnet-output.md」并给任务号）。
   - 判决要引的数照旧由主 agent 在入库的装置上重做一次（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」一节之后的「核查与判决由主 agent 做」第 3 条）；这一份读数在主 agent 跑出之前一律是「推的，待主 agent 跑」。
（`rerun.sh` 自带的 `--selftest` 只对未改 `rerun_steps` 的模板成立：它拷出去跑的是模板那一段，我改了那一段，这条自证对我这份会红，所以不列。）

等这条命令的结果才能往下做的：Q1 到 Q9 全部；第五节三张表里「乙」「甲」两列从「算的、前提待量」升为「量的」；第三节 E3、E4、E6 的字节半句；E5 没有机器实现，不在这条命令里。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果：第一节每行的「辩方判定」是辩方的读法；没往统一任务服务交任务；判决要引的数没复跑（命令在上面第七节）。
- 探针没编、没跑：三份 `.rs.in` 只过了 `rustfmt --check` 的解析，没过类型检查；所有 `PROBE_` 行的读数都还没有，预测写在第六节。
- E5（S=∅ 那一遍的读迹）没有实现，也没写：库里没有对池级 checker 的记读包装，要加库里的件，不在我的写范围里（我只写模型目录与草稿）。
- 没量「每个偏移恰两个写」「写长只有 32768 与 16384」「单元区边界」：探针打了 `group_sizes`、`e3_claimed_bytes_differ`、`PROBE_TIGHT_AREA`，E3 的「落在单元区」半句探针没实现（只打每块盘上单元写最低偏移与非单元写最高末尾，让主 agent 看区间是否分得开）。
- 没写 σ 段、整理与快照流的复用段、已准入的在飞头（续做之后挂完的盘）、GPU 判器一侧四类取样点（第六节写了原因）。
- 第 12 段两盘副本之间的偏移是否总相同、续做观察者放进单元段之后 E4 的挂载根怎么换：没追，写在第四节 L7 一行。
- 没读 `crates/singlefs-core` 的恢复与分配的全部代码；第三节第 5 点「指针带出生代号、读单元时核对」是推的，没逐行追 `crates/singlefs-checker` 的指针校验。
- 没碰别的腿这一轮的报告（禁读清单）。
- 没做 git 写操作；草稿目录里只有日志拷贝、`tools/recount.py`、`recount-out.txt`、`fmtcheck/` 的三个解析用拷贝，没有仓副本与编译目录（没有 `du` 要记）。
