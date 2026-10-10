# 第 20 批代码轮第一轮判决（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 F1 F2 F3 F5 H1 H2 -->

被判的文件：`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crates/singlefs-checker/src/image.rs`，`crates/mutations.tsv` 20-1 追加的 8 行。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b20-r1-body.md`、`_m3-formal-code-b20-r1-diff.md`、`_m3-formal-code-b20-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-b20-r1-snapshot/`（crates 285 份，冻结副本 `/tmp/claude-1000/m3-formal-code-b20-r1/tree/`；kb 3 份） | 冻结副本对 crates 快照 `sha256sum -c` 全过；kb 里 `.claude/kb/decisions/16-发布语义.md` 与 `.claude/kb/checks-owed.md` 在腿跑着时被别的会话改过（D16 那句「树表 0 条那一版上正常卸载一个字节都不写」从第 77 行挪到第 78 行，正推引的是第 78 行），引文对现行文件核 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b20-r1-sonnet-output.md`（sha256 2ac58aeb…），模型 `…-sonnet-model/` | 引文；K3-4 它读了禁读目录里的 20-1 原补丁（派发提示一边点名这份补丁、一边禁读整个目录，是主 agent 写错），这一格作废，主 agent 另核，见第二节 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b20-r1-opus-output.md`（sha256 e15cf76f…），模型 `…-opus-model/` | 主 agent 照它的 `tools/rerun.sh` 在会话草稿 `b20-opus-rerun/` 复跑，见第四节 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本的 crates 加主仓 .claude、research、records 拼的根> --background <背景材料>`：「✓ 核了 111 处引文，对上 111 处，没判 0 处（逐处列在上面）」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 F1 | 站住：所选那一版交 None 时判官只列「复算为假」，两条新断言都红，接对时都绿（判官层量过） | 站住：同一个靶向态正确接线两条成立、交 None 两条都不成立 | **站住**（复跑 `F1 正确接线`、`F1 变异 None` 两行与报告同） |
| K1 F2 与三行接线变异 | 站住：接对 (1,1)，2363 得 (1,0)，2364、2365 得 (0,0) | 站住：同；没点名的「所选那一版交 None」也是 (1,0) | **站住**。checker 档用例与变异没跑，接线层在 harness 档复做量过 |
| K1 第四种接错 | 第三截「所选那一版」交成第一次崩溃那一版，F2 用例分不出（这一例两个镜像只读恢复落同一版），变异表没有这一行 | 复跑输出 `F2 second crash effective_root=Some((InstanceGeneration(1), CheckpointTxg(3))) (first crash Some((InstanceGeneration(1), CheckpointTxg(3))))` 与之相容 | **采**：下一批造一个第一次与二次崩溃恢复落在不同版本的靶向态，补用例与变异；造不出就记账 |
| K1 F5 | 函数一层站住；调用点三个实参没钉 | 单测对点名的接错会红；随机历史里「崩溃恢复抛弃根」那一步 122 次 0 次发写，读故障认定恒是 NoneInjected，F5 修法今天改变不了任何判定，第三截写死 NoneInjected 这种接错也判不出 | **站住，射程坐实为空**（复跑同一行「F5 abandoning-recovery steps observed=122 steps_that_sent_something=0」）。立 C696（崩溃注入的读故障一支在随机历史里从没触发）；调用点实参与「写死 NoneInjected」都挂在它下面，等有一种步在读故障下真写盘才量得到 |
| K1 靶向摆法、薄壳 | 站住 | 站住：根槽 FUA 单独一段、两槽轮换是下一段，摆出来的态与文档逐句相符 | 站住 |
| K2-a 判法与写入口一一对应 | — | 15948 个段边界态加 250 个短续做态：判法为真时写入口一次都没交那两种拒，为假时每次都交其中一种 | 站住（复跑 TALLY 行逐行同） |
| K2-b nlink = 0 那一支 | — | 没打中假发现也没打中漏报：250 个「续做推满仍放不下」盘面上 20-1 的臂被单元区墙拒、不报；20-1 之前那一臂 250 / 250 报 Unexplained（被这次修掉）；崩溃注入真用的配置上这一支 33059 个段边界态里一次都没走到 | 没打中（一次抽样，不算定论）。20-1 实现员交上来的「不先走空删除队列」那一条：攻方量到会话里三种改动在同一盘面上照样报空间不够、盘一个字节不动，这一支在真用的配置上走不到，不另立条款 |
| K2 打中 H1：固定历史用例不核「建出了 inode」 | — | 同一态上「建 0 个 inode」也发成、池级 checker 也绿，盘上 inode 号表 [(2,1),(3,1)] 对 [(2,1)]（量过） | **坐实**（复跑 `K2 fixed arm 建 0 个 inode` 三行同）。改法 a 进下一批：断言发布之后 inode 号多出恰好一个，配变异「这一支建 0 个」 |
| K2 打中 H2：判法里 nlink 那一半没有用例走到 | — | 改成只看 `inode_record` 是不是 Some 时，250 个短续做态上翻回 Unexplained 假发现，全部用例照绿（推的） | **坐实**（复跑 k2a 四个数与 ARM 行同）。改法 b 进下一批：拿一个短续做盘面写一条 checker 档用例，断言单元区墙拒，配变异「判法只看 Some」 |
| K3-1 18-2 两行变异 | 站住：2368 只红「会话关」，2369 只红「F 不动」（量过） | — | 站住 |
| K3-2 `image.rs` 注释、K3-3 wgsl 常量 | 一致；wgsl 零引用 | — | 站住 |
| K3-4 重定基 | 作废（读了禁读文件） | — | 主 agent 另核（会话草稿 `k34/`）：合入前备份打重定基补丁，与冻结副本只差第 5999 行补的一行；备份到冻结副本删的行与 20-1 原补丁逐行相同（各 15 行），加的行 = 原补丁 488 行 + 第 5999 行那一行。**站住** |

## 三、去向

1. 20-1 的改动站住，不回改。
2. 下一批（第 21 批 21-2）：F3；改法 a（固定历史用例核建出一个 inode，变异「建 0 个」）；改法 b（短续做盘面的 checker 档用例，变异「判法只看 Some」）；第四种接错的靶向态与变异（造不出记账）。
3. 同日立 C696。
4. 提交时：20-1 与下一批的 checker 档用例、变异整表。

## 四、复跑核对

主 agent 在会话草稿 `b20-opus-rerun/` 照 `REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b20-r1/tree WORK=<草稿> THREADS=31 CAP=4G bash research/prompts/m3-formal-code-b20-r1-opus-model/tools/rerun.sh` 复跑，退 0：

- `k1.log` 以 F1、F2、F5、K2 起头的行，与攻方 `outputs/k1-run3.log` 排序后逐行相同（含「F5 abandoning-recovery steps observed=122 steps_that_sent_something=0」）。
- `k2a.log` 一趟跑完：「SUMMARY shapes_built=6604 mounts_refused=241 mounts_walked_to_empty=6113 mounts_short=250」，等于攻方 `outputs/k2a-final.log` 三片之和（1221 + 2554 + 2829、140 + 87 + 14、951 + 2368 + 2794、130 + 99 + 21）；ARM 行逐行相同。
- `k2b-map-240.log`、`k2b-map-256.log`、`k2b-map-384.log` 的 TALLY 行与攻方同名日志逐行相同。

核不动：checker 档用例与 6 行 checker 档变异没跑；随机历史层 0 全域（每段 1.09×10⁶ 到 2.8×10¹⁴ 个状态）没跑，攻方只取段边界态；4 GiB 盘带 unlink 的历史没跑。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | 改法 a、b | 攻方 | 「建 0 个」与「只看 Some」两臂量过，改法本身没实现 | 下一批实现员；不用用户定 |

不涉及要用户定的岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-04 不受影响：发布臂与写入口第 4、5 条一一对应（K2-a），第 6 条「会话」的射程不改 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-04 不受影响：18-2 两行变异只补证据 |
