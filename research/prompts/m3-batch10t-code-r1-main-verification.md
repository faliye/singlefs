# 根环实现第 10t–10w 批：代码轮第一轮判决（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 K4 S1 S2 S3 S4 S5 S6 G1 -->

正文 `research/prompts/_m3-batch10t-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10t-code-r1-sonnet-output.md`，sha256 928138d0…0feb，280 行；
- 攻方：`research/prompts/m3-batch10t-code-r1-opus-output.md`，sha256 ccf378b6…27c8，355 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10t-code-r1/tree/` 对 `research/prompts/m3-batch10t-code-r1-snapshot/crates-sha256.txt` 318 份 `sha256sum -c` 全过 |
| 引文 | `cite-check.py` 两份报告合跑，`--root` 冻结副本：`✓ 核了 141 处引文，对上 119 处，没判 22 处`；没判的 22 处都是腿引自己模型目录里的产物，路径不在冻结副本下。换仓根再跑一遍：`✓ 核了 141 处引文，对上 141 处，没判 0 处` |
| 腿在副本上量的数 | 都是副本或腿自己模型上的数，不进 kb，判决只拿来分辨候选、交用户 |

被判的源文件：
- `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`（K1、K2、K4）；
- `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker/src/image.rs`（K3）；
- `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`（钉值）。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 账给的容量大于真能用的 | **两条腿都量到同一形**：外人在进程唯一一次开账之后才来占卡，账不知道。<br>- 攻方：照账借满后超出 20000 MiB（配整卡额度），没配时超出 11846 MiB；<br>- 正推：容量 27440 对真空闲 5000。<br>正推另量一形：卡上空闲不到 2560 时，驱动那一份照样登记上（探针 G）。主 agent 认的零轮设计「偏大不超过两者里小的」，只对它说的那一形成立，不是唯一一形 | **打中，违反「只防 OOM」** |
| K1-2 先登记后建设备 | 两腿：三处都先登记 | 没打中 |
| K1-3 锁内一气做完 | 两腿：没打中。正推并发探针合计 32000，不超过上限 32320 | 没打中 |
| K2-1 白等或判红 | **两条腿量到的形相同**：<br>- 状态数按整张容量压，不按剩下的空压：别的进程借着 1 MiB，大流也要等满（攻方 S3、正推 C：空 5440 却等满）；<br>- 开账时外人在、之后走了，照样判红（攻方 S2、正推 b）；<br>- 两个产品路径进程都只登记了驱动那一份，就互相等满（攻方 S6）；<br>- 产品路径按开账那一刻读的空闲装状态（攻方 S5，读码加算术） | **打中，违反「不压并发」** |
| K2-2 重排 | 两腿：取卡登记或借草稿区那一刻满着的卡，整趟不用，不回头（攻方 S4、S4b，正推探针 D） | **打中** |
| K2-3 僵尸与驱动一份 | 攻方：产品路径在池里每张卡上都登记、起设备，用不用都占到进程退出。正推：没找到算重、算漏的形 | 打中一形（攻方），压并发、账不算错 |
| K3 I-3.14 第五形 | 两腿：五段头部在四处读法逐字节一致；三件事在六到七处读法都做到。正推 CPU 对实现差分 203579 条零不同，弄坏后 25 条不同，说明差分红得出来。I-2.4 不判这条读法站得住。两腿都提 `.claude/kb/invariants.md` I-2.4 行（第 133 行）要补一句第五形 | **站住**（两腿各抽样一次）。I-2.4 行补句进下一份 kb 写回 |
| K4-1 有卡用例 | 两腿：判据是产品自己填的字段，「借了又在判之前还掉」照绿，推的 | 打中（窄） |
| K4-2 扫源码绕法 | 两腿各量到几种：原始 C 字符串 `cr#"…"#` 之后的起卡调用、泛型里的逗号、裸 `#[test]`、非行首的 `cfg(test)` 写法、直接用 wgpu、放在另一份文件里的 `#[cfg(test)] mod x;` | 打中 |
| K4-3 钉值与变异行 | 正推量过：容量「估计 − 驱动」那一项换成 `u64::MAX`，30 条账本单测全绿，变异表里没有对应的行。攻方：GPU 第五形只有 MAC 一条对拍流；S5 那一处、`started_card` 先登记的断言都没有变异行 | 打中 |

## 三、交用户

显存账连着两轮被攻出成串的洞（第 10o 批代码轮 K4 一组，这一轮 K1、K2 一组），每一轮都是零轮设计被攻破。所以下一批之前先交用户定显存账的形态：岔路单 `research/prompts/m3-batch10t-code-r1-forks.md` G1 那一行。

不随 G1 变、下一批照做的：
- K4-1、K4-2、K4-3：补有卡用例的判据、堵扫源码绕法、补变异行与对拍流；
- kb I-2.4 行补第五形一句。

## 四、用户定案

用户 2026-10-07 弹窗定（岔路单 `research/prompts/m3-batch10t-code-r1-forks.md` G1）：「二：去掉跨进程账」。每次借现读空闲、只记本进程借的；两个进程同时读到同一份空闲的竞态，靠运行时认出显存不够（内存包装退 245）报本次未跑兜底。去向：下一批（第 10x 批）照这个形态改 `gpu_memory_ledger.rs`、`gpu_unit_checks.rs`、`crash_amplification.rs`，连同 K4-1、K4-2、K4-3；跨进程账那几格（K1-1 外人后来占卡、K2 一族等满、K2-3 每张卡都登记）随形态一起换掉，新形态零轮，下一轮代码轮攻。

## 回看决策

不涉及决策：这一轮判的是显存账、I-3.14 第五形的实现与测试。I-3.14 那一条决策（D18（块里携带什么信息） 已定项 11）已在上一轮判决的回看里改过，这一轮没有翻任何分项。
