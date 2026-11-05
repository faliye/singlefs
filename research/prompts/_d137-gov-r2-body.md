# d137-gov-r2：一百三十七节那一轮的治理文件改动——第二轮（2026-10-29）

## 一、题面

第一轮判决 `research/prompts/d137-gov-r1-main-verification.md` 打中的一批已经修完，修补各件的报告与只含自己改动的 diff 在 `~/.local/share/singlefs/work/` 下：`d137-r1fix-hooks/`、`d137-r1fix-hooks-2/`、`d137-r1fix-hooks-3/`、`d137-r1fix-hooks-4/`（钩子与共用切词）、`d137-r1fix-dispatch/`（派发闸）、`d137-r1fix-watch/`（看门狗与 core_usage、detached-run 单元归属）、`d137-serial-seconds/`（写串行声明的几处补预计秒数）、`d137-r1fix-defs/`（共用约束与主 agent 定义），各目录的 `result.md` 与 `own.diff`。这一轮是第二轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」的第二轮分工攻：第一轮的修补本身、产生结论的方法、从未被看过的地方。派法：一条 Sonnet 辩方腿复核第一轮判决，一条 Opus 攻方腿攻修补。

用户在这一轮定的口径同第一轮正文 `research/prompts/_d137-gov-r1-body.md`「一、题面」，另加（逐字见调度记录 `records/2026-10-29-一百三十七节那一轮调度记录.md`）：
- verus-proof-check.py 的串行声明补预计秒数（付一次 Verus 重判）；GPU 长命令超过 7200 秒那一截照叫醒，主 agent 看到后查进度。
- E185 装置的串行声明不改（改回去，叫醒了再查）。

被判的文件（快照 `research/prompts/d137-gov-r2-snapshot/files-sha256.txt`）：
- 钩子：`.claude/hooks/lib_shell_words.py`、`heavy-test-guard.sh`、`bash-command-detector.sh`、`lib_heavy_tests.py`、`runner-dispatch-guard.sh`。
- 定义与共用约束：`.claude/agent-common.md`、`.claude/main-agent.md`。
- 研究脚本：`scripts/agents/agent-watch.py`、`scripts/hosts/core_usage.py`、`scripts/hosts/detached-run.sh`、`scripts/hosts/multi-host-run.sh`、`scripts/crash/gpu-quiesce-run.sh`、`scripts/crash/crash-amplification.sh`、`scripts/mutation/crates-mutation-rows.py`、`scripts/gate/test-targets-parallel.py`、`scripts/gate/verus-proof-check.py`。
- 登记表：`.claude/gate.d/break-switch-replay-files.tsv`。

## 二、实现今天的样子（主 agent 的观测，2026-10-29）

- `crates/` 里没有相关实现：`grep -rln 'shell_tokens\|heredoc_feeds_shell\|registered_light_selftest\|claude_session_temporary_root\|declaration_excuses_low' crates/` 零命中（E185 那一处补丁已按用户定案退掉）。方案按钩子、定义与脚本今天的代码来谈。
- 主 agent 现喂修补后的钩子（主 agent 身份、不带前缀，脚本或代码里起 `cargo test --all`）：转发函数包 `BASH_ENV`、`{ bash; } <脚本`、`cp /bin/bash ./mysh && ./mysh -c …` 都退 2；同形无重活 `f(){ bash -c true; }; f` 退 0。
- 主 agent 自己撞上的两条误拒（修补之后，量的）：一条 python heredoc 里 `subprocess.run(["python3","scripts/kb/replace-once.py","crates/mutations.tsv",cur[0]+"\n",…])`（命令词与脚本路径是字面、只有参数带变量）被重型闸拒成「python 代码里 subprocess.run(…) 的命令不是字面的」；一条 `sed -n "$((n-3)),$((n))p" <文件>` 被拒成「sed 脚本解析不下去」。这两条不用再报本身，报同类。
- verus-proof-check.py 改后自证红格集合与改前基线逐项相同（24 格加一行汇总，那 24 格归别的会话暂存的一批，不判）；memory-serial-declared 那一格的正则改成认新写法。
- 别的会话没提交的改动在被判文件里（`run-with-memory-cap.sh` 的峰值口径、`runner-dispatch-guard.sh`、`write-guard.sh`、`agent-watch.py` 里「弄坏开关只在自证里认」、`crates-mutation-rows.py` 等）：只判这一轮的改动。

## 三、格

### R1　复核第一轮判决（辩方腿）
第一轮判决每一条「打中，采」与「采为线索」：打中的东西够不够得着（是不是真漏拒、真违规，还是装置或样本造出来的）、同一个打中是不是同样打中所有替代判法（例：「认不出的一律拒」打中的地方，换成「判不了放行并记检出」是不是也一样打中）；判决写「撑住」的，有没有被修补改坏；第一轮判决本身的方法（主 agent 只现喂了七条、其余采为线索）漏没漏。替被判出局或被改掉的一方辩护：这一轮修补里哪一处是过度修补、把正当命令拒掉的（主 agent 撞上的两条之外的同类）。

### R2　攻修补本身与从未被看过的地方（攻方腿）
- 修补本身：钩子四段修补（H1–H13、A2、已知没认四类）、派发闸、看门狗声明豁免与单元归属、串行声明补秒数、共用约束句子，每一处还能被一种普通写法绕过、或在并发与负载下判错的地方。
- 产生结论的方法：各件报告说「全绿」「回归全拒」「开关只红点名格」的，量的范围够不够撑这句话（例：只跑了受影响的开关、复跑装置抽样）。
- 从未被看过的地方：第一轮两条腿都没看、这一轮修补又碰到的文件与判法（例：看门狗 `unit_owner` 与 detached-run 登记的 owner-session、core_usage 的 `declaration_excuses_low`、派发闸 ⑨ 的分法、检出钩子 ⑬ 读脚本正文那一支）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 辩方腿（Sonnet） | R1 | 复核第一轮判决够不够得着、是不是同样打中替代方案；替被修掉的一方辩护，找过度修补 |
| 攻方腿（Opus） | R2 | 攻修补本身、方法与从未被看过的地方；不重复辩方腿的复核 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写被判文件在主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 喂钩子在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里拷一份钩子喂，cwd 放草稿目录，不在主工作区里改任何文件；喂法与原样输出写进报告。
- 起真 shell 的判法在脚本里用 `setsid --wait` 起每条样本，超时与结束在脚本里杀整组；交回前 `ps` 核自己起的进程一个不剩；不真起会无限递归的样本，只喂钩子。
- 不跑重型测试、不起 cargo、不碰 GPU；自证与探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限，排不上可以带 `ulimit -v` 与 `nice` 直接跑并写明。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
