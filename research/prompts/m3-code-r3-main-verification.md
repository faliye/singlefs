# 里程碑三第一批实现：代码轮第三轮（最终轮）判决（2026-10-01）

<!-- doc-lint:not-numbers W1 W2 W3 J1 J2 J3 J4 J5 L1 L2 L3 RU RGC RG RP EC AC Z0 E1 MOD2 FV -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_m3-code-r3-body.md` | — |
| 正推腿（Sonnet，W1） | `research/prompts/m3-code-r3-sonnet-output.md`（326 行），模型目录 `m3-code-r3-sonnet-model/` | b42f7701 |
| 攻方腿（Opus，W2、W3） | `research/prompts/m3-code-r3-opus-output.md`（366 行），模型目录 `m3-code-r3-opus-model/` | 1da23ced |
| 被判文件快照 | `research/prompts/m3-code-r3-snapshot/crates-sha256.txt`（冻结副本 `/tmp/claude-1000/m3-code-r3/tree/`） | — |

- 冻结副本 17 份被判文件 `sha256sum -c` 全部 OK；攻方模型目录 `SHA256SUMS-logs.txt` 193 份日志全部 OK。
- `cite-check.py`（主仓为根）：「核了 37 处引文，对上 37 处，没判 0 处」。
- 腿跑着时，别的会话把这一批合进了主工作区（`records/2026-09-30-里程碑三开工第一轮.md` 第五节那一条）：17 份里 14 份与冻结副本逐字节相同；`Cargo.lock`、`crates/mutations.tsv`、`crates/singlefs-core/Cargo.toml` 另叠了 Verus 那条线的改动，腿没有引这三份里那部分的行。
- 攻方的量都在冻结副本的拷贝上做，副本上的数不进 kb。

## 二、逐条判决

| # | 格 | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|---|
| 1 | W2-① 第二轮原样六个错实现 | 全被新用例抓到：RG 红 J1 那 1 条，RP 红 J2 那 3 条，EC 红 J3 那 6 条，AC 红 J3 txg 25 那 3 条，Z0 红 core 长度 0 与稀疏盘对拍各 1 条，E1 红 k = 2 那 1 条 | 与报告相符 | 没打中：J1–J5 做到了 |
| 2 | W2-② 稀疏盘那一行 Z0 | 「跑了 1 条，跳过 0 条，跑的都抓到了」 | — | 证红了 |
| 3 | W2-③ RU「缺根那一半只在第一次读不出时才重读」 | entries 31 条、稀疏盘 2 条、core 139 条、harness 全包 90 个二进制都过；根槽有一次读回坏一位时，注入真发生的 146 格原样全做成，RU 下 21 格把健康盘拒成落后支缺根，五个入口加可写挂载都中；违背 D16（发布语义） 已定项 1「读不出或自证不过就重读一次」 | 日志行 `WRONG RU_reread_only_after_a_read_error entries_after_mount_refuse_swapped_or_behind_devices rc=0 test result: ok. 31 passed; 0 failed …`、`… core-lib rc=0 test result: ok. 139 passed …` 逐字在 | **打中**（测试缺口：J2、H1 只注入 `ReadFails`），改（L1） |
| 4 | W2-④ RGC「只在卸载、抬 F、回退那条路上把区域号当盘号」 | 同样全绿（J1 只在会话发布上取了区域 2）；区域 2 且根环转过一圈（txg 26、29、32）时，三个入口做成 10 格，池级 checker 红 I-3.1 | `WRONG RGC_region_number_as_device_in_the_three_other_entries entries_after_mount_refuse_swapped_or_behind_devices rc=0 test result: ok. 31 passed …` 等逐字在 | **打中**（测试缺口），改（L2） |
| 5 | W3 三块盘 | 今天的 mkfs 交三块盘会 panic，两块盘归属不是 0/1/0 就拒，往挂着的池里多交一块盘四个入口都拒；改 mkfs 断言造的三盘副本上，MOD2、FV 两种在两块盘上与对的实现同值，在三块盘上让 txg 8 会话发布做成 | — | 三块盘不在今天的取样域里，MOD2、FV 今天算等价变异，不算 `.claude/rules/mutation-sampling.md` 第三类；记零轮，三块盘开线时补（L3，不在这一轮做） |
| 6 | W1 J1–J5 做到了没有 | 都做到；J3 那一句判决字面写「EC、AC 两种错实现必须红」在 txg 27 上，实情是 AC 只在 txg 25 红（txg 27 的根正落在区域 0 那块盘上），实现员另加的 txg 25 三条是对的 | — | 采：第二轮判决第三节 J3 应读作「EC 在 txg 27 与 25 红，AC 只在 txg 25 红」，冻结的那份不改，以这一条为准 |
| 7 | W1 前两轮判决引的数 | 都找得到；第二轮判决「3 处对不上」今天重跑是 5 处（多的 2 处在 `command-safety.md`，当时的输出复核不了）；背景材料「98 个测试二进制」实为 94 个目标加 4 个文档测试的 98 行 `test result` | — | 采：两处说法以这一条为准，「98 个测试二进制」应读作「98 行 test result（94 个目标加 4 个文档测试）」 |
| 8 | W1 留白 | checker 档 `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` 第 834、1251 行一带还有旧成员名的注释 | — | 采，并进 L1、L2 那一批 |
| 9 | W1 AC 单处写法与原样的差 | 只在「盘 0 落后而 P 的根槽住盘 1」那一格不同 | W2 量过：AC 原样被 txg 25 抓到 | 不改 |

## 三、改法（最终轮，修完不再开第四轮；都被攻过零轮）

- **L1**：J2 的三条与 H1 那两条各加一版 `ReadReturnsCorruptedBytes` 注入（根槽第一次读回坏一位、第二次读得出），断言照常做成；配一行变异「缺根那一半只在第一次读不出时才重读」（RU），证红。
- **L2**：正常卸载、准入抬 F、管理员回退各加 txg 26（区域 2、根环转过一圈）一格；那一槽上一圈留下的根是哪一条，写用例时现核（攻方推的是实例 1、txg 2，日志那一段被截断）。配一行变异「三个入口把区域号当盘号」（RGC），证红。
- 第二节第 8 行那两处旧注释跟新名字。

改在主工作区（这一批已由别的会话合进主工作区）：实现员在副本里改、交补丁，主 agent 用 `scripts/gate/apply-writer-patch.py` 打进主工作区，打完跑合入后验证。

## 四、交用户

| 项 | 状态 |
|---|---|
| L1、L2 | **零轮**：最终轮里新提的改法，按规矩不再为它们开一轮，验收只跑证红与合入后验证 |
| 三块盘（L3） | 今天 mkfs 不收三块盘；开线时补区域 2 落在第三块盘上的一格，记进 C585（会话发布放进发布途中取下的旧快照） 的「怎么拦」 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） 已定项 11 | 支撑 | 2026-10-01 改了：随这一批写回（可写挂载补句、落后支改成改法 C、成员新名字、重读一次） |
| D16（发布语义） 已定项 1 | 支撑 | 2026-10-01 不受影响：L1 补「自证不过也重读」那一半的用例，条款不变 |
