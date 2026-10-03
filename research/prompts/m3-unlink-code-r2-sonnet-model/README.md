# m3-unlink-code-r2 云端正推腿的模型与脚本（2026-10-03）

只读冻结副本 `/tmp/claude-1000/m3-unlink-code-r2/tree/`；所有跑动都在草稿副本 `/tmp/claude-1000/m3-unlink-r2-forward/tree/`（`rsync` 自冻结副本），编译目录 `/tmp/claude-1000/m3-unlink-r2-forward/target`。

| 文件 | 做什么 |
|---|---|
| `check_anchors.py` | 核 `crates/mutations.tsv` 每行：文件在、原文恰好命中一次、替换与原文不同、点名的测试函数在对应目标文件里有定义 |
| `probe.py` + `probes/p*.orig/.repl` + `run_probe.sh` | 在草稿副本里改一处（恰好命中一次才改）、跑指定 harness 档测试二进制、还原 |
| `run_base.sh` | 在草稿副本（未改）上跑一个 harness 档测试二进制 |
| `probe_tests_admission_raises.rs.txt` | 追加在草稿副本 `admission_raises_the_floor_before_refusing.rs` 末尾的探针：各次发布的写种类、续做推抬 F 之后设备写错时账与设备写数的差与次序、挂载透明 F 判 |
| `probe_tests_unlink_mount_continuation.rs.txt` | 追加在草稿副本 `unlink_mount_continuation_and_the_emptied_version.rs` 末尾的探针：账与设备写数相等（无抬 F 的形） |
| `probe_campaign_segments.rs.txt` | harness 档复核 checker 档战役用例写死的段形与写数（草稿副本 `tests/probe_campaign_segments.rs`） |
| `kb-quotes.md` | `quote-kb.py` 整抄的 D8 已定项 16 与 D16 「准入」那一行 |

内存包装的并行理由与等待秒数由环境变量给（见 `run_probe.sh`）；`CAP` 变量是包装的上限，峰值表里有这条命令时写 `auto`。
