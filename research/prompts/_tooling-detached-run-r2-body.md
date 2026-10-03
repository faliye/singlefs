# 长跑脱离会话入口：工具轮第二轮正文（2026-10-01）

<!-- doc-lint:not-numbers F1 F2 F3 F4 F5 F6 F7 V1 V2 V3 V4 -->

## 一、题面

第一轮判决 `research/prompts/tooling-detached-run-r1-main-verification.md` 打中五处，另采三处，改法 F1–F7 已由工具实现员改完（报告 `/tmp/claude-1000/m3-r1/detached-run-2/report.md`）。用户 2026-10-01 要求「写完后 走三方捶打代码 从这几次的反馈来看 这部分代码质量一般」。这一轮只攻 F1–F7 这批改法本身，以及第一轮没攻到的面；辩方复核第一轮判决。

主 agent 带前缀跑过验收（日志 `/tmp/claude-1000/-home-fy5090-code-singlefs/8adc3bc8-1d0b-43bd-9a3f-9e63dae04aaf/scratchpad/detached-accept2/summary.txt`）：
- 自证末行「✓ detached-run.sh 自证 21 格都对」；16 个 `DETACHED_RUN_BREAK` 开关各退 1。
- 第一轮攻方的 HUP 错实现（`repro-t4-watch-hup-gap.sh`）现在让自证红在「Ⓔ watch 被 HUP 停」那一格。
- `bash-command-detector.sh`、`heavy-test-guard.sh`、`lib_heavy_tests.py`、`lib_shell_words.py` 四份自检退 0。`BASH_COMMAND_DETECTOR_DETACHED_RUN_BY_NAME=1` 下红 6 格，`LIB_HEAVY_TESTS_BREAK=detached-run-by-name-only` 下红 15 格。
- 钩子样本 `cases-f1.json` 20 行全对，两个开关一起开时前 11 行全红。
- 残留单元 0。

## 二、实现今天的样子（主 agent 的观测，2026-10-01）

被判的是脚本、钩子与定义，不是 `crates/`：`crates/` 里没有相关实现（`grep -rln 'systemd-run\|detached' crates/` 零命中）。

腿读冻结副本 `/tmp/claude-1000/tooling-detached-run-r2/tree/`（sha256 在 `research/prompts/tooling-detached-run-r2-snapshot/files-sha256.txt`）。这一轮的改动 = `research/prompts/tooling-detached-run-r2-diff.patch`：拿实现员开工时的备份对现在的文件逐份 diff，1 311 行。备份里的 6 份与第一轮冻结副本逐字节相同，所以 diff 里只有 F1–F7。

逐条落点（行号以冻结副本为准，详见实现员报告第二到第五节）：
- **F1**：`detached-run.sh` 第 2 行加签名行 `# detached-run-script-signature:`。
  - 认法放进共用的 `lib_shell_words.py`：`detached_run_script` 按名字、符号链接、签名行认；`detached_run_start_command` 是两道钩子原先各一份的 start 参数解析，合成了这一份。
  - 子 agent 闸认 `source` / `.` 读进来的、改名拷贝、符号链接，以及同一条命令里先 cp / install / ln / mv 再起的。
  - 重型闸认改名拷贝、符号链接，以及同一条命令里先 cp 或 heredoc 写出再起的。
  - 实现员列了 9 类认不出的写法（报告「F1 认不出的写法」）。
- **F2**：Ⓔ 抽成函数，TERM、INT、HUP 各一格；Ⓓ 补核 `PATH`、`HOME`。
- **F3**：
  - 查不到单元状态的单列「查不到」，watch 不当成结束、接着看；start、stop 查不到时退 1。
  - 登记被删的单元，list 列「登记里没有」；`systemd-run` 失败的记 start-failed，list 列「没起成」；同名重起时 watch 照报旧一次的退出码。
  - 没修的两种：同一秒里同名重起两次；两次轮询之间起了又跑完的短命令，watch 不报。
- **F4**：设了 `DETACHED_RUN_BREAK` 而不在 `--selftest` 起的进程里（临时登记目录的令牌对不上），就拒、退 4。
- **F5**：`--stop-timeout` 默认 180 秒，经 `TimeoutStopSec` 设进单元；stop 读不到退出码就在登记里记 stop-killed。180 秒是推的。
- **F6**：start 读 Linger，为 no 或读不到时警告一行、照起。
- **F7**：`agent-common.md` 第 66 行前提句改成「没另写的一样拒；按角色分开判的在条目里写明」；`main-agent.md` 第 40 行补「脱离单元由 `detached-run.sh watch` 上报，`watch.sh --processes` 看不到它们」。
- 已知事实：两个脱离单元 `singlefs-detached-broad-mutation-local.service`、`singlefs-detached-broad-mutation-peer.service` 是第一轮那一版起的，正在跑；本机与第二台都已 `Linger=yes`。

## 三、格

### V1　复核第一轮判决（云端辩方）
逐条复核第一轮判决第二节那 12 行：打中的判得够不够得着（例如 T3 的口子，第一轮说主 agent 不带前缀经改名拷贝起 `cargo test --all` 重型闸放过，这件事在「主 agent 自己起长活」的实际用法里是不是真的危害）；收严的改法（F1–F7）有没有把合法用法也拒掉（主 agent 在 worktree 里用主仓绝对路径起、`nice` / `cd … &&` 前缀、label 带日期）；F4「设了开关就拒」会不会挡住门禁或别的脚本里合法设着的同名变量；F5 的 180 秒与 `multi-host-run.sh` 收到 TERM 后等第二台至多 30 秒、广谱变异驱动的收尾是不是合得上。替被收严的一方辩护；同样的打法打不打得中所有替代方案。

### V2　改法本身哪一格会错（云端攻方）
① F1 的签名行认法：签名行能不能被伪造到别的脚本里让它被当成 detached-run（误拒）、反过来删掉或改一个字符的拷贝认不出；共用的 `detached_run_start_command` 合并之后，两道钩子原有的判定有没有变（回归）；`lib_shell_words.py` 被别的钩子共用，这次改动有没有影响它们。② F3 的「查不到」：查询失败与单元真不在怎么区分、区分不了的边界；登记被删、start-failed、同名重起三种修法在竞争下对不对。③ F4 的令牌：令牌放在临时登记目录里，自证之外的进程能不能拿到它、伪造它；`--selftest` 起的子进程里开关还生不生效。④ F5：`TimeoutStopSec` 设进去之后 stop 的等待、stop-killed 的记法；旧版起的单元（登记里没有 stop-timeout）怎么报。⑤ F6：读 Linger 失败时的行为。

### V3　第一轮没攻到的面（云端攻方，同一条腿、另成一节）
`list` / `watch` 在登记目录很大、几十条历史登记时的输出与耗时；`--log` 指到已存在的文件、不可写的目录；单元里的命令把 stdout 关掉或写满盘；`watch` 自己被 SIGKILL 之后再起；两个 `watch` 同时跑。

### V4　自证与开关（云端攻方，同一条腿、再另成一节）
对新加的自证格与钩子样本，各写至少一种「看起来对、其实错」的实现（不是现有那 16 个与钩子的新开关），看放不放得过。

## 四、分工

| 腿 | 格 | 立场 | 理由 |
|---|---|---|---|
| 云端辩方（Sonnet） | V1 | 复核第一轮判决、替被收严的一方辩护 | 有前一轮判决可复核（`.claude/rules/three-way-inference.md`「多轮」：至少一轮派辩方） |
| 云端攻方（Opus） | V2、V3、V4 | 攻改法本身与新面，不重复第一轮攻过的角度 | 三节分开报 |

## 五、跑前写死的判据

- V2 造出主 agent 的合法用法被新闸拒（误拒），或子 agent 用一种不在实现员「认不出」清单里的写法起得出单元 ⇒ 打中，改。
- V2 造出 F3 之后 watch 仍把在跑的报成结束、或把结束的报成在跑 ⇒ 打中，改。
- V2 造出 F4 的令牌在自证之外拿得到，或合法的同名变量被误拒 ⇒ 打中，改。
- V1 判出第一轮某条打中是够不着的 ⇒ 那条的改法交用户定留不留。
- V4 写出一种错实现、自证与钩子自检全绿 ⇒ 打中，补格。
- 一条腿报「没打中」的格只抽了一次样，不支撑否定结论。

交付：腿的报告写 `research/prompts/tooling-detached-run-r2-<模型>-output.md`，分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；攻方的复现脚本、钩子输入样本与输出放 `research/prompts/tooling-detached-run-r2-opus-model/`。草稿放腿自己的目录 `/tmp/claude-1000/<腿名>/`。引规则写规则文件名加那份文件自己的行号。不在主仓里改任何文件。钩子对子 agent 拒 `detached-run.sh start` / `stop` 与各份 `--selftest`：要真起单元的写成复现脚本交主 agent 跑；钩子的判定喂 JSON 输入在副本里判，不执行。
