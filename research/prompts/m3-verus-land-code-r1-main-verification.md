# Verus 核接回 `crates/`：代码轮第一轮判决（2026-10-01）

<!-- doc-lint:not-numbers M2 M4 G1 G2 G3 G6 G7 G8 G9 C1 C2 C3 C4 C5 C6 C7 C8 K02 K19 K20 V01 V12 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / diff | `research/prompts/_m3-verus-land-code-r1-body.md`、`_m3-verus-land-code-r1-background.md`、`_m3-verus-land-code-r1-diff.md` | — |
| 开工快照 | `research/prompts/m3-verus-land-code-r1-snapshot/`：`crates-sha256.txt`（212 份，冻结副本 `/tmp/claude-1000/m3-verus-land-code-r1/tree/`）、`kb-sha256.txt` | 腿交齐之后冻结副本 `sha256sum -c` 全 OK；`kb-sha256.txt` 在主仓全 OK |
| 云端正推（Sonnet） | `research/prompts/m3-verus-land-code-r1-sonnet-output.md`（sha256 8e9e4929…bdcd）；模型 `research/prompts/m3-verus-land-code-r1-sonnet-model/` | 它的对拍（56 924 864 个输入）没复跑：与攻方腿互不共享代码的另一份对拍（60 000 个输入）结论相同，两条路径各自带阳性对照 |
| 云端攻方（Opus） | `research/prompts/m3-verus-land-code-r1-opus-output.md`（sha256 0e897b08…bd59）；模型 `research/prompts/m3-verus-land-code-r1-opus-model/` | 定点项复跑见第四节；C6 表格单元格里的引文 `cite-check.py` 认不出，主 agent 在冻结副本上逐行 `sed -n` 核过：14 处原文都在（例：`lib.rs:467`「if record.ordinal_within_publish as u64」；`[原文` 共 46 处；`recovery.rs:3122` 的 `unreachable!`；改前 fcc4d340 第 3143 行是 `u64::from(record.ordinal_within_publish.0)`） |
| 引文 | 两份腿报告 | `python3 scripts/agents/cite-check.py <两份报告> --root /tmp/claude-1000/m3-verus-land-code-r1/tree --root . --background research/prompts/_m3-verus-land-code-r1-background.md`：「✓ 核了 34 处引文，对上 34 处，没判 0 处」 |
| 主 agent 现跑的 Verus | `/tmp/claude-1000/m3-verus-land-code-r1-main-check/verus/`、`…/verus-green/`；Verus 0.2026.09.20.aef82ed `--no-cheating` | 新 crate 核原样「14 verified, 0 errors」；主循环条件加 `&& false`（一条都不施加）「14 verified, 0 errors」；攻方 `verus-green-mutations.tsv` 12 行（每行只改一处可执行判定：反向链不判、提交标记不看、末条不带提交标记照施加、C540 不判、点名单元不验、在飞上限不看、一条都不施加、锚点读不出时不看 txg、`Err` 一臂恒报、最大事务号取最小、水位之上条数不数、锚点不看 txg）**12 行全是「14 verified, 0 errors」**（`summary.txt`） |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| C1 核与原文等价 | 一致：56 924 864 个输入零差异（结局、报告五字段、报错、读盘日志、panic），四个改前变异体作阳性对照都红 | 产品输入上没打中：60 000 个随机表结局与读盘序列逐个相同；键与字段不一致的 `BTreeMap` 上改前施加 2 条、改后 0 条 | **重构在产品输入上等价，两条腿两份互不共享的对拍同向**。键与字段不一致那一格**打中**，两条腿都独立找到：今天 8 处调用方都按字段建键（攻方逐处读过），产品上走不到；改法取最小的一改——胶水建表前断言每个键等于 (实例代号, 计数器)，核把「入参按 (实例代号, 计数器) 严格升序」写成 `requires`（零轮） |
| C2 证明证到了什么 | 打中（spec 太弱）：只含「整次发布」「严格大于所选根」；`Err` 一臂空真；38 行必红世界只有第一行改了可执行判定 | 打中两处：同左，给了 12 行只改判定的变异；38 行没有一行只改可执行判定 | **打中，主 agent 现跑坐实**（第一节最后一行）：今天的 14 项证明只管住「施加的是若干整次发布、都在所选根之上」，D23（journal 的角色与格式） 已定项 14 前缀口径里的其余几条、读者规则四格、在飞上限、计数器连续与链首、报告字段都不在证明里，一条都不施加也验过。38 行必红世界证的是「改坏 spec 会红」，证不到「改坏代码会红」。并进第 4 批扩证明：12 行（加「删 I-8.3 那一判」）在 spec 补上之后都要转红，进 `verus-mutations.tsv` |
| C3 编进二进制的与 Verus 看的 | 一致：展开读过，没有 cfg、没有 `#[verifier::…]` | 没打中：依赖逐个 `=` 钉死 | **没打中**。两条腿同向 |
| C4 胶水 | 没做 | 打中三处：G6（K19：核里锚点筛选不看 txg，Verus 推绿，产品在 `recovery.rs:3122` 的 `unreachable!` 上 panic）；G2（同 C1 那一格）；公开 API 的次序前提没写 | **G6 打中**（复跑见第四节）：`unreachable!` 靠的是核与 `recovery.rs` 两份手抄的同一个筛选永远一致，那是没被证明的假设，盘上数据走得到的分支用 panic 收，违 `.claude/singlefs-ai-sop/rules/code-discipline.md`「能恢复的失败走 Result」。改法取攻方的最小一改：核的失败带出它看到的计数器，胶水直接拼报错，删掉再调原函数与 `unreachable!`，没人调的那份手抄删掉（零轮）。G2 与公开 API 同 C1 那一格 |
| C5 测试与变异 | 两条新单测只拦 J、K、L 三种改法 | 打中四处：514 / 515 / 613 被反向链兜住（补取样点三行转红）；C540 测试拦不住「只跳紧挨着的下一条」（K20）；在飞上限差一不红（K02）；重建根的 `rollback_floor` 取所选根的不红（G08a） | **打中**（复跑见第四节）。改法都是只动测试：`rewrite_record` 改完重算后面记录的反向链（攻方 `sampling-fix.tsv` 两行）；C540 测试加「标志挪到 jsn 7、jsn 8 抹掉」一格；补一条在飞上限的 harness 用例与一行变异；在施加记录的用例上钉 `rollback_floor` 等于记录新根段的那一份 |
| C6 编码纪律 | 没做 | 打中 14 处 | **打中**，14 处原文主 agent 逐行核过。并进第 4 批：`as` 换回 `from`（Verus 不支持的写明理由）、计数器 / txg / 事务号 / 实例代号在核里各给 newtype（核不依赖 `singlefs-format`，在核 crate 里自己立）、`ends_its_publish` 与 `verify_named_units` 两个布尔换枚举、`open` / `taken` / `found` 改名、同一失败统一一个名字、`m2_holds` 改成不带编号缩写的名字、为变异锚点留的死子式删掉并同步改第 23、355 行变异、46 处 `[原文 N]` 行号注释删掉（与改前的对照留在实现员报告与这份判决里）、核头注释改对、`Segment` 装下标那一处换名、测试里的魔法数起名 |
| C7 判法版本闭包 | 变：44 份变 45 份，`use` 自动进闭包成立 | 没打中 | **没打中**，两条腿同向（正推量过、攻方读代码） |
| C8 E165 装置 | 没做 | 没打中（看得浅） | **只看了并行与次序**，跑前登记没逐条对；E165 装置三处 clippy 红另记（并进第 4 批 3-6） |

两条腿提的改法都只修错与补测试，没有一条是为性能改 core（正文第五节最后一条）。

## 三、两条腿之外主 agent 记的

- 攻方自报用 `sed -i` 改了一次自己新建的模型脚本，「执行前拒绝的写法」那道闸没拦下：工具簇欠账。
- `crates/mutations.tsv` 第 43、723、738、1039、1296 行锚点在 HEAD 上就腐化（不在这一轮射程），只修锚点并进第 4 批。

## 四、复跑

主 agent 在 `/tmp/claude-1000/m3-verus-land-code-r1-main-check/opus/` 从冻结副本拷一份，用攻方入库的 `apply-mutation.py` 与它的总表（`make-combined-table.sh` 现生成）逐项套上、只跑点名的 harness 目标、换回（脚本 `run.sh`，经 `run-with-memory-cap.sh 12G`）。`logs/summary.txt` 原样：

```
NONE rc=0 passed=8 unreachable_panics=0 failed: 
T514 rc=0 passed=8 unreachable_panics=0 failed: 
T515 rc=0 passed=8 unreachable_panics=0 failed: 
T613 rc=0 passed=8 unreachable_panics=0 failed: 
T614 rc=101 passed=7 unreachable_panics=0 failed: last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens 
K20_c540_skips_the_very_next_record rc=0 passed=8 unreachable_panics=0 failed: 
T514+S rc=101 passed=7 unreachable_panics=0 failed: an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied 
T515+S rc=101 passed=7 unreachable_panics=0 failed: transaction_whose_commit_marker_never_appears_before_the_next_transaction_keeps_its_publish_unapplied 
T613+S rc=101 passed=7 unreachable_panics=0 failed: with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens 
S-only rc=0 passed=8 unreachable_panics=0 failed: 
G08a rc=0 passed=23 unreachable_panics=0 failed: 
K02 rc=0 passed=48 unreachable_panics=0 failed: 
```

K19 那一行（三个目标：`journal_last_record_flag_of_a_multi_record_publish`、`writable_mount_takes_a_second_instance`、`recovery_reads_the_created_file_after_reopen`）：`rc=101 passed=1 unreachable_panics=20`，红的测试含 `cold_start` 等四条。攻方报的「1948 处」是 13 个目标上的数，主 agent 没在同一组目标上重量，判决只引自己量的 20 处。

C5 四处与 G6 全部坐实：514 / 515 / 613 今天不红、补取样点之后各红点名的那一条、只补取样点全绿（取样点不改今天的结局）；阳性对照 T614 红；K20、G08a、K02 在点名的目标上全绿。

## 五、写回与下一步

第 4 批（照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 4 条按文件聚簇）：

1. **核与胶水一簇**（`crates/singlefs-replay-prefix/`、`crates/singlefs-core/src/recovery.rs`、`crates/mutations.tsv` 原位改第 23、355 行）：扩证明（3-4 原定的计数器连续、在飞上限、反向链、提交标记、点名校验结果、溢出，加 C2 的全部条款）；C4 的 `unreachable!` 与键序断言、`requires`；C6 十四处；12 行加「删 I-8.3」进 `verus-mutations.tsv` 并证红。
2. **测试一簇**（`journal_last_record_flag_of_a_multi_record_publish.rs`、新加的在飞上限用例、施加记录的那几份用例；`crates/mutations.tsv` 追加与第 43、723、738、1039、1296 行只修锚点）：C5 四处。
3. 3-6（E165 装置）、3-7（门禁跟着新 crate 改、规则与定义）照原计划。
4. 第 4 批合入之后开代码轮第二轮，只攻这一轮打中之后的改法本身。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 备料 | 2026-10-01 不受影响：证明今天只管到 M2 的「整次发布」一半，补 spec 归第 4 批，题面写回等第三轮 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-01 不受影响：只被引作 spec 该写进去的口径 |
