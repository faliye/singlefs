# 长跑脱离会话入口：工具轮第二轮判决（2026-10-01）

<!-- doc-lint:not-numbers V1 V2 V3 V4 F1 F4 F5 G1 G2 G3 G4 G5 G6 W1 W2 h4 h5 h7 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_tooling-detached-run-r2-body.md` | — |
| 辩方腿（Sonnet，V1） | `research/prompts/tooling-detached-run-r2-sonnet-output.md`（189 行），模型目录 `tooling-detached-run-r2-sonnet-model/` | 6e8335d9 |
| 攻方腿（Opus，V2–V4） | `research/prompts/tooling-detached-run-r2-opus-output.md`（298 行），模型目录 `tooling-detached-run-r2-opus-model/` | e439761d |
| 被判文件快照 | `research/prompts/tooling-detached-run-r2-snapshot/files-sha256.txt` | — |

- 写判决前，被判的 7 份文件 `sha256sum -c` 全部 OK。
- `cite-check.py` 跑两份报告：「核了 52 处引文，对上 44 处，没判 8 处」。没判的 8 处都是不在冻结副本里的主仓路径，主 agent 逐行在主仓核过，内容与报告所述对得上：`tooling-detached-run-r1-opus-output.md:132`、`code-tooling.sh:245`、`broad-mutation.py:297`、`multi-host-run.sh:67`、`peer-host-lib.sh:189`、`broad-mutation.py:1271`、`preflight.sh:31`、`evidence-discipline.md:122`。
- 攻方的 6 份复现由主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 跑，在草稿副本里跑（副本放 `detached-run.sh` 与 `.claude/{scripts,hooks,singlefs-ai-sop}`，用 git 初始化），日志 `/tmp/claude-1000/-home-fy5090-code-singlefs/8adc3bc8-1d0b-43bd-9a3f-9e63dae04aaf/scratchpad/r2repro/`，收尾残留单元 0。

## 二、逐条判决

| # | 格 | 腿报的 | 主 agent 复跑或现查 | 判 |
|---|---|---|---|---|
| 1 | V2② W1 登记被删 | 单元还在跑时 watch 报「都结束了」 | `repro-w1`：「start 退 0」「watch 退 0；此刻单元 ActiveState=active」「✓ … 观察到的 1 条都结束了」 | **打中**（上报把在跑的报成结束），改（G1） |
| 2 | V2② W2 start 报失败而单元在跑 | list 列「没起成」，watch 报「观察到 0 条」 | `repro-w2`：「start 退 1」「单元 ActiveState=active」「没起成 … 命令一行都没跑」「开跑时就没有还在跑的，观察到 0 条」；触发条件（`systemd-run` 报错而单元照起）是腿推的，复现靠替身造出 | **打中**，改（G1） |
| 3 | V3 `--log` 写不进 | start 报成功而命令没跑 | `repro-v3-log-unwritable`：「start 退 0」「Permission denied」「✓ 起了 …（已经跑完）」「marker：不在」「已结束 … 退出码 1」 | **打中**（假成功），改（G2） |
| 4 | V3 登记目录写不进 / 盘满 | 退出码丢了，list 把原因报成被 KILL | 模型量过机制，盘满本身是推的，主 agent 没复跑 | 采，与 G2 一起改（start 先核登记目录写得进；写不进的结局单列，不报成 KILL） |
| 5 | V2① `--force` 摘法对不上 | 预检摘掉 `--` 之后的 `--force`，共用解析不摘；`-- --force cargo test --all` 主 agent 不带前缀，重型闸放行 | `repro-b1-force-inner`：「start 退 0」「marker：ran」，登记里的命令已不带 `--force`。闸那一半主 agent 喂过攻方的样本，放行与报告相符 | **打中**（闸看的命令与真跑的命令不一致），改（G3）。旧版就有，不是 F1 引进的 |
| 6 | V4 自证的五种错实现 | 按代码推，都放得过 | `repro-v4-selftest-wrong-impls`：a（读不到 Linger 不警告）自证红 1 格；b（默认时限不设进单元）、c（stop 不等）、f（查询检查挪到归档之后）、g（watch 跳过查不到的）都「--selftest 退 0；0 格不对」。b 的后果：单元 `TimeoutStopUSec=1min 30s` 而登记写 180；c 的后果：stop 5 秒就报「被 KILL」、10 秒后 exit=9 与 stop-killed 并存 | a **撤回**；b、c、f、g **打中**，补自证格（G4） |
| 7 | V4 钩子自检 | h4（子 agent 闸不认 install / mv）、h7（认法不跟 `cd`）、h5（重型闸不认 heredoc 写出的拷贝）放得过 | `repro-v4-hook-selftests`：h4、h7 的 `bash-command-detector --selftest` 都退 0；`heavy-test-guard` 的对照组在副本里退 1，照攻方写的作废条款，h5 那一半作废 | h4、h7 **打中**，补格（G4）；h5 未坐实，G4 里补格时一并造 |
| 8 | V2① 认不出清单之外的写法 | `time -p`、`env -S`、`builtin source`、`bash --rcfile`、`bash --`、`function`、`cp X{,.bak}`、`//` 路径；500 层嵌套抛 RecursionError 后放行。同一批也打到重型闸 | 主 agent 没逐条复跑；攻方的钩子输入与输出在 `hook-probes-v2.txt` | 字面打中。都是共用库 `lib_shell_words.py` 原有的盲区，不是这一轮引进的；辩方指出任何按写法认的闸都堵不全（第一轮攻方报告第 132 行同意）。**交用户定范围**（第四节） |
| 9 | V2③ F4 令牌可伪造 | 令牌放在临时登记目录，自证之外拿得到 | 模型量过 | 字面打中，但属于「判别子观测不到」：开关是防误用，不防有意伪造。判据改成「误设的开关不生效、会被拒」，这一条现在成立；不改代码 |
| 10 | V1 第一轮条目 1、2 | 字面够得着、实际危害窄；按摘要认、禁 cp、只按名字三种替代方案同样被打中 | — | F1 已经实现、验收过，代价是共用库多一段认法。**交用户定留不留**（第四节） |
| 11 | V1 同类开关 | `multi-host-run.sh:67`、`mutation-shard-run.sh:69` 的弄坏开关在生产里同样生效，F4 只收了这一份 | 现查第 67 行属实 | 采，记欠账，下一批工具改动处理，这一轮不碰那两份 |
| 12 | V1 其余 | 第一轮条目 3–10 站得住；F1–F7 没有误拒主 agent 的合法用法 | — | 采 |
| 13 | V1 线索 | watch 对「查不到」没有上限；180 秒长于 Bash 前台默认 120 秒；单元被 KILL 时内存包装的 scope 可能活下来；别的会话的自证单元在 list 里是噪声 | 都是推的 | 前两条并进 G5；后两条记欠账 |
| 14 | V2④⑤、V3 其余 | F5、F6 没打中判据；watch 被 SIGKILL、两个 watch 同时跑没打中；1 000 条登记时 list 要 11 秒 | — | 只抽了一次样，不支撑「没有错」；1 000 条登记的耗时不判打中 |

## 三、改法（交工具实现员，一批做完）

- **G1**：
  - watch 按单元名核状态，不只信登记：登记被删、但单元还 active 的，报成「还在跑（登记里没有）」，不算结束。
  - start 遇到 `systemd-run` 退非 0，回读单元状态；单元在跑就记成在跑，并写明 `systemd-run` 的退出码。
- **G2**：start 起单元之前先核 `--log` 与登记目录都写得进，写不进就退 1、不起单元，绝不打「✓ 起了」。单元里写登记失败的结局单列一类，不报成被 KILL。
- **G3**：共用的 start 解析与预检对 `--force` 的处理一致。闸看到的命令要与单元里真跑的命令逐字相同；做不到就让 detached-run 在 `--` 之后完全不经预检摘参数，选哪种写理由。判红样本：`-- --force cargo test --all` 主 agent 不带前缀要被重型闸拒。
- **G4**：
  - 自证补格，b、c、f、g 四种错实现各喂一次必须红，攻方的 `v4_mutations.py` 照搬成开关；
  - 钩子自检补 h4（install / mv 拷贝）、h7（`cd` 之后的相对路径）两格；h5（heredoc 写出的拷贝）在重型闸自检里造一格，对照组先跑绿。
- **G5**：watch 对「查不到」连续若干次（照 `peer-host-lib.sh:189` 连续次数上限的写法，次数你定）后在清单里单列报警，仍不退出、不停单元。`--stop-timeout` 默认值与 stop 的等待写进文件头，并写明前台调用 stop 时 Bash 工具的超时要给够（或改成 stop 只发停机、不在前台等）；二选一，写理由。

改完开第三轮（最终轮）：只攻 G1–G5 的改法与第二轮还开着的格（`.claude/rules/three-way-inference.md`「多轮」：第三轮之后停）。

## 四、交用户

1. F1（按签名行认改名拷贝、`source` / `.`）留不留：已实现、验收过；辩方判它挡的是「子 agent 先被拒、再换写法绕」这一类，危害窄，而且任何按写法认的闸都堵不全。主 agent 倾向留：代价已付，拆掉也要改。
2. 共用库 `lib_shell_words.py` 原有的那批写法盲区（第二节第 8 条，含 500 层嵌套抛异常后放行），修到哪一步：这一批里一起修，还是记欠账另开一批。主 agent 倾向另开一批：它们不是这次引进的，而且这个共用库被所有钩子共用，改动面大，要单独走一轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| 不涉及决策：这一轮判的是研究脚本、钩子与 agent 定义，不碰文件系统的设计决策 | 不影响 | 2026-10-01 不受影响：没有一条决策正文引到这几份文件 |
