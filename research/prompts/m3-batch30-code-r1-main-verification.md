# 删除批第 30 批合入的代码：代码轮第一轮判决（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 M1 M2 M3 M4 F-W F-IT F-L0 G1 G2 G3 G4 O1 O2 O3 S1 S2 S3 S4 S5 S6 X2 X13 X14 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch30-code-r1-body.md`、`_m3-batch30-code-r1-diff.md`、`_m3-batch30-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch30-code-r1-snapshot/`（crates 5 份、kb 2 份）；冻结副本 `~/.local/share/singlefs/work/m3-batch30-code-r1/tree/` | `snapshot-check.sh m3-batch30-code-r1`：7 份里 6 份一致，`crates/mutations.tsv` 对不上。拿冻结副本那一份（sha256 cba4e052…，就是快照）与工作区 diff：差的只有第 1432–1434、1442、1709 行（根环线 10s 换文件路径）与末尾追加的 9 行（10s），共 19 行；两条腿引的变异行（第 2919、3036–3039 行与第 30 批追加的 10 行）都不在里面 |
| 云端正推（Sonnet） | `research/prompts/m3-batch30-code-r1-sonnet-output.md`（272 行，sha256 3d808b0e…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch30-code-r1-opus-output.md`（149 行，sha256 187e2eb6…），模型 `…-opus-model/` | — |
| 引文 | 两份腿报告 | `cite-check.py <两份> --root <冻结副本> --root . --background <背景材料>`：「核了 101 处引文，对上 101 处，没判 0 处」 |
| 主 agent 复跑：攻方 | 攻方模型拷进会话草稿目录 `m3b30-model/`，`SINGLEFS_REPO=<冻结副本> SINGLEFS_TOOLS=<仓根> rerun.sh <空目录> all` 再 `compare` | 原样比对只差两行测试耗时的秒数（攻方的汇总没去掉计时字段），去掉计时字段后「SUMMARY_MATCHES_EXCEPT_TIMING 12 行」；b29 写法下 5 条新坏镜像用例红，b30 9 条全过 |
| 主 agent 复跑：正推 | 正推模型拷进会话草稿目录 `m3b30-sonnet-model/`（拷贝与原目录逐文件 sha256 相同），`SINGLEFS_REPO=<冻结副本> SONNET_WORK=~/.local/share/singlefs/work/m3b30-sonnet-rerun DRIVE_BUDGET_SECONDS=100000 rerun.sh` 再 `compare` | 「same: baseline.txt」「same: mutations.txt」（sha256 da6d935b…、a2382add…）；追加变异原样：`S1 … SURVIVES 169 条跑过 红 0`、`S2 … SURVIVES 169 条跑过 红 0`、`S3 … RED 169 条跑过 红 4`、`S4 … SURVIVES 169 条跑过 红 0`、`S5 … RED 169 条跑过 红 11`、`S6 … RED 169 条跑过 红 6` |
| 读代码 | 冻结副本 `crates/singlefs-checker/src/walk.rs` 第 3159–3167 行与全文件 `locations[0]` 的用法 | 见第二节 O3 那一行 |

被判的 5 份文件（K3），每份一格判到。

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 M1 F-W | 一致：CPU 三处与 GPU 三处判法相同（另有一份头里类标签是类型段那一类、整单元校验和对得上才往下走）；往下走那一支与类型段那一支是同一段代码；没认出别类的条目一份不多读 | — | **采** |
| K1 M2 F-IT | 一致（读码）：GPU 链循环在走过、容器格式、I-1.12 之后判 I-1.2，偏移同 CPU，不判 I-4.2 | — | **采**（GPU 没跑，提交时随对拍） |
| K1 M3 测试尖度 | 一致：判决点名的 X2、X13、X14 与 F-W 五形都钉住；另有缺口 G1–G4 | — | **采**；缺口见下四行 |
| K1 M3 缺口 G1 | 类型段 0 那一支的 F-W 没有测试钉：S1（墓碑、谱系）、S2（inode）把这一支关掉，`checker_known_bad_images` 169 条一条不红 | — | **坐实**（主 agent 复跑同上）。第 31 批 N 已在 GPU 容器树类型段 0 那一处补了对拍流与变异，CPU 那两处仍没钉；进第 32 批补 CPU 一侧坏镜像（断言类型段 0 一支独有的后果：往下走那一份子树里的违例照红）与 S1、S2 两行变异 |
| K1 M3 缺口 G2 | `some_copy_of_the_child_verifies_as_the_class_the_entry_type_names` 里「头里类标签等于类型段那一类」没有测试钉：S4 去掉它 169 条不红；「校验和」那个条件（S3）由 4 条已有陈旧副本用例钉着 | — | **坐实**（复跑同上）。进第 32 批：补「一份在类型段那一类的长度上校验和对得上、头里类标签是同长度的别的类（码 1 对码 3，都是 32 KiB）」一形与 S4 那行变异 |
| K1 M3 缺口 G3 | F-W 五形与 X2、X13、X14 只用层级 1 的根，类型段 0 的孩子是下一层节点、F-W 往下走那一份子树这一形没有测试 | — | **采**（读用例得出）。进第 32 批补层级 2 的一形 |
| K1 M3 缺口 G4 | F-W 形只断言槽数那三条（I-3.1、I-3.11、I-5.1）不红，没断言 I-2.1 一族红不红（读断言得出，没做变异验证） | 攻方探针：F-W 那一形 I-2.1 / I-4.8 / I-7.4 在 b29、b30 都红 | **采**。I-2.1 一族照红是第 30 批 M 报告「要你定的」第 1 条，主 agent 已认、随 C739 写回；进第 32 批把这一条写成断言 |
| K1 M4 变异行 | 一致：追加 10 行成形，CPU 9 行（8 行加第 2919 行）改坏后点名的测试都红；第 2919 行只换了锚点；参数筛得到。另报两件：第 3036 与 3037 行、第 3038 与 3039 行原文、替换文、参数逐字相同，只是点名的测试不同；F-IT 三条对拍流里只有 birth_txg 那一条被变异行点名 | — | **采**。重复的两对行各证一条测试，不删；F-IT 另两条流的变异进第 32 批 |
| K2 M1 F-W CPU（inode / 墓碑 / 谱系，层级 1，盘 0 / 盘 1） | — | **没打中**：b30 比 b29 多判出 F-W 那一份子树里的 I-9.15 违例，不再多红 I-3.1 / I-3.11 / I-5.1，没引入新的漏判或误红 | **记「没打中（一次抽样）」**；主 agent 复跑坐实攻方的探针差 |
| K2 M1 F-W GPU、M2 F-IT GPU、CPU 与 GPU 判得不一样 | 观察 O2：GPU 在 inode 树根上对类型段 0 的 F-W 形记判不了、CPU 给结论（主 agent 已认） | **没打中（读码）** | **记「没打中（一次抽样）」**。第 31 批 N 已为 O2 那一形补对拍流（期望原因字 4、回落 CPU 后同判） |
| 观察 O1 | 类型段 0 的 F-W 形今天有行为：inode 那一形多红 I-1.1、I-1.2、I-3.1、I-3.11、I-5.1、I-7.2，墓碑与谱系那一形多红 I-7.2（原因是推的） | — | **记**：是 C739 后一半「槽数按类型段那一类」的后果，随写回；G1 补的坏镜像钉这几条 |
| 观察 O3 | F-W 之后三棵树「走过」的键取 `child.locations[0]`，读回的可能是位置条目 1 那一份；实例表链那一处改过（`placement_it_was_read_from`）；没造出反例，两种盘序红的集合逐条相同 | — | **记**：冻结副本 `walk.rs` 里「走过」的键几乎都取第一条位置条目（第 1022、1282、1473、2525、2590、2814、3366 行等），只有实例表链第 3159–3167 行改成读回的那一份，不是 F-W 才有的。进 m3-batch31-code-r1 攻方的格 |
| K2 第二次抽样：m3-batch29-code-r1 的 L1 合法状态多红、L1 一份读不出、L2 链长与候选集里外 | — | **没做**：要层 0 崩溃枚举，属 checker 档，交主 agent | **还差第二次抽样**：三格仍是「没打中（一次抽样）」。层 0 快档归提交时；m3-batch31-code-r1 攻方腿再抽一次 |
| K3 | 5 份各一格，都一致：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/mutations.tsv` | — | **采** |

## 三、进下一批的改法（各自验收标准在派发时写）

1. G1：CPU 墓碑、谱系、inode 三棵树类型段 0 那一支的 F-W 各补坏镜像，加 S1、S2 两行变异。
2. G2：补「头里类标签是同长度别类、校验和对得上」一形，加 S4 那行变异。
3. G3：补层级 2 的树里类型段 0、F-W 往下走那一份子树的一形。
4. G4：F-W 五形加 I-2.1 一族的断言。
5. F-IT：另两条对拍流（实例、尾段）各补一行变异。
6. 写回：F-W 读法与 C739 后一半（槽数按类型段那一类）写进 I-9.2、I-9.17、I-9.23；第 30 批 M 报告「要你定的」第 1、2 条（F-W 之后别类那一份按类型段长度读对不上时 I-2.1 一族照红；GPU inode 类型段 0 而有一份按码 2 自证过的条目落判不了）一并写。

F-W、F-IT 都在这一轮被攻过一次（一次抽样没打中），还差第二次抽样。

## 四、交用户

不交：这一轮没有岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-07 不受影响：内部条目形态不改，这一轮判的是 checker 走读认出别类之后往不往下走、实例表链判不判 I-1.2，与测试补没补 |
