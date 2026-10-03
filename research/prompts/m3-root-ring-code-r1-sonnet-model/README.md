# m3-root-ring-code-r1 正推腿的模型目录（草稿探针与脚本，2026-10-03）

不入 `crates/`；放这里只为主 agent 复跑。基线是冻结副本（`/tmp/claude-1000/m3-root-ring-code-r1/tree/`，主 agent 的 31 份 sha256 在
`research/prompts/m3-root-ring-code-r1-snapshot/crates-sha256.txt`）。所有命令都在冻结副本的拷贝里跑，harness 档，`capped.sh 15`，经内存包装，`--release`。

| 文件 | 用途 |
|---|---|
| `sonnet_probe_k1_k4.rs` | 拷进冻结副本拷贝的 `crates/singlefs-harness/tests/` 当一个新测试目标。含：卸载记号其余位在读者与池级 checker 的口径对照（`probe_marker_bits_core_set_vs_checker_prefix`）、写行之后第一个文件版本记录单落那一形（`probe_first_file_after_a_row_record_only`、`probe_first_file_after_row_wrong_alloc_root_in_record`、`probe_next_mount_after_first_file_after_row_record_only`）、卸载记号取记录还是照抄所选根（`probe_unmount_marker_distinguishes_record_from_chosen_root`，补洞用例草稿）、手算腿用的盘面转储（`probe_dump_records_and_roots_for_hand_decoding`） |
| `sonnet_probe_images_tail.rs` | 追加到 `unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 的拷贝（另存为 `sonnet_probe_images.rs`）末尾：转储默认环新池新建文件盘面的整盘摘要输入流 |
| `scripts/hand_decode.py` | 不经 core、不经 checker：自写 CRC-32C，按 kb 字段偏移手解转储出来的三条记录与三条根槽，重算头校验和、载荷校验和、反向链，七段与同代根逐段比 |
| `scripts/mutant.sh` | 在拷贝上改一行（核锚点）、跑指定的 harness 测试目标、还原；用法写在文件头 |
| `scripts/runchunk.sh`、`scripts/T1-targets.txt` | 全 harness 档分四批跑（每批 ≤ 225 秒）；T1 是十个相关测试目标 |
| `scripts/cl.sh` | 报告里每条引文行号的现查：今天主工作区与冻结副本各 grep -nF 一次 |
| `logs/` | 上面各条的原样输出摘录（PROBE 行、test result 行） |

旧值复现（K5「改之前」）：把 `_m3-root-ring-code-r1-diff.md` 里三块 ```` ````diff ```` 各存一份，对冻结副本拷贝依次 `git apply -R` 第 3、2、1 份；
第 1 份在 `crates/singlefs-core/src/mount.rs` 上两处冲突（第 12 批改过周围），手工还原两行：第 81 行 import 补回 `SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE`（旧文件里它在单行 import 里），
`raise_the_floor_through` 里 `reclaim_floor(new_floor, oldest_valid_root_in_the_window)` 改回 `oldest_valid_root`。之后同一个转储用例得 `test-crc=1733593715`。

补：探针里转储路径写死 `/tmp/claude-1000/m3-root-ring-code-r1-sonnet/dump/`（先 `mkdir -p`）。`scripts/kb_markers_check.py` 比 kb 的 JOURNAL_* 标记与冻结副本常量；
`logs/python-crc-and-sector-diff.txt` 是对 `dump/default-ring-fed-{new,old}.bin` 的 Python 复算与新旧逐扇区比（两份转储由 `sonnet_probe_images_tail.rs` 的用例在冻结副本与反向应用后的树上各跑一次得到）。
`logs/mut-M-C.log`、`logs/mut-M-D.log` 是两个小变异的 cargo 原样输出；M-A、M-F 的全套结果摘要在 `logs/mutants-full-suite.txt`。
