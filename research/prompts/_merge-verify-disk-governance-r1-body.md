# 合入后验证压盘：治理文件轮第一轮（merge-verify-disk-governance-r1，2026-11-19）

这一轮判的是为治系统盘压满而改的几份治理文件与仓根工作区清单，改动都已在主工作区。改前的基线在草稿目录 `~/.local/share/singlefs/work/merge-verify-drop-build-step/base/` 下（保留原相对路径）。

| 文件 | 改前基线 sha256 前 16 位 | 这一轮判的改动 |
|---|---|---|
| `scripts/gate/merge-and-verify.sh` | 8b580ee426327c0d | 去掉 build 一步；挑测试目标认包 Cargo.toml 的 `[[test]]` / `[[bin]]` 段；删进度行那一套；撞内存上限重来时删同一任务上一次尝试的快照；收尾只在起这一趟的 shell 里做；执行位 100644→100755 |
| `.claude/main-agent.md` | 2778cba21bb6f4d7 | 第 57 行去掉「build（build 只编补丁改到的包与依赖它们的包）」 |
| `.claude/skills/merge-verify/SKILL.md` | 12f01e7eafac6a1a | 第 22 行去掉「build」 |
| `.claude/hooks/lib_run_scope.py` | d00c941c8f925380 | 只判第 529 行合入后验证的范围说明去掉「build」那一句；这份之后又被 mainagent(A) 换过一批（分片入口套在驱动脚本里那一套），那一批不在这一轮判 |
| `Cargo.toml` | HEAD 那一版 | 加 `[profile.dev] debug = "line-tables-only"` |

起因（主 agent 量的）：合入后验证的 build 那一步用调试构建把改到的包与依赖它们的包的全部目标链一遍，一趟编译目录 158G，其中 326 个调试可执行合计 138.2 GB，抽 5 个读节表 `.debug_*` 占 73%–94%；系统盘此时写 98.7 MB/s、每次写平均等 997 ms、util 84%。用户在弹窗里选「两样都做」：dev 档只留行号表、合入后验证去掉 build。

## 问题

1. 换上的实现做的是不是文件头与两份流程文字说的事（正推腿）。
2. 哪一格会错（攻方腿）：
   - 去掉 build 之后有没有该红而不再红的：链接错、只有调试构建才暴露的编译错、挂特性目标与产品 GPU 包的重编核对改读 clippy 的 Checking 行之后放过了没重编的包。
   - 挑测试目标认 `[[test]]` / `[[bin]]` 段会不会把该退 3 的放成挑了目标：段外的键、缺省路径认错、旧侧倒不掉。
   - 删上一次尝试的快照会不会删错：别的任务、同一任务尝试号不小的、带 KEEP_COPY 留给验证段第 2 步的、还有进程在用的。
   - 收尾守卫（`finish` 认 `BASHPID` 等于 `$$`）会不会让该删的不删：主 shell 自己跑收尾、被 INT / TERM 停时。
   - `[profile.dev] debug = "line-tables-only"` 会不会改掉哪一道判定的结局或复用键（`scripts/gate/admission.py:867` 的 `PROFILE_SETTINGS_LEFT_OUT` 不进键；广谱变异 `scripts/mutation/mutation-prune.py:1049` 取整张 `[profile]`）。

## 实现今天的样子

主 agent 的观测（读的是此刻主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码；`Cargo.toml` 的 `[profile.dev]` 只改调试构建的调试信息量，不动 `crates/` 任何一份源码。
- **去掉 build**：`scripts/gate/merge-and-verify.sh` 文件头写合入后验证做 fmt、clippy、测试、验证包四步；`read_cargo_command`（说明在第 129、779 行）只从 `.claude/singlefs-ai-sop/scripts/check.sh` 现读 fmt、clippy、test 三条；删了开关 skip-build、build-whole-workspace 与只为 build 存在的三格自证。
- **重编核对改读 clippy**：`clippy_checking_as_compiling`（第 2634 行起，文件头第 148 行）把 clippy 日志的「Checking <包> v…」换写成「Compiling」，与测试日志一起交 `scripts/gate/lib_rebuilt_packages.py`；新开关 rebuild-evidence-test-only。
- **挑测试目标认 Cargo.toml 段**：文件头第 86–103 行；只落在 `[[test]]` / `[[bin]]` 段里的改动落到段点名的目标，段外有任何差别、name 缺或重名、旧侧倒不掉、没有段有差别，照旧判不出、退 3。新开关 manifest-sections-refused、manifest-other-sections-unchecked、manifest-default-path-ignored。
- **删上一次尝试的快照**：文件头第 50–52 行；`record_task_attempt`（第 905 行起）在日志目录写 `task-attempt`，`remove_earlier_attempt_snapshots`（第 912 行起）删同一任务号、尝试号更小的日志目录的 `repo/`，删前经 `run_build_directory.py holders` 核有没有进程开着里面的文件。新开关五条（earlier-attempt-*）。任务服务每次尝试交 `TASK_SERVER_TASK` 与 `TASK_SERVER_ATTEMPT`（`task-server/task_server.py:8933`）。
- **删进度行**：任务服务已不读进度行（`task-server/task_server_lib.py` 里 `PROGRESS_LINE` 0 命中），删了打这一行的代码、说明、3 格自证与 8 条 case-progress-* 开关。
- **收尾守卫**：文件头第 38 行起与第 858 行；进度进程是 `run_step` 放后台 fork 出的子 shell，fork 之后、bash 换回缺省 trap 之前收到 TERM，会照继承的 EXIT trap 删这一趟的快照与编译目录。工具员在本机照同样写法复刻，2000 次里 862 次收尾跑在子 shell 里，加守卫后 0 次（复刻脚本在 `~/.local/share/singlefs/work/merge-verify-drop-build-step/batch5/repro/race.sh`）。新开关 finish-in-forked-shell。
- **验过的**（换上之后在主工作区）：自证 type=succeeded exit=0、没有「✗ 自检」；开关 finish-in-forked-shell 只红点名那一格（实际「删了快照 9 行」）；开关 logs-single 只红点名那一格；门禁开关复跑「复跑了 117 条开关（1 份），自证红且点名的格与实际红的格逐格相同 117 条」，再交一趟同样 117 条对上（那一趟复用了前一趟的判定，不算第二次独立观测）。
- **没验的**：`[profile.dev]` 改动之后没有起过真 cargo 调试构建量二进制大小，138.2 GB→约 26.8 GB 是按节表推的（只留 `.debug_line`）。

## 判据与规则

- 合入后验证该做哪几步：`.claude/main-agent.md`「写与验分两段」验证段第 1 步与 `.claude/skills/merge-verify/SKILL.md`。
- 弄坏开关都要红在点名那一格：`.claude/singlefs-ai-sop/rules/show-me-test.md`「每一个弄坏开关都必须会红」。
- 测试镜像与快照收尾删：`.claude/singlefs-ai-sop/rules/command-safety.md`「测试镜像一律放临时目录」。
