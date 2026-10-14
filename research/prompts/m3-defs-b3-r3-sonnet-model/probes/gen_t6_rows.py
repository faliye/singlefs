#!/usr/bin/env python3 -I
"""T6 表格行生成：引处的行号由 token 在被引文件里现查（grep -nF 同义），不手写行号。
用法：gen_t6_rows.py <仓根> > rows.md
每条： (编号, [(文件键, token[, 第几次出现])…], 引的数, 现算（命令或 out/count_checks.txt 的键）, 结果, 判, 口径缺什么)
判： 对 / 对不上 / 复核不了
"""
import sys, os
REPO = sys.argv[1]
F = {"V1": "research/prompts/m3-defs-b3-r1-main-verification.md", "V2": "research/prompts/m3-defs-b3-r2-main-verification.md",
     "B2": "research/prompts/_m3-defs-b3-r2-body.md", "B3": "research/prompts/_m3-defs-b3-r3-body.md",
     "RC": "records/2026-10-12-里程碑三工具与治理批.md"}
SHORT = {"V1": "判决一", "V2": "判决二", "B2": "正文二", "B3": "正文三", "RC": "调度记录"}
TXT = {k: open(os.path.join(REPO, v), encoding="utf-8").read().split("\n") for k, v in F.items()}

def where(refs):
    out = []
    for r in refs:
        k, tok = r[0], r[1]; occ = r[2] if len(r) > 2 else 1
        hits = [i + 1 for i, l in enumerate(TXT[k]) if tok in l]
        if len(hits) < occ:
            MISSING.append((k, tok)); out.append(f"{SHORT[k]}:?"); continue
        out.append(f"{SHORT[k]}:{hits[occ - 1]}")
    return "、".join(out)

ROWS = []
MISSING = []
def C(i, refs, claim, how, res, verdict, gap=""):
    ROWS.append((i, refs, claim, how, res, verdict, gap))

# ── 判决一 ──
C("A01", [("V1", "60ed02db")], "正推腿报告 sha256 60ed02db…a8a8", "sha256sum 报告", "60ed02db…d8a8（K:legsha.m3-defs-b3-r1-sonnet-output）", "对不上", "前 8 位对，末 4 位 a8a8 应为 d8a8；调度记录 L104 同一处同错")
C("A02", [("V1", "afb4a1fb")], "攻方腿报告 sha256 afb4a1fb…6645", "K:legsha.m3-defs-b3-r1-opus-output", "afb4a1fb…6645", "对", "")
C("A03", [("V1", "21 份")], "冻结副本 21 份 sha256sum -c 全过", "K:snapshot.r1.*", "lines=21 ok=21 notok=0", "对", "")
C("A04", [("V1", "88 处")], "cite-check 两份报告 核 88 处、对上 88 处、没判 0 处", "cite-check.py 两份报告 --root r1 冻结副本 --background _m3-defs-b3-r1-background.md", "核了 88 处引文，对上 88 处，没判 0 处", "对", "")
C("A05", [("V1", "五行")], "攻方模型 rerun.sh 五行 same", "rerun-legs.sh 第 1 条（拷到草稿目录、经 4G 内存包装）", "5 行 compare 全 same（out/rerun-r1-opus.log）", "对", "")
C("A06", [("V1", "find … | sha256sum")], "模型目录 sha256 与报告抄的逐行相同", "K:modelsha.r1opus", "model_files=11 report_lines=11 identical=True", "对", "")
C("A07", [("V1", "grep -c E161")], "capped.sh --print 8 | grep -c E161 输出 0", "K:r1.capped_print8_E161（r2、r3 冻结副本同命令）", "r1 冻结副本 0；r2、r3 冻结副本 1", "对", "数的是第一轮那一版；改后（G3）已是 1，正文别处引时要带版本")
C("A08", [("V1", "750–765")], "layer0 第 750–765 行 --test-threads 0 处", "K:r1.layer0_750_765_test_threads", "r1 冻结副本 0（整份文件也是 0）；r2、r3 冻结副本该区间仍 0、整份 5", "对", "区间是改前的行号；改后 --test-threads 在别的行（整份 5 处）")
C("A09", [("V1", "8 份")], "tools: 只有 Read、Bash 的 8 份（prior-art 另有 WebFetch、WebSearch）", "K:r1.agents_total / agents_without_write_edit", "共 14 份，tools 里没有 Write 也没有 Edit 的 8 份", "对", "「只有 Read、Bash」其实含 prior-art 的 Web 两样；判的是无 Write/Edit；investigator、kb-scribe 有 Edit 无 Write 算有")
C("A10", [("V1", "第 1004 行")], "records/2026-09-30… 第 1004 行含 PEER_SSH_HOST 的值，已提交（3e3cdc3c）", "probes/value_hits.py（只打行号）", "3e3cdc3c、ce1457c3、HEAD、origin/master、r1/r2/r3 冻结副本都是 [1004]；主工作区工作树 []", "对", "")
C("A11", [("V1", "295、302")], "m3-batch10x-code-r1-opus-output.md 第 295、302 行含本机 hostname，已提交（ce1457c3）", "同上", "同上各处 [295, 302]；主工作区工作树 []", "对", "")
C("A12", [("V1", "ce1457c3")], "HEAD 与 origin/master 同为 ce1457c3", "git rev-parse --short HEAD、remotes/origin/master", "ce1457c3、ce1457c3", "对", "是今天现查；git status -sb 首行 ## master...origin/master")
C("A13", [("V1", "第 4 行残留旧编号")], "lib_writer_lock.py 第 4 行残留旧编号 ⑮", "读 r1、r2 冻结副本该文件第 4 行", "r1 含 ⑮；r2 含 ⑯", "对", "")
C("A14", [("V1", "G1 | ⑮")], "G1 的任务带重活类型 六种", "K:r2.thread_limit_required_types", "r2 冻结副本类型表 6 个名字", "对", "")
C("A15", [("V1", "doc-lint:not-numbers")], "判决一 编号：D1–D7、G1–G11、H1–H3", "K:v1.rows_D / rows_G / fork_items", "D 行 7、G 行 11、H 项 3", "对", "")
# ── 判决二 ──
C("B01", [("V2", "30 份")], "冻结副本 30 份 sha256sum -c 全过（非 OK 0 行）", "K:snapshot.r2.*", "lines=30 ok=30 notok=0", "对", "")
C("B02", [("V2", "104 处")], "cite-check 核 104 处、对上 104 处、没判 0 处", "cite-check.py 辩方、攻方两份 --root r2 冻结副本 --background _m3-defs-b3-r2-background.md", "核了 104 处引文，对上 104 处，没判 0 处", "对", "")
C("B03", [("V2", "第 20–77 行")], "攻方报告第 20–77 行的 sha256 与模型目录逐行相同", "K:modelsha.r2opus", "model_files=58 report_lines=58 identical=True", "对", "")
C("B04", [("V2", "八行")], "攻方模型 rerun.sh 八行 same（带 FROZEN_ROUND1）", "rerun-legs.sh 第 2 条", "8 行 compare 全 same（out/rerun-r2-opus.log）", "对", "")
C("B05", [("V2", "第 7 节")], "辩方报告第 7 节 sha256 与模型目录逐行相同", "K:modelsha.r2sonnet", "model_files=29 report_lines=29 identical=True", "对", "")
C("B06", [("V2", "九行里八行 same")], "辩方 rerun.sh 九行里八行 same、r7_abandoned 一行不同、diff 只有第 29 行", "rerun-legs.sh 第 3 条；diff 输出与模型 out/r7_abandoned.txt", "9 行 compare：8 行 same、r7_abandoned differs；diff 只有第 29 行（工作目录绝对路径）", "对", "")
C("B07", [("V2", "八份样本")], "攻方 tools/run-gate-samples.sh 八份样本", "K:opus_gate_sample_dirs", "gate-samples 下样本目录 8 个（doc-text 4、doc-process-records 3、code-tooling 1）", "对", "八份「结局」要门禁入口，子 agent 起不了：复核不了；数是 8")
C("B08", [("V2", "四个样本根")], "辩方 make_gate_sample_roots.py 四个样本根", "K:r2sonnet.sample_roots", "脚本里 root(\"gate-…\") 4 个", "对", "四个根的红绿结局要门禁入口：复核不了")
C("B09", [("V2", "命中都是 0")], "主工作区两份文件里 PEER_SSH_HOST 的值与本机主机名命中都是 0", "probes/value_hits.py main-wt / head / origin-master", "工作树 [] 与 []；HEAD 与 origin/master 仍是 [1004] 与 [295, 302]", "对", "「0」只对工作树；提交与远端还带着（H1 定「历史不动」，换掉的改动还在暂存区、没提交）")
C("B10", [("V2", "831 份")], "heavy-thread-defaults 扫 831 份", "tools/run_htd_cell.py <r2 冻结副本>（抽出格的 python 直接跑）", "r2 冻结副本：扫了 831 份（scripts 455、gate.d 24、hooks 16、crates 335、cargo-command-prefix 1）", "对", "只对第二轮那一版（after-b4-fixup2）；第三轮冻结副本（after-b5-fixup）这一格是红的，见 D03")
C("B11", [("V2", "94 处")], "可疑写法 94 处全在排除表", "同上", "r2 冻结副本：可疑写法 94 处全在排除表里", "对", "同 B10")
C("B12", [("V2", "66 行")], "表 66 行每行都命中", "K:exempt_rows.* 与格输出", "r2、r3 冻结副本排除表 66 行；r2 格输出「表 66 行，每行都命中」", "对", "")
C("B13", [("V2", "16 份", 1), ("V2", "16 份", 2)], "J07 的 16 份测试文件（挂 gpu 特性）", "K:r2.gpu_cfg_test_files", "tests/ 下含 feature = \"gpu\" 的 .rs 16 份", "对", "")
C("B14", [("V2", "十四组")], "辩方第 2 节十四组", "K:r2sonnet.section2_first_table_rows", "第 2 节第一张表 14 行", "对", "")
C("B15", [("V2", "改前→改后 0→2")], "S08、S09 改前→改后 0→2", "辩方模型 rerun（r1_thread_limit）", "九行 compare 中 r1_thread_limit same（与 out/ 逐字节相同）", "对", "")
C("B16", [("V2", "1→0")], "S01 改前→改后 1→0", "辩方模型 rerun（r2_capped）", "r2_capped same", "对", "")
C("B17", [("V2", "replay.sh 22 行")], "replay.sh 22 行钉线程数", "K:r1/r2/r3.replay_sh_thread_pins", "grep -c 'E1[0-9][0-9]_[A-Z_]*THREADS=[0-9]' 三个冻结副本都是 22；E161_THREADS=10 在第 722 行", "对", "")
C("B18", [("V2", "攻方九种写法")], "-p/bench 攻方九种写法、S10、S11 改前→改后 0→2", "K:r2opus.r5_*", "r5 out 中 -p 与 bench 的 9 个 case：8 个 0→2，p-env-C 仍 0（它是 C2），run-cwd-probe-bin 不在这 9 个里（2→2）", "对不上", "九种里 8 种 0→2；p-env-C 0→0 属 C2（K7）；该句没分")
C("B19", [("V2", "改后 0、改前 2")], "C1 改后 0、改前 2", "r5-r2.txt 与 r5-r1.txt（已 rerun same）", "copy-light-selftest-body-heavy：改后 rc=0、改前 rc=2", "对", "")
C("B20", [("V2", "10 份产物")], "仓里已有 10 份产物超过 8 MiB", "K:main.results_files_over_8MiB", "主工作区 research/results 下 >8 MiB 的 10 份，最大 115269707 字节", "对", "冻结副本不带 research/results（0 份）；数的是主工作区")
C("B21", [("V2", "主工作区 0 份")], "带 NUL 的文本主工作区 0 份", "K:main.results_files_with_NUL_head", "research/results 下前 8 KiB 带 NUL 的 3 份，全是 .gz", "对", "判「文本」靠后缀；格本身不看后缀，3 份 .gz 会被它算「没判的二进制」")
C("B22", [("V2", "六种类型缺行")], "六种类型缺行拒", "K:r2.thread_limit_required_types", "6", "对", "")
C("B23", [("V2", "七种类型")], "⑭ 扩到七种类型", "K 无；读 r2 冻结副本 LONG_RUN_AGENTS = {designer} | 六种", "7（r3：10）", "对", "")
C("B24", [("V2", "| R1 |"), ("V2", "| R9 |")], "R 行：3 站得住、6 部分站不住", "K:v2.verdict_tally / v2.rows_R", "R 行 9；站得住 3、部分站不住 6", "对", "")
C("B25", [("V2", "| K1 |"), ("V2", "| K11 |")], "改法 K1–K11", "K:v2.rows_K", "K 行 11；岔路 H 项 1", "对", "")
C("B26", [("V2", "另立一笔账（改前也中")], "另立账 J03、J12、B3、J11、J10、S26、J08 后半、J14，加第一轮 S21、O09", "K:v2.pile_line_ids / pile_first_round_ids", "本行 8 笔 + 第一轮 2 笔", "对", "R5 行里的 C2 另立账不在本行（它并进 K7）")
C("B27", [("V2", "9976f8c1")], "辩方、攻方报告 sha256 9976f8c1…1fec、42b53504…9126", "K:legsha", "9976f8c1…1fec、42b53504…9126", "对", "")
C("B28", [("V2", "本机 32")], "available_parallelism 本机 32", "nproc", "32", "对", "")
C("B29", [("V2", "| K2 |")], "K2 加 clerk、three-way-forward、three-way-defense 三种", "K:r3.thread_limit_required_types", "r3 冻结副本类型表 9 个", "对", "")
# ── 正文二 ──
C("C01", [("B2", "after-b4-fixup2")], "冻结副本 = 集成副本标签 after-b4-fixup2 那一版", "K:frozen_vs_tag.r2", "30 份里 30 份与标签同 sha", "对", "")
C("C02", [("B2", "六种任务带重活")], "R1 六种类型", "K:r2.thread_limit_required_types", "6", "对", "")
C("C03", [("B2", "七份定义")], "七份定义「输入」的「预计挂钟」", "K:r2.agent_defs_with_预计挂钟", "r2 冻结副本含「预计挂钟」的定义 7 份", "对", "数的是整份定义里有这个词，没限「输入」一节")
C("C04", [("B2", "133 项")], "verify3 派 133 项、收 133 项", "K:verify3.total", "verify3.summary 133 行", "对", "")
C("C05", [("B2", "16 项自证与门禁格")], "16 项自证与门禁格、约 113 个弄坏开关按期望判", "K:verify3.tally", "期望绿 16 项（判对 14、判错 2）；弄坏开关 117 项（判对 115、判错 2）", "对不上", "对的总数 129 相同，拆分不对：绿 14 不是 16，开关 115 不是 113；「约」不是现算")
C("C06", [("B2", "判错的 4 项")], "判错 4 项分诊：一项真冲突，另三项是挂错脚本、无 git 历史、31 路并行时探工具链版本没回来", "K:verify3.wrong、verify3.cell_code_tooling", "4 项 = 第 14（code-tooling 格）、16（abandoned-rounds 格）、93、94（挂错脚本）", "对", "「项」与「原因」不是一一对应：93、94 两项同一原因；第 14 项一项里就含「真冲突」和「31 路并行」两种原因")
C("C07", [("B2", "4 行")], "heavy-thread-defaults 红 4 行；登记 4 行后转绿", "K:verify3.hdd_red_lines、exempt_rows.after-b4-fixup(2)", "红行 runner-dispatch-guard.sh 1182、1791、1792、2107；排除表 62→66（+4）；fixup2 版格转绿", "对", "")
C("C08", [("B2", "8 格")], "双机驱动 8 格「现算线程」会红", "K:verify1.two_hosts_*", "--selftest 53 格里 8 格判错，8 个格名都是 *-threads-*", "对", "")
C("C09", [("B2", "8 个字")], "private-values 8 个字以上的词", "读 doc-text.sh 文件头与常量", "门槛 8 个字（r2、r3 都是）", "对", "")
C("C10", [("B2", "6 字")], "叫停出处原话 6 字以上", "读 runner-dispatch-guard.sh ABANDONED_QUOTE_MINIMUM_CHARACTERS", "= 6", "对", "")
C("C11", [("B2", "| R1 |")], "R1–R9 九格", "K:b2.rows_R", "9", "对", "")
# ── 正文三 ──
C("D01", [("B3", "35 份")], "冻结副本 35 份 sha256", "K:snapshot.r3.*、frozen_vs_tag.r3", "35/35 OK；35 份与集成副本标签 after-b5-fixup 同 sha", "对", "")
C("D02", [("B3", "被判的定义与规则文件")], "被判文件 14 份、都在 35 份里", "K:r3body.judged_files(_in_snapshot)", "14 份路径，14 份都在 35 份清单里", "对", "T1 对象里的 scripts/hosts/heavy-threads.sh、heavy_threads.py 不在 35 份清单里：读到的版本没被钉住")
C("D03", [("B3", "94 处")], "可疑写法 94 处、排除表 66 行、扫 831 份（现跑绿）", "tools/run_htd_cell.py 在 r2 冻结副本、after-b5、after-b5-fixup 上", "after-b4-fixup2：831/94/66 绿；after-b5：832/94/66 绿；第三轮冻结副本（= after-b5-fixup）：红，1 处（scripts/agents/dispatch-prompt.py:262）", "对不上", "三个数只对 fixup2；正文三讲的第三轮冻结副本上这一格红；批 5 收尾复跑（L124）没跑这一格")
C("D04", [("B3", "133 项")], "verify3 133 项", "K:verify3.total", "133", "对", "")
C("D05", [("B3", "148 项", 1)], "verify4 148 项", "K:verify4.total", "148", "对", "")
C("D06", [("B3", "504 行")], "records 能当出处的 504 行", "K:records_scan.r2/r3", "能当出处 557 行；其中没有叫停类词的 504 行；records 95 份（调度记录按禁读跳过）", "对不上", "504 是「能当出处且行里没有叫停/停掉/停了/撂下/不判」的行，能当出处的是 557")
C("D07", [("B3", "22 行")], "replay.sh 22 行", "K:r3.replay_sh_thread_pins", "22", "对", "")
C("D08", [("B3", "19835 行")], "19835 行新增命中", "tools/measure-config-words.py（批 5 丙的脚本）在集成副本现树", "19844 行 / 2232 份（345 个词、59 个有命中、原词源 3 行）；引的 19835 / 2223", "对", "引的是批 5 丙只打自己补丁的那棵树（13654 路径）；集成副本 HEAD 13713 路径，多了 9 行 9 份；结论（≠0）不变")
C("D09", [("B3", "148 项", 2)], "批 5 派 148 项、收 148 项、判错 1 项", "K:verify4.total / verify4.wrong", "148、148、1（第 6 项 runner-dispatch-guard）", "对", "")
C("D10", [("B3", "九种类型骨架")], "九种类型骨架打线程上限：31，十种打预计挂钟", "K:r3.skeleton_*", "14 种类型里骨架带「线程上限：31」的 9 种、带「预计挂钟：」的 10 种", "对", "")
C("D11", [("B3", "| T1 |")], "T1–T6 六格", "K:b3.rows_T", "6", "对", "")
# ── 调度记录 ──
C("E01", [("RC", "第 91–104 行")], "hook-swap 第 91–104 行旧路径（课题 2 表里列第 91、94、95、99、101、104 行）", "读 base 提交 c840472 的 scripts/gate/hook-swap.py", "91、94、95、99、101、104 六行含旧式 scripts/*.sh 路径；selftest-registry 第 130 行是 lib_workspace_cargo_lock.py light", "对", "")
C("E02", [("RC", "527543072")], "du -sc 合计 527543072 KiB；16 个 cargo 各一个 w* 目录", "du -sc cross-run-cache/singlefs-crates-mutation-target*", "现在 1417166004 KiB；w* 目录 15 个（另有不带后缀的一个）", "复核不了", "时点量，那一趟还在写；调研报告里 730940872 也是另一个时点")
C("E03", [("RC", "min(16, 核数 // 2)")], "worker_count_for 取 min(16, 核数 // 2)；门禁文件头第 38 行同写", "读 base 提交", "crates-mutation-rows.py 第 1855 行 by_processor = max(1, min(16, processor_count // 2))；门禁文件头第 38 行是「核数的一半、至多 16」", "对", "")
C("E04", [("RC", "约 970 GiB")], "31 份编译目录约 970 GiB（推的）", "算术 527543072 KiB / 16 × 31", "974.7 GiB", "对", "推的数，约数；基数是 16 份合计的一个时点")
C("E05", [("RC", "27 行")], "事实表 27 行", "K:facts_rows", "27（facts.tsv 28 行含表头）", "对", "")
C("E06", [("RC", "353 行")], "候选 353 行（247 个不同的行）", "K:candidate_rows / candidate_distinct", "353；按载体去重 247", "对", "")
C("E07", [("RC", "约 179 行")], "F1-F13 约 179 行、F14-F27 约 174 行", "K:candidates_F1_13 / F14_27", "179、174（和 353）", "对", "")
C("E08", [("RC", "盘点表 54 行")], "盘点表 54 行", "K:inventory_rows", "54（产品代码 4、checker 档 8、门禁 6、脚本 27、显式参数 1、文字 8）", "对", "")
C("E09", [("RC", "事件句不改 101")], "课题 0 后半：174 行：不改 101、不相干 63、要改 8、要补 2；反向问 3", "K:judge_b_tally；报告 sha 07ee6db7…7fac5", "101、63、8、2（和 174）；sha 对", "对", "反向问 M1–M3 的 3 条没单独的表可数")
C("E10", [("RC", "360 行处置")], "360 行处置：不改 321、改了 32、补了 7；原始证据 5 份", "K:sync_rows / sync_tally / sync_evidence_rows", "360；321、32、7；原始证据表 5 行", "对", "")
C("E11", [("RC", "checks-owed 17 处")], "写回员 checks-owed 17 处；milestone 03、D3、D23、D28、变更史 23 处", "读两份 report.md", "17 条规格/17 处替换（1 个文件）；23 条/23 处（5 个文件）", "对", "")
C("E12", [("RC", "24 处编号缺简称")], "24 处编号缺简称、153 段补了 23 段、整仓 doc-lint 红 4 处、写回 0 行", "无产物", "—", "复核不了", "只有主 agent 当时的文字；没有保存 doc-lint 输出")
C("E13", [("RC", "26 次")], "singlefs-53 提到 12-twelfth-txn.md 26 次、49 分钟前起", "无（会话记录不归我读）", "—", "复核不了", "")
C("E14", [("RC", "22 条全对")], "hook-swap 8 条喂法；--dry-run 22 条全对；旧版 4 条红；+159/−22", "读 b2-guard 的 patch 与 report", "hook-swap.py +8/−8；报告写喂了 22 条全对、旧版四条红；补丁 4 份 +159/−22", "对", "")
C("E15", [("RC", "0.52 GiB")], "共用调研：0.52 GiB、6.7 GiB（40.1 GiB 的 17%）、22.4、9.75、81–92%、每片 20–22 GiB、31 片 0.6–0.7 TiB", "读 report.md；算术", "报告里都有；6.7/40.1 = 16.7%；31×20 = 0.61 TiB、31×22 = 0.67 TiB", "复核不了", "是磁盘上量的数，现场已变；只核了报告与记录一致、算术成立")
C("E16", [("RC", "根分区可用 962G")], "根分区可用 962G；M2 轻型门禁 39 项", "无", "—", "复核不了", "时点量；39 项只是计划")
C("E17", [("RC", "29 份 1248 行")], "补丁 29 份 1248 行", "K:patch.b2-defaults", "files=29，补丁文件 1248 行（+365/−102）", "对", "1248 是补丁文件的行数，不是改动行数；与别处的「+a/−d」口径不同")
C("E18", [("RC", "六个装置")], "六个装置 include! 共用的 threads.rs", "grep include! 补丁", "补丁里 6 处 include!(\"heavy_threads_default/threads.rs\")", "对", "")
C("E19", [("RC", "30 份 +1221/−94")], "丙补丁 30 份 +1221/−94；排除表 47 行；自补 18 行；SELFTEST_RUNNERS 加三条；登记两行、改三行理由", "K:patch.b2-threads；读补丁", "30、+1221/−94；报告写 47 行、18 行；补丁里 SELFTEST_RUNNERS 新增 3 条；selftest-registry 补丁 +5/−3（新两行、改三行）", "对", "")
C("E20", [("RC", "13604 份")], "主工作区跟踪与未跟踪文件 13604 份", "K:int.base_files", "集成副本 base 提交 13604 个文件", "对", "")
C("E21", [("RC", "41 个文件 +1711/−247")], "批 2 四份补丁 41 个文件 +1711/−247", "K:b2stack.*（临时 index 依次叠四份补丁）", "全部 68 个文件 +2550/−247；其中改动的 41 个 +1711/−247，新增的 27 个 +839", "对不上", "41/+1711 只算改动的文件，没算 27 个新增文件")
C("E22", [("RC", "派 67 项")], "verify 派 67 项、收 67 项、判错 0；23 份自证全绿、42 个新开关全红；取证两项", "K:verify.tally / total / wrong", "67；绿 23、红 42、取证 2；判错 0", "对", "")
C("E23", [("RC", "53 格里 8 格")], "双机驱动 --selftest 53 格里 8 格判错", "K:verify1.two_hosts_*", "53 格里 8 格判错；8 个失败格名", "对", "")
C("E24", [("RC", "红 19 行")], "新格在集成后的真树上红 19 行", "K:verify1.cell_hdd_red_items", "19", "对", "")
C("E24b", [("RC", "红 19 行")], "19 行里「弄坏开关自己的代码 6 行、真漏改 8 行、判法太宽把超时 600 当线程 1 行、单测期望式 1 行等」", "读 verify-out/067 的 19 行清单", "清单只有路径:行与原行，没有归类；6+8+1+1 = 16，「等」是另外 3 行", "复核不了", "归类是判断，不是计数；能核的只有 19 和 16 < 19")
C("E25", [("RC", "抽 6 份开关日志")], "抽 6 份开关日志看红在点名那一格", "无", "—", "复核不了", "抽样数，没写抽的是哪 6 份")
C("E26", [("RC", "13 份 +154/−36")], "批 2 收尾 13 份 +154/−36；19 行 = 改代码 3 + 判法 1 + 排除表 15；排除表 47→62", "K:patch.b2-fixup、exempt_rows.b088a84", "13、+154/−36；3+1+15 = 19；排除表 62", "对", "3、1 的归类是判断，只核了加法与 15 = 62−47")
C("E27", [("RC", "17 项 16 对")], "verify2：17 项 16 对；5 个开关红；run-with-memory-cap 72 项绿、两开关红", "K:verify2.*", "17 项、判错 1（cell-selftest-coverage）；开关红 8 = 5 + 2 + 1；rwmc 自检「查了 72 项」", "对", "")
C("E28", [("RC", "831 份、可疑 90 处")], "verify2 时格：扫 831 份、可疑 90 处、表 62 行", "K:verify2.cell_hdd；run_htd_cell.py r1 冻结副本", "831、90、62（r1 冻结副本重算同）", "对", "")
C("E29", [("RC", "6 份 +25/−14")], "批 3 补丁 6 份 +25/−14；旧公式改前 5 行、改后 0 行", "K:patch.b3-defs；grep 同一组词（报告里的命令）", "6、+25/−14；base 5 行（commit-stages 1、main-agent 3、implementation-workflow 1）、before-b4 0 行", "对", "")
C("E30", [("RC", "rules-lint 绿（17 份 1957 行）")], "rules-lint 绿（17 份 1957 行）", "无（rules-lint 是提交时才跑的检查）", "SOP 规则目录 17 份、wc -l 共 2004 行；.claude/rules 10 份 876 行", "复核不了", "17 与 SOP 规则 17 份同数，行数 2004 ≠ 1957，扫的目录与计行口径不明")
C("E31", [("RC", "1370 行")], "doc-lint 滤掉「日期早于第一个提交」1370 行", "无", "—", "复核不了", "")
C("E32", [("RC", "324 行")], "正推腿报告 324 行；sha256 60ed02db…a8a8；cite-check 51/51", "K:leglines；cite-check.py 只 r1 正推报告", "324 行；sha 末 4 位错（同 A01）；核了 51 处、对上 51 处", "对不上", "行数、51/51 对；sha 末 4 位 d8a8 不是 a8a8")
C("E33", [("RC", "三个名字与共同前缀全仓 0 处")], "三个名字与共同前缀全仓 0 处", "probes/value_tree_hits.py（git 跟踪 + 未跟踪未忽略的 14412 份）", "配置里的两个值（第二台 SSH 别名、本机主机名）命中 0 份；第三个名字不在配置里", "对", "「三个名字」「共同前缀」没写出来是哪几个：第三个只能按文字信")
C("E34", [("RC", "门禁 66 号")], "abandoned-rounds.tsv 第 9 行 m3-batch63-code-r1、文件头写「门禁 66 号」", "读 base 提交", "第 9 行是 m3-batch63-code-r1，文件头含「门禁 66 号」", "对", "")
C("E35", [("RC", "四个新开关")], "批 4 丙：四个新开关；private-values 五个开关", "补丁里新增开关名", "heavy-thread-defaults 4 个、private-values 5 个", "对", "")
C("E36", [("RC", "整仓比只中那两份旧行 3 行")], "真配置在集成副本整仓比只中那两份旧行 3 行", "K:value_hits（行数）", "1004 一行 + 295、302 两行 = 3 行", "对", "")
C("E37", [("RC", "应为七个")], "runner-dispatch-guard.sh 第 247 行写「五个开关」，应为七个", "K:writer_lock_modes.*", "before-b4：lib_writer_lock.py 5 个、钩子头写五个；after-b4：7 个、钩子头仍写五个；撤回 nearest-lock-only 后 after-b4-fixup：6 个、钩子头写六个", "对", "「七」是批 4 乙补丁那一版的数；撤回后现在是六，文件头已是六")
C("E38", [("RC", "12 个钩子开关")], "批 4 甲 12 个钩子开关；第 1595 行", "补丁里新增 break_switch；读 records/2026-10-01-删除与并发事务.md 第 1595 行", "12 个新开关；第 1595 行有轮名要的用户原话", "对", "")
C("E39", [("RC", "30 份 +1283/−237")], "批 4 四份补丁打进集成副本 30 份 +1283/−237", "K:int.shortstat.before-b4..after-b4(.M/.A/.D)", "全部 52 个文件 +1425/−237；其中改动 29 个 +1283/−232、删除 1 个 −5（共 30 个 +1283/−237），新增 22 个 +142", "对不上", "30/+1283 不含 22 个新增文件；四份补丁自己的和 52 个文件、+1425 才对得上")
C("E40", [("RC", "5c2c79b")], "批 4 收尾 16 份；after-b4-fixup = 5c2c79b", "K:int.shortstat；git rev-parse", "16 份 +57/−89；标签 after-b4-fixup 提交 5c2c79b", "对", "")
C("E41", [("RC", "约 120 个弄坏开关")], "verify3 涉及 16 项自证与门禁格、约 120 个弄坏开关", "K:verify3.tally", "绿 16、开关 117", "对", "「约 120」是约数，117")
C("E42", [("RC", "第 14 项 code-tooling 六格五红")], "第 14 项 code-tooling 六格五红", "K:verify3.cell_code_tooling", "6 格里 5 格红（research-script-selftests、research-script-selftest-coverage、stage-owners、fixture-claims、heavy-thread-defaults）", "对", "")
C("E42b", [("RC", "122 + 950")], "fixture-claims 日期 122 + 950 处", "K:verify3.fixture_dates", "122 个文件名里的日期、950 处样本正文里的日期", "对", "")
C("E42c", [("RC", "单独重跑各 19、24、59 格全对")], "单独重跑 multi-host-run.sh 19 格、mutation-shard-run.sh 24 格", "K:verify3.parallel_selftests", "日志里 mutation-shard-run.sh 共 24 格（16 格判错）、multi-host-run.sh 共 19 格（10 格判错）", "对", "总格数对；「单独重跑全对」的结果没有留存日志")
C("E42d", [("RC", "单独重跑各 19、24、59 格全对")], "crates-mutation-rows.py 自证 59 格", "无（登记自证，不许跑；verify3 日志只有堆栈）", "—", "复核不了", "")
C("E43", [("RC", "其余 129 项对")], "verify3 其余 129 项对：16 项自证与门禁格绿、约 113 个弄坏开关各自红", "K:verify3.tally", "129 = 绿 14 + 红 115", "对不上", "总数 129 对，拆分错（同 C05）")
C("E44", [("RC", "快照 30 份")], "第二轮快照 30 份、kb 3 份、清单抄 48 不抄 6", "K:snapshot.r2.* / checklist.r2", "30；kb 3；抄 48、不抄 6", "对", "")
C("E45", [("RC", "材料 1 份 kb、清单抄 32 不抄 2")], "第一轮材料 1 份 kb、清单抄 32 不抄 2；快照 21 份", "K:checklist.r1 / snapshot.r1", "kb 1；抄 32、不抄 2；21", "对", "")
C("E46", [("RC", "A1–A4、B1–B3、C1–C2、D1–D2、E1–E3")], "攻方 14 个编号、改法 P1–P10、八行 same、八份样本", "K:r2opus.rows_hit_ids / rows_P", "A1–A4、B1–B3、C1–C2、D1–D2、E1–E3 共 14 个；P 行 10", "对", "")
C("E47", [("RC", "第一轮 41 条打中的历史修到 33")], "辩方：41 条修到 33、部分 2、没修到 6，改过头 0；判轻 14、判重 1", "K:r2sonnet.table1_tally / table3_tally / section1_rows", "41 行；33、2、6；判轻 14、判重 1", "对", "「修到」含 5 行写「文字修到」的（钩子没改）；报告首段自己说明了")
C("E48", [("RC", "另立账八笔")], "判决二另立账八笔", "K:v2.pile_line_ids", "8 笔", "对", "")
C("E49", [("RC", "9 份文字")], "批 5 丁 9 份、乙 4 份、丙 50 份、甲 19 份", "K:patch.b5-*", "9、4、50、19", "对", "")
C("E50", [("RC", "82 份，标签 `after-b5`")], "批 5 四份补丁 82 份（无冲突）", "K:int.shortstat.after-b4-fixup2..after-b5", "82 个文件 +1175/−257（改动 23、新增 59）", "对", "这一处的 82 是全部文件（含新增），批 4 那处的 30 不含，两处口径不同")
C("E51", [("RC", "d16-item1-r2、c364-r3、smalldata-ttime-r1、c196-r1")], "2026-10-12 之前的登记行 4 行", "读 r3 冻结副本 research/prompts/abandoned-rounds.tsv", "c196-r1、d16-item1-r2、c364-r3、smalldata-ttime-r1 四行（第 7–10 行）", "对", "")
C("E52", [("RC", "13 份自证与 4 道样本档绿")], "verify4：13 份自证与 4 道样本档绿，其余 147 项对", "K:verify4.tally；读 summary 项名", "绿期望 17 项（判对 16、判错 1）；其中 stage-* 4 项、cell-* 2 项、其余 11 项（含判错 1、env-chdir 2）；开关 131 项全红", "对不上", "13 是非 stage 的绿期望项（含 2 个门禁格、含判错的那 1 项），判对的自证只有 10 份")
C("E53", [("RC", "六格 5 红")], "—", "—", "—", "对", "")
C("E54", [("RC", "四份自证绿")], "批 5 收尾：四份自证绿、四个开关各自红（types-before-nine 红 7 格）", "verify5-out 日志", "dp、rdg、htg、lht 四份绿；rdg-anyref、htg-unrec、lht-unrec、dp-types 四个红；dp-types 7 个「✗ 自检」", "对", "")
C("E55", [("RC", "快照 35 份")], "第三轮快照 35 份；kb 1 份、清单抄 32 不抄 2", "K:snapshot.r3 / checklist.r3", "35；1；32、2", "对", "")
C("E56", [("RC", "原始证据 5 份拷进")], "同 E10", "同 E10", "同 E10", "对", "")
for r in [x for x in ROWS if x[0] in ("E53", "E56")]:
    ROWS.remove(r)
C("P01", [("RC", "766f46bd")], "批 4 丁补丁 4 份 +15/−15", "K:patch.b4-defs", "files=4 +15/-15", "对", "")
C("P02", [("RC", "9be52da9")], "批 4 丙补丁 39 份 +488/−30", "K:patch.b4-gates", "files=39 +488/-30", "对", "")
C("P03", [("RC", "90406101")], "批 4 乙补丁 4 份 +434/−41", "K:patch.b4-guard", "files=4 +434/-41", "对", "")
C("P04", [("RC", "b2faf14e")], "批 4 收尾补丁 16 份", "K:patch.b4-fixup", "files=16 +57/-89", "对", "")
C("P05", [("RC", "d7e8a3a7")], "批 4 收尾二补丁加 4 行", "K:patch.b4-fixup2", "files=1 +4/-0", "对", "")
C("P06", [("RC", "44b58a14")], "批 5 丁 9 份文字；乙 4 份（多改 lib_shell_words.py 与登记表一行注释）；丙 50 份", "K:patch.b5-defs/-guard/-gates；K:patchfile.b5-guard.*", "9 份 +19/−11；4 份 +176/−39（lib_shell_words.py +50/−11、selftest-registry.tsv +1/−1）；50 份 +456/−60", "对", "")
C("P07", [("RC", "2faf527c")], "批 5 甲 19 份，新库 lib_abandoned_rounds.py", "K:patch.b5-dispatch；K:patchfile.b5-dispatch.lib_abandoned_rounds.py", "19 份 +524/−147；lib_abandoned_rounds.py 是新文件 254 行", "对", "")
C("P08", [("RC", "5c4b1c60")], "批 5 收尾 6 份，多改 lib_heavy_tests.py 文件头 8 行注释", "K:patch.b5-fixup；K:patchfile.b5-fixup.lib_heavy_tests.py", "6 份 +89/−19；lib_heavy_tests.py +5/−3（合计动 8 行）", "对", "「8 行」是增 5 删 3 的合计，没写口径")
C("P09", [("RC", "7 个新弄坏开关")], "—", "—", "—", "对", "")
for r in [x for x in ROWS if x[0] == "P09"]:
    ROWS.remove(r)

print("| 编号 | 引处 | 引的数 | 现算（命令 / out 键） | 现算结果 | 判 | 口径缺什么 |")
print("|---|---|---|---|---|---|---|")
for i, refs, claim, how, res, verdict, gap in ROWS:
    cells = [i, where(refs), claim, how, res, verdict, gap]
    print("| " + " | ".join(str(c).replace("|", "\\|") for c in cells) + " |")

if MISSING:
    print("MISSING:", MISSING, file=sys.stderr); sys.exit(3)

# ── sha256 前缀…末几位（逐个） ──
print()
print("| 编号 | 文件 | 记录/判决里写的位置与末位 | 现算末位 | 判 |")
print("|---|---|---|---|---|")
import ast
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "out", "count_checks.txt")
n = 0
for l in open(OUT, encoding="utf-8"):
    if not l.startswith("shaclaim."): continue
    key, _, val = l.rstrip("\n").partition("=")
    if val.startswith("记录里没写前缀"): continue
    name = key[len("shaclaim."):]
    items = ast.literal_eval(val)
    for pos, tail, ok in items:
        n += 1
        pos_txt = pos.replace("RC:", "调度记录:").replace("V1:", "判决一:").replace("V2:", "判决二:")
        print(f"| H{n:02d} | {name} | {pos_txt} 写「…{tail}」 | {'末位相同' if ok else '末位不同'} | {'对' if ok else '对不上'} |")
