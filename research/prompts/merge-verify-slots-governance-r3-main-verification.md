# merge-verify-slots-governance-r3：判决（2026-11-21）

正文 `research/prompts/_merge-verify-slots-governance-r3-body.md`，开工快照 `research/prompts/merge-verify-slots-governance-r3-snapshot/`。这是这一题的第三轮，也是最后一轮（`.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」最多三轮）。前两轮判决 `research/prompts/merge-verify-slots-governance-r1-main-verification.md`、`research/prompts/merge-verify-slots-governance-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet） | `research/prompts/merge-verify-slots-governance-r3-sonnet-output.md` | 第 2 格三项成立，另报看门狗新格之外有十一种改坏现有断言杀不掉（次序反转、空行过滤最该补）；第 3 格三处成立 |
| 攻方（opus） | `research/prompts/merge-verify-slots-governance-r3-opus-output.md` | 第 1 格打中一处：连同 `SINGLEFS_DATA_ROOT` 一起换，工作根比对就放行，清理删掉真编译根下各槽的编译目录；反过来只换数据根会误拒；第 4 格第二轮不改的四处站得住，「已经量的」十三句十句核到、第 10 句被日志推翻 |

核查（照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做，不派子 agent 去核子 agent」）：

- 倒推开工快照的树：`task-server/task submit --cwd ~/.local/share/singlefs/work/merge-verify-slots-r3-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py merge-verify-slots-governance-r3 --out ~/.local/share/singlefs/work/merge-verify-slots-r3-verdict/merge-verify-slots-governance-r3-tree --repository /home/fy5090/code/singlefs` type=succeeded exit=0：「清单点名 11 份，倒推出 11 份，sha256 对上 11 份」。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/merge-verify-slots-governance-r3-sonnet-output.md research/prompts/merge-verify-slots-governance-r3-opus-output.md --root ~/.local/share/singlefs/work/merge-verify-slots-r3-verdict/merge-verify-slots-governance-r3-tree --background research/prompts/_merge-verify-slots-governance-r3-background.md`：「核了 63 处引文、61 处裸行号，对上 25 处，没判 99 处」，没有对不上的。判决用到的没判几处主 agent 逐处现查：`scripts/gate/merge-and-verify-snapshot-lib.sh:410`–`413`、`.claude/hooks/lib_work_root.py` 第 5 行与第 42 行（数据根先取 `SINGLEFS_DATA_ROOT`、没设才取多机配置的 `SHARED_DATA_ROOT`）、`scripts/agents/agent-watch.py:404`–`406`。
- 两条腿的模型都拷到 `~/.local/share/singlefs/work/merge-verify-slots-r3-verdict/` 下的副本里复跑（拷完逐份 sha256 与报告开头记的相同）：
  - 正推：`task-server/task submit --cwd <空的运行目录> -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs <运行目录> <副本>` type=succeeded exit=0。读数：「MODEL_BASELINE tail_lines=3 real_red=none proposed_red=none」「MODEL_SUMMARY mutants=20 killed-by-real=7 only-proposed-kills=13 survived-everything=0」，与报告推的相同。
  - 攻方：第一次交在判决草稿目录下一层，外层脚本退 2「运行目录……不是工作根下一层的目录」；换到工作根下一层重交 `task-server/task submit --cwd <运行目录> -- bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs <冻结副本> <运行目录> <副本>` type=succeeded exit=0。读数：
    - 「name=slot_root_pairing_summary arm=intact cases=192 ok=152 deleted_paired_targets=16 false_refusal=24 accepted_unpaired=0 other=0 guard_refused=0 prediction_miss=0」；
    - 「name=slot_root_pairing_summary arm=fix-configured-root cases=192 ok=192 deleted_paired_targets=0 false_refusal=0」；
    - 「name=slot_root_pairing_leftover_processes count=0」；
    - `verdict=deleted-paired-targets` 的格都在 `data=other-data` 上（例「arm=intact layout=plain cfg=data data=other-data work=unset slots=3 … idle_target=删了 kept_target=删了 verdict=deleted-paired-targets prediction=match」）。
- 主 agent 另跑：
  - `bash .claude/gate.d/code-tooling.sh --check memory-bounded-parallelism` type=succeeded exit=0。
  - `python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py <仓根>` type=command-failed exit=1「判了 89 个脚本……54 处不合格」，e203 四份模块不在里面，剩的是入口 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 与 `scripts/experiment-apparatus/e203_admission_scan_cost_histories.rs` 各三行，不是这一批改的。
  - 点 `scripts/agents/agent-watch.py` 的开关复跑 type=command-failed exit=1「开关 140 条、对上 138 条」。不符的 `AGENT_WATCH_BREAK=progressnote` 与 `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` 都多红了第二轮新加的「进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行」那一格，文件头没点名。
  - 点 `scripts/gate/merge-and-verify.sh` 的开关复跑交了、等结局。

## 二、判决

1. **第 1 格（第二轮改法第 1 条的修补）**：打中一处，坐实。
   - `lib_work_root.py shared work` 跟着 `SINGLEFS_DATA_ROOT` 走，编译根只读多机配置，两边不同源。
   - 连同 `SINGLEFS_DATA_ROOT` 一起换了工作根（或换了数据根、不设工作根），比对就放行，清理拿那一份槽那一层对着真编译根，删掉成对的编译目录：模型 192 种情形 16 种。
   - 只换数据根、工作根用服务给的那一个时误拒：24 种。
   - 判定不会因此错，代价是那几槽下一趟从头编，或一趟本该占槽的照一趟一份跑。正常的一趟（服务交的真路径、符号链接、NFS、第二台）不误拒；三格自证不碰真的那一份。
   - 「取不到缺省工作根」那一句写「跑不起来」，不是唯一的原因，说法改。
   - 两条新开关会不会连带红新格，等点 `scripts/gate/merge-and-verify.sh` 那一趟开关复跑的结局。
2. **第 2 格（第二轮改法第 2–4 条）**：都成立。
   - `crash-library-totals.sh` 第 280、281 行、文件头与 `task-server/task_server_lib.py:287` 是同一说法。
   - 看门狗新格分得出取头与取尾。正推腿另报的十一种改坏里，次序反转与空行过滤两种现有断言杀不掉、改的人最容易碰到，补断言；其余几种（截断字数、文案、时长）只换措辞，不补。
   - `.claude/agent-common.md:37` 新句与服务那两处是同一件事；服务的 clean 另删分片父任务的测试二进制，新句只写「现场」，不影响「不删编译目录」那个要点。
3. **第 3 格（三处登记行）**：都成立。
4. **第 4 格**：
   - 第二轮写明不改的四处站得住。
   - 「已经量的」第 10 句「点 `scripts/agents/agent-watch.py` 的开关复跑在跑……到 128 / 140 条都对上」不对，坐实：第 122 条 `progressnote` 那时已经不符（见核查），调度记录第 183 行同一句一并更正。
   - 第 1 句「换上了 3 份」只在真换那一条的日志里，dry-run 那一条是「--dry-run：停在草稿」。
   - 第 11、12 句是直接跑的文本工具，日志里核不到。

## 三、改法

第三轮的改法没被再攻过，标「零轮」（`.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」）。都是治理文件，不写进产品代码，不做 A/B。

1. 零轮。`choose_slot` 的缺省工作根改照攻方腿模型 `fix-configured-root` 那一臂：取多机配置里 `SHARED_DATA_ROOT` 下的 work，与编译根同源，不看 `SINGLEFS_DATA_ROOT`。模型那一臂 192 种情形全对（见核查）。同一处把「取不到缺省工作根」那一句的原因说法改实。自证加一格造「连同数据根一起换」的情形，配弄坏开关。
2. 零轮。`scripts/agents/agent-watch.py` 文件头 `AGENT_WATCH_BREAK=progressnote`、`SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` 两条补点名「进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行」。主 agent 已在草稿里改好、出了规格（`~/.local/share/singlefs/work/merge-verify-slots-r2-fixes/header-names/spec.json`），这一份在第三轮开工快照里，判决写完再换。
3. 零轮。`scripts/agents/agent_watch_alerts.py` 那一格补次序与空行的断言（正推腿报告给的反例输入）。
4. 调度记录第 183 行那一句更正。

## 交用户的

没有要用户定的。

## 这一轮被判过的文件

- scripts/gate/merge-and-verify.sh：开关表槽位一族两条新开关与文件头工作根那一条，连带红新格等开关复跑结局
- scripts/gate/merge-and-verify-snapshot-lib.sh：choose_slot 的工作根比对与编译根不同源，换了数据根时放行或误拒
- scripts/gate/merge-and-verify-selftest-lib.sh：三格自证不碰真的工作根与编译根，缺换了数据根那一格
- scripts/crash/crash-library-totals.sh：第 280、281 行与文件头、服务同一说法
- scripts/agents/agent_watch_alerts.py：新格分得出取头与取尾，次序与空行两种改坏杀不掉
- scripts/agents/agent-watch.py：开关表 progressnote 与 new-root-unknown 两条没点名新格
- .claude/agent-common.md：第 37 行 task clean 新句与服务现行说法是同一件事
- .claude/memory-bounded-parallelism.tsv：第 10 行三格与三条开关配对对，门禁那一格绿
- .claude/preflight-exclude：e203 四行锚串与新理由对
- scripts/experiment-apparatus/README.md：第 7 行按 main.rs 列 31 份与 Cargo.toml 对上

## 回看决策

不涉及决策：这一轮判的是合入后验证的槽位、看门狗自证、一份崩溃脚本的失败句、共用约束一句与三张登记表，都是治理文件，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里的任何一条。
