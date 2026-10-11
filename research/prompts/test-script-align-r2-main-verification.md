# test-script-align-r2 判决（2026-10-01）

正文 `research/prompts/_test-script-align-r2-body.md`；腿：辩方 `research/prompts/test-script-align-r2-sonnet-output.md`（U3、U4）、攻方 `research/prompts/test-script-align-r2-opus-output.md`（U1、U2），模型目录各自 `-model/`。被复核的第一轮判决 `research/prompts/test-script-align-r1-main-verification.md`。

## 一、核查

- `python3 scripts/cite-check.py <两份报告> --root . --background research/prompts/_test-script-align-r2-background.md`：「✓ 核了 100 处引文，对上 100 处，没判 0 处」。
- 开工快照 `research/prompts/test-script-align-r2-snapshot/` 判决前 `sha256sum -c`：只有 `.claude/kb/decisions-history.md` 对不上，是别的会话（「里程碑3 删除和并发事务」）开工后改的，它给的 diff `/tmp/claude-1000/m3-batch6/kb-changes-for-open-rounds.patch` 只动 D18、D3、D8、D15 四节；两条腿都没引 decisions-history 里那几节，不影响这一轮。
- 攻方 U1-1、U1-2 主 agent 现查：在本仓起 `bash research/prompts/test-script-align-r2-opus-model/rerun.sh <草稿目录>`（它只喂本仓 `.claude/hooks/heavy-test-guard.sh` 的 JSON、看退出码，输出写草稿目录，模型目录 `sha256sum -c` 全对）。通配写法 `bash .claude/gate.d/checker-tier-layer0-replay.s? <根>`、`…/checker-tier-qemu-device-streams.s? <根>`、`bash scripts/mutate.s? crates/mutations.tsv`、`bash scripts/vm-bench*.sh --selftest`、`bash scripts/crash-amplification-two-host?.sh --selftest`、`python3 scripts/compile-then-swap.p? --selftest` 以 implementation-writer 身份全退 0；不带通配的 `checker-tier-layer0-replay.sh`、`vm-bench.sh --selftest`、`cargo test -p singlefs-checker-tier` 退 2。不带 `--item` 的改过正文的 `stage-selftest.sh` 拷贝喂字节相同的轻样本退 0，带 `--item` 或样本不同退 2。（第一次在草稿目录的拷贝上跑时，探针按自己在仓里的位置算仓根、全报 127，那一次不算。）
- 辩方的钩子坏法（bash 层语法错让任何 Bash 退 2、内嵌 python 语法错与缺名字静默放行）没现查，照它模型目录里的探针采；`rerun.sh` 它跑过九行全 0。

## 二、判定与去向

| 格 | 判定 | 去向 |
|---|---|---|
| U1-1 命令词带通配的整条看不见，展开后跑真重活 | 采，最优先（现成的绕过） | 改钩子：判定时把带通配的命令词按命令的当前目录展开，逐个判；有一个展开成重型、按名字判的重型脚本或门禁阶段就拒；展不开（零命中）的照字面判。改法是攻方推的、零轮 |
| U1-2 不带 `--item` 的 stage-selftest.sh 只按名字认 | 采 | 不带 `--item` 的那条路也先判命令词是本仓那一份 stage-selftest.sh。零轮 |
| U2 入口、`mutate.sh`、54 号新判法 | 只抽一次，没打中 | 不作「没打中」用；54 号假 rustc 不核内容记进 C613（自证登记表靠读代码判轻重，脚本改了没人核） |
| U3 第一轮判决逐条 | 辩方判站得住，无一条采错 | 维持；T3-3「列出没判的是哪几类」够不着（驱动行没有类名字段），记进 C605（崩溃放量的复用与剪枝没有关掉再判的对照） 的 ignore 那一半 |
| U3 同一形态在别的臂上 | 采 | C622（门禁的弄坏开关平常跑也认） 射程扩到 `scripts/` 与 `.claude/hooks/`；C604（快档不开特性，崩溃放量与 GPU 对拍编进来零条） 补「`src/lib.rs` 里挂特性的模块」；C613 补「阶段 source 的 `lib/` 与读的登记表不核」与登记表头和「-V 不算」对齐 |
| U3 T1-3 写回照字面做不了 | 采 | D13（验证路线） 已定项 9 与 `.claude/main-agent.md`、C20（罕见布局无人跑） 补三样：「全量」指 54 号 `--full` 与崩溃放量全量两样；「动了崩溃点重放或判器」按暂存改动碰到的路径判（`crates/singlefs-checker/`、`crates/singlefs-checker-tier/src/` 下 `crash*`、`layer0_progress`、`verdict_store`、`crash_identity`、`crash_judge_*`、`crates/singlefs-replay-prefix/`）；「上一次全量」由主 agent 写进当前里程碑记录一行（日期、两样各自的结论），另记一笔欠账做成机器可读的标记。「准入条件」与「异步」改成一句：提交前弹窗请用户跑；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑 |
| U4 欠账逐笔 | 采 | 照辩方报告第四节逐笔补句：C604、C605、C608、C610、C611、C613、C622、C623、C625 各补落成要的那一句；C609 写明前置两件待定、今天落不成；C604、C608 里「用 Vulkan 判没卡」与 C620（GPU 对拍认「没卡」只看 Vulkan 枚举，驱动坏了被当成本次未跑） 冲突处改成指向 C620；C612 的行号改成函数名与判结局的分支名，不写行号 |
| U4 改钩子的流程会拦死会话 | 采 | 新写一份先验后换的脚本（仿 `scripts/compile-then-swap.py`）：在草稿里对改后的钩子跑语法检查、内嵌 python 编译、喂一组固定 JSON 核退出码，全对才换上，换上后再喂一遍，不对就按反向定点替换回滚（不整份覆盖，免得盖掉别的会话的改动）；`.claude/agents/tooling-writer.md` 补「改 `.claude/hooks/` 下的钩子只经这份脚本换上」 |
| C20 尾部两句是主 agent 自己改的 | 记一笔 | `checks-owed.md` 不带历史（`.claude/rules/changelog-format.md`），不补变更史；这两句与本轮的改写一样标「零轮」，第三轮攻 |

这一轮采的改法被攻过零轮；第三轮是最后一轮，只攻第二轮站住的形态与这一轮的改法，第三轮里新冒出来的零轮形态写进交用户表、标「零轮」，不再开第四轮。

## 三、被判文件里点名的定义与共用约束

`.claude/agents/tooling-writer.md`（改钩子只经先验后换脚本）、`.claude/main-agent.md`（「禁止」第一条的三样补全）这一轮判了，改法见上表。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 改 | 2026-10-01 改了：补「全量」的两个所指、「动了崩溃点重放或判器」的路径判法、「上一次全量」的记法，「准入条件」与「异步」合成一句；写回交 `kb-scribe` |
| D13（验证路线） 已定项 15 | 不影响 | 2026-10-01 不受影响：这一轮没碰它的定案 |
