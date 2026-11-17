# checker-src-backfill-code-r1：判决（2026-11-17）

正文 `research/prompts/_checker-src-backfill-code-r1-body.md`，开工快照 `research/prompts/checker-src-backfill-code-r1-snapshot/`。这是这一题的第一轮，补的是池级 checker `src` 五份在途改动没走过的三方。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推 | `research/prompts/checker-src-backfill-code-r1-sonnet-output.md` | 四件改动与条款对得上，改名一批是纯改名；「I-9.16 ④ 判红」只对顶掉十二种之一的形态真；检查单「类型与名字」不成立（`walk.rs:825` 通配臂）；三处判定分支没有变异行 |
| 攻方 | `research/prompts/checker-src-backfill-code-r1-opus-output.md` | CRC 按内容复用、环长那一格、共享树表的版本没打中；I-3.5 那一路打中 C1、C2；变异打中 C3；kb 里残留旧成员名 C4；检查单「类型与名字」不成立，其余七格成立 |

核查：

- 写判决之前 `sha256sum -c research/prompts/checker-src-backfill-code-r1-snapshot/crates-sha256.txt research/prompts/checker-src-backfill-code-r1-snapshot/kb-sha256.txt` 全部 OK，被判的五份与点名的 kb 文件自开工没变，所以没倒推快照树，引文直接对着主工作区核。
- `python3 scripts/agents/cite-check.py research/prompts/checker-src-backfill-code-r1-opus-output.md research/prompts/checker-src-backfill-code-r1-sonnet-output.md --root . --background research/prompts/_checker-src-backfill-code-r1-body.md` 末行「✓ 核了 81 处引文、121 处裸行号，对上 80 处，没判 122 处（其中裸行号没有引文可比 94 处；逐处列在上面）」。
- 主 agent 现读的行：`crates/singlefs-checker/src/walk.rs:814-828`、`:3743-3762`、`:5990-5997`、`:6024`、`:6136-6141`，`crates/singlefs-checker/src/image.rs:952-980`，`crates/singlefs-checker/src/lib.rs:150-156`，`crates/singlefs-core/src/recovery.rs:7462-7466`，`crates/singlefs-checker/invariants/09-inode树结构.md:28` 的 ③，`crates/singlefs-checker/invariants/05-空间记账.md:11` 的内容格与状态格。
- 复跑的读数（都经任务服务）：
  - 攻方两条探针，在它的副本里（任务 18222，退 0，`test result: ok. 2 passed`）：`ATTACK_PROBE table=干净的十二种 entries=12 i_9_16=Holds i_9_27=Holds kind_without_a_clause=None`；`ATTACK_PROBE table=十二种加一条种类 16（树 ID 65540、根指针全零） entries=13 i_9_16=Holds i_9_27=Holds kind_without_a_clause=Some(16)`；`ATTACK_PROBE table=十二种、livelist 根指针非全零 entries=12 i_9_16=Holds i_9_27=Holds kind_without_a_clause=Some(6)`；同一对由 `pair_without_kind=Violated(…)` 变成 `pair_with_kind_16=PartiallyJudged(…)`。
  - 攻方候选变异的证红，对它的副本（任务 18229，退 1 在期望内）：「根指针一律读成全零（库单测那一格）」没红，「（探针那一格）」抓到，「记了部分没判之后照判那一对」抓到；验证包那一行（任务 18234，退 0）：「折成 I-3.5 输入的那一步丢掉种类」抓到。
  - 主工作区两个验证包测试目标（任务 18225，退 0）：`pool_checker_judges_the_deletion_state_of_every_walked_version` 45 条、`system_configuration_fields_core_and_checker_refuse_the_same_table` 6 条都过。
- 没核的：整份坏镜像（回退那次发布的树表里再插一条种类 16）没有造出来跑整条 `check_pool_image`，「整份镜像一条违例都没有」是攻方推的，探针只钉住了树表那几道不变量成立与那一对改报部分没判两半；正推腿要的池级 checker 库测试整个目标没跑（不带测试名的那条被服务当整包测试拒了），探针那一趟在副本里编过同一份库测试；21 行新变异的逐行现判（正推腿第 8 节 C5）没交，留到提交时的 crates-mutation-replay。

## 二、判决

1. **环长判据加「装进 4 字节」一格（`image.rs`）**：两条腿都没打中，记「没打中」。式子与 core 那一侧同表由任务 18225 的测试目标罩着。
2. **CRC 表法按内容复用（`lib.rs`）**：两条腿都没打中，记「没打中」。取记下的值之前整段逐字节比对，`#[cfg(test)]` 的计数与上限覆盖不进非测试构建（主 agent 现读 `lib.rs:150-156`）。两条线索不进这一轮：每条线程 8 MiB 的上限只数内容字节、不算每条记录的额外开销；E161（崩溃状态去重与耗时拆分） 的装置对这个函数计时，复用之后量到的是命中的代价。两条都没量过，登记进欠账（见「三、改法」第 6 条）。
3. **I-3.5「走到没有条款的形状」那一路（`walk.rs`）**：打中，两条腿一致，主 agent 现查坐实。
   - C1：注释 `walk.rs:3747` 与 I-3.5 状态格写「I-9.16 ④ 判红」。④ 只查十二种在不在（`walk.rs:6136-6141`），③ 的字面是「每一种种类码（登记过的与没登记的都算）至多一条」，许没登记的种类出现一次；探针量到十二种齐全再多一条种类 16 时 I-9.16、I-9.27 都成立，而新路让那一对由违例改报部分没判。那半句担保不成立。
   - C2：livelist、稀疏旁表根指针非全零那一支（`walk.rs:824`）同样没有 I-9.16 去红（探针：`i_9_16=Holds i_9_27=Holds kind_without_a_clause=Some(6)`）；I-3.5 内容格只点了「种类没登记的条目」，这一支只写在状态格里。core 冷走读对这一形报「树的种类没登记」是攻方读码推的，没在镜像上跑。
   - 共享树表的版本记的种类、每版复位、只连坐相邻一对：攻方没打中，正推对得上，记「没打中」。
4. **扫描重建三个错误成员改名（`rebuild.rs`）与改名一批**：两条腿都判纯改名，记「没打中」。`crates`、`scripts` 下旧成员名零命中（主 agent 的 grep）；kb 里 `.claude/kb/checks-owed.md:961` 与英、日两份还写旧成员名（攻方 C4，正推第 7 节同报），那一行说的是当时改名这件事还是现状，主 agent 没判，并进写回 kb 的那一批看。
5. **新加的 21 行变异**：打中 C3。`walk.rs:809`（根指针全零那一位从盘上字节接到判式）没有变异行，现有库单测也抓不到（任务 18229 那一行没红）；`walk.rs:2270`、`walk.rs:1101` 测试抓得到、缺变异行（18234、18229 抓到）。21 行里 5 行是旧行随改名换键、16 行是新断点（两条腿各自数的，一致）。
6. **检查单八格**：
   - 走不到的理由二次查实：成立。正文照事务员的事实表写「`lib.rs` 三处 `expect`」是错的：真新加的只有 `lib.rs:154`（在 `#[cfg(test)]` 里），`:163`、`:178` 是 HEAD 里就有、挪进闭包的旧句（两条腿一致，主 agent 现读 `lib.rs:150-156`）。
   - 类型与名字：两条腿都判不成立，点的是同一处 `walk.rs:825` 的 `other_kind => key_schema_for_tree_kind(other_kind).is_none()`。被匹配的 `kind` 是盘上读来的 `u16`，仓里没有树种类的封闭枚举，这一臂写不成穷举；要不要为树种类立枚举交用户（见「交用户的」）。
   - 其余六格：两条腿都判成立，记成立。
7. **事实表与正文写错的**（两条腿各列，一致的几条）：`lib.rs` 新测试 5 条不是 6 条；`truncation_intent_names_the_record` 是改名不是新加；正文写的 `walk.rs:823` 的臂在 825、`walk.rs:7270` 在 7273；变异 21 行里 5 行是换键。判决里的数以这一节为准。

## 三、改法

1. I-3.5 那一路的漏红（C1、C2），照用户定案（`records/2026-11-16-三方豁免取消与存量补轮.md` 第 37 条，弹窗选「没登记种类判红，非零根记走读失败」）：I-9.16 加一道「每条条目的种类都在登记表里」，并改 I-9.16 ③「登记过的与没登记的都算」那半句；livelist、稀疏旁表根指针非全零记成走读失败，与 core 冷走读报「树的种类没登记」对齐；注释与 I-3.5 状态格里「I-9.16 ④ 判红」那半句照改后的判法重写。**零轮**：腿提的、没实现；池级 checker 不是产品代码，A/B 由改完之后的必红用例与变异行顶（整份坏镜像 S1、S2 各造一份，改前不红、改后红），改完照代码轮收法再攻一轮。
2. `walk.rs:809` 加一条从带根指针非全零的 livelist 条目字节经 `tree_table_entry_of_the_walk` 走到判式的库单测，挂一行变异。**量过**：攻方的探针与候选变异在副本里红在该红的地方（任务 18229）。
3. `walk.rs:2270`、`walk.rs:1101` 各挂一行变异。**量过**（任务 18234、18229 抓到）。
4. 新加的测试 `a_fingerprint_keeps_at_most_the_candidate_limit_and_evicts_the_earliest_remembered_content`（`lib.rs:1577`）与这一批去掉 `a_` 前缀的口径不齐：并进第 1 条那一份补丁改名。
5. `.claude/kb/checks-owed.md:961`（三语）的旧成员名：并进写回 kb 的那一批判是现状句还是经过句。
6. 两条没量过的线索登记欠账：按内容复用的内存上界没算每条记录的额外开销；E161 的装置对 `crc32_castagnoli_table` 的计时在复用之后量的是什么。

## 交用户的

| 事 | 状态 |
|---|---|
| I-3.5 那一路的漏红怎么改 | 用户已定（第 37 条） |
| 要不要为树种类立一个封闭枚举，让 `walk.rs:825` 这类对盘上 `u16` 的兜底臂消掉 | 开着；两条腿都判检查单「类型与名字」不成立，被攻过零轮 |

## 这一轮被判过的文件

- crates/singlefs-checker/src/image.rs：判了环长判据加「在飞上限装进 4 字节」一格与新测试，两条腿都没打中；8 个测试改名判为纯改名。
- crates/singlefs-checker/src/lib.rs：判了 CRC 表法的按内容复用（同值、线程局部、测试计数不进非测试构建）与 5 条新测试，两条腿都没打中；一条新测试的名字口径不齐要改。
- crates/singlefs-checker/src/position_addressed.rs：判了两处测试改名，纯改名。
- crates/singlefs-checker/src/rebuild.rs：判了 `ScanRebuildNotDecided` 三个成员改名与测试改名，纯改名。
- crates/singlefs-checker/src/walk.rs：判了 I-3.5「走到没有条款的形状」那一路，打中两处漏红（C1、C2）与三处判定分支缺变异或缺测试（C3），照第三节改；改名一批判为纯改名；`walk.rs:825` 的兜底臂交用户。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 18 | 不影响 | 2026-11-17 不受影响：环长那一格的代码与条款对得上，两条腿都没打中 |
| D8（核心索引结构） 已定项 9 | 不影响 | 2026-11-17 不受影响：种类码登记表本身没被判，打中的是不变量清单 I-3.5、I-9.16 的写法 |
