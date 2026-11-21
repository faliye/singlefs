# 合入后验证压盘治理轮 · 正推腿报告（2026-11-19）

被判对象与判据见 `research/prompts/_merge-verify-disk-governance-r1-body.md`；这一条腿只做正推：从文件头与 `crates/`、`scripts/` 今天的实现推出「应该是什么」，再与被判对象逐格比。

## 先交代

- 没写模型：这一轮的格都能靠读代码、`grep`、只读 `git`、读 ELF 节表核；没有模型目录，也就没有复跑脚本与 sha256。要起项目代码才能核的格写在「要主 agent 跑的命令」。
- 被判六份的哈希：`scripts/gate/merge-and-verify.sh`、`.claude/main-agent.md`、`.claude/skills/merge-verify/SKILL.md`、`.claude/hooks/lib_run_scope.py`、`Cargo.toml` 与背景材料本身，此刻都与 `research/prompts/merge-verify-disk-governance-r1-snapshot/judged-sha256.txt` 登记的一致（`sha256sum` 前 16 位比过：8b580ee 那一份基线已换成 c6badd7bdfb55050）。改前基线与此刻的 `diff` 行数：脚本 900 行、`main-agent.md` 16 行、`SKILL.md` 4 行、`lib_run_scope.py` 181 行（后者含别的会话的分片入口那一批，不判）。
- 披露：核「别处还有没有残留的 build 说法」时，两条 `grep` 没排除 `records/`，输出里带出了 `records/2026-11-18-合入后验证压盘.md` 的几行（标题、用户定案、一条交回摘要）。那份在禁读清单里；这几行没进任何结论，之后的 `grep` 都加了 `--exclude-dir=records`。主 agent 要不要把这一条当作污染，自己判。
- `crates/` 下 `git diff HEAD --stat` 显示 75 个文件有改动（别的会话的），分辨不出哪些是这一轮的；背景材料说这一轮没改 `crates/`，我只核了被判六份没碰它，没核别的会话的改动。
- 行号都是此刻主工作区里 `grep -nF` 命中的那一行；文件在别的会话手里还会动，主 agent 复核时以 `sha256sum` 对 `judged-sha256.txt` 为准。

## 结论先说（打中 5 处）

| 打中 | 在哪 | 性质 |
|---|---|---|
| 链接残留 | `.claude/agents/implementation-writer.md:32`、`.claude/rules/implementation-workflow.md:60` 写「链接留给合入后验证」；脚本现在不跑 build | 文字与实现不符（读出来的） |
| 注释前提残留 | `Cargo.toml:30` 注释的「合入后验证一趟要链几百个」 | 文字与实现不符（读出来的） |
| 广谱复用键 | `scripts/mutation/mutation-prune.py:1049` 取整张 `[profile]` 进全局键，`Cargo.toml:31-32` 新加的 `[profile.dev]` 会让广谱变异的复用键整体变一次 | 实现行为（算出两个哈希不同） |
| 缺省路径目录形 | 文件头第 99 行认 `tests/<名>/main.rs`、`src/bin/<名>/main.rs` 为缺省根文件，`scripts/gate/rerun-failed-tests.py:250`、`:267` 的目标表不认 | 推的（读代码），偏安全：退 3、不放过；探针待主 agent 跑 |
| 判据措辞 | 文件头第 68、136 行「同一套判据」「用户要的是同一套判据」，而 `.claude/singlefs-ai-sop/scripts/check.sh:86` 还跑 `cargo build --all-targets`、文件头第 131 行明说不读它 | 文字；行为上三步（fmt、clippy、test）确实与 check.sh 一致 |

背景材料「问题」第 1 条的结论：六份里脚本、`main-agent.md` 第 57 行、`SKILL.md` 第 22 行、`lib_run_scope.py` 第 529 行各自「成立」；`Cargo.toml` 的 profile 改动「成立」、它的注释「不成立」一句（上表第二行）；背景材料自己有 2 个行号漂移，见该节。
三条判据：合入后验证的步「成立」；开关表点名「成立」（新旧各 10 条逐条对上）；快照与编译目录收尾删「成立，两处与规则字面有差」（见 2.3 节）。

## 1. 逐份核：换上的实现做的是不是文件头与两份流程文字说的事

### 1.1 `scripts/gate/merge-and-verify.sh`

逐条对：文件头怎么说 → 代码做了什么 → 判。

| 文件头说的 | 代码里 | 判 |
|---|---|---|
| 第 7 行「合入后验证做 fmt、clippy、测试、验证包四步，不跑 cargo build」 | `cargo_command_for`（第 2610 行）现在只被 clippy 用（第 2630 行），fmt 命令在第 2749 行，test 按包、按目标另拼，没有 build 命令；`grep -nE 'skip-build\|build-whole-workspace\|CARGO_BUILD\|build_packages\|build-packages\|build\.log\|build 本次\|cargo build'` 全文件只命中第 7、131 行两处说明 | 成立 |
| 第 131 行「check.sh 里 cargo build 那一条不读、不要求恰好一条（这里没有 build 一步）」 | `read_check_sh`（第 1132 行起）只读 `fmt`、`clippy`、`test` 三次 `read_cargo_command`；`CARGO_BUILD_WORDS` 与 `build-packages` 在代码里 0 命中 | 成立 |
| 第 147–148 行「编完核 test 与 clippy 两趟的编译输出……clippy 打的是「Checking <包名> v…」，核对时当同一种行」 | 第 2825 行每个 clippy 组合不论红绿都把日志追进 `clippy_logs`；第 2852–2854 行 `clippy_checking_as_compiling` 把行首 `Checking` 换成 `Compiling`（第 2639 行的 `sed -E`）写进 `clippy-checking-as-compiling.log`，与 `test.log` 一起交 `lib_rebuilt_packages.py check`；`lib_rebuilt_packages.py:63` 的 `COMPILING_LINE` 认行首空白加 `Compiling <名> v…` | 成立 |
| 第 148 行「测试那一步不编的包……靠 clippy 那一趟的行证明这一趟重编了」 | 没有 `test.log`（测试本次未跑）时 `compile_logs` 只剩 clippy 那份（第 2849–2860 行）；`rebuild-evidence-test-only` 开关只读 test | 成立（依赖「clippy 不带 -p、打工作区每个成员」：`clippy_command_for`（第 2618 行起）只把 check.sh 读来的词与特性、lint 表拼上，没有 `-p`；产品 GPU 包是工作区成员，`Cargo.toml:15`） |
| 第 98–103 行「包的 Cargo.toml：改动只落在 [[test]] / [[bin]] 段里的……」 | 见下一节 | 成立，一处缺省路径目录形不一致（打中「缺省路径目录形」） |
| 第 38–40 行「收尾只在起这一趟的那个 shell 里做（finish 认 BASHPID 等于 $$）」 | 第 858 行 `if [[ "$BASHPID" != "$$" && … ]]; then trap - EXIT; exit "$rc"; fi`，在 `finish` 里取到 `rc=$?` 之后、任何删除之前（第 855–860 行）；EXIT、INT、TERM 的 trap 在第 5021–5023 行一次装好 | 成立 |
| 第 48–55 行「上一次尝试留下的快照」 | 见下一节 | 成立 |
| 第 139–143 行「进度行……每 PROGRESS_INTERVAL（300）秒打一行」 | `progress_ticker`（第 807 行）仍在、`run_step` 仍起它（第 1303 行）。这一轮删的是另一样：打 `TASK_SERVER_PROGRESS done=… total=…` 的机读进度行（`case_progress_*`、3 格自证、8 条开关）。`task_server_lib.py` 与 `task_server.py` 里 `PROGRESS_LINE`、`TASK_SERVER_PROGRESS` 各 0 命中（`grep -c` 四个数都是 0），服务不再读；服务判卡住用 `task_server.py:799` 的 `STALL_SECONDS = 600.0`（日志满 10 分钟没涨且 CPU 几乎没涨），第 139 行说的「满 10 分钟没涨」成立 | 成立（背景材料「删进度行」指机读那一套，人看的 5 分钟进度行没删） |

**挑测试目标认 `[[test]]` / `[[bin]]` 段**（第 98–103 行 对 第 1761–1819、2030–2061、2096–2102 行）：

- 新旧两侧怎么来：`manifest_old_text`（第 1770 行）在 `tempfile.mkdtemp` 建的临时 git 仓里放树里此刻那一份，`reversed(patch_directories)` 逐份 `git apply --reverse --include=<路径>`；`mkdtemp` 默认落 `$TMPDIR` 否则 `/tmp`，与第 101 行说的「${TMPDIR:-/tmp}」同；倒不掉交 `(None, 原因)`；`finally` 里 `shutil.rmtree`。成立。
- 比什么：`manifest_target_changes`（第 1796 行）对 `test`、`bin` 之外的任何键 `old.get(key) != new.get(key)` 就交原因（第 1801–1804 行）；段名缺、非字符串、重名交原因（第 1809–1812 行）；没有段有差别交原因（第 1817 行）。第 102–103 行列的三类原因与这三处一一对上。成立。
- 落到哪：`land_manifest_changes`（第 2030 行）对新加、改了、删掉的段，新旧两侧的根文件都走 `land_on_targets`；要特性的由 `land_on_targets` 里的 `required_entry` 记「要特性」；落不到时 `after is None or after.get("test") is False` 报一句不跑，别的整份判不出（第 2056–2060 行）。与第 100 行逐句对上。成立。
- 缺省路径：`default_root_paths`（第 1761 行）候选是 `["src/main.rs"（[[bin]] 且名字与包名同）, "<tests 或 src/bin>/<名>.rs", "<…>/<名>/main.rs"]`（第 1765 行），在树里的都交、都不在交第一个；第 99 行「（[[test]] 是 tests/<名>.rs、tests/<名>/main.rs；[[bin]] 是 src/main.rs……）」与它一致。
- **打中「缺省路径目录形」**：第 99 行与代码都认 `<名>/main.rs` 形，但 `land_on_targets` 最后拿根文件去 `rerun-failed-tests.py` 的 `test_targets_of_package` 给的表里找。那一份第 250 行的缺省根文件只有 `src/main.rs` 与 `<目录>/<名>.rs`，第 267 行 `if relative not in declared and name not in declared` 又把「段已声明名字」的自动发现挡掉；一个 `[[test]] name = "extra"`（不写 path）而根文件只在 `tests/extra/main.rs` 时，表里没有它。我顺着 `roots_of_path` → `subdirectory_roots` 读下来：落不到、`landed` 为假、`after` 不是 None 也不是 `test = false`，于是整份判不出退 3，原因句写成「根文件 tests/extra/main.rs 不是 cargo test 会跑的目标」（事实上 cargo 会跑它）。方向偏安全（退 3、不跑、不放过），所以不是该退 3 的放成了挑目标；但第 99 行说「认」而实际退 3。自证 6 格里没有这一形（`tool` 用的是 `src/bin/tool.rs`）。这是读出来的，探针在草稿目录，待主 agent 跑（命令见「要主 agent 跑的命令」第一条）。
  - 什么现象会推翻：探针打出 `tests/extra/main.rs 在目标表里：True`，或 `test_targets_of_package` 的第 250–268 行另有一处把声明过名字的 `<名>/main.rs` 加进来。

**删上一次尝试的快照**（第 48–55 行 对 第 905–950、5021–5026 行）：

- 记：`record_task_attempt`（第 906 行）要 `TASK_SERVER_TASK` 非空且 `TASK_SERVER_ATTEMPT` 匹配 `^[1-9][0-9]*$`（第 908 行）才写 `<日志目录>/task-attempt` 一行「任务号 尝试号」；手跑没有任务号，不写。对第 50 行。成立。
- 找：`remove_earlier_attempt_snapshots`（第 916 行）要任务号有、尝试号匹配、`> 1`（第 918–919 行）；在 `dirname "$current"` 同一层的 `*/task-attempt` 里（第 921 行）跳过当前目录、读不出正整数尝试号的、任务号不等的；尝试号 `< 当前` 才继续（第 930 行）；`repo/` 是目录且不是链接才删。对第 51–52 行「别的任务号、尝试号不小于这一次的、没有 task-attempt 的日志目录不碰」。成立；带 `MERGE_AND_VERIFY_KEEP_COPY=1` 留的也删，代码里不看这个变量，对第 52 行「带……留下的也删」。
- 核：删之前 `python3 "$RUN_BUILD_DIRECTORY" holders "$snapshot"`，退 3 打「没删……本机有进程开着里面的文件」、别的非 0 打「核不成」，都 `continue`（第 935–942 行）；`run_build_directory.py:18` 文档与 `EXIT_HOLDERS = 3`（第 27 行）对得上第 53 行「退 3 是有，别的非 0 是核不成」。第 53 行自己写明「核的只是本机的进程」，实现也只核本机。成立。
- 删：`rm -rf -- "${snapshot:?}"`（第 945 行）后回读，仍在打 `!`、不在打「删了上一次尝试留下的快照 …（共 …）」；函数末 `return 0`，不改退出码。对第 54–55 行。成立。
- 时点：第 5021–5026 行：`trap finish EXIT` 之后、`record_task_attempt`、`remove_earlier_attempt_snapshots`，再往下才是指纹与 `take_snapshot`；`mkdir -p "$logs"` 在前。对第 49 行「建好日志目录之后、拷快照之前」。成立。
- 服务一侧：`task_server.py:9185`（`number = attempt_count + 1`，从 1 起）与 `:9196`（`environment.update(TASK_SERVER_TASK=…, TASK_SERVER_ATTEMPT=…)`）。背景材料写 `task_server.py:8933`，此刻那一行在 9196（该文件此刻在别的会话手里，漂了）；变量名与语义对。成立。

**1.1 结论**：成立；两处不一致已记（缺省路径目录形、文件头第 68、136 行的「同一套判据」，见 2.4 节）。
推翻它的现象：自证某格红（待主 agent 复跑，背景材料说过一遍绿，我没复跑）、或上面任一行号的代码在复核时与文件头对不上。

### 1.2 `.claude/main-agent.md` 第 57 行

第 57 行原文（`awk 'NR==57'` 取）与文件头对：

| 第 57 行说的 | 文件头 / 代码 | 判 |
|---|---|---|
| 「在主工作区的快照上打这一批全部补丁」 | 文件头第 29、35 行：开跑那一刻拷快照、`apply-writer-patch.py --root <快照>` 打；拷快照在代码第 5055–5056 行，打补丁在第 2696 行（`--root "$tree"`，`tree` 此时是快照） | 成立 |
| 「另跑 fmt、clippy」（没有 build） | 第 7 行；代码第 2749、2630 行；无 build 命令 | 成立 |
| 「只跑补丁改到的测试目标与变异行点名的测试目标（不跑整个包，也不带上依赖它们的包）」 | 第 5–8、69–70、82–83 行 | 成立 |
| 「逐个写成 `--item <包>:<目标根文件>`（不带 `--item` 的提交时被服务拒）」 | 第 16 行与第 279–280 行 | 成立 |
| 「主工作区一个字节不写」 | 第 28 行 | 成立 |
| 「退 3、退 2、退 75」 | 第 225–231 行退出码表 | 成立 |
| 「编一次」 | 脚本里没有单独的编译一步：测试那一步先 `--no-run` 编一次（第 77–78 行），clippy 另是 check 的一趟；两样共用一个 `CARGO_TARGET_DIR`。去掉 build 之前这三个字就是这个意思 | 推不出：字面像有独立编译步；不构成与实现冲突 |
| 没提「验证包那一步」「重编核对」 | 第 5–6、236 行有；`SKILL.md` 第 22 行提了验证包那一步 | 去 build 之前就没提；不属于这一轮的改动 |

结论：成立。推翻：脚本又加回 build 或换掉 clippy，而第 57 行没跟。

### 1.3 `.claude/skills/merge-verify/SKILL.md` 第 22 行

「脚本编译之后另跑 fmt、clippy，再跑验证包那一步（逐项交门禁 `crash-point-replay`、不起整格）；快照留在它日志目录的 `repo/`。」对文件头：fmt、clippy（第 7 行）；验证包那一步逐项交 `crash-point-replay`、不起整格（第 236–240、246 行）；快照在 `<日志目录>/repo`（第 29 行，代码第 5055 行）。「没带 `env MERGE_AND_VERIFY_KEEP_COPY=1` 的那一趟不留快照、不打合进主工作区的命令，带上重交」对第 168–170 行与代码里 `merge_in_commands` 的分支（第 41–44 行说明）。成立。
没提的：上一次尝试的快照由脚本自己删（第 48–55 行）。同一份第 23 行写的是主 agent 手删「被下一趟替掉的那一趟」，两件不冲突（前者只管同一任务号的重试，后者管不同任务）。

### 1.4 `.claude/hooks/lib_run_scope.py` 第 529 行

「合入后验证点名的项 …（测试那一步只跑点名的测试目标；打补丁、fmt、clippy 照旧，验证包那一步只交点名的验证包目标里的）」对第 15 行「fmt、clippy、重编核对照旧，验证包那一步只交点名的验证包目标里的」：成立，没提重编核对（改前也没提）。全文件 `grep -n build` 只命中 `build_selftest_tree` 等与合入后验证无关的名字（第 1896、2044、2126、2165、2182 行）；第 2044 行「cargo build 不要声明」是别的格。
该文件另有 181 行 diff（分片入口套在驱动脚本里那一批），背景材料说不在这一轮判，我没核。

### 1.5 `Cargo.toml`

| 说的 | 核到的 | 判 |
|---|---|---|
| 第 31–32 行 `[profile.dev]` `debug = "line-tables-only"` | `tomllib` 读出 `profile` 是 `{'release': {…}, 'dev': {'debug': 'line-tables-only'}}`，`grep -c '^\[profile.dev\]'` 为 1（没有重复表）；同一取值第 25 行 `[profile.release]` 已在用，工具链 `cargo 1.98.0` 认；`build_toolchain.rs:157` 说 `test` 继承 dev，测试二进制随它 | 成立 |
| 第 30 行「完整调试信息占每个测试二进制的 73%–94%」 | 我量了本机仅有的一个调试可执行 `target/debug/deps/unit_graph-3aec26e79f64e211`（`unit-graph` 的 bin，不是测试二进制）：总 52,291,768 字节，`readelf -S -W` 的 `.debug_*` 八个节合计 40,205,330 字节，占 76.9%（量的，一个样本）；`.debug_line` 3,195,555 字节，占 6.1%。落在 73%–94% 里；别的四个样本的读数我没有（转述） | 一个样本支持，其余转述 |
| 第 30 行「合入后验证一趟要链几百个」 | 合入后验证现在只起补丁点名的测试目标（第 69–70 行），clippy 只做类型检查不链接，没有 build（第 7 行）；一趟链几百个的是改前的 build（这一轮删的）。留下的链几百个的是提交时 `.claude/singlefs-ai-sop/scripts/check.sh:86` 的 `cargo build --all-targets` | 不成立（打中「注释前提残留」）：这句说的是改前的合入后验证 |
| 背景材料「138.2 GB→约 26.8 GB 按节表推（只留 `.debug_line`）」 | 我这一个样本：只留 `.debug_line` 是 (52,291,768−40,205,330+3,195,555)/52,291,768 = 29.2%，背景是 19.4%。`line-tables-only` 仍会写编译单元与字符串的小节，std 预编译库自带的调试信息也不受 `[profile.dev]` 管；所以 26.8 GB 更像下界 | 推的，不当预期值；真 cargo 调试构建量大小没做（待主 agent 跑，命令见下） |

**会不会改掉哪道判定的结局或复用键**（正推出处）：

- 进键的 profile 设置：`build_toolchain.rs:21` 的 `PROFILE_SETTING_KEYS` 与 `:37` 的 `PROFILE_SETTINGS_READ` 都没有 `debug`；`crates/singlefs-verification/build.rs:127` 与 `crash_identity.rs:1132` 经 `effective_profile_lines` 读根清单，故验证包判器组键不受 `[profile.dev] debug` 影响。`scripts/gate/admission.py:868` 的 `PROFILE_SETTINGS_LEFT_OUT` 与 `scripts/mutation/lib_mutation_units.py:103` 的 `PROFILE_SETTINGS_NOT_AFFECTING_OUTPUT` 都是 `("debug", "incremental")`，文件头第 50 行写「编译档里的 debug、incremental 两项三处都不进」。三处不变。
- 没有读 DWARF 的代码：`grep -rlE 'addr2line|gimli|debuginfo|\.debug_|DW_TAG|std::backtrace|Backtrace'` 在 `crates`、`scripts`、`.claude/gate.d` 里的命中是 `Formatter::debug_struct`（`crates/singlefs-mount-runtime/src/lib.rs`）与变异脚本的 `-Cdebuginfo` 样本串，没有读调试节的；测试结局不依赖调试信息量。
- **打中「广谱复用键」**：`scripts/mutation/mutation-prune.py:1049` `lines.append(f"profile {canonical(parsed.get('profile', {}))}")` 把根清单整张 `[profile]` 解析后放进构建环境行，没有按 `debug` 过滤；这些行经 `global_lines`（第 1055 行起）进每条判定的键，`scripts/mutation/broad-mutation.py:1419` 取它。HEAD 与工作区的这一行我各算了一遍（`tomllib` 读，`json` 规范化后取 sha256 前 16 位）：`657baa2db7b25363` 对 `957ed70c2573e39b`，不等。结论：这次加 `[profile.dev]` 会让广谱变异已存的复用键整体对不上一次（只多判一遍，不改结局）；以后 `debug` 再改一次又对不上。另外三处读者都明说 `debug` 不进键，这一处没跟，是口径不一致。我的规范化写法与它的 `canonical` 不一定逐字节相同，哈希值只用来说「不等」。
  - 什么现象会推翻：广谱变异的键在别处先去掉了 `profile.dev`（`mutation-prune.py:1049` 之后另有过滤），或广谱变异的复用键本来就不进 profile（`global_lines` 的 `build_lines` 调用方没传 `build_environment_lines`）。我读到的调用链是 `broad-mutation.py:1419` → `build_environment_lines`，没起脚本跑。

### 1.6 背景材料「实现今天的样子」那一节（六份里的第六份，judged-sha256.txt 第一行）

背景材料自己的事实断言，逐条现查：

| 断言 | 现查 | 判 |
|---|---|---|
| `read_cargo_command` 说明在第 129、779 行 | 第 129 行「cargo 命令现读：fmt、clippy、test 三条……」、第 779 行「fmt、clippy、test 三条 cargo 命令的写法不在这里抄第二份……」 | 对 |
| `clippy_checking_as_compiling` 第 2634 行起、文件头第 148 行 | 说明从第 2634 行起，函数在第 2636 行；文件头第 147–148 行 | 对 |
| 挑测试目标认段，文件头第 86–103 行 | 第 86 行「正是一个测试目标的根文件」起、第 98–103 行是 Cargo.toml 那一条 | 对 |
| 删快照文件头第 50–52 行；`record_task_attempt` 第 905 行起、`remove_earlier_attempt_snapshots` 第 912 行起 | 第 50–52 行是三条要点；说明在第 905、912 行，函数在第 906、916 行 | 对（「起」含说明行） |
| 收尾守卫文件头第 38 行起与第 858 行 | 对 | 对 |
| `task-server/task_server.py:8933` 交 `TASK_SERVER_TASK` 与 `TASK_SERVER_ATTEMPT` | 此刻在第 9196 行（`task_server.py` 此刻有别的会话的 1058 行改动） | 行号漂移，内容对 |
| `scripts/gate/admission.py:867` 的 `PROFILE_SETTINGS_LEFT_OUT` | 此刻第 868 行 | 行号差 1 |
| `scripts/mutation/mutation-prune.py:1049` 取整张 `[profile]` | 第 1049 行，原样 | 对 |
| `task_server_lib.py` 里 `PROGRESS_LINE` 0 命中 | 两份服务文件各 0（`grep -c`），`TASK_SERVER_PROGRESS` 同 0 | 对 |
| 删了开关 `skip-build`、`build-whole-workspace`、8 条 `case-progress-*`，加 `finish-in-forked-shell`、`rebuild-evidence-test-only`、manifest 3 条、`earlier-attempt-*` 5 条 | 见第 2.2 节：新旧文件头开关名集合逐条对，删 10、增 10，名字与这句一致 | 对 |
| 表格里「删进度行那一套」 | 删的是机读的 `TASK_SERVER_PROGRESS` 那一套；每 300 秒打一行人读进度的 `progress_ticker`（第 807 行）还在（第 1303 行起它） | 措辞有歧义，不是错 |
| 「这一轮没有改 `crates/`」 | `git diff HEAD --stat -- crates` 此刻 75 个文件，是别的会话的；分不出 | 复核不了；我只核了被判六份没碰 `crates/` |
| 「2000 次里 862 次收尾跑在子 shell 里，加守卫后 0 次」 | 复刻脚本我没跑（要起样本进程，交主 agent） | 复核不了 |
| 「门禁开关复跑 117 条（1 份）逐格相同」 | `.claude/gate.d/break-switch-replay-files.tsv:72` 的登记行写「MERGE_AND_VERIFY_BREAK 一百一十六条、SINGLEFS_WORK_ROOT_BREAK 一条」；我数文件头是 116 + 1 = 117 | 条数对得上；那一趟的结局我没复跑 |

## 2. 三条判据

### 2.1 合入后验证做的步，与 `main-agent.md` 验证段第 1 步一致吗

代码里实际的次序（读出来的）：打补丁（第 2696 行）→ `rocksdb-prebuilt.py entry`（第 2755 行）→ 测试那一步（挑目标、按条复用、`test-targets-parallel.py` 按目标并行起）→ fmt（第 2819 行）→ clippy 每个特性组合（第 2824 行）→ 重编核对（第 2859 行）→ 验证包那一步（第 2870 行）。cargo 的三步是 fmt、clippy、test，没有 build。
对第 57 行：打补丁、fmt、clippy、只跑点名的测试目标、`--item`、退出码三支，逐项对得上（1.2 节）；第 57 行没写的验证包那一步与重编核对，`SKILL.md` 第 22 行与文件头第 6–8 行有，去 build 之前就没写，不属于这一轮。
结论：一致。推翻它的现象：脚本某一步起了 `cargo build`（`grep -n 'cargo build' scripts/gate/merge-and-verify.sh` 只应命中文件头第 7、131 行两处说明），或第 57 行 / `SKILL.md` 第 22 行再写回 build。

**一处文字没跟上**（打中「链接残留」）：`.claude/agents/implementation-writer.md:32` 写「不写 `cargo build`，链接留给合入后验证，`scripts/gate/merge-and-verify.sh` 文件头」，`.claude/rules/implementation-workflow.md:60` 写「clippy（点名改到的包；不写 build，链接留给合入后验证）」。文件头第 7 行写的是「不跑 cargo build：clippy 带 --all-targets 已对全部目标做类型检查，调试构建把测试二进制全链一遍只占盘」，实现里没有任何一步链接补丁改到的包的全部目标：测试那一步只链点名的测试目标，clippy 只类型检查。没点到的 bin、example、别的测试目标，链接错只在提交时 `.claude/singlefs-ai-sop/scripts/check.sh:86` 的 `cargo build --all-targets` 才暴露。这两处还把「链接」指到文件头，而文件头没有这一项的交代；`investigator.md:33` 写的是「链接留给提交时的门禁」，与实现一致。这是文档与实现的不一致，不是脚本的缺陷；该红而不红的格归攻方腿判。
- 什么现象会推翻：文件头或 clippy 命令里有一步会链接全部目标（例如 clippy 命令带了 `--release` 之类的出码档，我读 `.claude/singlefs-ai-sop/scripts/check.sh:77` 是 `cargo clippy --all-targets … -- -D warnings`，没有）。

### 2.2 每条新开关与删掉的开关在文件头开关表里点名齐吗

做法：`switches.py`（草稿目录）取新旧两份脚本文件头（第一行以外、第一个非 `#` 行之前）里所有 `MERGE_AND_VERIFY_BREAK=<名>`，取集合；再在代码体（非注释行）里找每个名字。

- 文件头里新旧各 116 个名字；删掉的 10 个：`build-whole-workspace`、`skip-build`、`case-progress-at-end-only`、`case-progress-before-step`、`case-progress-dropped`、`case-progress-green-only`、`case-progress-indented`、`case-progress-total-wrong`、`case-progress-zero-before-estimate`、`case-progress-zero-dropped`；这 10 个名字在新脚本全文 0 命中（含注释）。
- 增加的 10 个：`earlier-attempt-not-recorded`（文件头第 641 行、代码第 907 行）、`earlier-attempt-snapshot-kept`（第 644、920 行）、`earlier-attempt-task-ignored`（第 646、928 行）、`earlier-attempt-not-smaller-removed`（第 648、929 行）、`earlier-attempt-open-files-unchecked`（第 650、934 行）、`finish-in-forked-shell`（第 471、858 行）、`manifest-sections-refused`（第 532、2097 行）、`manifest-other-sections-unchecked`（第 536、1799/1801/1817 行）、`manifest-default-path-ignored`（第 539、2049 行）、`rebuild-evidence-test-only`（第 392、2850 行）；每个都在文件头点了名、代码里有非注释的判断。
- 反方向：从代码里抓 `MERGE_AND_VERIFY_BREAK:-}" ==/!= "<名>"`、`broken ==/!= "<名>"`、`MERGE_AND_VERIFY_BREAK=<名>` 三种写法得 98 个名字，全在文件头里；文件头 116 个名字每个在代码体非注释行至少出现一次。
- 新自证变量 `MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH`：文件头第 685–686 行点了名，`BREAK_VARIABLES`（第 687 行）里有它，`settle_break_switches` 平常设了拒（第 705 行）。
- 点名的格：新开关文件头里「」里的格名，逐个在代码里找 `✗ 自检：<格名前缀>`，全部命中（`Checking`、`都重编了`、`上一次尝试留下的快照` 三个落在「」里的是词不是格名，没有对应格，不算缺）。
- 与登记表对：`.claude/gate.d/break-switch-replay-files.tsv:72` 登记 116 + 1 = 117，与我数的相同。
结论：点名齐。推翻：任何一个 `MERGE_AND_VERIFY_BREAK=` 值在代码里有判断而文件头没有，或反过来（`switches.py` 重跑即得）。
一个说明：新开关「点名的格红且只红点名的格」是弄坏开关的要求（`show-me-test.md`「每一个弄坏开关都必须会红」），那要起脚本，我没复跑（命令见第 3 节）。

### 2.3 快照与编译目录的收尾删，照 `command-safety.md`「测试镜像一律放临时目录」吗

规则四条（`.claude/singlefs-ai-sop/rules/command-safety.md` 第 88–90 行，标题第 86 行）逐条对：

| 规则 | 实现 | 判 |
|---|---|---|
| 第 88 行「镜像不放仓库里；路径从环境变量取，默认落到 `${TMPDIR:-/tmp}`」 | 编译目录：第 5081 行 `mktemp -d "${TMPDIR:-/tmp}/merge-and-verify-target.XXXXXXXX"`，只在调用方没设 `CARGO_TARGET_DIR` 时建（第 5078–5087 行）。快照：`$MERGE_AND_VERIFY_LOG_DIRECTORY` 或默认 `<工作根>/merge-and-verify/<日期>-<进程号>` 下的 `repo/`；日志目录落在要验的树里面时拒（第 5019 行的 `die`），所以不在仓库里；默认落点是工作根、不是 `$TMPDIR`，文件头第 29、55–58 行写了原因：工作根两台共享、快照要第二台同一路径读得到 | 编译目录成立；快照路径来源成立，默认落点与规则字面不同（有写明的理由，不是这一轮引入的） |
| 第 89 行「建镜像的装置在测试结束时自己删掉它：成功、失败、panic 都删……shell 用 `trap … EXIT`」 | 第 5021–5023 行 `trap finish EXIT`、`trap 'exit 130' INT`、`trap 'exit 143' TERM`；`finish`（第 854 行起）先删编译目录（第 869 行）再删快照（第 888 行），与文件头第 59 行「trap 的第一步、在删快照之前」一致；第 858 行的守卫让 `&` 放后台 fork 出来的子 shell 跑到 `finish` 时不删（背景材料说在 fork 之后、换回缺省 trap 之前收到 TERM 会照继承的 EXIT trap 删这一趟的东西，这一条在读代码上讲得通：第 5023 行的 TERM trap 是 `exit 143`，`progress_ticker` 在第 1303 行放后台，它第 812 行才重设自己的 TERM trap）。主 shell 自己跑 `finish`（正常退出、`exit 130`、`exit 143` 都走 EXIT trap）时 `BASHPID` 恒等于 `$$`，守卫不拦。cgroup 杀（SIGKILL）trap 跑不到，文件头第 48–49 行自己写了；这一轮加的第 905–950 行补的是「同一任务下一次尝试开跑时删上一次的快照」，补不到编译目录（第 55 行明说不归这一步）、也补不到没有下一次尝试的任务 | 成立；SIGKILL 的一支规则字面（shell 里 trap 做不到）覆盖不了，靠「同一任务重试」兜快照 |
| 第 90 行「要留着看现场，由显式的环境变量打开，留下时打印路径」 | `MERGE_AND_VERIFY_KEEP_COPY=1`（第 864 行）留着、打「留着快照 … 与编译目录 …」与「看完自己删：rm -rf -- …」（第 882–883 行）；文件头第 41 行写明 | 成立 |
| 门禁阶段「跑完没留下临时文件」（规则第 94 行） | 编译目录在 `$TMPDIR` 下、`finish` 第一步删；经服务起时编译目录由服务给（`task_server.py:2069` 的说明与第 6316 行的判据），脚本不建也不删，文件头第 61 行写「归调用方删」，主 agent 用 `task clean` | 成立 |

新加的「删上一次尝试的快照」是否与第 90 行「留着看现场」冲突：它也会删带 `KEEP_COPY=1` 留的上一次尝试的快照（第 52 行明说）。服务重排的原因是撞内存上限、被 slice 挤掉、卡住一类（`task_server_lib.py` 第 126–150 行与异常表「服务自己修」一栏），上一次尝试已经被杀、没有「要看的现场」；但我没逐种核「正常以退出码 1、3 收尾的一趟绝不会被重排」，这一句是推的。推翻：服务对 `exit=1` 或 `exit=3` 的尝试也重排，且主 agent 要读那一棵快照。

结论：成立，两处与规则字面有差（快照默认落点在工作根；SIGKILL 靠重试兜）。

### 2.4 同一套判据的措辞（打中「判据措辞」）

文件头第 68 行「与上游 `.claude/singlefs-ai-sop/scripts/check.sh` 同一套判据」、第 136 行「（用户要的是同一套判据）」；`.claude/singlefs-ai-sop/scripts/check.sh:86` 仍有 `cargo build --all-targets`，文件头第 131 行明说「cargo build 那一条不读」。三步（fmt、clippy、test）的参数确实现读自 check.sh、与它同一套；build 一步是有意不同。读者只看第 68、136 行会以为合入后验证与 check.sh 的 cargo 命令集相同。性质：文字，没有行为缺陷；归主 agent 要不要在第 68 行补半句「build 除外」。

## 3. 要主 agent 跑的命令

每条一项，我没交、没跑。

1. 缺省路径目录形探针（1.1 节「缺省路径目录形」）：
   - 命令：`task-server/task submit -- python3 /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-sonnet/manifest_default_path_probe.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-sonnet/probe-default-root`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看：`TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；日志里「tests/extra/main.rs 在目标表里：」「src/bin/tool/main.rs 在目标表里：」两行。我读出来的预期是两行都是 `False`（那就是第 99 行说「认」而目标表不认）；任一行是 `True` 推翻这一条。脚本只在样本目录里建小样本、读完删。
2. 广谱复用键探针（1.5 节「广谱复用键」）：
   - 命令：`task-server/task submit -- python3 /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-sonnet/profile_key_probe.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-sonnet/probe-profile-key`
   - 在哪起：仓根。
   - 看：`type=succeeded`、`exit=0`；日志里 `head [...]`、`worktree [...]` 两个 `profile ` 行，与末尾「profile 行相等：」。预期 `False`（HEAD 那一版没有 `dev`，工作区有）；`True` 推翻（那就是 `build_environment_lines` 别处过滤了）。
3. 真调试构建量二进制大小（背景材料「没验的」那一项，也核 `Cargo.toml:30` 的 73%–94% 与 26.8 GB 的推法）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo build 点名 singlefs-unit-graph 包的 bin unit-graph; 范围=1 个包的 1 个二进制; 为什么=量 [profile.dev] debug=line-tables-only 之后调试可执行里 .debug_* 节占比，核 Cargo.toml 注释与 26.8 GB 的推法' task-server/task submit -- cargo build -p singlefs-unit-graph --bin unit-graph`
   - 在哪起：仓根。写法照范围声明闸（`.claude/hooks/run-scope-guard.sh` 文件头）与 `project-code-guard.sh` 判，被拒照说明改。
   - 看：`type=succeeded`、`exit=0`。之后对服务给的编译目录里 `debug/unit-graph` 跑 `readelf -S -W <文件>`，把 `.debug_*` 八个左右的节大小加起来除以文件大小，与我量的改前同一个可执行（`target/debug/deps/unit_graph-3aec26e79f64e211`：52,291,768 字节、`.debug_*` 合计 40,205,330 字节、76.9%）比。改后占比若仍超过 50%，26.8 GB 那个数就站不住。用完 `task-server/task clean <号>`。
4. 自证与开关（背景材料说已绿过，我没复跑；弄坏开关「点名格红」与「只红点名格」要起脚本才核得了）：
   - 命令：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   - 在哪起：仓根（写法照 `.claude/agent-common.md`「子 agent 不交任务」里 required 那一条与 `.claude/hooks/selftest-temporary-service.tsv:56`）。
   - 看：`type=succeeded`、`exit=0`，日志里没有「✗ 自检」。
5. 收尾守卫的竞争复刻（背景材料的 2000 次 / 862 次 / 0 次）：复刻脚本在 `merge-verify-drop-build-step/batch5/repro/race.sh`，起样本进程，我不跑也没细读；要复核请主 agent 自己交，并在复跑里把 `BASHPID` 守卫换成不带守卫的一版作对照。

## 4. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1.1 脚本去掉 build | 成立 | 文件头第 7、131 行与代码一致，全文件 build 只剩两处说明 |
| 1.1 重编核对读 clippy 的 Checking 行 | 成立 | 第 2825、2852–2854、2639 行与文件头第 147–148 行逐步对上；依赖 clippy 不带 `-p` |
| 1.1 挑测试目标认 `[[test]]` / `[[bin]]` 段 | 成立，一处不一致 | 缺省路径 `<名>/main.rs` 形文件头认、目标表不认，退 3、偏安全，读出来的，探针待跑 |
| 1.1 删上一次尝试的快照 | 成立 | 记、找、核、删、时点五样与第 48–55 行一一对上；别的任务、尝试号不小、手跑都不碰 |
| 1.1 收尾守卫 | 成立 | 第 858 行与第 38–40 行一致；主 shell 走 EXIT、INT、TERM 时守卫不拦 |
| 1.1 删机读进度行 | 成立 | 服务两份文件里 `PROGRESS_LINE`、`TASK_SERVER_PROGRESS` 各 0 命中；人读进度行没删 |
| 1.2 `main-agent.md:57` | 成立 | 步骤与文件头一致；「编一次」字面推不出，不冲突 |
| 1.3 `SKILL.md:22` | 成立 | 与文件头第 6、29、168–170 行一致 |
| 1.4 `lib_run_scope.py:529` | 成立 | 与第 15 行一致；别的 181 行 diff 不在这一轮 |
| 1.5 `Cargo.toml` `[profile.dev]` | 成立 | 只有一个 dev 表、`debug` 不进验证包判器组键、`admission.py:868`、`lib_mutation_units.py:103` 的键 |
| 1.5 `Cargo.toml:30` 注释 | 不成立（一句） | 「合入后验证一趟要链几百个」说的是改前；打中「注释前提残留」 |
| 1.5 广谱复用键 | 打中 | `mutation-prune.py:1049` 取整张 `[profile]`，加 `[profile.dev]` 让广谱变异键对不上一次；哈希两个不等；探针待跑 |
| 1.5 26.8 GB 推法 | 推的 | 一个样本 `.debug_*` 占 76.9%，只留行号表剩 29.2%，背景是 19.4%；真构建待跑 |
| 1.6 背景材料的行号 | 对，两处行号漂 | `task_server.py:8933`→9196、`admission.py:867`→868；另两条断言「复核不了」 |
| 2.1 步与 `main-agent.md` 第 1 步 | 一致，一处文字残留 | `implementation-writer.md:32`、`implementation-workflow.md:60` 的「链接留给合入后验证」不再成立；打中「链接残留」 |
| 2.2 开关表点名 | 点名齐 | 删 10、增 10，文件头 116 名与代码体逐个对上；登记表 117 条 |
| 2.3 快照与编译目录收尾删 | 成立，两处字面有差 | 快照默认落工作根、SIGKILL 靠同任务重试兜；经服务起时编译目录归服务 |
| 2.4 「同一套判据」措辞 | 文字有差 | 文件头第 68、136 行未提 build 除外；`.claude/singlefs-ai-sop/scripts/check.sh:86` 仍有 build |

打中 5 处：链接残留、注释前提残留、广谱复用键、缺省路径目录形（推的）、判据措辞（文字）。

## 没做什么

- 没起任何项目代码：自证、弄坏开关、收尾守卫的复刻、真 cargo 调试构建都没跑；命令在第 3 节，要引的读数由主 agent 复跑。没往统一任务服务交任务、没替主 agent 采纳或出判决。
- 攻方腿的格（去掉 build 之后有没有该红而不红、`[[test]]` / `[[bin]]` 段把该退 3 的放成挑目标、删快照删错、守卫让该删的不删）我只在正推顺路碰到的地方写了读到的事实，没有构造反例；那几格的结论归攻方腿。
- 没核 `lib_run_scope.py` 那 181 行 diff 里分片入口那一批，没核 `.claude/main-agent.md` 第 24、38、100 行的别的改动（背景材料只判第 57 行）。
- 没核 `crates/` 下别的会话的 75 个文件改动。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r1-sonnet/` 里有 `mv.diff`、`switches.py`、新旧开关名清单与两个探针脚本，没有仓副本与编译目录（`du -sh` 约几十 KB）；探针脚本留给主 agent 跑，跑完它们自己删样本目录。
