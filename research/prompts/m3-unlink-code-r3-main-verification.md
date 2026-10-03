# 删除批第 12 批合入的代码：代码轮第三轮判决（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 N7 N8 V1 V2 V3 V4 V5 V6 U4 U8 U13 LR1 LR5 Q2 MU1 MU2 GA GB GC GD GE -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-unlink-code-r3-body.md`、`_m3-unlink-code-r3-diff.md`、`_m3-unlink-code-r3-background.md` | — |
| 开工快照 | `research/prompts/m3-unlink-code-r3-snapshot/`（crates 15 份、kb 3 份）；冻结副本 `/tmp/claude-1000/m3-unlink-code-r3/tree/` | `snapshot-check.sh`：18 份里 2 份对不上，逐份核清，见下面两行 |
| 快照之后被改的 `crates/mutations.tsv` | 与冻结副本 `diff` 的差正好是第 13 批那两处（C511 那一行整行替换、LR5 一行追加） | 腿没有引这份文件里会变的那两行 |
| 快照之后被改的 `.claude/kb/checks-owed.md` 与 D8 | 形式化接入那条线写回改的，逐处旧串新串拷在快照目录 `peer-checks-owed-strings.md`、`peer-c673-writeback-report.md` | 三份腿报告引 `checks-owed.md` 0 处；正推腿引的 D8 第 459、462、464 行（共 5 处），在对方写回之后原样下移 5 行，落到 464、467、469 行，引文没被改 |
| 云端正推（Sonnet） | `research/prompts/m3-unlink-code-r3-sonnet-output.md`（234 行，sha256 7b781171…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus）第一次抽样 | `research/prompts/m3-unlink-code-r3-opus-output.md`（248 行，sha256 5f0faf33…），模型 `…-opus-model/` | 主 agent 复跑，见下一行 |
| 云端攻方（Opus）第二次抽样 | `research/prompts/m3-unlink-code-r3-opus-s2-output.md`（283 行，sha256 4de9af79…），模型 `…-opus-s2-model/` | 没复跑；腿自报在空目录里从头跑通 `rerun.sh`，24 份产物判定行相同 |
| 主 agent 复跑（第一次抽样的模型拷到 `/tmp/claude-1000/m3-unlink-code-r3-main-checks/`） | X 那一份盘面、4 GiB 盘循环 h4，两种实现：START、CUT、INTERSECTION 行与腿的逐行相同。窄盘网格三种实现：汇总行与腿的 `out/sweeps-summary.txt` 第 1–3 行逐字相同 | 腿的 `rerun.sh` 有缺陷：拷过来的 `run.sh` 没有执行位，`sweep-batch.sh` 每一件都退 126。主 agent 补上执行位之后另跑一趟 |
| 引文 | 三份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：正推与第一次抽样「✓ 核了 191 处引文，对上 191 处」（对方写回 D8 之前）；第二次抽样「✓ 核了 19 处引文，对上 19 处」 |

被判的 15 份文件（K3），每份一格判到：
- `crates/mutations.tsv`
- `crates/singlefs-checker-tier/src/crash_injection.rs`
- `crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs`
- `crates/singlefs-core/src/mount.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-harness/src/history.rs`
- `crates/singlefs-harness/src/model_comparison.rs`
- `crates/singlefs-harness/src/model.rs`
- `crates/singlefs-harness/src/mount_transparency.rs`
- `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs`
- `crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs`
- `crates/singlefs-harness/tests/fault_injection_fast_tier.rs`
- `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs`
- `crates/singlefs-harness/tests/random_histories.rs`
- `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs`

正推腿第三节逐份判：一致 10，部分一致 3（`mount.rs`、`transaction.rs`、`random_histories.rs`），读代码一致而跑复核不了 2（两份 checker 档文件），冲突 0。

## 二、逐格判决

| 格 | 正推 | 攻方（两次抽样） | 判决 |
|---|---|---|---|
| N1 丙1 只改卸载那一臂 | 一致：`mount.rs` 第 2258–2259 行，唯一的调用点在第 2765 行；4 条用例与 4 行变异复跑都红 | — | **坐实** |
| N1 第 2251–2255 行文档「引用它们的那几条根都已低于 F_生效」 | 只对回退候选集成立，恢复择根不看 F（`recovery.rs` 第 1260 行，`choose_root` 的入参与择根键里没有 F） | 两次都打中这一句 | **坐实不成立**：恢复择根照 `(checkpoint_txg, 实例代号)` 取最大，单盘幸存时择到那块盘上最新的根，它可以低于系统配置里的 F |
| N1 X 第 1404 行那一份盘面的交集 | 单点算出为空 | 两次抽样都为空（第一次：前缀刀与层 0 枚举 8223 个状态；第二次：同样） | **坐实为空** |
| N1 放开扫 | — | 第一次：窄盘 1900 个起点里 174 个有交集，丙1 读不出 37 对；改之前那一版 0 个、卸载被拒 252；GA 0 个、卸载被拒 0；普通盘循环（单元区 384、256、240 槽与 4 GiB 四档）400 个起点里 1 个有交集，落在 4 GiB 那一档，活文件的 extent 叶与 inode 叶被写掉，只读挂载之后打开文件失败（`MappingStillUnreadable`）。第二次：三族 1684 格，丙1 卸载做成的 1263 格里 74 格有交集；改之前那一版 0 格，卸载被拒 189 格；层 0 枚举三段历史上，丙1 幸存盘字节被改的状态 2051 / 2051 / 3843 个，改之前那一版 0 个；用户看得到的是读文件 `DataUnitChecksumMismatchEverywhere`（2051 个状态）与只读挂载 `ERR:Open`（3075 个状态） | **打中，坐实**：两次独立抽样同向；第一次的窄盘网格、X 盘面与 h4 由主 agent 在副本上复跑，逐字相同 |
| N1 第五节跑前判据的字面 | — | 两次都报：那六条（I-1.1、I-2.1、I-4.8、I-7.2、I-7.4、I-7.6）在卸载一个字都没写之前的单盘镜像上就全红，两种实现的条名分不开 | **判据写错，属于「判别子观测不到」那一形**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」）。判据拿 checker 的条名分两种实现，而在单盘镜像上条名已经饱和。分得开两种实现的观测是：被择中的根引用的单元字节有没有被改、只读挂载与读回成不成。按这个观测，丙1 打中、改之前那一版不中。这是第三轮，不为改判据另开一轮，按打中判，交用户 |
| N1 两块盘都在 | 没跑 | 两次都没打中（第一次：前缀刀全部族、层 0 枚举 5 份 `checker_violated={}`；第二次：9 次枚举 86328 个状态，被改 0 个） | **没打中，两次抽样**：记「没打中」 |
| N1 卸载—再挂循环 | — | 第一次打中（窄盘循环 186 个起点里 13 个，普通盘循环四档 400 个里 1 个、落在 4 GiB 那一档）；第二次没量 | **打中，一次抽样**：主 agent 复跑了 h4 那一份，与腿的逐行相同 |
| N2 V1–V3 写账 | 一致；只比调用数、不比字节；注错点只摆在发布的第一个写；续做那一处没有「抬 F 先写系统配置报错」的用例 | 第一次 74132 个注错点、第二次 6370 个，带账的错都满足「落下的写 − 取号两次 = 写账合计」；份数对不上的全是第二轮判决 H4 那一形 | **没打中，两次抽样**：站住。两处 `writes_of_persisted_publishes` 口径不同、错值不带出处，两次都量到，与 X 报告 Q2 自报相同；今天没有算合计的消费者，记欠账 |
| N2 取界（准入与续做都推） | — | 第一次 1550 个起点 0 个对不上；第二次没打中（推的） | **没打中**，第二次是推的，按一次抽样记 |
| N3 Q-F | `change_count` = 本次 txg，①b 用例仍绿；用户定案选项里的「ctime 跟着改」没做；代码注释「记录里没有 ctime 字段」与 `records.rs`、D8 字段表不符 | — | **坐实收窄**：记录偏移 64 起 atime、mtime、ctime 三格写同一个 `write_time_seconds`（`crates/singlefs-core/src/records.rs` 第 76–78 行；D8（核心索引结构） 字段表第 188 行）。「没有 ctime 字段」那句是主 agent 写进派发提示的，错的；收窄掉「ctime 跟着改」没回问用户。`UserChange::Unlink { inode }` 不带时间（`crates/singlefs-core/src/mounted_session.rs` 第 63 行），挂载续做里的几批没有调用方。交用户 |
| N4 U8 | 不重写根的发布读上一版码 2 头，一致；重写根那一支读形状，条款没写，没有变异钉 | — | **一致**；重写根那一支补一行变异，进下一批 |
| N5 挂载透明 | 一致：界仍按位置取，F 与树根指针都认续做，5 行变异红 | 取界没打中（见 N2 那一行） | **一致** |
| N6 崩溃注入 | 读代码一致；checker 档用例与 7 行变异没跑 | — | **读代码一致**；跑归提交时 checker 档 |
| N7 分批 | 一致 | 两次都没打中：同一个常量 144，同一种「剩下不多于一批就是最后一批」 | **站住** |
| N7 映射树上界会不会低于真节点数 | 推导读得通 | 第一次量过 5497 次发布，0 次低于；第二次推的 | **没打中**，量过的只有一次 |
| N7 上界只涨不落 | — | 第一次：最松到真值的 333 倍，300 段里 8 段单这一项就过了 4 GiB 盘「允许撞单元区墙」的门槛，后果是推的；第二次：小盘上「允许拒」从第一个文件起就恒成立，早就是这样，与 N7 无关（推的） | **打中松紧，后果没量**：记欠账（GE），不进这一批的改法 |
| N8 | 一致；LR1 验收满足，重钉、A3b-15、清单为空、守护变异各复跑红；全表锚点命中一次 | 没攻 | **一致** |

正推腿另报一处：`mount.rs` 第 2234 行与第 2610 行写「两个入口只差这两样」，丙1 之后差三样。**坐实**，进下一批。

## 三、进下一批的改法（各自验收标准在派发时写）

1. 第 2251–2255 行文档（GB）：照用户对丙1 那一问的选择改写。
2. 单盘幸存的读回用例（GC）：卸载那一串第一次空发布的单元写完之后，丢掉住着现行那一版根的那块盘，剩下那块盘只读挂载、整读每个活文件，判与卸载之前同一块盘上读出的相同。取 h2、h4 两形。丙1 下这条要红，改之前那一版与 GA 下要绿（两次抽样各自在原型上量过）。
3. `mount.rs` 第 2234、2610 行的「只差这两样」改成现值。
4. Q-F 那句代码注释：「记录里没有 ctime 字段」照第二节 N3 那一行改；改不改时间，照用户那一问。
5. U8 重写根那一支补一行变异，点名会红的用例。
6. N2 补两样用例：续做那一处「抬 F 先写系统配置报错」；注错点放到发布第一个写之后的位置。
7. 两块盘都在、卸载那一串某次空发布的记录已落而根没落的刀上，再可写挂载之后池级 checker 红 I-7.9（第一次抽样「顺手看到」那一节，改之前那一版同样红）：派调查员先复现、二分，不先改。

## 四、交用户

岔路单 `research/prompts/m3-unlink-code-r3-forks.md`：丙1 单盘幸存那一格怎么改，Q-F 的时间字段改不改。两条都是这一轮新冒出来的零轮形态，第三轮之后不再开轮。

记欠账（写回那一批落表）：
- N2 两处写账口径不同、错值不带出处。
- N7 映射树上界只涨不落（GE）。
- 池级 checker 没有「掉了一块盘」的读法：单盘镜像上卸载之前就红六七条，第二次抽样报 I-7.4 在单盘上是误红。
- LR5 双故障形下 I-3.1 该不该并（第 13 批实现员报告第七节第 1 问）。
- LR5 小世界没罩到的四样。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-03 不受影响：「抬 F 那一串」「正常卸载」「生效」三行的丙1 补句还没写回，等用户选了丙1 那一问再写；恢复择根不看系统配置里的 F，条款没写单盘幸存时 F 高于剩下那块盘上全部根时择什么，交用户 |
| D8（核心索引结构） 已定项 6 | 备料 | 2026-10-03 不受影响：偏移 88 改动计数照用户定案改了实现；偏移 64 起三个时间秒共写一个值，ctime 改不改交用户 |
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-03 不受影响：第 2、5、6、7 条的实现与条款对上（N1–N7 各格）；第 6 条卸载那一句等丙1 那一问 |
