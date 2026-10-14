# m3-defs-b3-r3 正推腿（Sonnet）报告：T5、T6（2026-10-13）

立场：逐条核「两轮判决与正文说的数和清单，就是产物与冻结副本里的样子」。引处的行号都是我用 `grep -nF` 现取的；冻结副本里的文件写绝对路径。模型目录 `research/prompts/m3-defs-b3-r3-sonnet-model/`，表里「K:键名」指它的 `out/count_checks.txt` 里的那个键。

## 0 现场与复跑

- 冻结副本：第三轮 `~/.local/share/singlefs/work/m3-defs-b3-r3/repo/` 对 `research/prompts/m3-defs-b3-r3-snapshot/frozen-copy-sha256.txt` 35 份 `sha256sum -c` 全 OK、0 份不对（K:snapshot.r3.*）；第一、二轮冻结副本 21/21、30/30。三份冻结副本各与集成副本标签 before-b4、after-b4-fixup2、after-b5-fixup 逐份同 sha（K:frozen_vs_tag.*）。
- 读了什么：两轮判决与岔路单、第二/三轮正文、调度记录、四条腿报告、前两轮三个模型目录、`~/.local/share/singlefs/work/m3-tooling-*` 的 patch.diff 与 report.md、集成副本的 verify*.summary 与 verify*-out。没读这一轮攻方腿的报告与模型目录。开工先读的 `agent-common.md`、`three-way-inference.md`、`implementation-first.md` 读的是主工作区那一份（定义要求的开工读物），没拿它们当被判文件的证据；被判文件的内容与行号全取自冻结副本。
- 复跑：`bash ~/.local/share/singlefs/work/m3-defs-b3-r3/repo/scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-defs-b3-r3-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-defs-b3-r3/repo --work <新的草稿目录>`，末行 `name=compare file=count_checks same`；我连跑两次，第二次与 `out/count_checks.txt` 逐字节相同。前两轮腿的模型目录另拷进草稿目录，用 `tools/rerun-legs.sh` 经内存包装复跑，输出在 `out/rerun-*.log`。
- 门禁格（`--check`）子 agent 起不了。heavy-thread-defaults 一格我把 `code-tooling.sh` 里那一格的 python 抽出来直接跑（`tools/run_htd_cell.py`，不经门禁入口，基准按「没有」），在第一、二轮冻结副本和集成副本 after-b4-fixup2、after-b5 上复现了记录里的 831/90/62、831/94/66、832/94/66，所以它在第三轮冻结副本上的结果可信；仍要主 agent 在冻结副本上跑格坐实（§3 第 1 条）。
- 私有值：`multi-host.env` 只读来数命中，报告与模型目录里没有值；用同一份值扫了模型目录，命中 0 份。

## 1 T6　两轮判决、第二/三轮正文、调度记录里的数，一个数一行

判：对 / 对不上 / 复核不了。「引处」是我现查的行号（判决一、判决二、正文二、正文三、调度记录各指 `research/prompts/m3-defs-b3-r1-main-verification.md`、`…r2-main-verification.md`、`_m3-defs-b3-r2-body.md`、`_m3-defs-b3-r3-body.md`、`records/2026-10-12-里程碑三工具与治理批.md`）。

结果：132 行里 112 对、10 对不上、10 复核不了；sha256 末位 37 个，35 对、2 对不上。去掉重复，对不上的是八处：

1. 正推腿报告的 sha256 写成 `60ed02db…a8a8`，真值末 4 位是 d8a8（A01、E32、H31、H32；判决一 L5 与调度记录 L104 同错）。
2. 判决二 L28「攻方九种写法……0→2」：九个 case 里 8 个 0→2，p-env-C 仍 0（B18）。
3. 正文二 L30 与调度记录 L115「16 项自证与门禁格、约 113 个弄坏开关」：真拆分是绿 14 对 2 错、开关 115 对 2 错，只有总数 129 对（C05、E43）。
4. 正文三 L22 的 94 处、66 行、831 份只对 after-b4-fixup2；正文三点名的第三轮冻结副本（after-b5-fixup）上这一格是红的，1 处（D03，另见 §2.3 W01）。
5. 正文三 L22「records 能当出处的 504 行」：能当出处的是 557 行，504 是其中不带叫停类词的（D06）。
6. 调度记录 L73「41 个文件 +1711/−247」：只算改动的文件；四份补丁共 68 个文件 +2550/−247（E21）。
7. 调度记录 L112「30 份 +1283/−237」：不含 22 个新增文件；四份补丁共 52 个文件 +1425/−237（E39）。
8. 调度记录 L123「13 份自证与 4 道样本档绿」：判对的自证只有 10 份（E52）。

复核不了的 10 行是 E02、E12、E13、E15、E16、E24b、E25、E30、E31、E42d，原因写在各行；八份样本与四个样本根的红绿结局要门禁入口，我只核了个数（B07、B08）。

推翻条件：任一行的「现算结果」在 `out/count_checks.txt` 对应键重跑得到引的数，该行就不成立；D03 在主 agent 于冻结副本上跑 `--check heavy-thread-defaults` 得绿时不成立。

| 编号 | 引处 | 引的数 | 现算（命令 / out 键） | 现算结果 | 判 | 口径缺什么 |
|---|---|---|---|---|---|---|
| A01 | 判决一:5 | 正推腿报告 sha256 60ed02db…a8a8 | sha256sum 报告 | 60ed02db…d8a8（K:legsha.m3-defs-b3-r1-sonnet-output） | 对不上 | 前 8 位对，末 4 位 a8a8 应为 d8a8；调度记录 L104 同一处同错 |
| A02 | 判决一:5 | 攻方腿报告 sha256 afb4a1fb…6645 | K:legsha.m3-defs-b3-r1-opus-output | afb4a1fb…6645 | 对 |  |
| A03 | 判决一:11 | 冻结副本 21 份 sha256sum -c 全过 | K:snapshot.r1.* | lines=21 ok=21 notok=0 | 对 |  |
| A04 | 判决一:12 | cite-check 两份报告 核 88 处、对上 88 处、没判 0 处 | cite-check.py 两份报告 --root r1 冻结副本 --background _m3-defs-b3-r1-background.md | 核了 88 处引文，对上 88 处，没判 0 处 | 对 |  |
| A05 | 判决一:13 | 攻方模型 rerun.sh 五行 same | rerun-legs.sh 第 1 条（拷到草稿目录、经 4G 内存包装） | 5 行 compare 全 same（out/rerun-r1-opus.log） | 对 |  |
| A06 | 判决一:13 | 模型目录 sha256 与报告抄的逐行相同 | K:modelsha.r1opus | model_files=11 report_lines=11 identical=True | 对 |  |
| A07 | 判决一:14 | capped.sh --print 8 \| grep -c E161 输出 0 | K:r1.capped_print8_E161（r2、r3 冻结副本同命令） | r1 冻结副本 0；r2、r3 冻结副本 1 | 对 | 数的是第一轮那一版；改后（G3）已是 1，正文别处引时要带版本 |
| A08 | 判决一:14 | layer0 第 750–765 行 --test-threads 0 处 | K:r1.layer0_750_765_test_threads | r1 冻结副本 0（整份文件也是 0）；r2、r3 冻结副本该区间仍 0、整份 5 | 对 | 区间是改前的行号；改后 --test-threads 在别的行（整份 5 处） |
| A09 | 判决一:14 | tools: 只有 Read、Bash 的 8 份（prior-art 另有 WebFetch、WebSearch） | K:r1.agents_total / agents_without_write_edit | 共 14 份，tools 里没有 Write 也没有 Edit 的 8 份 | 对 | 「只有 Read、Bash」其实含 prior-art 的 Web 两样；判的是无 Write/Edit；investigator、kb-scribe 有 Edit 无 Write 算有 |
| A10 | 判决一:15 | records/2026-09-30… 第 1004 行含 PEER_SSH_HOST 的值，已提交（3e3cdc3c） | probes/value_hits.py（只打行号） | 3e3cdc3c、ce1457c3、HEAD、origin/master、r1/r2/r3 冻结副本都是 [1004]；主工作区工作树 [] | 对 |  |
| A11 | 判决一:15 | m3-batch10x-code-r1-opus-output.md 第 295、302 行含本机 hostname，已提交（ce1457c3） | 同上 | 同上各处 [295, 302]；主工作区工作树 [] | 对 |  |
| A12 | 判决一:15 | HEAD 与 origin/master 同为 ce1457c3 | git rev-parse --short HEAD、remotes/origin/master | ce1457c3、ce1457c3 | 对 | 是今天现查；git status -sb 首行 ## master...origin/master |
| A13 | 判决一:26 | lib_writer_lock.py 第 4 行残留旧编号 ⑮ | 读 r1、r2 冻结副本该文件第 4 行 | r1 含 ⑮；r2 含 ⑯ | 对 |  |
| A14 | 判决一:34 | G1 的任务带重活类型 六种 | K:r2.thread_limit_required_types | r2 冻结副本类型表 6 个名字 | 对 |  |
| A15 | 判决一:3 | 判决一 编号：D1–D7、G1–G11、H1–H3 | K:v1.rows_D / rows_G / fork_items | D 行 7、G 行 11、H 项 3 | 对 |  |
| B01 | 判决二:11 | 冻结副本 30 份 sha256sum -c 全过（非 OK 0 行） | K:snapshot.r2.* | lines=30 ok=30 notok=0 | 对 |  |
| B02 | 判决二:12 | cite-check 核 104 处、对上 104 处、没判 0 处 | cite-check.py 辩方、攻方两份 --root r2 冻结副本 --background _m3-defs-b3-r2-background.md | 核了 104 处引文，对上 104 处，没判 0 处 | 对 |  |
| B03 | 判决二:13 | 攻方报告第 20–77 行的 sha256 与模型目录逐行相同 | K:modelsha.r2opus | model_files=58 report_lines=58 identical=True | 对 |  |
| B04 | 判决二:13 | 攻方模型 rerun.sh 八行 same（带 FROZEN_ROUND1） | rerun-legs.sh 第 2 条 | 8 行 compare 全 same（out/rerun-r2-opus.log） | 对 |  |
| B05 | 判决二:14 | 辩方报告第 7 节 sha256 与模型目录逐行相同 | K:modelsha.r2sonnet | model_files=29 report_lines=29 identical=True | 对 |  |
| B06 | 判决二:14 | 辩方 rerun.sh 九行里八行 same、r7_abandoned 一行不同、diff 只有第 29 行 | rerun-legs.sh 第 3 条；diff 输出与模型 out/r7_abandoned.txt | 9 行 compare：8 行 same、r7_abandoned differs；diff 只有第 29 行（工作目录绝对路径） | 对 |  |
| B07 | 判决二:15 | 攻方 tools/run-gate-samples.sh 八份样本 | K:opus_gate_sample_dirs | gate-samples 下样本目录 8 个（doc-text 4、doc-process-records 3、code-tooling 1） | 对 | 八份「结局」要门禁入口，子 agent 起不了：复核不了；数是 8 |
| B08 | 判决二:15 | 辩方 make_gate_sample_roots.py 四个样本根 | K:r2sonnet.sample_roots | 脚本里 root("gate-…") 4 个 | 对 | 四个根的红绿结局要门禁入口：复核不了 |
| B09 | 判决二:16 | 主工作区两份文件里 PEER_SSH_HOST 的值与本机主机名命中都是 0 | probes/value_hits.py main-wt / head / origin-master | 工作树 [] 与 []；HEAD 与 origin/master 仍是 [1004] 与 [295, 302] | 对 | 「0」只对工作树；提交与远端还带着（H1 定「历史不动」，换掉的改动还在暂存区、没提交） |
| B10 | 判决二:17 | heavy-thread-defaults 扫 831 份 | tools/run_htd_cell.py <r2 冻结副本>（抽出格的 python 直接跑） | r2 冻结副本：扫了 831 份（scripts 455、gate.d 24、hooks 16、crates 335、cargo-command-prefix 1） | 对 | 只对第二轮那一版（after-b4-fixup2）；第三轮冻结副本（after-b5-fixup）这一格是红的，见 D03 |
| B11 | 判决二:17 | 可疑写法 94 处全在排除表 | 同上 | r2 冻结副本：可疑写法 94 处全在排除表里 | 对 | 同 B10 |
| B12 | 判决二:17 | 表 66 行每行都命中 | K:exempt_rows.* 与格输出 | r2、r3 冻结副本排除表 66 行；r2 格输出「表 66 行，每行都命中」 | 对 |  |
| B13 | 判决二:18、判决二:26 | J07 的 16 份测试文件（挂 gpu 特性） | K:r2.gpu_cfg_test_files | tests/ 下含 feature = "gpu" 的 .rs 16 份 | 对 |  |
| B14 | 判决二:24 | 辩方第 2 节十四组 | K:r2sonnet.section2_first_table_rows | 第 2 节第一张表 14 行 | 对 |  |
| B15 | 判决二:24 | S08、S09 改前→改后 0→2 | 辩方模型 rerun（r1_thread_limit） | 九行 compare 中 r1_thread_limit same（与 out/ 逐字节相同） | 对 |  |
| B16 | 判决二:25 | S01 改前→改后 1→0 | 辩方模型 rerun（r2_capped） | r2_capped same | 对 |  |
| B17 | 判决二:25 | replay.sh 22 行钉线程数 | K:r1/r2/r3.replay_sh_thread_pins | grep -c 'E1[0-9][0-9]_[A-Z_]*THREADS=[0-9]' 三个冻结副本都是 22；E161_THREADS=10 在第 722 行 | 对 |  |
| B18 | 判决二:28 | -p/bench 攻方九种写法、S10、S11 改前→改后 0→2 | K:r2opus.r5_* | r5 out 中 -p 与 bench 的 9 个 case：8 个 0→2，p-env-C 仍 0（它是 C2），run-cwd-probe-bin 不在这 9 个里（2→2） | 对不上 | 九种里 8 种 0→2；p-env-C 0→0 属 C2（K7）；该句没分 |
| B19 | 判决二:28 | C1 改后 0、改前 2 | r5-r2.txt 与 r5-r1.txt（已 rerun same） | copy-light-selftest-body-heavy：改后 rc=0、改前 rc=2 | 对 |  |
| B20 | 判决二:32 | 仓里已有 10 份产物超过 8 MiB | K:main.results_files_over_8MiB | 主工作区 research/results 下 >8 MiB 的 10 份，最大 115269707 字节 | 对 | 冻结副本不带 research/results（0 份）；数的是主工作区 |
| B21 | 判决二:32 | 带 NUL 的文本主工作区 0 份 | K:main.results_files_with_NUL_head | research/results 下前 8 KiB 带 NUL 的 3 份，全是 .gz | 对 | 判「文本」靠后缀；格本身不看后缀，3 份 .gz 会被它算「没判的二进制」 |
| B22 | 判决二:24 | 六种类型缺行拒 | K:r2.thread_limit_required_types | 6 | 对 |  |
| B23 | 判决二:31 | ⑭ 扩到七种类型 | K 无；读 r2 冻结副本 LONG_RUN_AGENTS = {designer} \| 六种 | 7（r3：10） | 对 |  |
| B24 | 判决二:24、判决二:32 | R 行：3 站得住、6 部分站不住 | K:v2.verdict_tally / v2.rows_R | R 行 9；站得住 3、部分站不住 6 | 对 |  |
| B25 | 判决二:38、判决二:48 | 改法 K1–K11 | K:v2.rows_K | K 行 11；岔路 H 项 1 | 对 |  |
| B26 | 判决二:50 | 另立账 J03、J12、B3、J11、J10、S26、J08 后半、J14，加第一轮 S21、O09 | K:v2.pile_line_ids / pile_first_round_ids | 本行 8 笔 + 第一轮 2 笔 | 对 | R5 行里的 C2 另立账不在本行（它并进 K7） |
| B27 | 判决二:5 | 辩方、攻方报告 sha256 9976f8c1…1fec、42b53504…9126 | K:legsha | 9976f8c1…1fec、42b53504…9126 | 对 |  |
| B28 | 判决二:26 | available_parallelism 本机 32 | nproc | 32 | 对 |  |
| B29 | 判决二:39 | K2 加 clerk、three-way-forward、three-way-defense 三种 | K:r3.thread_limit_required_types | r3 冻结副本类型表 9 个 | 对 |  |
| C01 | 正文二:13 | 冻结副本 = 集成副本标签 after-b4-fixup2 那一版 | K:frozen_vs_tag.r2 | 30 份里 30 份与标签同 sha | 对 |  |
| C02 | 正文二:17 | R1 六种类型 | K:r2.thread_limit_required_types | 6 | 对 |  |
| C03 | 正文二:24 | 七份定义「输入」的「预计挂钟」 | K:r2.agent_defs_with_预计挂钟 | r2 冻结副本含「预计挂钟」的定义 7 份 | 对 | 数的是整份定义里有这个词，没限「输入」一节 |
| C04 | 正文二:30 | verify3 派 133 项、收 133 项 | K:verify3.total | verify3.summary 133 行 | 对 |  |
| C05 | 正文二:30 | 16 项自证与门禁格、约 113 个弄坏开关按期望判 | K:verify3.tally | 期望绿 16 项（判对 14、判错 2）；弄坏开关 117 项（判对 115、判错 2） | 对不上 | 对的总数 129 相同，拆分不对：绿 14 不是 16，开关 115 不是 113；「约」不是现算 |
| C06 | 正文二:30 | 判错 4 项分诊：一项真冲突，另三项是挂错脚本、无 git 历史、31 路并行时探工具链版本没回来 | K:verify3.wrong、verify3.cell_code_tooling | 4 项 = 第 14（code-tooling 格）、16（abandoned-rounds 格）、93、94（挂错脚本） | 对 | 「项」与「原因」不是一一对应：93、94 两项同一原因；第 14 项一项里就含「真冲突」和「31 路并行」两种原因 |
| C07 | 正文二:30 | heavy-thread-defaults 红 4 行；登记 4 行后转绿 | K:verify3.hdd_red_lines、exempt_rows.after-b4-fixup(2) | 红行 runner-dispatch-guard.sh 1182、1791、1792、2107；排除表 62→66（+4）；fixup2 版格转绿 | 对 |  |
| C08 | 正文二:31 | 双机驱动 8 格「现算线程」会红 | K:verify1.two_hosts_* | --selftest 53 格里 8 格判错，8 个格名都是 *-threads-* | 对 |  |
| C09 | 正文二:25 | private-values 8 个字以上的词 | 读 doc-text.sh 文件头与常量 | 门槛 8 个字（r2、r3 都是） | 对 |  |
| C10 | 正文二:23 | 叫停出处原话 6 字以上 | 读 runner-dispatch-guard.sh ABANDONED_QUOTE_MINIMUM_CHARACTERS | = 6 | 对 |  |
| C11 | 正文二:17 | R1–R9 九格 | K:b2.rows_R | 9 | 对 |  |
| D01 | 正文三:13 | 冻结副本 35 份 sha256 | K:snapshot.r3.*、frozen_vs_tag.r3 | 35/35 OK；35 份与集成副本标签 after-b5-fixup 同 sha | 对 |  |
| D02 | 正文三:11 | 被判文件 14 份、都在 35 份里 | K:r3body.judged_files(_in_snapshot) | 14 份路径，14 份都在 35 份清单里 | 对 | T1 对象里的 scripts/hosts/heavy-threads.sh、heavy_threads.py 不在 35 份清单里：读到的版本没被钉住 |
| D03 | 正文三:22 | 可疑写法 94 处、排除表 66 行、扫 831 份（现跑绿） | tools/run_htd_cell.py 在 r2 冻结副本、after-b5、after-b5-fixup 上 | after-b4-fixup2：831/94/66 绿；after-b5：832/94/66 绿；第三轮冻结副本（= after-b5-fixup）：红，1 处（scripts/agents/dispatch-prompt.py:262） | 对不上 | 三个数只对 fixup2；正文三讲的第三轮冻结副本上这一格红；批 5 收尾复跑（L124）没跑这一格 |
| D04 | 正文三:22 | verify3 133 项 | K:verify3.total | 133 | 对 |  |
| D05 | 正文三:22 | verify4 148 项 | K:verify4.total | 148 | 对 |  |
| D06 | 正文三:22 | records 能当出处的 504 行 | K:records_scan.r2/r3 | 能当出处 557 行；其中没有叫停类词的 504 行；records 95 份（调度记录按禁读跳过） | 对不上 | 504 是「能当出处且行里没有叫停/停掉/停了/撂下/不判」的行，能当出处的是 557 |
| D07 | 正文三:22 | replay.sh 22 行 | K:r3.replay_sh_thread_pins | 22 | 对 |  |
| D08 | 正文三:22 | 19835 行新增命中 | tools/measure-config-words.py（批 5 丙的脚本）在集成副本现树 | 19844 行 / 2232 份（345 个词、59 个有命中、原词源 3 行）；引的 19835 / 2223 | 对 | 引的是批 5 丙只打自己补丁的那棵树（13654 路径）；集成副本 HEAD 13713 路径，多了 9 行 9 份；结论（≠0）不变 |
| D09 | 正文三:27 | 批 5 派 148 项、收 148 项、判错 1 项 | K:verify4.total / verify4.wrong | 148、148、1（第 6 项 runner-dispatch-guard） | 对 |  |
| D10 | 正文三:27 | 九种类型骨架打线程上限：31，十种打预计挂钟 | K:r3.skeleton_* | 14 种类型里骨架带「线程上限：31」的 9 种、带「预计挂钟：」的 10 种 | 对 |  |
| D11 | 正文三:17 | T1–T6 六格 | K:b3.rows_T | 6 | 对 |  |
| E01 | 调度记录:35 | hook-swap 第 91–104 行旧路径（课题 2 表里列第 91、94、95、99、101、104 行） | 读 base 提交 c840472 的 scripts/gate/hook-swap.py | 91、94、95、99、101、104 六行含旧式 scripts/*.sh 路径；selftest-registry 第 130 行是 lib_workspace_cargo_lock.py light | 对 |  |
| E02 | 调度记录:38 | du -sc 合计 527543072 KiB；16 个 cargo 各一个 w* 目录 | du -sc cross-run-cache/singlefs-crates-mutation-target* | 现在 1417166004 KiB；w* 目录 15 个（另有不带后缀的一个） | 复核不了 | 时点量，那一趟还在写；调研报告里 730940872 也是另一个时点 |
| E03 | 调度记录:39 | worker_count_for 取 min(16, 核数 // 2)；门禁文件头第 38 行同写 | 读 base 提交 | crates-mutation-rows.py 第 1855 行 by_processor = max(1, min(16, processor_count // 2))；门禁文件头第 38 行是「核数的一半、至多 16」 | 对 |  |
| E04 | 调度记录:39 | 31 份编译目录约 970 GiB（推的） | 算术 527543072 KiB / 16 × 31 | 974.7 GiB | 对 | 推的数，约数；基数是 16 份合计的一个时点 |
| E05 | 调度记录:42 | 事实表 27 行 | K:facts_rows | 27（facts.tsv 28 行含表头） | 对 |  |
| E06 | 调度记录:42 | 候选 353 行（247 个不同的行） | K:candidate_rows / candidate_distinct | 353；按载体去重 247 | 对 |  |
| E07 | 调度记录:42 | F1-F13 约 179 行、F14-F27 约 174 行 | K:candidates_F1_13 / F14_27 | 179、174（和 353） | 对 |  |
| E08 | 调度记录:62 | 盘点表 54 行 | K:inventory_rows | 54（产品代码 4、checker 档 8、门禁 6、脚本 27、显式参数 1、文字 8） | 对 |  |
| E09 | 调度记录:59 | 课题 0 后半：174 行：不改 101、不相干 63、要改 8、要补 2；反向问 3 | K:judge_b_tally；报告 sha 07ee6db7…7fac5 | 101、63、8、2（和 174）；sha 对 | 对 | 反向问 M1–M3 的 3 条没单独的表可数 |
| E10 | 调度记录:61 | 360 行处置：不改 321、改了 32、补了 7；原始证据 5 份 | K:sync_rows / sync_tally / sync_evidence_rows | 360；321、32、7；原始证据表 5 行 | 对 |  |
| E11 | 调度记录:64 | 写回员 checks-owed 17 处；milestone 03、D3、D23、D28、变更史 23 处 | 读两份 report.md | 17 条规格/17 处替换（1 个文件）；23 条/23 处（5 个文件） | 对 |  |
| E12 | 调度记录:64 | 24 处编号缺简称、153 段补了 23 段、整仓 doc-lint 红 4 处、写回 0 行 | 无产物 | — | 复核不了 | 只有主 agent 当时的文字；没有保存 doc-lint 输出 |
| E13 | 调度记录:66 | singlefs-53 提到 12-twelfth-txn.md 26 次、49 分钟前起 | 无（会话记录不归我读） | — | 复核不了 |  |
| E14 | 调度记录:68 | hook-swap 8 条喂法；--dry-run 22 条全对；旧版 4 条红；+159/−22 | 读 b2-guard 的 patch 与 report | hook-swap.py +8/−8；报告写喂了 22 条全对、旧版四条红；补丁 4 份 +159/−22 | 对 |  |
| E15 | 调度记录:69 | 共用调研：0.52 GiB、6.7 GiB（40.1 GiB 的 17%）、22.4、9.75、81–92%、每片 20–22 GiB、31 片 0.6–0.7 TiB | 读 report.md；算术 | 报告里都有；6.7/40.1 = 16.7%；31×20 = 0.61 TiB、31×22 = 0.67 TiB | 复核不了 | 是磁盘上量的数，现场已变；只核了报告与记录一致、算术成立 |
| E16 | 调度记录:69 | 根分区可用 962G；M2 轻型门禁 39 项 | 无 | — | 复核不了 | 时点量；39 项只是计划 |
| E17 | 调度记录:70 | 补丁 29 份 1248 行 | K:patch.b2-defaults | files=29，补丁文件 1248 行（+365/−102） | 对 | 1248 是补丁文件的行数，不是改动行数；与别处的「+a/−d」口径不同 |
| E18 | 调度记录:70 | 六个装置 include! 共用的 threads.rs | grep include! 补丁 | 补丁里 6 处 include!("heavy_threads_default/threads.rs") | 对 |  |
| E19 | 调度记录:72 | 丙补丁 30 份 +1221/−94；排除表 47 行；自补 18 行；SELFTEST_RUNNERS 加三条；登记两行、改三行理由 | K:patch.b2-threads；读补丁 | 30、+1221/−94；报告写 47 行、18 行；补丁里 SELFTEST_RUNNERS 新增 3 条；selftest-registry 补丁 +5/−3（新两行、改三行） | 对 |  |
| E20 | 调度记录:73 | 主工作区跟踪与未跟踪文件 13604 份 | K:int.base_files | 集成副本 base 提交 13604 个文件 | 对 |  |
| E21 | 调度记录:73 | 批 2 四份补丁 41 个文件 +1711/−247 | K:b2stack.*（临时 index 依次叠四份补丁） | 全部 68 个文件 +2550/−247；其中改动的 41 个 +1711/−247，新增的 27 个 +839 | 对不上 | 41/+1711 只算改动的文件，没算 27 个新增文件 |
| E22 | 调度记录:74 | verify 派 67 项、收 67 项、判错 0；23 份自证全绿、42 个新开关全红；取证两项 | K:verify.tally / total / wrong | 67；绿 23、红 42、取证 2；判错 0 | 对 |  |
| E23 | 调度记录:74 | 双机驱动 --selftest 53 格里 8 格判错 | K:verify1.two_hosts_* | 53 格里 8 格判错；8 个失败格名 | 对 |  |
| E24 | 调度记录:74 | 新格在集成后的真树上红 19 行 | K:verify1.cell_hdd_red_items | 19 | 对 |  |
| E24b | 调度记录:74 | 19 行里「弄坏开关自己的代码 6 行、真漏改 8 行、判法太宽把超时 600 当线程 1 行、单测期望式 1 行等」 | 读 verify-out/067 的 19 行清单 | 清单只有路径:行与原行，没有归类；6+8+1+1 = 16，「等」是另外 3 行 | 复核不了 | 归类是判断，不是计数；能核的只有 19 和 16 < 19 |
| E25 | 调度记录:74 | 抽 6 份开关日志看红在点名那一格 | 无 | — | 复核不了 | 抽样数，没写抽的是哪 6 份 |
| E26 | 调度记录:75 | 批 2 收尾 13 份 +154/−36；19 行 = 改代码 3 + 判法 1 + 排除表 15；排除表 47→62 | K:patch.b2-fixup、exempt_rows.b088a84 | 13、+154/−36；3+1+15 = 19；排除表 62 | 对 | 3、1 的归类是判断，只核了加法与 15 = 62−47 |
| E27 | 调度记录:76 | verify2：17 项 16 对；5 个开关红；run-with-memory-cap 72 项绿、两开关红 | K:verify2.* | 17 项、判错 1（cell-selftest-coverage）；开关红 8 = 5 + 2 + 1；rwmc 自检「查了 72 项」 | 对 |  |
| E28 | 调度记录:76 | verify2 时格：扫 831 份、可疑 90 处、表 62 行 | K:verify2.cell_hdd；run_htd_cell.py r1 冻结副本 | 831、90、62（r1 冻结副本重算同） | 对 |  |
| E29 | 调度记录:102 | 批 3 补丁 6 份 +25/−14；旧公式改前 5 行、改后 0 行 | K:patch.b3-defs；grep 同一组词（报告里的命令） | 6、+25/−14；base 5 行（commit-stages 1、main-agent 3、implementation-workflow 1）、before-b4 0 行 | 对 |  |
| E30 | 调度记录:102 | rules-lint 绿（17 份 1957 行） | 无（rules-lint 是提交时才跑的检查） | SOP 规则目录 17 份、wc -l 共 2004 行；.claude/rules 10 份 876 行 | 复核不了 | 17 与 SOP 规则 17 份同数，行数 2004 ≠ 1957，扫的目录与计行口径不明 |
| E31 | 调度记录:103 | doc-lint 滤掉「日期早于第一个提交」1370 行 | 无 | — | 复核不了 |  |
| E32 | 调度记录:104 | 正推腿报告 324 行；sha256 60ed02db…a8a8；cite-check 51/51 | K:leglines；cite-check.py 只 r1 正推报告 | 324 行；sha 末 4 位错（同 A01）；核了 51 处、对上 51 处 | 对不上 | 行数、51/51 对；sha 末 4 位 d8a8 不是 a8a8 |
| E33 | 调度记录:106 | 三个名字与共同前缀全仓 0 处 | probes/value_tree_hits.py（git 跟踪 + 未跟踪未忽略的 14412 份） | 配置里的两个值（第二台 SSH 别名、本机主机名）命中 0 份；第三个名字不在配置里 | 对 | 「三个名字」「共同前缀」没写出来是哪几个：第三个只能按文字信 |
| E34 | 调度记录:109 | abandoned-rounds.tsv 第 9 行 m3-batch63-code-r1、文件头写「门禁 66 号」 | 读 base 提交 | 第 9 行是 m3-batch63-code-r1，文件头含「门禁 66 号」 | 对 |  |
| E35 | 调度记录:110 | 批 4 丙：四个新开关；private-values 五个开关 | 补丁里新增开关名 | heavy-thread-defaults 4 个、private-values 5 个 | 对 |  |
| E36 | 调度记录:110 | 真配置在集成副本整仓比只中那两份旧行 3 行 | K:value_hits（行数） | 1004 一行 + 295、302 两行 = 3 行 | 对 |  |
| E37 | 调度记录:111 | runner-dispatch-guard.sh 第 247 行写「五个开关」，应为七个 | K:writer_lock_modes.* | before-b4：lib_writer_lock.py 5 个、钩子头写五个；after-b4：7 个、钩子头仍写五个；撤回 nearest-lock-only 后 after-b4-fixup：6 个、钩子头写六个 | 对 | 「七」是批 4 乙补丁那一版的数；撤回后现在是六，文件头已是六 |
| E38 | 调度记录:112 | 批 4 甲 12 个钩子开关；第 1595 行 | 补丁里新增 break_switch；读 records/2026-10-01-删除与并发事务.md 第 1595 行 | 12 个新开关；第 1595 行有轮名要的用户原话 | 对 |  |
| E39 | 调度记录:112 | 批 4 四份补丁打进集成副本 30 份 +1283/−237 | K:int.shortstat.before-b4..after-b4(.M/.A/.D) | 全部 52 个文件 +1425/−237；其中改动 29 个 +1283/−232、删除 1 个 −5（共 30 个 +1283/−237），新增 22 个 +142 | 对不上 | 30/+1283 不含 22 个新增文件；四份补丁自己的和 52 个文件、+1425 才对得上 |
| E40 | 调度记录:114 | 批 4 收尾 16 份；after-b4-fixup = 5c2c79b | K:int.shortstat；git rev-parse | 16 份 +57/−89；标签 after-b4-fixup 提交 5c2c79b | 对 |  |
| E41 | 调度记录:114 | verify3 涉及 16 项自证与门禁格、约 120 个弄坏开关 | K:verify3.tally | 绿 16、开关 117 | 对 | 「约 120」是约数，117 |
| E42 | 调度记录:115 | 第 14 项 code-tooling 六格五红 | K:verify3.cell_code_tooling | 6 格里 5 格红（research-script-selftests、research-script-selftest-coverage、stage-owners、fixture-claims、heavy-thread-defaults） | 对 |  |
| E42b | 调度记录:115 | fixture-claims 日期 122 + 950 处 | K:verify3.fixture_dates | 122 个文件名里的日期、950 处样本正文里的日期 | 对 |  |
| E42c | 调度记录:115 | 单独重跑 multi-host-run.sh 19 格、mutation-shard-run.sh 24 格 | K:verify3.parallel_selftests | 日志里 mutation-shard-run.sh 共 24 格（16 格判错）、multi-host-run.sh 共 19 格（10 格判错） | 对 | 总格数对；「单独重跑全对」的结果没有留存日志 |
| E42d | 调度记录:115 | crates-mutation-rows.py 自证 59 格 | 无（登记自证，不许跑；verify3 日志只有堆栈） | — | 复核不了 |  |
| E43 | 调度记录:115 | verify3 其余 129 项对：16 项自证与门禁格绿、约 113 个弄坏开关各自红 | K:verify3.tally | 129 = 绿 14 + 红 115 | 对不上 | 总数 129 对，拆分错（同 C05） |
| E44 | 调度记录:116 | 第二轮快照 30 份、kb 3 份、清单抄 48 不抄 6 | K:snapshot.r2.* / checklist.r2 | 30；kb 3；抄 48、不抄 6 | 对 |  |
| E45 | 调度记录:102 | 第一轮材料 1 份 kb、清单抄 32 不抄 2；快照 21 份 | K:checklist.r1 / snapshot.r1 | kb 1；抄 32、不抄 2；21 | 对 |  |
| E46 | 调度记录:117 | 攻方 14 个编号、改法 P1–P10、八行 same、八份样本 | K:r2opus.rows_hit_ids / rows_P | A1–A4、B1–B3、C1–C2、D1–D2、E1–E3 共 14 个；P 行 10 | 对 |  |
| E47 | 调度记录:118 | 辩方：41 条修到 33、部分 2、没修到 6，改过头 0；判轻 14、判重 1 | K:r2sonnet.table1_tally / table3_tally / section1_rows | 41 行；33、2、6；判轻 14、判重 1 | 对 | 「修到」含 5 行写「文字修到」的（钩子没改）；报告首段自己说明了 |
| E48 | 调度记录:118 | 判决二另立账八笔 | K:v2.pile_line_ids | 8 笔 | 对 |  |
| E49 | 调度记录:121 | 批 5 丁 9 份、乙 4 份、丙 50 份、甲 19 份 | K:patch.b5-* | 9、4、50、19 | 对 |  |
| E50 | 调度记录:122 | 批 5 四份补丁 82 份（无冲突） | K:int.shortstat.after-b4-fixup2..after-b5 | 82 个文件 +1175/−257（改动 23、新增 59） | 对 | 这一处的 82 是全部文件（含新增），批 4 那处的 30 不含，两处口径不同 |
| E51 | 调度记录:122 | 2026-10-12 之前的登记行 4 行 | 读 r3 冻结副本 research/prompts/abandoned-rounds.tsv | c196-r1、d16-item1-r2、c364-r3、smalldata-ttime-r1 四行（第 7–10 行） | 对 |  |
| E52 | 调度记录:123 | verify4：13 份自证与 4 道样本档绿，其余 147 项对 | K:verify4.tally；读 summary 项名 | 绿期望 17 项（判对 16、判错 1）；其中 stage-* 4 项、cell-* 2 项、其余 11 项（含判错 1、env-chdir 2）；开关 131 项全红 | 对不上 | 13 是非 stage 的绿期望项（含 2 个门禁格、含判错的那 1 项），判对的自证只有 10 份 |
| E54 | 调度记录:124 | 批 5 收尾：四份自证绿、四个开关各自红（types-before-nine 红 7 格） | verify5-out 日志 | dp、rdg、htg、lht 四份绿；rdg-anyref、htg-unrec、lht-unrec、dp-types 四个红；dp-types 7 个「✗ 自检」 | 对 |  |
| E55 | 调度记录:124 | 第三轮快照 35 份；kb 1 份、清单抄 32 不抄 2 | K:snapshot.r3 / checklist.r3 | 35；1；32、2 | 对 |  |
| P01 | 调度记录:109 | 批 4 丁补丁 4 份 +15/−15 | K:patch.b4-defs | files=4 +15/-15 | 对 |  |
| P02 | 调度记录:110 | 批 4 丙补丁 39 份 +488/−30 | K:patch.b4-gates | files=39 +488/-30 | 对 |  |
| P03 | 调度记录:111 | 批 4 乙补丁 4 份 +434/−41 | K:patch.b4-guard | files=4 +434/-41 | 对 |  |
| P04 | 调度记录:114 | 批 4 收尾补丁 16 份 | K:patch.b4-fixup | files=16 +57/-89 | 对 |  |
| P05 | 调度记录:116 | 批 4 收尾二补丁加 4 行 | K:patch.b4-fixup2 | files=1 +4/-0 | 对 |  |
| P06 | 调度记录:121 | 批 5 丁 9 份文字；乙 4 份（多改 lib_shell_words.py 与登记表一行注释）；丙 50 份 | K:patch.b5-defs/-guard/-gates；K:patchfile.b5-guard.* | 9 份 +19/−11；4 份 +176/−39（lib_shell_words.py +50/−11、selftest-registry.tsv +1/−1）；50 份 +456/−60 | 对 |  |
| P07 | 调度记录:122 | 批 5 甲 19 份，新库 lib_abandoned_rounds.py | K:patch.b5-dispatch；K:patchfile.b5-dispatch.lib_abandoned_rounds.py | 19 份 +524/−147；lib_abandoned_rounds.py 是新文件 254 行 | 对 |  |
| P08 | 调度记录:124 | 批 5 收尾 6 份，多改 lib_heavy_tests.py 文件头 8 行注释 | K:patch.b5-fixup；K:patchfile.b5-fixup.lib_heavy_tests.py | 6 份 +89/−19；lib_heavy_tests.py +5/−3（合计动 8 行） | 对 | 「8 行」是增 5 删 3 的合计，没写口径 |

### 1.1 sha256 前缀…末几位，逐个对真文件

记录与判决里「sha256 xxxxxxxx…yyyy」一类的写法，前 8 位在真文件里找到的，逐个比末几位（K:shaclaim.*）。记录里只写了补丁/报告其一的，另一份不在此表。

| 编号 | 文件 | 记录/判决里写的位置与末位 | 现算末位 | 判 |
|---|---|---|---|---|
| H01 | b2-guard/patch.diff | 调度记录:68 写「…6761」 | 末位相同 | 对 |
| H02 | b2-guard/report.md | 调度记录:68 写「…53ad」 | 末位相同 | 对 |
| H03 | b2-defaults/patch.diff | 调度记录:70 写「…11c3」 | 末位相同 | 对 |
| H04 | b2-defaults/report.md | 调度记录:70 写「…808e」 | 末位相同 | 对 |
| H05 | b2-threads/patch.diff | 调度记录:72 写「…72de」 | 末位相同 | 对 |
| H06 | b2-threads/report.md | 调度记录:72 写「…ccabd」 | 末位相同 | 对 |
| H07 | b2-dispatch/report.md | 调度记录:73 写「…1578」 | 末位相同 | 对 |
| H08 | b2-fixup/patch.diff | 调度记录:75 写「…81ec」 | 末位相同 | 对 |
| H09 | b2-fixup/report.md | 调度记录:75 写「…20f3」 | 末位相同 | 对 |
| H10 | b3-defs/patch.diff | 调度记录:102 写「…e93f」 | 末位相同 | 对 |
| H11 | b3-defs/report.md | 调度记录:102 写「…8a58」 | 末位相同 | 对 |
| H12 | b4-defs/patch.diff | 调度记录:109 写「…5c49」 | 末位相同 | 对 |
| H13 | b4-defs/report.md | 调度记录:109 写「…31a1cf」 | 末位相同 | 对 |
| H14 | b4-gates/patch.diff | 调度记录:110 写「…efe」 | 末位相同 | 对 |
| H15 | b4-gates/report.md | 调度记录:110 写「…497a」 | 末位相同 | 对 |
| H16 | b4-guard/patch.diff | 调度记录:111 写「…c1e4」 | 末位相同 | 对 |
| H17 | b4-guard/report.md | 调度记录:111 写「…fd73」 | 末位相同 | 对 |
| H18 | b4-dispatch/patch.diff | 调度记录:112 写「…6535」 | 末位相同 | 对 |
| H19 | b4-dispatch/report.md | 调度记录:112 写「…e270」 | 末位相同 | 对 |
| H20 | b4-fixup/patch.diff | 调度记录:114 写「…f74f」 | 末位相同 | 对 |
| H21 | b4-fixup/report.md | 调度记录:114 写「…78d5」 | 末位相同 | 对 |
| H22 | b4-fixup2/patch.diff | 调度记录:116 写「…8549」 | 末位相同 | 对 |
| H23 | b4-fixup2/report.md | 调度记录:116 写「…b0ab」 | 末位相同 | 对 |
| H24 | b5-defs/patch.diff | 调度记录:121 写「…a6f9」 | 末位相同 | 对 |
| H25 | b5-guard/patch.diff | 调度记录:121 写「…cb40」 | 末位相同 | 对 |
| H26 | b5-gates/patch.diff | 调度记录:121 写「…408d」 | 末位相同 | 对 |
| H27 | b5-dispatch/patch.diff | 调度记录:122 写「…c8b9」 | 末位相同 | 对 |
| H28 | b5-fixup/patch.diff | 调度记录:124 写「…480b」 | 末位相同 | 对 |
| H29 | m3-tooling-b1-sync-judge-b/report.md | 调度记录:59 写「…7fac5」 | 末位相同 | 对 |
| H30 | m3-tooling-b2-mutation-share/report.md | 调度记录:69 写「…6fb5」 | 末位相同 | 对 |
| H31 | m3-defs-b3-r1-sonnet-output.md | 调度记录:104 写「…a8a8」 | 末位不同 | 对不上 |
| H32 | m3-defs-b3-r1-sonnet-output.md | 判决一:5 写「…a8a8」 | 末位不同 | 对不上 |
| H33 | m3-defs-b3-r1-opus-output.md | 判决一:5 写「…6645」 | 末位相同 | 对 |
| H34 | m3-defs-b3-r2-sonnet-output.md | 调度记录:118 写「…1fec」 | 末位相同 | 对 |
| H35 | m3-defs-b3-r2-sonnet-output.md | 判决二:5 写「…1fec」 | 末位相同 | 对 |
| H36 | m3-defs-b3-r2-opus-output.md | 调度记录:117 写「…9126」 | 末位相同 | 对 |
| H37 | m3-defs-b3-r2-opus-output.md | 判决二:5 写「…9126」 | 末位相同 | 对 |

### 1.2 对不上的八处，两边并排

左边是判决、正文或调度记录里的那一句，右边是现算输出里的那一行（`out/count_checks.txt` 的行）。

| 编号 | 引的一句 | 现算的一句 |
|---|---|---|
| A01 / E32 | 「sha256 60ed02db…a8a8」（`research/prompts/m3-defs-b3-r1-main-verification.md:5`）；「sha256 60ed02db…a8a8」（`records/2026-10-12-里程碑三工具与治理批.md:104`） | 「60ed02db...d8a8」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:13`） |
| B18 | 「攻方九种写法、辩方 S10、S11，0→2」（`research/prompts/m3-defs-b3-r2-main-verification.md:28`） | 「r2opus.r5_p_and_bench_cases=9」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:131`）、「r2opus.r5_cases_0to2=8」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:132`）、「p-env-C」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:133`） |
| C05 / E43 | 「16 项自证与门禁格、约 113 个弄坏开关按期望判」（`research/prompts/_m3-defs-b3-r2-body.md:30`）；「16 项自证与门禁格绿、约 113 个弄坏开关各自红」（`records/2026-10-12-里程碑三工具与治理批.md:115`） | 「('其他', 'green', '对'): 14」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:103`）、「('开关', 'red', '对'): 115」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:103`） |
| D03 | 「可疑写法 94 处」（`research/prompts/_m3-defs-b3-r3-body.md:22`） | 「✗ 1 处自己定并行度默认值的写法没经登记位取」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:217`）、「scripts/agents/dispatch-prompt.py:262」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:218`） |
| D06 | 「records 能当出处的 504 行」（`research/prompts/_m3-defs-b3-r3-body.md:22`） | 「source_lines=557」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:171`）、「source_lines_without_halt_words=504」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:171`） |
| E21 | 「41 个文件 +1711/−247」（`records/2026-10-12-里程碑三工具与治理批.md:73`） | 「68 files changed, 2550 insertions(+), 247 deletions(-)」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:81`） |
| E39 | 「30 份 +1283/−237」（`records/2026-10-12-里程碑三工具与治理批.md:112`） | 「52 files changed, 1425 insertions(+), 237 deletions(-)」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:60`）、「22 files changed, 142 insertions(+)」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:62`） |
| E52 | 「13 份自证与 4 道样本档绿」（`records/2026-10-12-里程碑三工具与治理批.md:123`） | 「('其他', 'green', '对'): 16」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:106`）、「('其他', 'green', '错'): 1」（`research/prompts/m3-defs-b3-r3-sonnet-model/out/count_checks.txt:106`） |

## 2 T5　开着的线：四条腿报告里打中的每一条的去向

三类与改法表的位置（行号现查）：第一轮 改法 G1–G11 判决一:34–44；另立账 判决一:46（compile-then-swap、「快照冲突已判」）；交用户 判决一:50–52（H1–H3）；核不动 判决一:16（GPU「本次未跑」、重启后旧进程回执、看门狗「已失效」）。第二轮 改法 K1–K11 判决二:38–48；另立账 判决二:50（八笔，加第一轮两笔 S21、O09）；交用户 判决二:54（H4）；核不动 判决二:18（J07、C1 后果）。三类之外还有一份「还没改」名单：正文二第二节（正文二:29–32）与正文三第二节（正文三:26–28），下表标「名单外有去向」。

去向类：改法 / 另立账 / 交用户 / 核不动 / 名单外有去向（三类里没有，正文第二节、排除表或调度记录里有）/ 无去向（两轮都没写）/ 撤回或判重。腿报告里判「一致」「没打中」「构造不出反例」的行不是打中，不列。「去向出处」一列里的括注是那一处的编号。

### 2.1 逐条去向表（四条腿）

腿报告位置一列：一轮正推 `research/prompts/m3-defs-b3-r1-sonnet-output.md`，一轮攻方 `…r1-opus-output.md`，二轮辩方 `…r2-sonnet-output.md`，二轮攻方 `…r2-opus-output.md`。

T5-01 至 T5-31 是一轮正推，T5-32 至 T5-47 一轮攻方，T5-48 至 T5-77 二轮辩方，T5-78 至 T5-94 二轮攻方。表里的 U01 至 U08 在 §2.2。

| 编号 | 腿报告位置 | 打中的历史 | 去向类 | 去向出处 | 备注 |
|---|---|---|---|---|---|
| T5-01 | 一轮正推:26 | E161 装置只读 E161_THREADS，capped.sh N 到不了它 | 改法 | 判决一:36（G3）；判决二:42（K5） |  |
| T5-02 | 一轮正推:27、一轮攻方:119 | 变异门禁 32 核机器上只开 16 个工作进程 | 名单外有去向 | 判决一:38（G5）；正文二:31（正文二第二节） | 判决里只有 G5「写明去哪查」，真正的改（并进依赖产物共用）在正文第二节与调度记录 L58，不在判决三类里 |
| T5-03 | 一轮正推:28 | E171 按此刻空闲线程取小（忙时 0），与「不看忙闲」不一致 | 名单外有去向 | 判决一:38（G5）；正文二:31（正文二第二节）；调度记录:75（调度记录） | 同上：判决只有 G5；固定 31 归 E171 下一次修订，只在正文与调度记录 |
| T5-04 | 一轮正推:29 | gpu 那一趟 RUST_TEST_THREADS=31、无 --test-threads 1 | 改法 | 判决一:37（G4）；判决二:43（K6） |  |
| T5-05 | 一轮正推:30、一轮正推:31 | capped.sh 8 与提示里 31 并存、命令不经 capped.sh，闸都放行 | 改法（文字半）；钩子半无去向 | 判决一:35（G2） | G2 只改共用约束的文字；重型测试闸与派发闸都不比 N、不要求经 capped.sh（见 U01） |
| T5-06 | 一轮正推:32 | SINGLEFS_THREAD_CAP 已设时 capped.sh default 取它，不取登记值 | 改法 + 另立账 | 判决一:35（G2）；判决二:50（第二轮另立账） | 第二轮 J12 另立账 |
| T5-07 | 一轮正推:59 | 用户临时让出 CPU：没有规则管在跑的子 agent | 交用户 | 判决一:52（H3）；判决二:41（K4） | H3 已定「在跑的不动」；K4 把「让出」限定成明说 |
| T5-08 | 一轮正推:68 | ⑮ 放过 31 ÷ 2、31/2、31 的一半；不写这一行不判 | 改法 | 判决一:34（G1）；判决二:38（K1）；判决二:39（K2） |  |
| T5-09 | 一轮正推:90 | gpu 一趟与 verification 第 59 行不一致 | 改法 | 判决一:37（G4）；判决二:43（K6） |  |
| T5-10 | 一轮正推:96 | 带 gpu 的一趟是 --lib 加多个目标，--test-threads 1 让 CPU 用例也串行，挂钟没量 | 核不动 | 判决二:18（J07 提交时量挂钟）；判决二:18（第二轮核不动） |  |
| T5-11 | 一轮正推:128、一轮正推:130 | -p 点名独立探针放行；bench 不判；直接执行探针编出来的二进制罩不住 | 改法 | 判决一:40（G7） |  |
| T5-12 | 一轮正推:131 | 虚拟工作区根 cargo run 不判，共用约束文字没写 | 无去向 | 判决一:40（G7） | G7 只写明「直接执行二进制」「脚本里起的 cargo」两样不判，没有这一样（U02） |
| T5-13 | 一轮正推:168 | 合入锁放行的子命令只有六个，cargo metadata/tree/fmt 被拒；文字没说是哪几个 | 另立账 | 判决二:50（J10） |  |
| T5-14 | 一轮正推:169 | 合入锁不读脚本正文，脚本里的 cargo 不判 | 改法 | 判决一:40（G7） |  |
| T5-15 | 一轮正推:170 | cp -r 把 .git 带进副本，副本里 cargo 照被锁拒 | 另立账 | 判决二:50（J10） |  |
| T5-16 | 一轮正推:181 | 嵌套草稿目录里 A 写 E 的子目录被拒、E 写父目录被拒 | 改法（撤回后只补文字） | 判决一:41（G8）；判决二:29（R6 站得住） | 判决一原把它读成要改钩子，主 agent 更正撤回 |
| T5-17 | 一轮正推:191 | lib_writer_lock.py 第 4 行残留 ⑮ | 改法 | 判决一:41（G8） |  |
| T5-18 | 一轮正推:214 | 8 份只有 Bash 的定义「开工先认锁」无从触发 | 改法 | 判决一:41（G8） |  |
| T5-19 | 一轮正推:215 | 回执找不到时先到先得，比文字强 | 改法（文字）+ 另立账（钩子） | 判决一:41（G8）；判决二:50（J11） |  |
| T5-20 | 一轮正推:216 | 重启后的旧进程回执是否还在记录里 | 核不动 | 判决一:16（第一轮核不动）；判决二:50（J11） |  |
| T5-21 | 一轮正推:217 | 回执对不上时拒绝说明不点名持锁者 | 改法 | 判决一:41（G8） |  |
| T5-22 | 一轮正推:228 | 看门狗「有子 agent 交回时」漏写「被停」也算 | 改法 | 判决一:42（G9） |  |
| T5-23 | 一轮正推:229 | 看门狗「已失效」判定没喂，只读码 | 核不动 | 判决一:16（第一轮核不动） |  |
| T5-24 | 一轮正推:247 | halted 引自己、引无关一句也把还要判的轮关掉 | 改法 | 判决一:43（G10）；判决二:45（K8） |  |
| T5-25 | 一轮正推:265 | 「快照冲突已判」理由非空就放行 | 另立账 | 判决一:46（第一轮另立账）；判决二:50（S21 照旧） |  |
| T5-26 | 一轮正推:268 | ⑥ 的「」括、6 个字、同一行三条要求文字没写 | 改法 | 判决一:43（G10） |  |
| T5-27 | 一轮正推:277 | halted 与 abandoned-rounds.tsv 同一件事两处登记 | 改法 | 判决一:43（G10）；判决二:46（K9） |  |
| T5-28 | 一轮正推:295 | 多机行「每台线程现算」与固定 31 二义 | 改法 | 判决一:44（G11） |  |
| T5-29 | 一轮正推:296 | ⑭ 只判设计员与执行员 | 改法 | 判决一:44（G11）；判决二:39（K2） |  |
| T5-30 | 一轮正推:297 | 写死台数与额度只在多机行按数字判 | 另立账 | 判决一:28（判决一 D7 行）；判决二:50（S26） |  |
| T5-31 | 一轮正推:322 | 前台派发的子 agent 在主会话记录里有没有 async_launched 回执（没有则写者锁全是先到先得） | 无去向 | 判决一:16（第一轮核不动） | 判决一核不动只列重启后旧进程那一条；前台派发这一问没人答也没列（U03） |
| T5-32 | 一轮攻方:51 | SINGLEFS_THREAD_CAP 设了时 capped.sh N 与 capped.sh default 交不同的数 | 改法 + 另立账 | 判决一:35（G2）；判决二:50（第二轮另立账） |  |
| T5-33 | 一轮攻方:92 | 用户要让出 CPU，在跑的子 agent 没有路降线程 | 交用户 | 判决一:52（H3）；判决二:41（K4） |  |
| T5-34 | 一轮攻方:102 | ⑮ 放过公式、第二台另一个数、不写这一行 | 改法 | 判决一:34（G1）；判决二:38（K1） |  |
| T5-35 | 一轮攻方:118 | GPU 对拍 CPU 一侧读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，不在 capped.sh 清单 | 名单外有去向 | 判决二:26（J06 / O04）；判决二:43（K6） | 归代码批；排除表代码批行承接；K6 只改注释文字 |
| T5-36 | 一轮攻方:123 | 轻型 --quick 与门禁 gpu 一趟让 GPU 测试按 31 个线程跑 | 改法 | 判决一:37（G4）；判决二:43（K6） |  |
| T5-37 | 一轮攻方:141 | 借不到显存时报「本次未跑」不判红 | 核不动 | 判决一:16（第一轮核不动） |  |
| T5-38 | 一轮攻方:145 | 「配置里的机器与卡都用」与单个 GPU 测试二进制只用本机点名的卡 | 改法 | 判决一:39（G6） | 辩方 J13 判重（弱） |
| T5-39 | 一轮攻方:151 | 「20 万态量级」没写单位、与「挂钟长不是收窄它的理由」拉扯 | 改法 + 另立账 | 判决一:39（G6）；判决二:50（J08 后半） |  |
| T5-40 | 一轮攻方:163 | 照现状哈希登记，别人没审过的代码改动拿到豁免 | 交用户 | 判决一:51（H2） | H2 已定乙（不登记）；门禁层没改（见 U04） |
| T5-41 | 一轮攻方:193、二轮辩方:53 | compile-then-swap 只核开跑之后的改动 | 另立账 | 判决一:46（第一轮另立账）；判决二:50（O09 照旧） |  |
| T5-42 | 一轮攻方:209 | 仓里已有两处私有值，已推到远端 | 交用户 | 判决一:50（H1）；判决二:16（判决二 L16 现查工作树 0） | H1 定「只改工作区，历史不动」：HEAD 与 origin/master 仍带（今天现查），工作树已换、在暂存区没提交 |
| T5-43 | 一轮攻方:234 | 射程只写第二台；与「原样输出」「整行抄原话」「证据不许事后改」三条撞 | 交用户 + 另立账 | 判决一:50（H1）；判决二:50（J14） |  |
| T5-44 | 一轮攻方:245 | 能做成只判新加行、按长词的会红检查 | 改法 | 判决一:25（判决一 D4 行）；判决二:47（K10） | 做成 private-values 格；词源加 ~/.config/singlefs/ 一半没并入（见 U05） |
| T5-45 | 一轮攻方:278 | halted 自引/引正文/引规则关掉还要判的轮 | 改法 | 判决一:43（G10）；判决二:45（K8） |  |
| T5-46 | 一轮攻方:295 | 出处写 research/../../ 指到仓外 ⑥ 照收 | 改法 | 判决一:43（G10） | 辩方 S20 修到 |
| T5-47 | 一轮攻方:299 | halted 与 abandoned-rounds.tsv 两处登记各管一半 | 改法 | 判决一:43（G10）；判决二:46（K9） |  |
| T5-48 | 二轮辩方:20 | E171 按忙闲取小没修到（预期内） | 名单外有去向 | 正文二:31（正文二第二节） | 同 T5-03 |
| T5-49 | 二轮辩方:21 | 变异门禁 16 个进程没修到（预期内） | 名单外有去向 | 正文二:31（正文二第二节） | 同 T5-02 |
| T5-50 | 二轮辩方:22 | gpu 一趟 --test-threads 1 修到单机这一趟；J05–J07 另记 | 改法 | 判决二:43（K6） |  |
| T5-51 | 二轮辩方:23 | 子 agent 命令不经 capped.sh、capped.sh 8 与提示 31 并存：文字修到，钩子照旧 | 无去向 | — | 同 T5-05 的钩子半（U01） |
| T5-52 | 二轮辩方:24 | SINGLEFS_THREAD_CAP 已设时 default 取它 | 另立账 | 判决二:50（J12） |  |
| T5-53 | 二轮辩方:29 | bench 修到；直接执行二进制与虚拟根 run 文字改成「不判」 | 无去向（虚拟根一样） | — | agent-common.md 写了直接执行与脚本里 cargo 两样，没写虚拟根（U02） |
| T5-54 | 二轮辩方:30 | 合入锁子命令集合二义与 cp -r 带 .git：没改 | 另立账 | 判决二:50（J10） |  |
| T5-55 | 二轮辩方:34 | 先到先得钩子行为没改，旧进程仍可先认领 | 另立账 | 判决二:50（J11） |  |
| T5-56 | 二轮辩方:39 | 「快照冲突已判」理由非空即放行 | 另立账 | 判决二:50（S21 照旧） |  |
| T5-57 | 二轮辩方:44 | 写死额度只在多机行按数字判 | 另立账 | 判决二:50（S26） |  |
| T5-58 | 二轮辩方:45 | 环境变量通道被 ⑮ 与骨架忽略（部分修到） | 另立账 | 判决二:50（J12） |  |
| T5-59 | 二轮辩方:48 | GPU 对拍 CPU 一侧变量没修到；新写的注释失实 | 改法 | 判决二:43（K6） | 变量本身归代码批 |
| T5-60 | 二轮辩方:49 | 双机分支、crash-case、崩溃放量没带 --test-threads 1，规则写成无例外 | 改法 + 名单外有去向 | 判决二:43（K6）；正文三:28（正文三第二节「还没改」） | 批 5 丙：双机分支做不成（调度记录 L121） |
| T5-61 | 二轮辩方:52 | 门禁判不出「别人没审过」，登记哈希的门禁输出没变 | 交用户（文字）；门禁层无去向 | 判决一:51（H2） | （U04） |
| T5-62 | 二轮辩方:54 | 冻结副本里两处私有值仍在 | 交用户 | 判决一:50（H1）；判决二:16（判决二 L16） |  |
| T5-63 | 二轮辩方:100 | investigator、tooling-writer 定义「输入」没写线程上限一行 | 改法 | 判决二:40（K3） |  |
| T5-64 | 二轮辩方:101 | 「线程上限：<登记值>」非 <填 尖括号被当占位 | 改法 | 判决二:38（K1） |  |
| T5-65 | 二轮辩方:102 | 「线程上限已判」理由非空就放行 | 另立账 | 判决二:24（J03） |  |
| T5-66 | 二轮辩方:103 | clerk、三方正推腿/辩方腿定义许编译却不在类型表 | 改法 | 判决二:39（K2） |  |
| T5-67 | 二轮辩方:104 | 规则与文件头写成无例外，双机分支等没带 | 改法 | 判决二:43（K6） |  |
| T5-68 | 二轮辩方:105 | CPU 一侧注释失实（读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取核数） | 改法 | 判决二:43（K6） |  |
| T5-69 | 二轮辩方:106 | --lib 与 16 份文件纯 CPU 用例一起串行，挂钟没量 | 核不动 | 判决二:18（第二轮核不动） |  |
| T5-70 | 二轮辩方:107 | verification 第 60 行前半与第 59 行读法歧义；harness 档 20 万态怎么压 | 改法 + 另立账 | 判决二:43（K6）；判决二:50（J08 后半） |  |
| T5-71 | 二轮辩方:108 | H3「在跑的不发消息」与「派出去之后」两句发消息的条款并存 | 改法 | 判决二:41（K4） |  |
| T5-72 | 二轮辩方:109 | 合入锁子命令集合、cp -r 带 .git 第一轮判决没列进改法 | 另立账 | 判决二:50（J10） |  |
| T5-73 | 二轮辩方:110 | 回执不在记录里时先到先得没改 | 另立账 | 判决二:50（J11） |  |
| T5-74 | 二轮辩方:111 | 环境变量通道：⑮ 与骨架忽略 SINGLEFS_THREAD_CAP | 另立账 | 判决二:50（J12） |  |
| T5-75 | 二轮辩方:112 | （判重，弱）B2 原句在「不压并发」一条里本来指驱动 | 撤回或判重 | — | 判决二没提；无需去向 |
| T5-76 | 二轮辩方:113 | 上游证据规矩「一个字都不改」与共用约束私有值那一条两层各说各的 | 另立账 | 判决二:50（J14） |  |
| T5-77 | 二轮辩方:114 | 括注写中文数字或比例词放行 | 改法 | 判决二:38（K1） |  |
| T5-78 | 二轮攻方:140 | 括注里写中文数字或比例词 ⑮ 放行 | 改法 | 判决二:38（K1） |  |
| T5-79 | 二轮攻方:152 | 非 <填 的尖括号占位绕过缺行拒 | 改法 | 判决二:38（K1） |  |
| T5-80 | 二轮攻方:160 | clerk、three-way-defense 不写线程上限，三处文字三种做法 | 改法 | 判决二:39（K2） |  |
| T5-81 | 二轮攻方:183 | 「机器忙也固定」与「让出 CPU 写别的数」同一行，主 agent 分不出 | 改法 | 判决二:41（K4） |  |
| T5-82 | 二轮攻方:191 | 第二台另起一行但标签不在行首（第二台线程上限：16）⑮ 不管 | 无去向 | — | 报告自己标「不另算打中」；判决二没提（U06） |
| T5-83 | 二轮攻方:215 | cfg(test) 之后整份 break；回退按整份文件找字样 | 改法 | 判决二:42（K5） |  |
| T5-84 | 二轮攻方:241 | 命令前缀赋值、-j N、--test-threads N、:=N 格一种都不认；replay.sh 22 行 | 另立账 | 判决二:50（B3） |  |
| T5-85 | 二轮攻方:211 | E175 装置源码在 research/results 下，扫描目录看不到 | 无去向 | — | 报告标「后果只是 capped.sh N 到不了它」；判决二没提（U06） |
| T5-86 | 二轮攻方:283 | 仓副本里 light 自证正文改成起 checker 档测试照放 | 交用户 | 判决二:54（H4）；判决二:48（K11） | H4 定乙「收回放宽」 |
| T5-87 | 二轮攻方:300 | env -C 换目录起 cargo 闸按 JSON cwd 判，放行 | 改法 | 判决二:44（K7） | 判决二 R5 行写「另立账、并进下一批工具活」 |
| T5-88 | 二轮攻方:340 | 同一行别的轮的叫停原话、开轮行、引规则行、「先放一放」都收 | 改法 + 名单外有去向 | 判决二:45（K8）；判决二:30（判决二 R7 行：由主 agent 核） |  |
| T5-89 | 二轮攻方:362 | ⑥ 与门禁 abandoned-rounds 格判法不一 | 改法 | 判决二:46（K9） |  |
| T5-90 | 二轮攻方:380 | 未跟踪文件只读前 8 MiB | 改法 | 判决二:47（K10） |  |
| T5-91 | 二轮攻方:388 | 文件名里的值不判；文件名与内容都带值时红行把值打出来 | 改法 | 判决二:47（K10） |  |
| T5-92 | 二轮攻方:394 | 词源不读 ~/.config/singlefs/ | 改法（条件并入）；结果无去向 | 判决二:47（K10）；调度记录:121（调度记录：丙量出 19835 行，没并入） | 判决二写「E3 照 K10 验收量了再并」；量了不并，之后判决与正文三都没写这条还开着（U05） |
| T5-93 | 二轮攻方:443 | 喂钩子入口时没带主会话记录，⑦ 并发、⑯ 写者锁、回执没参与 | 无去向 | — | R1、R5、R7 的结论没覆盖这三样联动（U07） |
| T5-94 | 二轮攻方:447 | 每格只抽一次样；没打中的格按规则要另抽一次才算没打中 | 无去向 | — | 三方规则要求；R4、R6、R8 由正文三 T2–T4 补第二条腿，R2、R5、R7、R9 的「没打中」没有第二抽（U08） |

统计：94 行里 75 行的去向全在三类或改法表里；7 行（T5-02、T5-03、T5-35、T5-48、T5-49、T5-60、T5-88）的去向在三类之外的正文第二节、排除表或判决的格行里；11 行（T5-05、T5-12、T5-31、T5-51、T5-53、T5-61、T5-82、T5-85、T5-92、T5-93、T5-94）至少一部分没有去向，归并成 U01–U08（75 行里 T5-40、T5-44 另有残余记在 U04、U05）；1 行（T5-75）是辩方自己判重，不需要去向。「去向类」是我逐行判的，不是机器算的；推翻条件：判决里该编号的那一行读不出表里写的去向。

### 2.2 两轮都没写去向的（U01–U08）

| 编号 | 打中的历史 | 腿报告位置 | 两轮判决的状态 | 现查证据 | 建议归类 |
|---|---|---|---|---|---|
| U01 | 子 agent 手起命令：`capped.sh 8`（提示里线程上限是 31）或不经 capped.sh 起 cargo，两道闸都放行；G2 只改了共用约束的文字 | 一轮正推:30–31；二轮辩方:23 | 判决一:35 只有 G2 的文字；判决二没提 | 重型测试闸入口喂四条命令（out/feed_guard_n.txt）：capped.sh 31 退 0、capped.sh 8 退 0、只经内存包装退 0、什么包装都没有退 2；heavy-test-guard.sh 与 lib_heavy_tests.py 里「线程上限」各 0 处 | 补进另立账；改前的 G2 之前同样不比，不否这次改动 |
| U02 | 虚拟工作区根上不带 -p 的 cargo run 不判，共用约束文字没写这一样 | 一轮正推:131；二轮辩方:29 | 判决一:26 写「文字没写」，G7（判决一:40）的改法没含它 | 「直接执行它编出来的二进制、脚本里起的 cargo」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/agent-common.md:81`）只列两样；agent-common.md、main-agent.md、rules 里「虚拟」0 处，只在 lib_heavy_tests.py 第 64 行文件头 | 补进另立账，或并进 K7 那批工具活的文字 |
| U03 | 前台派发的子 agent 在主会话记录里有没有 async_launched 回执（没有则写者锁全是先到先得） | 一轮正推:322 | 判决一:16 核不动只列「重启后旧进程回执」；判决二没提 | 我也核不了：要真实主会话记录 | 补进核不动 |
| U04 | 门禁层不识别「别人没审过」：按现状哈希登记照样放，H2 乙只靠文字 | 一轮攻方:163；二轮辩方:52 | H2 已定乙（判决一:51）；门禁代码没改，判决二没把「门禁仍放」列为开着 | 辩方 O08：攻方 d3 探针的输出改前改后逐字相同（模型 out/opus-r1-d3-on-before.txt 与 on-after.txt 同 sha256 e83c7d17…） | 交用户确认 H2 是否就是只靠文字 |
| U05 | 词源加 ~/.config/singlefs/ 没并入，状态只在调度记录 | 一轮攻方:245；二轮攻方:394 | 判决二:47 写「E3 照 K10 验收量了再并」；调度记录:121 量出新增 19835 行不并 | 现算 19844 行 / 2232 份（D08）；判决二、正文三都没写它还开着 | 补进交用户表（零轮）或另立账 |
| U06 | 报告标「不算打中」的两处：第二台另起一行、标签不在行首的「线程上限」行不判；E175 装置源码在 research/results 下，扫描目录看不到 | 二轮攻方:191、211 | 判决二没提 | 报告自己写了「按文字没写到记」「后果只是 capped.sh N 到不了它」 | 备查 |
| U07 | 攻方喂钩子入口时没带主会话记录，⑦ 并发、⑯ 写者锁、回执三样没参与 | 二轮攻方:443 | 判决二没提 | 报告「这条腿自己的限度」一节自己写了 | 备查：R1、R5、R7 的结论没覆盖这三样的联动 |
| U08 | 「没打中」没有第二抽 | 二轮攻方:447 | 判决二没提 | 「至少再抽一次」（`/home/fy5090/.local/share/singlefs/work/m3-defs-b3-r3/repo/.claude/rules/three-way-inference.md:153`）；R4、R6、R8 由正文三 T2–T4 的攻方补；R2、R5、R7、R9 里「没打中」的小项（如 R9 退 77）没有第二抽 | 备查 |

### 2.3 调度记录里开着、两轮判决与正文三也没写去向的线（W01–W12）

这一节超出「腿报告里打中的」，是对着正文 T5「合起来是不是这一批留下的全部开着的线」补的。

| 编号 | 开着的线 | 调度记录 | 判决/正文三的状态 | 证据 | 建议 |
|---|---|---|---|---|---|
| W01 | heavy-thread-defaults 格在第三轮冻结副本上红：scripts/agents/dispatch-prompt.py:262 读自己的线程变量 THREAD_LIMIT_REQUIRED_AGENTS。这一行是批 5 收尾加的（after-b5 绿、after-b5-fixup 红） | :124 批 5 收尾只复跑四份自证，没复跑这一格 | 都没写；正文三 L22 引的 94/66/831 是这一格在 after-b4-fixup2 转绿时的数 | out/htd_cell_r3_frozen.txt；K:htd_cell.tag_after-b5（832/94/66 绿）与 K:htd_cell.tag_after-b5-fixup（红 1 处）；`git -C <集成副本> diff after-b5 after-b5-fixup -- scripts/agents/dispatch-prompt.py` 里新增该行 | 主 agent 在冻结副本跑格坐实（§3 第 1 条）；再定登记进排除表（涨一行，撞「只缩不涨」）还是改写法 |
| W02 | 本来就红、记为「不是这一批的」：research-script-selftest-coverage 三份脚本没人跑；stage-owners 缺 doc-text 的 session-names 一行；fixture-claims 的 layer0 两格缺红样本 | :115 | 都没写 | verify3 的 code-tooling 格 6 格 5 红（K:verify3.cell_code_tooling）；「主工作区同样如此」没有产物可核 | 补进另立账，标「改前就红」 |
| W03 | 变异门禁依赖产物共用的六条岔路，等两台 s2 跑完量代价（M1–M3）再弹窗 | :69 | 正文二:31、正文三:28 只写「变异门禁 worker_count_for 等」 | 调研报告里六条岔路与数的一致性见 E15 | 补进交用户（带「代价没量」） |
| W04 | admission.py 合入后判法摘要变，崩溃枚举用例现有全绿标记全部不作数，下一趟 --full 逐条重跑 | :72 | 都没写 | 只有调度记录这一句 | 记入合入清单 |
| W05 | 批 2 至批 5 的补丁全在集成副本，主工作区最早 2026-10-14 才打；H1 的两处替换在暂存区没提交，HEAD 与 origin/master 仍带值 | :57、:106 | 正文二:13 提到「主工作区没有这几份补丁」；H1 的提交状态没写 | A10、A11、B09 | 记入合入清单 |
| W06 | 门禁 abandoned-rounds 格对 2026-10-12 之前的 4 行登记（c196-r1、d16-item1-r2、c364-r3、smalldata-ttime-r1）不要求出处；批 5 甲「照收，进第三轮」 | :122 | 正文三没有对应格 | E51（tsv 第 7–10 行） | 补进交用户（零轮） |
| W07 | 批 5 额外做的判法与文字：丙加两处判法（`["X"] = 值` 不算读；`;`、`&&`、`||` 之后同一行赋的算局部变量），乙多改 lib_shell_words.py 与登记表一行，收尾多改 lib_heavy_tests.py 文件头 | :121、:124 | 正文三只说 K1–K11 零轮 | P06、P08 | 并进零轮交用户表 |
| W08 | 等两台 s2 跑完才做：六个 .rs 装置的 heavy_threads_default_tests、E176 装置只编不跑、prove-red.sh 31 路与 REPLAY_JOBS 31 条的内存没量、replay.sh 最坏 31×31 | :70、:75 | 都没写 | 只有调度记录 | 补进交用户（代价没量） |
| W09 | 流程缺口：子 agent 在副本里跑不了登记自证，工具活交回前证不了红；集成副本要补 SOP、被忽略配置与被点名产物，建副本时别的会话改到一半的文件造成假红 | :68、:77 | H4（判决二:54）只收了「仓副本放宽」那一半 | verify3 的 4 个判错里 3 个是假红（C06） | 另立账 |
| W10 | 「只缩不涨」没比：基准里还没有 heavy-thread-exempt 这张表 | :116 | 都没写 | 抽出的格按「没有基准」跑，同样没比 | 合入主工作区后补比一次 |
| W11 | 私有 memory 第 1–8 行改指针：「第 1 行等批 3 写进项目后再改成指针」没有后续记录 | :100 | 都没写 | 复核不了：私有目录不归我读 | 备查 |
| W12 | 推迟到代码批的：课题 6 六·1 覆盖脚本与门禁格、课题 10 第 8 项 crates/ 默认值 | :37、:53 | 正文二/三第二节写了 crates/ 与 worker_count_for，没写六·1 | 排除表「代码批」行 13 行（K:exempt_rows.frozen_r3.crates） | 备查 |

## 3 要主 agent 跑的命令与复核不了的

1. 坐实 W01 / D03（门禁格子 agent 起不了）：

   ```
   cd ~/.local/share/singlefs/work/m3-defs-b3-r3/repo
   SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults "$PWD"
   ```

   我的推（`out/htd_cell_r3_frozen.txt`）：红，1 处，`scripts/agents/dispatch-prompt.py:262`。推翻：退 0。同一格在 after-b5 上我的抽出版是绿（832/94/66，与 verify4 的日志一致），所以抽出版的判法与门禁一致。
2. 复跑本报告的 T6 现算：见 §0 的 `tools/rerun.sh` 命令，预期末行 `name=compare file=count_checks same`；只在冻结副本、集成副本和这个仓的 HEAD 没变时逐字节相同（`git.HEAD`、`main-wt`、`value_tree_hits` 三处随仓变）。
3. 复跑前两轮腿的模型目录：`research/prompts/m3-defs-b3-r3-sonnet-model/tools/rerun-legs.sh <仓根> ~/.local/share/singlefs/work <模型拷贝所在目录> <新草稿目录>`，预期五行 same、八行 same、九行里八行 same（r7_abandoned 只差第 29 行的绝对路径）。
4. E30 若要核：在集成副本里 `SINGLEFS_HEAVY_TESTS=commit bash .claude/singlefs-ai-sop/scripts/rules-lint.sh` 看末行「扫了 N 份文件 M 行」；我读到的 SOP 规则目录是 17 份 2004 行，`.claude/rules` 是 10 份 876 行，都不是 1957。

复核不了（原因）：E02、E15、E16 时点量，现场已变；E12、E13、E31 只有当时的文字，没留输出；E24b 是归类；E25 抽样没写抽哪几份；E30 见上；E42d 是登记自证，不许跑；B07、B08 的红绿结局要门禁入口；U03 要真实主会话记录；W11 私有目录。

## 4 判定一览

| 格 | 判定 | 一句话 | 推翻条件 |
|---|---|---|---|
| T6 判决一（A01–A15） | 冲突 1 处 | 15 个数 14 个对得上；正推腿 sha256 末 4 位写成 a8a8，真值 d8a8（A01，同错在调度记录 L104） | `sha256sum` 报告得 …a8a8 |
| T6 判决二（B01–B29） | 冲突 1 处 | 29 个数 28 个对得上；「九种写法 0→2」是 8 种（B18）；B09、B10、B11 的「0」「831/94/66」只对工作树与第二轮那一版 | out/ 里 r5 的 9 个 case 全 0→2 |
| T6 正文二（C01–C11） | 冲突 1 处 | 11 个数 10 个对得上；「16 项自证与门禁格、约 113 个弄坏开关按期望判」真拆分是 14 与 115（C05） | verify3.summary 里期望绿判对 16 项 |
| T6 正文三（D01–D11） | 冲突 2 处 | 11 个数 9 个对得上；94/66/831 对第三轮冻结副本不成立（格红，D03）；「能当出处的 504 行」是 557 行里的 504（D06）；T1 的两份对象不在 35 份清单里（D02） | 门禁入口跑格得绿；r7-records-scan 的 source_lines 是 504 |
| T6 调度记录（E01–E55、P01–P08） | 冲突 5 处，复核不了 10 行 | 66 行里 51 行对得上；sha 末位（E32）、「41 个文件」（E21）、「30 份」（E39）、「16 项/约 113」（E43）、「13 份自证」（E52）对不上 | 各行「现算」列的命令得到引的数 |
| T6 sha256 末位（H01–H37） | 冲突 2 处（同一个错） | 37 个里 35 个对；错的是 r1 正推腿报告 | 同 A01 |
| T5 四条腿打中（T5-01–T5-94） | 有漏列 | 94 行里 75 行去向在三类或改法表里，7 行在三类之外，11 行有一部分没去向（U01–U08），1 行判重 | 判决里找得到 U01–U08 任一条的去向 |
| T5 调度记录里开着的线（W01–W12） | 有漏列 | 12 条判决与正文三没写去向；其中 W01（heavy-thread-defaults 在第三轮冻结副本红）最要紧 | 门禁入口跑格得绿（W01）；各条在判决里找得到去向 |

## 5 没做什么

- 不判别的腿的格（T1–T4）：没读、没判；没替主 agent 采纳或出判决；U、W 两张表里的「建议」只是归类建议。
- 没读这一轮攻方腿的报告与模型目录；没读主工作区里被判的定义、规则、钩子、脚本与门禁（开工先读的三份规则文件除外，见 §0）。
- 没跑登记的 `--selftest`、门禁入口、`cargo`、重型测试、rules-lint、doc-lint；heavy-thread-defaults 一格是抽出判法直接跑，不是门禁入口。
- 没核主 agent 在门禁入口上跑出的八份样本、四个样本根的红绿结局，只核了个数。
- 没核磁盘现场类的数（E02、E15、E16）和会话记录类的数（E13）。
- 没替前两轮的腿重判它们的「推」与「量」，只核判决与正文引的数、清单和去向。

## 6 模型目录各文件 sha256

`cd research/prompts/m3-defs-b3-r3-sonnet-model && find . -type f | sort | xargs sha256sum` 原样：

```
9c8e46c451cac42819df9b040817f0e20655c3e9761571d260be4c3d6f3b1e26  ./out/count_checks.txt
69561a0cf943ef900a436fd0514a9eb80ff44f21e4673d8605990ab74d1ffe87  ./out/feed_guard_n.txt
d92016829081e1e90107403c15c70e7f6f02ab3c2fbf6b81889f88e8f50a2141  ./out/htd_cell_r3_frozen.txt
91ddadbbfbdad9fc2e0a275a70a8c0e44269d2b70c7fcd425a441da68a59a7ce  ./out/rerun-r1-opus.log
01d5d60233e2d799ecc4cd2bead3b7032d2ad82de3c94028989ea8c75864b88d  ./out/rerun-r2-opus.log
98a852caf0b021f99b6c2daadece781259e934347327fd0be75e659d9a3982c1  ./out/rerun-r2-sonnet.log
76cbad926974af8d7b5169fde824017d6975f25e0a330196a1aadb93c9350e60  ./probes/count_checks.py
599706f8fee7afb622c908ee4954b05b836fd667e77b509ef80a5cf2950316ea  ./probes/feed_guard_n.py
17e854f60bf3feef9ab71157328eb07e6e351907296e1baca21f09e57d61c4c4  ./probes/gen_t5_rows.py
f51f7f59a2fa1798841813bc652051f9607aebd23b40143fe4aa7e4450b5148d  ./probes/gen_t6_rows.py
c686ac926d4c442e8fd1ce391c8f4d78eaeb28d7f0e55b5cbf148e8d0c1d56fa  ./probes/value_hits.py
8d12294f4d3da6fdd8c0db135a75a6160bcbfc31beb50122d1e271a7fb3ba2c5  ./probes/value_tree_hits.py
376d2714b9e14591dbaa4e41cf164548cc00d9259d88a539adb77afee21aba9f  ./tools/rerun-legs.sh
687a1eebb13c2621520822eed40e6e1eddebb4a1a887a87a86967e1f7103160d  ./tools/rerun.sh
57d6b10cdf026bb787aca353c10a7820db274530fee443130df31218dc56b1e0  ./tools/run_htd_cell.py
```
