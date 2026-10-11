# 长跑脱离会话入口：工具轮第一轮判决（2026-10-01）

<!-- doc-lint:not-numbers T1 T2 T3 T4 F1 F2 F3 F4 F5 F6 F7 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_tooling-detached-run-r1-body.md` | — |
| 正推腿（Sonnet，T1） | `research/prompts/tooling-detached-run-r1-sonnet-output.md`（210 行），模型目录 `tooling-detached-run-r1-sonnet-model/` | dd4b4481 |
| 攻方腿（Opus，T2–T4） | `research/prompts/tooling-detached-run-r1-opus-output.md`（138 行），模型目录 `tooling-detached-run-r1-opus-model/` | d594df29 |
| 被判文件快照 | `research/prompts/tooling-detached-run-r1-snapshot/files-sha256.txt` | — |

- 写判决前，被判的 7 份文件 `sha256sum -c` 全部 OK，腿跑着的时候没有被改过。
- `cite-check.py` 跑两份报告：「核了 23 处引文，对上 21 处，没判 2 处」。没判的 2 处（`rules-discipline.md:13`、`scripts/agents/watch.sh:11`，都是主仓路径、不在冻结副本里）主 agent 在主仓逐字核过，都对得上。

## 二、逐条判决

| # | 格 | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|---|
| 1 | T3 子 agent 闸 | `source` / `.` 起脚本、改名拷贝，钩子只按文件名认 | 把命令写成钩子输入喂给主仓的 `bash-command-detector.sh`，不执行：clerk 身份下 `bash scripts/hosts/detached-run.sh start …` 退 2；`source …`、`. …`、`cp … /tmp/dr2.sh && bash /tmp/dr2.sh start …` 三种都退 0 | **打中**，必须改（F1） |
| 2 | T3 重型闸 | 改名拷贝里藏 `cargo test --all`，重型闸看不穿 | 主 agent 不带前缀发 `bash /tmp/dr2.sh start --label p -- cargo test --all`，`heavy-test-guard.sh` 退 0（照规矩应拒）；clerk 身份同样退 0 | **打中**，必须改（F1） |
| 3 | T4 自证只验 TERM | 注入「watch 收 HUP 时停单元」的错实现，自证仍然全绿 | 代码里 TERM / INT / HUP 三种信号走同一个收尾函数（`detached-run.sh` 第 352–354 行），代码本身对。主 agent 在草稿副本里带前缀跑 `repro-t4-watch-hup-gap.sh`：注入成功，坏脚本 `--selftest` 末行「✓ detached-run.sh 自证 11 格都对」、退 0，残留单元 0 | **打中**（测试缺口），补自证格（F2） |
| 4 | T4 Ⓓ 只核三个变量 | 带进单元的 5 个变量，Ⓓ 只核了 3 个（漏 `PATH`、`HOME`） | 与报告所列的自证格写法相符，没有另跑 | 采，补格（F2） |
| 5 | T1 systemd 连不上时报错为「已结束」 | `systemctl --user show` 失败时输出为空，被当成「单元不在」；用 shim 复现：watch 退 0、报「观察到的 1 条都结束了」 | `detached_run_unit_state`（第 109 行）把 stderr 丢掉、失败交空串；第 168–169 行空串判成 `vanished` | **打中**（只影响上报，不停脚本），改（F3） |
| 6 | T1 弄坏开关在生产调用里也生效 | `DETACHED_RUN_BREAK=watch-stops-units` 设着时，实际调用的 watch 会停单元；第 548 行还把开关带进单元 | 第 332 行判开关不看是不是在自证里；第 548 行原样转进单元 | 打中，按收严改：开关只在 `--selftest` 的进程里生效（F4） |
| 7 | T1 stop 超过默认停机时限 | 没设 `TimeoutStopSec`，默认 90 秒后 KILL，退出码写不出来 | `grep -c TimeoutStopSec` 0 次 | 采，改（F5） |
| 8 | T1 会话关闭 | `Linger=no` 时，最后一个会话结束会把单元一起停 | 当天主 agent 核过并照用户定案在两台开了 linger（`records/2026-09-30-里程碑三开工第一轮.md` 第五节） | 环境上已解；脚本文件头仍写着 Linger=no 的前提，改成 `start` 检查 Linger、为 no 时打一行警告（F6） |
| 9 | T1 规则 | `agent-common.md` 第 66 行「主 agent 与子 agent 一样拒」与 ③ 的新放行并排；`main-agent.md` 新句没说 `watch.sh --processes` 看不到脱离单元 | 两处原文现查都对得上 | 采，改字面（F7） |
| 10 | T1 推的几条 | 登记被删就看不到单元；start 失败留下登记；同名重起时漏掉旧一次的退出码 | 没真跑 | 并进 F3 一起核；核不出就记欠账，不单开 |
| 11 | T2 | 没打中（`--force` 被剥、`env --` 开头的 `名=值`、`PATH` 里的相对项都是文件头写明的行为；`stop_requested` 是没用到的变量） | — | 只抽了一次样，不支撑「没有错」 |
| 12 | T3 主 agent 夹带 | `--` 之后的 ③⑥ 写法（`bash -c`、`systemd-run`、`kill -- -1`、`setsid -f`、`nohup &`、`pkill -f`）都被拦下；嵌套两层也认得出 | — | 没打中，同样只抽了一次样 |

## 三、改法（交工具实现员，一批做完）

- **F1**：子 agent 闸和重型闸都改成按脚本内容认 `detached-run.sh`，不再按文件名认：
  - 认的依据是签名行（同 `scripts/mutation/broad-mutation.py` 的 G3 做法）；
  - `source` / `.` 起的、改名或符号链接的拷贝都要认得出；
  - 认不出的写法照 G3 那样逐个记进欠账。
  - 做成判红样本：clerk 身份下 `source …`、`. …`、改名拷贝起 start 都要退 2；主 agent 不带前缀经改名拷贝起 `cargo test --all` 要被重型闸拒。
- **F2**：自证补两格「watch 收 HUP 不停单元」「watch 收 INT 不停单元」；Ⓓ 补核 `PATH`、`HOME`。拿攻方的 `repro-t4-watch-hup-gap.sh` 那种 HUP 错实现去喂，必须红。
- **F3**：`systemctl --user show` 失败要单独判成「查不到」，list 报成单独一类。watch 遇到「查不到」不能当作「都结束了」退 0，要继续观察，并在清单里写明。顺手核正推腿报的三种推的情形（登记被删、start 失败留下登记、同名重起漏旧退出码），能修的一起修。
- **F4**：`DETACHED_RUN_BREAK` 只在 `--selftest` 起的进程里生效。在别处设了它，就拒绝执行，退出码与出路写在文件头。
- **F5**：`start` 给单元设 `TimeoutStopSec`，用 `--stop-timeout` 参数给，默认值写在文件头。`stop` 等待退出码的时间不短于它。超时被 KILL 的，登记里写明。
- **F6**：文件头 Linger 那一段改成现状写法；`start` 读 `loginctl show-user -p Linger`，为 no 时 stderr 打一行警告（不拒）。
- **F7**：
  - `agent-common.md` 第 66 行改成「按角色分开判的在条目里写明」这一类字面；
  - `main-agent.md` 那一句补上：脱离单元由 `detached-run.sh watch` 上报，`watch.sh --processes` 看不到它们。

改完之后再攻一轮（第二轮只攻这一批改法与第一轮没攻到的面），用户要求「走三方捶打代码」。

## 四、两台广谱变异

已经在用这一版入口跑。第二节第 1、2、5、6 条都不影响「脚本不随会话停」：它们是闸的口子、测试缺口和上报不准。所以两台照跑，F1–F7 改完验收之后不必重起（入口只在 start 时起作用，在跑的单元不受改动影响）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| 不涉及决策：这一轮判的是研究脚本、钩子与 agent 定义，不碰文件系统的设计决策 | 不影响 | 2026-10-01 不受影响：没有一条决策正文引到这几份文件 |
