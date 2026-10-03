#!/usr/bin/env python3
# admission: always 每一次调都判此刻的红例账、钉住的表与变异表，上一次的结论不替这一次作保
# run-condition: none 追加、补钉与 --check 只读写点名的文件；--replay 要的 cargo 与装置由它自己判，缺了判红或报本次未跑
"""E165 红例钉进快速档：把全量那一档红例账里的每个红例按输入原样钉进 .claude/gate.d/e165-pinned-red-examples.tsv，并判钉得全不全、今天还绿不绿。

用户 2026-10-01 定「全量形式化红了的例子 可以加到快速形式化中」「这个应该在每一次跑完之后就加入 门禁来控制比较好」
（调度记录 records/2026-10-01-里程碑三形式化接入.md 第三节）。表的格式写在表自己的文件头。

用法（<根> 默认取这份脚本往上两级的仓；--table 默认 <根>/.claude/gate.d/e165-pinned-red-examples.tsv）：
  e165-pin-red-examples.py [--root 根] [--table 表] [--kill-bounds 产物]… <红例账…>
      追加：账里还没钉的行追加进表，一行 21 段：自己的 ①–⑧ 后面原样接账那一行的 13 段（⑨–㉑）。
      已钉过的不重复：表里有一行 ⑨ 起与这一行账逐字节相同，且 ⑥ 等于这一次换出的名字或 ⑤ 就是这一行账。
      同一个（①, ⑥）而 ⑨ 起不同的（另一趟的账，⑱ 首红与否或 ⑨⑩ 的变体与表行不同）另钉一行。
      账里没有反例状态的行（序号 none，驱动报 no_single_state_differs）钉不了，只数不钉。
      ⑥ 写「<表>#<变异名>」，换名与 --check 同一套：每份账配同一趟的 kill-bounds 产物，不给 --kill-bounds 按文件名配（与 --check 相同），
      给了就逐份对应（第 i 个产物配第 i 份账，个数要相等；第二段驱动收尾时这样调）；配对的四样（见 --check 一节）有一样不过就判红、一行都不钉，不退回按今天的行号换名；
      配上了取产物里这个变体那一行（item= 等于账第 1 列、table_row= 等于账第 2 列的第一个表行）的
      table= 与 title=，table= 要是这种变体的表（tsv-<行号> 是 crates/mutations.tsv，Z<n> 是 research/mutations/e165_replay_bounded_enumeration_custom.tsv），
      在今天的那张表里找第 1 列（空格记成 _）等于 title= 的数据行，恰好一行就用它的变异名：那一趟之后变异表增删过行也对得上，不按今天的行号换。
      table= 不对、按名字找不到、找到不止一行的，逐行报出来、不钉，别的行照钉，退 1。
      同一个（①, ⑥）在表里的 ⑦ 与账里的不同：一行都不钉、判红。
      写：读表时记 sha256，同目录临时文件写完、改名换上之前再算一次，变了就不写（别的会话刚写过，重跑）；写完回读，新钉的每一行都要在。
  e165-pin-red-examples.py --check [--root 根] [--table 表] [<红例账…>]
      门禁 pinned-red-examples-complete 格：不给账就取 <根>/research/results/e165-*-red-examples.tsv 全部。账里每一行（有反例状态的）都要钉着：
      表里有一行 ① 等于这一行算出的指纹、⑦ 等于账的第 12 列、⑨ 起用制表符接起来与账那一行逐字节相同，
      且 ⑤ 就是这一行账或 ⑥ 是出这份账的那一趟给这个变体的名字。同一个输入钉着、只是 ⑨ 起不同的，逐行报「没整行钉」。
      那一趟的名字不按今天的变异表换（表在跑完之后删一行，后面的 tsv-<行号> 就全错一位），取同一趟的 kill-bounds 产物：
      账 <目录>/<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out（E165 第二段驱动 e165-replay-mutation-kill-bounds.sh 一趟写这一对）。
      配对四样都要过，有一样不过就判红：产物在；产物的收尾行 E7RESULT name=verdict 在、它的 ledger_sha256= 等于这份账此刻的 sha256（驱动写完账才写这一行）；
      账里有行的变体与产物里 red_examples= 大于 0 的变体是同一批、每个变体的账行数等于它的 red_examples=；
      账的每一行按（第 1 列变体, 第 2 列的第一个表行）在产物里有 E7RESULT name=kill_bound 那一行（item=、table_row=），带 title=。
      ⑥ 的表（# 之前）等于那一行的 table=、变异名（# 之后，空格记成 _）等于它的 title=，才算同一个变异。
      门禁那一格照原样起（--check --root <根>），按账的文件名找产物，不多传参数；--check 不收 --kill-bounds。
      没钉全的出路逐份给 python3 research/scripts/e165-pin-red-examples.py <账>：追加按同一趟产物的 title= 到今天的变异表里按名字找变异，
      那一趟之后变异表增删过行也钉得进；按名字找不到或找到几行的逐行报出来、不钉，不钉进错的名字。
      表自己也判（--check、--replay 与追加都先判）：每行 21 段（8 段的旧行报「没整行抄红例账」，出路给 --backfill）、① 是 64 位十六进制、
      ① 等于拿 ②③⑦ 重算的指纹、⑪⑫⑬ 与 ②③④ 相同、⑳㉑ 与 ⑦⑧ 相同（不同的点名哪一行哪一段，与装置 --pinned-cases 同一种报法）、
      （⑥, ⑨ 起的整行）不重复。少一行判红，出路给补进去的命令。一份账都没有退 77。
  e165-pin-red-examples.py --backfill [--root 根] [--table 表]
      补钉：表里 8 段的旧行（整行抄红例账之前钉的）按 ⑤（<账文件名>:<行号>）回 <根>/research/results/<账文件名> 取那一行，原样接成 ⑨–㉑。
      取回的要是 13 段的账行、第 3、4、5、12、13 段与这一行的 ②③④⑦⑧ 逐字相同；账不在、没有那一行、段数不对或对不上的逐行报出来、不填，
      取得到的照补；有取不到的退 1。写法与追加相同（读表时记 sha256、换上之前复核、写完回读补过的每一行）。没有 8 段的行退 0。
  e165-pin-red-examples.py --replay [--root 根] [--table 表] [--binary 可执行文件] [--memory-cap 上限|none] [--threads N] [--target 编译目录]
      门禁 pinned-red-examples-replay 格：先判每行 ⑥ 点名的变异（<表>#<名>，几个用 + 连）还在它的表里（第 1 列等于那个名字的数据行），
      再用装置 e165_replay_bounded_enumeration --pinned-cases <表> 在今天的实现上逐行判：stdout 里以「E7RESULT name=state 」起头的行
      条数等于表的数据行数、第 i 行与第 i 个数据行的 ⑦ 逐字节相同、装置退 0（约定：每行一行结果，有行对不上或指纹漂了退非 0）。
      不给 --binary 时：装置源码（crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs）里没有 --pinned-cases 就退 77
      （变异那一半照判，红了照红）；有就 cargo build --release --locked -p singlefs-checker-tier --bin e165_replay_bounded_enumeration
      （CARGO_TARGET_DIR 默认 ${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-e165-pinned-target）再跑。
      装置经 research/scripts/run-with-memory-cap.sh（上限默认 $GATE_E165_PINNED_MEMORY_MAX，再默认 2G）与 capped.sh N 起；--memory-cap none 直接起（样本的桩用）；
      经包装起而包装自己退 250–254 的，判「没跑成」：退 1，✗ 行写「没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」，
      → 照包装文件头「退出码」一段，不指向变异行或实现；
      装置被信号停了的（直接起时 subprocess 交负的信号号，经包装或 bash 起时退 128 + 信号号、不在 250–254 里，例 137 = 外面 kill -9），判红，
      ✗ 行写「装置被信号停了」、这一次的输出不算判定，→ 照包装文件头「退出码」一段（撞自己上限的由包装认成 250，外面停的同样退 137），不写修实现；
      --binary 以 .sh 结尾的经 bash 起（样本目录里的桩不带执行位）。
      编装置与跑装置时每过 300 秒（PROGRESS_INTERVAL_SECONDS，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」定的进度间隔）往 stderr 打一行
      「E165_PINNED_PROGRESS 挂钟 <秒> 秒：<在做什么> 还在跑；装置已打 n/m 行结果」（编装置那一步不带行数）；判定照旧等它跑完、按全部输出判。
  e165-pin-red-examples.py --selftest
      在临时仓里造账、变异表、产物与桩装置，逐格判；弄坏开关逐个打开，每个都要让至少一格判错。
退出码：0 绿（追加：钉好了或没有要钉的；补钉：都补上了或没有要补的）；1 红；2 用法错；77 本次未跑（--check 没有账、--replay 装置不认 --pinned-cases）。

弄坏开关 E165_PIN_RED_EXAMPLES_BREAK=<项>[,<项>…]（只给证红用；--selftest 与门禁样本在开着时都必须判错）：
  pin-twice              追加不去重                        pin-again 那一格判错
  titles-unchecked       追加按 title 找不到变异时，退回按今天的行号换名、不报   titles-mismatch、pin-by-title-not-found、pin-by-title-not-found-detail 三格判错
  no-recheck-before-write  改名换上之前不复核 sha256       race 那一格判错
  check-ignores-missing  --check 不报没钉的行               check-missing 那一格（样本 pinned-red-examples-complete-red）判绿
  fingerprint-unchecked  --check 不拿 ②③⑦ 重算 ①         check-tampered 那一格判绿
  variant-unchecked      --replay 不判变异还在不在          replay-variant-missing 那一格（样本 pinned-red-examples-replay-variant-red）判绿
  replay-ignores-output  --replay 只看装置退出码，不比 ⑦    replay-mismatch 那一格（样本 pinned-red-examples-replay-red）判绿
  wrapper-exit-as-replay-red  --replay 不认包装自己的 250–254，照装置退非 0 报回归   replay-wrapper-exit 那一格（自证里的假包装退 252）判错
  killed-as-regression   --replay 不认装置被信号停了，照装置退非 0 报回归、出路写修实现
                                                     replay-killed（桩装置 kill -9 自己）、replay-wrapper-137（假包装退 137）两格判错
  names-from-today-table --check 照今天的变异表换 tsv-<行号>，不认产物里的名字   check-shifted-table（自证里删一行、末尾加一行、行号错位的合成表）判红
  kill-bounds-optional   --check 账配的产物不在也不红，照今天的变异表换名     check-kill-bounds-missing 那一格判绿
  title-missing-ignored  --check 产物里那个变体没有那一行或没有 title= 也不红，照今天的变异表换名   check-title-missing 那一格判绿
  title-not-compared     --check 产物里有 title= 就算钉着，不比 ⑥               check-title-other-variant 那一格判绿
  variant-sets-unchecked --check 不比账与产物的变体与行数                   check-variant-sets-differ 那一格判绿
  ledger-digest-unchecked  --check 不核产物收尾行记的账 sha256、不认没收尾     check-ledger-digest、check-run-unfinished 两格判绿
整行抄红例账（一行 21 段）的几项，开关一行、下一行起写打开之后判错的自证格与门禁样本（样本在 .claude/gate.d/fixtures/checker-tier-formal-verification.sh/ 下）：
  whole-row-unchecked    --check 不比 ⑨ 起的整行，只认 ①、⑦ 与 ⑤ / ⑥（整行抄之前的判法）
      自证 check-whole-row-differs 判绿；样本 pinned-red-examples-complete-whole-row-red 判绿
  old-rows-accepted      表里 8 段的旧行照收、不报没整行抄（装置改收 21 段之前的判法）
      自证 check-old-row、replay-old-row、replay-old-row-howto 判错；样本 pinned-red-examples-replay-old-row-red 判绿
  copied-columns-unchecked  不核 ⑪⑫⑬ 与 ②③④、⑳㉑ 与 ⑦⑧
      自证 replay-copied-domain-differs、replay-copied-original-differs 判绿；样本 pinned-red-examples-replay-copied-columns-red 判绿
  dedupe-by-input-variant  追加按（①, ⑥）去重：另一趟的账 ⑨ 起不同也算已钉（整行抄之前的判法）
      自证 pin-whole-row-variant 判错（该新钉 1 行，钉了 0 行），接着 check-whole-row-variant 判红
  backfill-unverified    --backfill 按 ⑤ 取回的账行不核第 3、4、5、12、13 段就接上
      自证 backfill-mismatched-origin、backfill-mismatched-origin-detail 判错（把别的账行接了上去，不报取不到原行）
  补钉写表的复核与追加共用 no-recheck-before-write：打开之后自证 backfill-race、backfill-race-wrote 同样判错
  variant-by-line-number  追加配上了产物也按今天的行号换名、再核 title 相同（按名字找之前的判法：那一趟之后变异表增删过行就一行都不钉）
      自证 pin-by-title-shifted-table、check-by-title-shifted-table、pin-by-title-same-names 判错（该钉进 3 行，换出的名字与 title 不同、一行都没钉）；
      pin-by-title-not-found、pin-by-title-not-found-detail、pin-by-title-ambiguous、pin-by-title-ambiguous-detail 判错（不报按名字找不到 / 找到几行，同名的照行号钉了进去）
追加按文件名配同一趟产物（与 --check 同一套）的两项：
  append-names-from-today-table  追加配上了产物也不用它的名字，照今天的变异表按行号换名（不给 --kill-bounds 时的旧判法）
      自证 pin-deleted-row-names 判错（今天的表删了一行、末尾加了一行，钉进的是错位那两行的名字）；
      titles-mismatch、pin-by-title-not-found、pin-by-title-not-found-detail、pin-by-title-ambiguous、pin-by-title-ambiguous-detail、pin-by-title-same-names 同样判错
  append-pairing-optional  追加配不上产物（产物不在、收尾行记的账 sha256 不同）也不红，照今天的变异表按行号换名照钉
      自证 pin-kill-bounds-missing、pin-kill-bounds-missing-wrote、pin-ledger-digest、pin-explicit-kill-bounds-digest、pin-deleted-row、pin-deleted-row-names 判错
--replay 的进度行：
  progress-silent        编装置、跑装置时不打进度行（跑完之前什么都看不到）
      自证 replay-progress 判错（假时钟下装置跑过进度间隔，一行 E165_PINNED_PROGRESS 都没有）
"""
import glob
import hashlib
import os
import re
import subprocess
import sys
import tempfile
import threading
import time

sys.dont_write_bytecode = True
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
DEFAULT_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
TABLE = ".claude/gate.d/e165-pinned-red-examples.tsv"
LEDGER_GLOB = "research/results/e165-*-red-examples.tsv"
LEDGER_SUFFIX, KILL_BOUNDS_SUFFIX = "-red-examples.tsv", "-kill-bounds.out"
CRATES_TABLE = "crates/mutations.tsv"
CUSTOM_TABLE = "research/mutations/e165_replay_bounded_enumeration_custom.tsv"
DEVICE_SOURCE = "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
DEVICE_BINARY = "e165_replay_bounded_enumeration"
DEVICE_FLAG = "--pinned-cases"
RESULT_PREFIX = "E7RESULT name=state "
LEDGER_COLUMNS, TABLE_OWN_COLUMNS = 13, 8
TABLE_COLUMNS = TABLE_OWN_COLUMNS + LEDGER_COLUMNS
COLUMN_MARKS = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳㉑"
# 抄来的账行与钉表自己的段要相同的几对：（账的第几段, 钉表自己的第几段）；账第 3、4、5 段是域、N、序号，第 12、13 段是两条逐态行
COPIED_PAIRS = ((3, 2), (4, 3), (5, 4), (12, 7), (13, 8))
TABLE_PROBLEMS_HOWTO = ("8 段的旧行（没整行抄红例账）跑 python3 research/scripts/e165-pin-red-examples.py --backfill 补成 21 段（按 ⑤ 回账里取原行）；"
                        "别的是手改坏了：表只由这份脚本写，从 git 找回再钉")
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
BREAK = {item for item in os.environ.get("E165_PIN_RED_EXAMPLES_BREAK", "").split(",") if item}
MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "run-with-memory-cap.sh")
# 内存包装自己的结局（run-with-memory-cap.sh 文件头「退出码」一段）：经它起时装置这一次没跑成，输出不算判定
WRAPPER_OWN_EXITS = {250: "撞了这一条自己的内存上限", 251: "包装用法错或起不了 scope", 252: "内存不够排不上", 253: "超过限时", 254: "被总上限挤掉"}
# --replay 编装置、跑装置时的进度行（文件头 --replay 一段）：间隔照 .claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」；时钟、轮询间隔与去处自证换成假的
PROGRESS_INTERVAL_SECONDS = 300
PROGRESS_POLL_SECONDS = 1.0
PROGRESS_CLOCK = time.monotonic
PROGRESS_SINK = None   # None 是 sys.stderr


class Red(Exception):
    def __init__(self, message, howto):
        super().__init__(message)
        self.message, self.howto = message, howto


class NotRun(Exception):
    pass


def report_red(message, howto):
    print(f"  ✗ {message}")
    print(f"     → 怎么办：{howto}")


def file_digest(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def state_before_outcome(line):
    """逐态行里 state= 的值（整行最后一段），截到 ;first= 之前；认不出交 None。"""
    match = re.search(r"(?:^| )state=(\S+)$", line)
    return match.group(1).split(";first=")[0] if match else None


def input_fingerprint(domain, size, original_line):
    state = state_before_outcome(original_line)
    if state is None:
        return None
    return hashlib.sha256(f"{domain}|{size}|{state}".encode("utf-8")).hexdigest()


# ── 读账、读表、换变异名 ──

def read_ledger(path):
    rows = []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        if len(fields) != LEDGER_COLUMNS:
            raise Red(f"红例账 {path} 第 {number} 行是 {len(fields)} 段，要 {LEDGER_COLUMNS} 段", "账由 E165 第二段驱动写，格式在它的表头；手改坏了从 git 找回")
        rows.append({"number": number, "fields": fields})
    return rows


def mutation_names(root, relative):
    """变异表的 {物理行号: 变异名}（数据行）。"""
    path = os.path.join(root, relative)
    if not os.path.isfile(path):
        raise Red(f"变异表 {relative} 不在", "它随仓走；被删了从 git 找回")
    names = {}
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if raw.strip() and not raw.startswith("#"):
            names[number] = raw.split("\t")[0]
    return names


def read_kill_bounds(path):
    """第二段产物里每一行 E7RESULT name=kill_bound 的 item、table、table_row、red_examples、title（缺了记 None），
    与收尾行 E7RESULT name=verdict 的 ledger_sha256（没有收尾行记 None，有行而没有这一段记空串）。"""
    records, ledger_digest = [], None
    for line in open(path, encoding="utf-8").read().split("\n"):
        if line.startswith("E7RESULT name=verdict "):
            match = re.search(r" ledger_sha256=([0-9a-f]{64})(?: |$)", line)
            ledger_digest = match.group(1) if match else ""
        if not line.startswith("E7RESULT name=kill_bound "):
            continue
        record = {}
        for key in ("item", "table", "table_row", "red_examples", "title"):
            match = re.search(rf" {key}=(\S+)", line)
            record[key] = match.group(1) if match else None
        records.append(record)
    return records, ledger_digest


def shown_path(path, root):
    """仓里的写相对仓根的路径，仓外的写绝对路径。"""
    relative = os.path.relpath(path, root)
    return os.path.abspath(path) if relative == ".." or relative.startswith("../") else relative


def paired_kill_bounds(ledger_path):
    """账 <目录>/<前缀>-red-examples.tsv 配同一趟的 <目录>/<前缀>-kill-bounds.out；文件名不是这个形态交 None。"""
    base = os.path.basename(ledger_path)
    if not base.endswith(LEDGER_SUFFIX):
        return None
    return os.path.join(os.path.dirname(ledger_path), base[:-len(LEDGER_SUFFIX)] + KILL_BOUNDS_SUFFIX)


def first_table_row(ledger_path, row):
    rows_text = row["fields"][1]
    first_row = rows_text.split("+")[0]
    if not first_row.isdigit():
        raise Red(f"{os.path.basename(ledger_path)}:{row['number']} 第 2 列「{rows_text}」认不出表行", "账的第 2 列是表行号（几行同一个改法用 + 连）")
    return int(first_row)


RUN_PAIRING_HOWTO = ("账与同一趟的产物按文件名配对（<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out，驱动 research/scripts/e165-replay-mutation-kill-bounds.sh 一趟写这一对）："
                     "产物或账被删、被改了从 git 找回同一趟的原样（git log --all --diff-filter=D --name-only -- '*<文件名>*'，git show <提交>:<路径>）；"
                     "找不回的重跑那一趟再钉；没有同一趟产物的账（手造的、改过名的）不放进 research/results/")


def run_variant_names(root, ledger_path, ledger_rows, product=None):
    """出这份账的那一趟给每个（变体, 表行）的（表, title）；配对不过 raise Red。开关 kill-bounds-optional 下产物不在交 None。
    product 不给就按文件名配（--check 只走这一支）。"""
    where = shown_path(ledger_path, root)
    product = product or paired_kill_bounds(ledger_path)
    if product is None or not os.path.isfile(product):
        if "kill-bounds-optional" in BREAK:
            return None
        shown = f"配的 kill-bounds 产物 {shown_path(product, root)} 不在" if product else f"文件名不以 {LEDGER_SUFFIX} 收尾，配不出 kill-bounds 产物"
        raise Red(f"红例账 {where} {shown}：认不出出这份账的那一趟给变体起的名字", RUN_PAIRING_HOWTO)
    records, ledger_digest = read_kill_bounds(product)
    if "ledger-digest-unchecked" not in BREAK:
        if ledger_digest is None:
            raise Red(f"{shown_path(product, root)} 没有收尾行 E7RESULT name=verdict：这一趟没跑完，配不上红例账 {where}",
                      "照驱动文件头：重跑同一条命令只补没跑的变体，26 个都落行才写收尾行（账的 sha256 记在那一行）；之后再判、再钉")
        actual = file_digest(ledger_path)
        if ledger_digest != actual:
            raise Red(f"{shown_path(product, root)} 收尾行记的账 sha256 是 {ledger_digest or '（没写）'}，红例账 {where} 此刻是 {actual}：账与产物不是同一趟",
                      RUN_PAIRING_HOWTO)
    if "variant-sets-unchecked" not in BREAK:
        ledger_counts = {}
        for row in ledger_rows:
            ledger_counts[row["fields"][0]] = ledger_counts.get(row["fields"][0], 0) + 1
        product_counts = {}
        for record in records:
            count = int(record["red_examples"]) if record["red_examples"] and record["red_examples"].isdigit() else 0
            product_counts.setdefault(record["item"], set()).add(count)
        differences = []
        for item in sorted(set(ledger_counts) | {item for item, counts in product_counts.items() if counts != {0}}):
            counts = product_counts.get(item)
            if counts is None:
                differences.append(f"{item}：账里 {ledger_counts[item]} 行，产物里没有这个变体")
            elif counts != {ledger_counts.get(item, 0)}:
                differences.append(f"{item}：账里 {ledger_counts.get(item, 0)} 行，产物的 red_examples= 是 {'/'.join(str(count) for count in sorted(counts))}")
        if differences:
            raise Red(f"红例账 {where} 与 {shown_path(product, root)} 的变体集合或行数不同（{len(differences)} 处：{'；'.join(differences[:5])}）：不是同一趟写的", RUN_PAIRING_HOWTO)
    names = {(record["item"], int(record["table_row"])): (record["table"], record["title"])
             for record in records if record["item"] and record["table_row"] and record["table_row"].isdigit()}
    if "title-missing-ignored" not in BREAK:
        for row in ledger_rows:
            key = (row["fields"][0], first_table_row(ledger_path, row))
            if names.get(key, (None, None))[1] is None:
                raise Red(f"{os.path.basename(ledger_path)}:{row['number']}：{shown_path(product, root)} 里没有 item={key[0]} table_row={key[1]} 那一行，或那一行没有 title=",
                          RUN_PAIRING_HOWTO)
    return names


class Unresolved(Exception):
    """追加给了 --kill-bounds 而这一行账按 title 找不到唯一的变异：逐行报、不钉这一行。"""
    pass


class VariantNames:
    def __init__(self, root):
        self.root, self.cache = root, {}

    def names_of(self, relative):
        if relative not in self.cache:
            self.cache[relative] = mutation_names(self.root, relative)
        return self.cache[relative]

    def variant(self, ledger_path, row, titles=None):
        """账的第 1、2 列换成「<表>#<名>」。titles（同一趟产物的 {(变体, 表行): (table=, title=)}，run_variant_names 交的）给了就按 title= 到今天的表里找名字，
        不给（--check 的开关那几支）就按今天的行号换。账写坏了 raise Red；按名字找不到或找到几行 raise Unresolved。"""
        item, rows_text = row["fields"][0], row["fields"][1]
        first_row = rows_text.split("+")[0]
        where = f"{os.path.basename(ledger_path)}:{row['number']}"
        if not first_row.isdigit():
            raise Red(f"{where} 第 2 列「{rows_text}」认不出表行", "账的第 2 列是表行号（几行同一个改法用 + 连）")
        line_number = int(first_row)
        match = re.fullmatch(r"tsv-(\d+)", item)
        if match:
            table = CRATES_TABLE
            if int(match.group(1)) != line_number:
                raise Red(f"{where}：变体 {item} 与第 2 列的表行 {rows_text} 对不上", "账写坏了；从 git 找回原样")
        elif re.fullmatch(r"Z\d+", item):
            table = CUSTOM_TABLE
        else:
            raise Red(f"{where}：变体「{item}」既不是 tsv-<行号> 也不是 Z<n>", "只认 E165 第二段驱动写的两种变体名")
        if titles is not None and "variant-by-line-number" not in BREAK:
            return self.variant_by_title(where, item, line_number, table, titles)
        name = self.names_of(table).get(line_number)
        if name is None or (table == CUSTOM_TABLE and not name.startswith(item + " ")):
            raise Red(f"{where}：{table} 第 {line_number} 行不是变体 {item} 的变异（读到 {name!r}）",
                      "表在跑完之后被改过、行号对不上：拿跑那一趟时的表（git show <那次提交>:<表>）核出变异名，或重跑那一趟再钉")
        if titles is not None and "titles-unchecked" not in BREAK:
            title = titles.get((item, line_number), (None, None))[1]
            if title is None or title != name.replace(" ", "_"):
                raise Red(f"{where}：变体 {item} 在第二段产物里的 title 是 {title!r}，今天的 {table} 第 {line_number} 行是 {name!r}",
                          "表在跑完之后被改过：拿跑那一趟时的表核出变异名，或重跑那一趟再钉")
        return f"{table}#{name}"

    def variant_by_title(self, where, item, line_number, table, titles):
        """出这份账的那一趟给这个变体的 title=，在今天的表里按名字找：第 1 列空格记成 _ 之后等于它的数据行恰好一行。"""
        table_of_run, title = titles.get((item, line_number), (None, None))
        if title is None:
            raise Unresolved(f"{where}：--kill-bounds 产物里没有 item={item} table_row={line_number} 那一行，或那一行没有 title=")
        if table_of_run != table:
            raise Unresolved(f"{where}：产物里变体 {item} 那一行的 table= 是 {table_of_run}，这种变体的表是 {table}")
        hits = [(number, name) for number, name in sorted(self.names_of(table).items()) if name.replace(" ", "_") == title]
        if not hits and "titles-unchecked" in BREAK and line_number in self.names_of(table):
            return f"{table}#{self.names_of(table)[line_number]}"
        if not hits:
            raise Unresolved(f"{where}：变体 {item} 那一趟的 title 是 {title!r}，今天的 {table} 里按名字找不到（第 1 列空格记成 _ 之后没有一行等于它）")
        if len(hits) > 1:
            raise Unresolved(f"{where}：变体 {item} 那一趟的 title 是 {title!r}，今天的 {table} 里找到 {len(hits)} 行同名（第 {'、'.join(str(number) for number, _name in hits)} 行），认不出是哪一个")
        return f"{table}#{hits[0][1]}"


UNRESOLVED_HOWTO = ("按名字找不到的：那一趟之后这条变异被删或改了名，被删的从 git 找回那一行（git log -S '<变异名>' -- <表>）再跑同一条命令，"
                    "改了名的交主 agent 定（在今天的变异表上重跑出这份账的那一趟再钉）；找到几行同名的：先把表里重名的变异改成不同的名字（变异名全表唯一），"
                    "再跑同一条命令；已钉进的不重复，不手填 ⑥")


def pinned_fields(ledger_path, row, variant):
    fields = row["fields"]
    domain, size, ordinal, original, mutant = fields[2], fields[3], fields[4], fields[11], fields[12]
    fingerprint = input_fingerprint(domain, size, original)
    if fingerprint is None:
        raise Red(f"{os.path.basename(ledger_path)}:{row['number']} 第 12 列认不出 state=", "账的第 12 列是原实现的逐态行（E7RESULT name=state …），手改坏了从 git 找回")
    return [fingerprint, domain, size, ordinal, f"{os.path.basename(ledger_path)}:{row['number']}", variant, original, mutant, *fields]


def copied_ledger_row(fields):
    """钉表一行 ⑨ 起用制表符接起来：整行抄的那一行账（8 段的旧行交空串）。"""
    return "\t".join(fields[TABLE_OWN_COLUMNS:])


def first_differing_column(fields, ledger_fields):
    """钉表一行 ⑨ 起与账那一行第一处不同的段，写成「⑱（账第 10 段）」；段数不同写段数。"""
    copied = fields[TABLE_OWN_COLUMNS:]
    if len(copied) != len(ledger_fields):
        return f"⑨ 起是 {len(copied)} 段、账那一行是 {len(ledger_fields)} 段"
    for index, (pinned_value, ledger_value) in enumerate(zip(copied, ledger_fields)):
        if pinned_value != ledger_value:
            return f"{COLUMN_MARKS[TABLE_OWN_COLUMNS + index]}（账第 {index + 1} 段）"
    return "（逐字节相同）"


def has_example(row):
    return row["fields"][4] != "none" and row["fields"][11] != ""


def read_table(path):
    if not os.path.isfile(path):
        raise Red(f"钉住的表 {path} 不在", "它随仓走，表头写着格式；被删了从 git 找回")
    text = open(path, encoding="utf-8").read()
    rows = []
    for number, raw in enumerate(text.split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        rows.append({"number": number, "fields": raw.split("\t")})
    return text, rows


def table_problems(rows):
    problems, seen = [], set()
    for row in rows:
        fields = row["fields"]
        if len(fields) == TABLE_OWN_COLUMNS:
            if "old-rows-accepted" in BREAK:
                continue
            problems.append(f"第 {row['number']} 行没整行抄红例账（{TABLE_OWN_COLUMNS} 段的旧格式，要 {TABLE_COLUMNS} 段）")
            continue
        if len(fields) != TABLE_COLUMNS:
            problems.append(f"第 {row['number']} 行是 {len(fields)} 段，要 {TABLE_COLUMNS} 段（自己的 {TABLE_OWN_COLUMNS} 段后面接红例账一行的 {LEDGER_COLUMNS} 段）")
            continue
        if not re.fullmatch(r"[0-9a-f]{64}", fields[0]):
            problems.append(f"第 {row['number']} 行的 ① 不是 64 位小写十六进制")
        elif input_fingerprint(fields[1], fields[2], fields[6]) != fields[0] and "fingerprint-unchecked" not in BREAK:
            problems.append(f"第 {row['number']} 行的 ① 与拿 ②③⑦ 重算的指纹不同（手改过这一行）")
        if "copied-columns-unchecked" not in BREAK:
            for ledger_column, own_column in COPIED_PAIRS:
                copied_index = TABLE_OWN_COLUMNS + ledger_column - 1
                if fields[copied_index] != fields[own_column - 1]:
                    problems.append(f"第 {row['number']} 行：{COLUMN_MARKS[copied_index]}（红例账第 {ledger_column} 段）与 {COLUMN_MARKS[own_column - 1]} 不同")
        key = (fields[5], copied_ledger_row(fields))
        if key in seen:
            problems.append(f"第 {row['number']} 行的 ⑥ 与 ⑨ 起的整行都与前面一行相同（重复钉了）")
        seen.add(key)
    return problems


def write_table(table, new_text, digest_at_read):
    """同目录临时文件写完、改名换上之前复核读表时的 sha256（变了就不写）；不成 raise Red。"""

    def unchanged_since_read():
        if "no-recheck-before-write" in BREAK:
            return
        if current_digest(table) != digest_at_read:
            raise ChangedSinceRead()

    try:
        replace_file_contents_by_rename(table, new_text, check_before_rename=unchanged_since_read)
    except ChangedSinceRead:
        raise Red(f"读到写之间 {table} 被别人改过，一行都没写", "别的会话刚写过这张表：重跑同一条命令")
    except ReplaceRefused as refused:
        raise Red(f"没换上：{str(refused).splitlines()[0]}", str(refused).splitlines()[-1] if len(str(refused).splitlines()) > 1 else "看上面的原因")


# ── 追加 ──

def run_names_for_append(root, ledgers, products):
    """追加换名用的那一趟的名字，与 --check 同一套：每份账配同一趟的产物（run_variant_names 的四样），
    products 给了（--kill-bounds，与账逐份对应，第二段驱动收尾时这样调）用给的那一份，不给按文件名配；
    有一份配不上就 raise Red，一行都不钉。{账: {(变体, 表行): (table=, title=)} 或 None（只在开关下）}。"""
    names = {}
    for index, ledger in enumerate(ledgers):
        try:
            names[ledger] = run_variant_names(root, ledger, read_ledger(ledger), products[index] if products else None)
        except Red as red:
            if "append-pairing-optional" not in BREAK:
                raise Red(f"{red.message}（追加按出这份账的那一趟的名字换名，配不上就一行都没钉）", red.howto)
            names[ledger] = None
        if "append-names-from-today-table" in BREAK:
            names[ledger] = None
    return names


def append(root, table, ledgers, products):
    text, existing = read_table(table)
    problems = table_problems(existing)
    if problems:
        raise Red(f"钉住的表 {table} 本身不对：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    digest_at_read = file_digest(table)
    run_names = run_names_for_append(root, ledgers, products)
    by_key = {}
    for row in existing:
        by_key.setdefault((row["fields"][0], row["fields"][5]), row["fields"])
    by_variant = {(row["fields"][5], copied_ledger_row(row["fields"])) for row in existing}
    by_origin = {(row["fields"][4], copied_ledger_row(row["fields"])) for row in existing}
    names = VariantNames(root)
    new_rows, already, without_example, conflicts, total, unresolved = [], 0, 0, [], 0, []
    for ledger in ledgers:
        for row in read_ledger(ledger):
            total += 1
            if not has_example(row):
                without_example += 1
                continue
            try:
                variant = names.variant(ledger, row, run_names[ledger])
            except Unresolved as problem:
                unresolved.append(str(problem))
                continue
            fields = pinned_fields(ledger, row, variant)
            key, whole = (fields[0], fields[5]), copied_ledger_row(fields)
            if "pin-twice" not in BREAK:
                if key in by_key and by_key[key][6] != fields[6]:
                    conflicts.append(f"{fields[4]}（{fields[5]}）：表里 ⑦ 是 {by_key[key][6][:120]}…，账里是 {fields[6][:120]}…")
                    continue
                if (fields[5], whole) in by_variant or (fields[4], whole) in by_origin or ("dedupe-by-input-variant" in BREAK and key in by_key):
                    already += 1
                    continue
            new_rows.append(fields)
            by_key.setdefault(key, fields)
            by_variant.add((fields[5], whole))
            by_origin.add((fields[4], whole))
    for problem in unresolved:
        print(f"     没钉：{problem}")  # gate-lint:detail
    unresolved_note = f"；{len(unresolved)} 行按 title 找不到唯一的变异、没钉" if unresolved else ""
    if conflicts:
        for conflict in conflicts[:5]:
            print(f"     {conflict}")  # gate-lint:detail
        raise Red(f"{len(conflicts)} 行账与表里同一个（输入, 变异）的原实现逐态行不同，一行都没钉",
                  "原实现在这个输入上的行为变了：先弄清是实现改了行为（要三方认）还是账跑在别的树上，再决定改表还是重跑")
    if not new_rows:
        if unresolved:
            raise Red(f"{len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态{unresolved_note}，一行都没新钉", UNRESOLVED_HOWTO)
        print(f"  ✓ {len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态，没有要新钉的；表里 {len(existing)} 行")
        return EXIT_GREEN
    new_text = (text if text.endswith("\n") or not text else text + "\n") + "".join("\t".join(fields) + "\n" for fields in new_rows)
    write_table(table, new_text, digest_at_read)
    _text, after = read_table(table)
    present = {"\t".join(row["fields"]) for row in after}
    lost = [fields for fields in new_rows if "\t".join(fields) not in present]
    if lost:
        raise Red(f"写完回读：新钉的 {len(new_rows)} 行里 {len(lost)} 行不在表里", "别的会话在改名换上之后又写过；重跑同一条命令")
    if unresolved:
        raise Red(f"钉进 {len(new_rows)} 行（回读都在；{len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态）{unresolved_note}",
                  UNRESOLVED_HOWTO)
    print(f"  ✓ 钉进 {len(new_rows)} 行（{len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态），回读都在；表里现有 {len(after)} 行")
    return EXIT_GREEN


class ChangedSinceRead(Exception):
    pass


# ── 补钉：8 段的旧行按 ⑤ 回账里取原行，接成 21 段 ──

def backfill(root, table):
    text, rows = read_table(table)
    digest_at_read = file_digest(table)
    old_rows = [row for row in rows if len(row["fields"]) == TABLE_OWN_COLUMNS]
    if not old_rows:
        print(f"  ✓ {os.path.relpath(table, root)} 的 {len(rows)} 行都不是 {TABLE_OWN_COLUMNS} 段的旧行，没有要补的")
        return EXIT_GREEN
    ledger_directory = os.path.dirname(LEDGER_GLOB)
    ledger_lines, filled, unrecoverable = {}, {}, []
    for row in old_rows:
        fields = row["fields"]
        name, _colon, number = fields[4].rpartition(":")
        if not name or not number.isdigit() or "/" in name:
            unrecoverable.append((row, f"⑤「{fields[4]}」不是 <红例账文件名>:<行号>"))
            continue
        if name not in ledger_lines:
            path = os.path.join(root, ledger_directory, name)
            ledger_lines[name] = open(path, encoding="utf-8").read().split("\n") if os.path.isfile(path) else None
        lines = ledger_lines[name]
        if lines is None:
            unrecoverable.append((row, f"⑤ 点的红例账 {ledger_directory}/{name} 不在"))
            continue
        if not 1 <= int(number) <= len(lines):
            unrecoverable.append((row, f"{ledger_directory}/{name} 没有第 {number} 行（共 {len(lines)} 行）"))
            continue
        line = lines[int(number) - 1]
        ledger_fields = line.split("\t")
        if line.startswith("#") or len(ledger_fields) != LEDGER_COLUMNS:
            unrecoverable.append((row, f"{ledger_directory}/{name} 第 {number} 行不是 {LEDGER_COLUMNS} 段的账行（{len(ledger_fields)} 段）"))
            continue
        if "backfill-unverified" not in BREAK:
            differing = [f"账第 {ledger_column} 段与 {COLUMN_MARKS[own_column - 1]}" for ledger_column, own_column in COPIED_PAIRS
                         if ledger_fields[ledger_column - 1] != fields[own_column - 1]]
            if differing:
                unrecoverable.append((row, f"{ledger_directory}/{name} 第 {number} 行与这一行对不上（{'、'.join(differing)} 不同）：账在钉过之后被改过，或 ⑤ 的行号错位"))
                continue
        filled[row["number"]] = fields + ledger_fields
    for row, reason in unrecoverable:
        print(f"     取不到原行：第 {row['number']} 行（⑤ {row['fields'][4]}）：{reason}")  # gate-lint:detail
    unrecoverable_howto = ("逐行照上面的原因找回那份账钉这一行时的原样（git log --all --diff-filter=D --name-only -- '*<账文件名>*'，git show <提交>:<路径>，"
                           "放回 research/results/）再跑 --backfill；找不回的这一行交主 agent 定（重跑出那份账的那一趟再钉），不手填、不照别的账行填")
    if not filled:
        raise Red(f"{os.path.relpath(table, root)} 里 {len(old_rows)} 行 {TABLE_OWN_COLUMNS} 段的旧行一行都取不到原行，一行都没补", unrecoverable_howto)
    new_text = "\n".join("\t".join(filled[number]) if number in filled else line for number, line in enumerate(text.split("\n"), 1))
    new_rows = [{"number": row["number"], "fields": filled.get(row["number"], row["fields"])} for row in rows]
    problems = table_problems([row for row in new_rows if len(row["fields"]) != TABLE_OWN_COLUMNS])
    if problems:
        raise Red(f"补完之后的表本身不对，一行都没写：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    write_table(table, new_text, digest_at_read)
    _text, after = read_table(table)
    present = {(row["number"], "\t".join(row["fields"])) for row in after}
    lost = [number for number, fields in filled.items() if (number, "\t".join(fields)) not in present]
    if lost:
        raise Red(f"写完回读：补过的 {len(filled)} 行里 {len(lost)} 行不在原来的行号上（第 {'、'.join(str(number) for number in lost[:5])} 行）",
                  "别的会话在改名换上之后又写过；重跑 --backfill")
    if unrecoverable:
        raise Red(f"补了 {len(filled)} 行（回读都在）；{len(unrecoverable)} 行取不到原行，没补、留着 {TABLE_OWN_COLUMNS} 段", unrecoverable_howto)
    print(f"  ✓ 补钉：{len(filled)} 行 {TABLE_OWN_COLUMNS} 段的旧行都按 ⑤ 取回账行接成 {TABLE_COLUMNS} 段，回读都在；表里现有 {len(after)} 行")
    return EXIT_GREEN


def current_digest(path):
    return file_digest(path)


# ── 格一：账都钉着 ──

def check(root, table, ledgers):
    if not ledgers:
        ledgers = sorted(glob.glob(os.path.join(root, LEDGER_GLOB)))
    if not ledgers:
        raise NotRun(f"{LEDGER_GLOB} 一份红例账都没有：这一格无对象可判")
    _text, rows = read_table(table)
    problems = table_problems(rows)
    if problems:
        raise Red(f"钉住的表本身不对：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    by_fingerprint = {}
    for row in rows:
        by_fingerprint.setdefault(row["fields"][0], []).append(row)
    names = VariantNames(root)
    missing, counted, without_example, paired, whole_row_differs = [], 0, 0, 0, 0
    for ledger in ledgers:
        ledger_rows = read_ledger(ledger)
        run_names = run_variant_names(root, ledger, ledger_rows)
        if run_names is not None:
            paired += 1
        if "names-from-today-table" in BREAK:
            run_names = None
        for row in ledger_rows:
            if not has_example(row):
                without_example += 1
                continue
            counted += 1
            origin = f"{os.path.basename(ledger)}:{row['number']}"
            fingerprint = input_fingerprint(row["fields"][2], row["fields"][3], row["fields"][11])
            same_input = [pinned_row for pinned_row in by_fingerprint.get(fingerprint, []) if pinned_row["fields"][6] == row["fields"][11]]
            if "whole-row-unchecked" in BREAK:
                candidates = [pinned_row["fields"] for pinned_row in same_input]
            else:
                # 整行抄：钉着的那一行 ⑨ 起用制表符接起来要与账这一行逐字节相同
                candidates = [pinned_row["fields"] for pinned_row in same_input if copied_ledger_row(pinned_row["fields"]) == "\t".join(row["fields"])]
                if same_input and not candidates:
                    whole_row_differs += 1
                    # 点名最像的那一行：⑨⑩（变体与表行）与账这一行相同的优先
                    nearest = next((pinned_row for pinned_row in same_input if pinned_row["fields"][8:10] == row["fields"][0:2]), same_input[0])
                    missing.append((ledger, f"{origin}（没整行钉：表第 {nearest['number']} 行钉着同一个输入与 ⑦，⑨ 起与账这一行不同，"
                                            f"第一处在 {first_differing_column(nearest['fields'], row['fields'])}）"))
                    continue
            pinned = any(fields[4] == origin for fields in candidates)
            if not pinned and candidates:
                table_of_run, title = (run_names or {}).get((row["fields"][0], first_table_row(ledger, row)), (None, None))
                if title is not None:
                    # 出这份账的那一趟的名字：⑥ 的表等于产物的 table=、变异名空格记成 _ 之后等于产物的 title=
                    pinned = "title-not-compared" in BREAK or any(
                        fields[5].partition("#")[0] == table_of_run and fields[5].partition("#")[2].replace(" ", "_") == title for fields in candidates)
                else:
                    # 只在开关下走到这里（产物不在、没有 title=，或不认产物）：照今天的变异表换名
                    try:
                        variant = names.variant(ledger, row)
                    except Red:
                        variant = None
                    pinned = any(fields[5] == variant for fields in candidates)
            if not pinned:
                missing.append((ledger, origin))
    if missing and "check-ignores-missing" not in BREAK:
        for _ledger, origin in missing[:10]:
            print(f"     没钉：{origin}")  # gate-lint:detail
        commands = [f"python3 research/scripts/e165-pin-red-examples.py {shown_path(ledger, root)}"
                    for ledger in sorted({ledger for ledger, _origin in missing})]
        whole_note = f"（其中 {whole_row_differs} 行表里钉着同一个输入，但没有一行 ⑨ 起与账那一行逐字节相同）" if whole_row_differs else ""
        raise Red(f"{len(ledgers)} 份红例账 {counted} 行里 {len(missing)} 行没钉进 {os.path.relpath(table, root)}{whole_note}",
                  f"补进去，一份账一条：{'；'.join(commands)}（已钉的不重复，⑨ 起不同的另钉一行整行抄；追加按文件名配同一趟的产物、按它的 title= 到今天的变异表里按名字找变异，按名字找不到或找到几行的逐行报、不钉，照它的出路办）")
    print(f"  ✓ {len(ledgers)} 份红例账 {counted} 行都钉在 {os.path.relpath(table, root)}，钉着的那一行 ⑨ 起与账那一行逐字节相同（{without_example} 行没有反例状态、不钉；"
          f"{paired} 份账与同一趟的 kill-bounds 产物配对都过，⑥ 按那一趟的名字认）；表里 {len(rows)} 行，① 都与 ②③⑦ 重算的相同，⑪⑫⑬⑳㉑ 与 ②③④⑦⑧ 相同")
    return EXIT_GREEN


# ── 格二：变异还在、今天的实现上绿 ──

def run_reporting_progress(command, cwd, environment, what, rows_total=None):
    """起 command，stdout、stderr 收进内存（判定照旧跑完再看）；跑着的时候每过 PROGRESS_INTERVAL_SECONDS 秒（按 PROGRESS_CLOCK）
    往 PROGRESS_SINK（默认 stderr）打一行「E165_PINNED_PROGRESS 挂钟 <秒> 秒：<what> 还在跑[；装置已打 n/m 行结果]」。交 CompletedProcess。"""
    process = subprocess.Popen(command, cwd=cwd, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    received = {"stdout": [], "stderr": []}

    def drain(stream, key):
        for line in stream:
            received[key].append(line)

    readers = [threading.Thread(target=drain, args=(process.stdout, "stdout"), daemon=True),
               threading.Thread(target=drain, args=(process.stderr, "stderr"), daemon=True)]
    for reader in readers:
        reader.start()
    sink = PROGRESS_SINK if PROGRESS_SINK is not None else sys.stderr
    started = last_line = PROGRESS_CLOCK()
    while True:
        try:
            process.wait(timeout=PROGRESS_POLL_SECONDS)
            break
        except subprocess.TimeoutExpired:
            pass
        now = PROGRESS_CLOCK()
        if now - last_line >= PROGRESS_INTERVAL_SECONDS and "progress-silent" not in BREAK:
            counted = ""
            if rows_total is not None:
                counted = f"；装置已打 {sum(1 for line in list(received['stdout']) if line.startswith(RESULT_PREFIX))}/{rows_total} 行结果"
            print(f"E165_PINNED_PROGRESS 挂钟 {now - started:.0f} 秒：{what} 还在跑{counted}", file=sink, flush=True)
            last_line = now
    for reader in readers:
        reader.join()
    return subprocess.CompletedProcess(command, process.returncode, "".join(received["stdout"]), "".join(received["stderr"]))


def replay(root, table, options):
    _text, rows = read_table(table)
    problems = table_problems(rows)
    if problems:
        raise Red("钉住的表本身不对：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    if not rows:
        raise NotRun(f"{os.path.relpath(table, root)} 一行都没有：这一格无对象可判")
    missing_variants = []
    if "variant-unchecked" not in BREAK:
        tables = {}
        for row in rows:
            for part in row["fields"][5].split("+"):
                relative, _hash, name = part.partition("#")
                if relative not in tables:
                    path = os.path.join(root, relative)
                    tables[relative] = set(mutation_names(root, relative).values()) if os.path.isfile(path) else None
                if tables[relative] is None or name not in tables[relative]:
                    missing_variants.append(f"第 {row['number']} 行：{part}")
    if missing_variants:
        for item in missing_variants[:10]:
            print(f"     不在了：{item}")  # gate-lint:detail
        raise Red(f"{len(rows)} 行里 {len(missing_variants)} 处点名的「改回去就红」的变异不在它的表里了",
                  "变异行被删或改了名：找回那一行（改法被悄悄撤回时它要先响），确是改名就照新名改这张表（要写明为什么）")
    binary = options["binary"]
    if binary is None:
        source = os.path.join(root, DEVICE_SOURCE)
        if not os.path.isfile(source) or DEVICE_FLAG not in open(source, encoding="utf-8").read():
            raise NotRun(f"{len(rows)} 行点名的变异都还在；装置 {DEVICE_SOURCE} 还不认 {DEVICE_FLAG}（第 4 批 3-6 才加），逐行重放这一半本次未跑")
        environment = dict(os.environ, CARGO_TARGET_DIR=options["target"])
        command = ["nice", "-n", "19", "cargo", "build", "--release", "--locked", "-p", "singlefs-checker-tier", "--bin", DEVICE_BINARY]
        completed = run_reporting_progress(command, root, environment, f"编装置 {DEVICE_BINARY}")
        if completed.returncode != 0:
            raise Red(f"编装置没成（{' '.join(command[3:])} 退 {completed.returncode}）：{' | '.join(completed.stderr.strip().split(chr(10))[-4:])}",
                      "先让它在平时的工具链上编得过；缓存缺包 cargo fetch --locked")
        binary = os.path.join(options["target"], "release", DEVICE_BINARY)
    # 以 .sh 结尾的（样本的桩）经 bash 起：样本目录里的文件不带执行位（script-modes.sh 不看 fixtures/）
    command = ["bash", binary, DEVICE_FLAG, os.path.abspath(table)] if binary.endswith(".sh") else [binary, DEVICE_FLAG, os.path.abspath(table)]
    wrapped = options["memory_cap"] != "none"
    if wrapped:
        command = ["bash", MEMORY_CAP_RUNNER, options["memory_cap"],
                   "bash", os.path.join(SCRIPT_DIRECTORY, "capped.sh"), str(options["threads"]), *command]
    environment = dict(os.environ, RUN_WITH_MEMORY_CAP_PARALLEL_REASON="门禁里逐行重放钉住的红例，与同一趟别的格各用各的上限",
                       RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="门禁的上限按钉住的表最大时给，表只涨不缩")
    completed = run_reporting_progress(command, root, environment, f"装置 {DEVICE_FLAG} 逐行重放钉住的 {len(rows)} 行", len(rows))
    if wrapped and completed.returncode in WRAPPER_OWN_EXITS and "wrapper-exit-as-replay-red" not in BREAK:
        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-4:] if line.strip())
        raise Red(f"没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 {completed.returncode}（{WRAPPER_OWN_EXITS[completed.returncode]}，上限 {options['memory_cap']}），"
                  f"钉住的 {len(rows)} 行这一次的输出不算判定{'（' + tail[:300] + '）' if tail else ''}",
                  "照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办：250 是装置要的比上限多，把 GATE_E165_PINNED_MEMORY_MAX（或 --memory-cap）设大再跑；"
                  "251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；"
                  "253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑；不拿掉包装跑，不改钉住的表、变异行或实现")
    signal_number = -completed.returncode if completed.returncode < 0 else completed.returncode - 128 if 128 < completed.returncode <= 128 + 64 else None
    if signal_number is not None and "killed-as-regression" not in BREAK:
        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-4:] if line.strip())
        raise Red(f"装置被信号停了（退 {completed.returncode}，信号 {signal_number}）：钉住的 {len(rows)} 行这一次没判完，输出不算判定{'（' + tail[:300] + '）' if tail else ''}",
                  "照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办：撞了这一条自己的上限由包装认成 250，外面 kill -9 同样退 137——这一次是外面停的："
                  "查是谁在那一刻停了它（同一时刻的 ps、journalctl --user、看门狗的记录），那件事不再发生之后重跑这一格；不拿掉包装跑，不改钉住的表、变异行或实现")
    results = [line for line in completed.stdout.split("\n") if line.startswith(RESULT_PREFIX)]
    mismatches = [] if "replay-ignores-output" in BREAK else \
        [f"第 {row['number']} 行（{row['fields'][4]}）：要 {row['fields'][6][:160]}，装置打的是 {(results[index] if index < len(results) else '（没打）')[:160]}"
         for index, row in enumerate(rows) if index >= len(results) or results[index] != row["fields"][6]]
    if completed.returncode != 0 or mismatches or (len(results) != len(rows) and "replay-ignores-output" not in BREAK):
        for mismatch in mismatches[:5]:
            print(f"     {mismatch}")  # gate-lint:detail
        raise Red(f"钉住的 {len(rows)} 行在今天的实现上：装置退 {completed.returncode}、打了 {len(results)} 行结果、{len(mismatches)} 行与 ⑦ 不同"
                  f"{'（' + completed.stderr.strip().split(chr(10))[-1][:200] + '）' if completed.stderr.strip() else ''}",
                  "今天的实现在钉住的输入上与原实现不同：是改动引入的回归就修实现；装置报指纹漂了就先查装置的枚举次序与状态编码变没变")
    print(f"  ✓ 钉住的 {len(rows)} 行：点名的变异都还在它们的表里；装置 {DEVICE_FLAG} 逐行打的结果都与 ⑦ 逐字节相同")
    return EXIT_GREEN


# ── 命令行 ──

def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest()
    options = {"root": DEFAULT_ROOT, "table": None, "kill_bounds": [], "binary": None, "threads": int(os.environ.get("SINGLEFS_THREAD_CAP") or os.cpu_count() or 1),
               "memory_cap": os.environ.get("GATE_E165_PINNED_MEMORY_MAX") or "2G",
               "target": os.path.join(os.environ.get("GATE_CROSS_RUN_TMPDIR") or os.environ.get("TMPDIR") or "/tmp", "singlefs-e165-pinned-target")}
    mode, ledgers, position = "append", [], 0
    valued = {"--root": "root", "--table": "table", "--binary": "binary", "--memory-cap": "memory_cap", "--threads": "threads", "--target": "target"}
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("--check", "--replay", "--backfill"):
            mode = argument[2:]
            position += 1
        elif argument in valued or argument == "--kill-bounds":
            if position + 1 >= len(arguments):
                report_red(f"{argument} 后面缺值", "看文件头的用法一节")
                return EXIT_USAGE
            if argument == "--kill-bounds":
                options["kill_bounds"].append(arguments[position + 1])
            else:
                options[valued[argument]] = int(arguments[position + 1]) if argument == "--threads" else arguments[position + 1]
            position += 2
        elif argument.startswith("--"):
            report_red(f"认不出的选项 {argument}", "看文件头的用法一节")
            return EXIT_USAGE
        else:
            ledgers.append(arguments[position])
            position += 1
    root = os.path.abspath(options["root"])
    table = os.path.abspath(options["table"] or os.path.join(root, TABLE))
    if options["kill_bounds"] and mode != "append":
        report_red(f"--{mode} 不收 --kill-bounds：--check 按文件名给每份账配同一趟的产物"
                   "（<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out），--replay 与 --backfill 用不上产物",
                   "去掉 --kill-bounds 再跑；要判的账照位置参数点名")
        return EXIT_USAGE
    if options["kill_bounds"] and len(options["kill_bounds"]) != len(ledgers):
        report_red(f"追加给了 {len(options['kill_bounds'])} 个 --kill-bounds、{len(ledgers)} 份红例账：给就逐份对应，第 i 个产物配第 i 份账",
                   "不给 --kill-bounds（按文件名配：<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out），或每份账给一个同一趟的产物")
        return EXIT_USAGE
    if mode == "backfill" and ledgers:
        report_red("--backfill 不收红例账：每行按自己的 ⑤ 回 research/results/ 取原行",
                   "去掉位置参数再跑：python3 research/scripts/e165-pin-red-examples.py --backfill")
        return EXIT_USAGE
    if mode == "append" and not ledgers:
        report_red("追加要至少一份红例账", "例：python3 research/scripts/e165-pin-red-examples.py research/results/<某一趟>-red-examples.tsv")
        return EXIT_USAGE
    missing = [ledger for ledger in ledgers + options["kill_bounds"] if not os.path.isfile(ledger)]
    if missing:
        report_red(f"点名的文件不在：{' '.join(missing)}", "路径相对当前目录；红例账在 research/results/ 下")
        return EXIT_USAGE
    try:
        if mode == "append":
            return append(root, table, [os.path.abspath(ledger) for ledger in ledgers], [os.path.abspath(product) for product in options["kill_bounds"]])
        if mode == "check":
            return check(root, table, [os.path.abspath(ledger) for ledger in ledgers])
        if mode == "backfill":
            return backfill(root, table)
        return replay(root, table, options)
    except Red as red:
        report_red(red.message, red.howto)
        return EXIT_RED
    except NotRun as not_run:
        print(f"  ! {not_run}")
        return EXIT_NOT_RUN


# ── 自证 ──

def selftest():
    global current_digest, MEMORY_CAP_RUNNER, PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK
    cases, wrong = 0, []
    import contextlib
    import io
    with tempfile.TemporaryDirectory(prefix="e165-pin-selftest-") as work:
        root = os.path.join(work, "root")
        for directory in (".claude/gate.d", "crates", "research/mutations", "research/results"):
            os.makedirs(os.path.join(root, directory))

        def write(relative, text):
            with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                handle.write(text)

        crates_table = "# 表头\n红的一条\tcrates/x.rs\ta\tb\t-p x\tt\n红的两条\tcrates/x.rs\tc\td\t-p x\tt\n"
        write(CRATES_TABLE, crates_table)
        write(CUSTOM_TABLE, "# 表头\nZ1 自造的\tcrates/x.rs\te\tf\n")
        header = "# 自证用的钉住的表\n"
        write(TABLE, header)

        def state_line(domain, ordinal, state, first):
            return f"E7RESULT name=state domain={domain} ring_slots=64 n=2 ordinal={ordinal} counters=states state={state};first={first}"

        ledger_rows = [
            ["tsv-2", "2", "L", "2", "3", "counters", "+J4", "true", "true", "true", "cmd", state_line("L", 3, "shape=1c0", "returned:a"), state_line("L", 3, "shape=1c0", "returned:b")],
            ["tsv-3", "3+9", "L", "2", "3", "counters", "+J4", "true", "true", "true", "cmd", state_line("L", 3, "shape=1c0", "returned:a"), state_line("L", 3, "shape=1c0", "returned:c")],
            ["Z1", "2", "X", "2", "5", "outcome", "outcome", "true", "true", "true", "cmd", state_line("X", 5, "shape=2c0", "returned:d"), state_line("X", 5, "shape=2c0", "returned:e")],
            ["tsv-3", "3+9", "X", "2", "none", "no_single_state_differs", "none", "false", "false", "false", "", "", ""],
        ]
        ledger = os.path.join(root, "research/results/e165-selftest-red-examples.tsv")
        write(os.path.relpath(ledger, root), "# 账头\n" + "".join("\t".join(fields) + "\n" for fields in ledger_rows))
        kill_bounds = os.path.join(root, "research/results/e165-selftest-kill-bounds.out")

        def run_product(ledger_path):
            """同一趟的产物：每个变体一行（red_examples= 等于账里它的行数），收尾行记账的 sha256。"""
            return ("E7RESULT name=kill_bound item=baseline table=none table_row=none outcome=baseline title=基线 complete=yes\n"
                    "E7RESULT name=kill_bound item=tsv-2 table=crates/mutations.tsv table_row=2 outcome=red red_examples=1 title=红的一条 complete=yes\n"
                    "E7RESULT name=kill_bound item=tsv-3 table=crates/mutations.tsv table_row=3 outcome=red red_examples=2 title=红的两条 complete=yes\n"
                    "E7RESULT name=kill_bound item=tsv-3 table=crates/mutations.tsv table_row=9 outcome=red red_examples=2 title=同一处的另一行 complete=yes\n"
                    f"E7RESULT name=kill_bound item=Z1 table={CUSTOM_TABLE} table_row=2 outcome=red red_examples=1 title=Z1_自造的 complete=yes\n"
                    f"E7RESULT name=verdict scope=segment2 variants=4 red=3 ledger_sha256={file_digest(ledger_path)}\n"
                    "E7RESULT name=done emitted=7\n")

        good_titles = run_product(ledger)
        write(os.path.relpath(kill_bounds, root), good_titles)
        stub = os.path.join(work, "stub-device.sh")
        with open(stub, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 桩装置：--pinned-cases <表> 逐个数据行打 ⑦；STUB_MODE=wrong 把最后一行打错，=fail 退 1，=killed 打一行之后 kill -9 自己\n"
                         "[[ \"$1\" == --pinned-cases ]] || exit 2\n"
                         "if [[ \"${STUB_MODE:-}\" == killed ]]; then grep -v '^#' \"$2\" | grep . | head -n 1 | cut -f7; kill -9 \"$$\"; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == slow ]]; then grep -v '^#' \"$2\" | grep . | head -n 1 | cut -f7; sleep 0.6; grep -v '^#' \"$2\" | grep . | tail -n +2 | cut -f7; "
                         "echo \"E7RESULT name=pinned_cases rows=$(grep -v '^#' \"$2\" | grep -c .)\"; exit 0; fi\n"
                         "rows=$(grep -v '^#' \"$2\" | grep -c .)\n"
                         "grep -v '^#' \"$2\" | grep . | cut -f7 | { if [[ \"${STUB_MODE:-}\" == wrong ]]; then sed '$ s/returned:/returned:WRONG/'; else cat; fi; }\n"
                         "echo \"E7RESULT name=pinned_cases rows=$rows\"\n"
                         "[[ \"${STUB_MODE:-}\" == fail ]] && exit 1\nexit 0\n")
        os.chmod(stub, 0o755)

        def expect(label, arguments, want_exit, want_text, environment=None):
            nonlocal cases
            cases += 1
            saved = dict(os.environ)
            os.environ.update(environment or {})
            buffer = io.StringIO()
            try:
                with contextlib.redirect_stdout(buffer):
                    got = main(["--root", root, *arguments])
            finally:
                os.environ.clear()
                os.environ.update(saved)
            output = buffer.getvalue()
            if got != want_exit or want_text not in output:
                wrong.append(label)
                print(f"    {label} 这一格判错：期望退 {want_exit}、输出含「{want_text}」，实测退 {got}：{output.strip()[-300:]}")  # gate-lint:detail
            return output

        table_path = os.path.join(root, TABLE)
        expect("check-empty-table", ["--check", ledger], 1, "3 行里 3 行没钉进")
        write(os.path.relpath(kill_bounds, root), good_titles.replace("title=红的两条", "title=别的变异"))
        expect("titles-mismatch", [ledger], 1, "title 是")
        write(os.path.relpath(kill_bounds, root), good_titles)
        write(TABLE, header)
        expect("pin-kill-bounds-count", ["--kill-bounds", kill_bounds, ledger, ledger], 2, "追加给了 1 个 --kill-bounds、2 份红例账")
        # 按 title 找名字：那一趟之后变异表头上插了一行，tsv-2、tsv-3 的行号都错一位，名字照样找得到、钉进 3 行（开关 variant-by-line-number 下一行都不钉）
        write(CRATES_TABLE, crates_table.replace("# 表头\n", "# 表头\n插在前面的一条\tcrates/x.rs\tm\tn\t-p x\tt\n"))
        expect("pin-by-title-shifted-table", [ledger], 0, "钉进 3 行")
        by_title_text = open(table_path, encoding="utf-8").read()
        expect("check-by-title-shifted-table", ["--check", ledger], 0, "3 行都钉在")
        write(TABLE, header)
        # 按名字找不到：tsv-3 的变异在那一趟之后改了名，这一行逐行报、不钉，另外两行照钉，退 1（开关 titles-unchecked 下照行号钉进 3 行）
        write(CRATES_TABLE, crates_table.replace("红的两条", "改了名的两条"))
        not_found = expect("pin-by-title-not-found", [ledger], 1, "钉进 2 行（回读都在；1 份账 4 行：0 行已钉、1 行没有反例状态）；1 行按 title 找不到唯一的变异、没钉")
        not_found_table = open(table_path, encoding="utf-8").read()
        if ("e165-selftest-red-examples.tsv:3：变体 tsv-3 那一趟的 title 是 '红的两条'" not in not_found or "按名字找不到" not in not_found
                or "改了名的两条" in not_found_table or "#红的两条" in not_found_table):
            wrong.append("pin-by-title-not-found-detail")
            print(f"    pin-by-title-not-found-detail 这一格判错：要逐行点名第 3 行、按名字找不到，表里不许有 tsv-3 那一行，实测：{not_found.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        write(TABLE, header)
        # 找到两行同名：变异表末尾多了一行也叫「红的两条」，认不出是哪一个，逐行报、不钉（开关 variant-by-line-number 下照行号钉进 3 行）
        write(CRATES_TABLE, crates_table + "红的两条\tcrates/y.rs\ti\tj\t-p y\tt\n")
        ambiguous = expect("pin-by-title-ambiguous", [ledger], 1, "1 行按 title 找不到唯一的变异、没钉")
        if "找到 2 行同名（第 3、4 行）" not in ambiguous or "#红的两条" in open(table_path, encoding="utf-8").read():
            wrong.append("pin-by-title-ambiguous-detail")
            print(f"    pin-by-title-ambiguous-detail 这一格判错：要点名找到 2 行同名（第 3、4 行），表里不许有 tsv-3 那一行，实测：{ambiguous.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        write(CRATES_TABLE, crates_table)
        write(TABLE, header)
        # 读到写之间被改过：让复核那一次读到不同的 sha256
        original_digest = current_digest
        current_digest = lambda path: "0" * 64
        expect("race", [ledger], 1, "被别人改过")
        current_digest = original_digest
        if open(table_path, encoding="utf-8").read() != header:
            wrong.append("race-wrote")
            print("    race-wrote 这一格判错：复核判红之后表还是被写了")  # gate-lint:detail
        cases += 1
        expect("pin", [ledger], 0, "钉进 3 行")
        if open(table_path, encoding="utf-8").read() != by_title_text:
            wrong.append("pin-by-title-same-names")
            print("    pin-by-title-same-names 这一格判错：行号错位时按 title 钉出的表应当与表没改时钉出的逐字节相同")  # gate-lint:detail
        cases += 1
        expect("pin-again", [ledger], 0, "3 行已钉")
        expect("check-green", ["--check", ledger], 0, "3 行都钉在")
        expect("check-default-ledgers", ["--check"], 0, "1 份红例账 3 行都钉在")
        pinned_text = open(table_path, encoding="utf-8").read()
        write(TABLE, "\n".join(line for line in pinned_text.split("\n") if "\tZ1 " not in line and "#Z1" not in line) + "\n")
        expect("check-missing", ["--check", ledger], 1, "3 行里 1 行没钉进")
        # ⑦ 与抄来的 ⑳ 一起改：⑳ 与 ⑦ 仍相同，只剩重算的指纹对不上（开关 fingerprint-unchecked 下这一格判错）
        if pinned_text.count("shape=2c0;first=returned:d\tE7") != 2:
            wrong.append("check-tampered-setup")
            print("    check-tampered-setup 这一格判错：Z1 那一行的 ⑦ 与 ⑳ 应当各有一处「shape=2c0;first=returned:d」")  # gate-lint:detail
        cases += 1
        write(TABLE, pinned_text.replace("shape=2c0;first=returned:d\tE7", "shape=2c1;first=returned:d\tE7"))
        expect("check-tampered", ["--check", ledger], 1, "重算的指纹不同")
        write(TABLE, pinned_text)

        def edited_table(edit):
            """钉好的表逐个数据行交给 edit(段列表, 第几个数据行) 改，交回整份文本。"""
            lines, index = [], 0
            for line in pinned_text.split("\n"):
                if line.strip() and not line.startswith("#"):
                    fields = line.split("\t")
                    edit(fields, index)
                    line, index = "\t".join(fields), index + 1
                lines.append(line)
            return "\n".join(lines)

        def set_field(row_index, column_index, value):
            def edit(fields, index):
                if index == row_index:
                    fields[column_index] = value
            return edit

        # 整行抄：Z1 那一行的 ⑲（账第 11 段，单条重放的命令）与账不同，别的段都对（开关 whole-row-unchecked 下判绿）
        write(TABLE, edited_table(set_field(2, 18, "cmd-hand-edited")))
        expect("check-whole-row-differs", ["--check", ledger], 1, "没整行钉：表第")
        # 8 段的旧行：--check 与 --replay 都报没整行抄、出路给 --backfill（开关 old-rows-accepted 下 replay 判绿）
        old_row_table = edited_table(lambda fields, index: fields.__delitem__(slice(TABLE_OWN_COLUMNS, None)) if index == 2 else None)
        write(TABLE, old_row_table)
        expect("check-old-row", ["--check", ledger], 1, "没整行抄红例账（8 段的旧格式，要 21 段）")
        old_row_output = expect("replay-old-row", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "第 4 行没整行抄红例账")
        if "--backfill" not in old_row_output:
            wrong.append("replay-old-row-howto")
            print(f"    replay-old-row-howto 这一格判错：出路要给 --backfill，实测：{old_row_output.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        # 抄来的账行与自己的段对不上（开关 copied-columns-unchecked 下两格都判绿：桩照 ⑦ 打）
        write(TABLE, edited_table(set_field(0, 10, "X")))
        expect("replay-copied-domain-differs", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "第 2 行：⑪（红例账第 3 段）与 ② 不同")
        write(TABLE, edited_table(set_field(0, 19, state_line("L", 3, "shape=1c0", "returned:q"))))
        expect("replay-copied-original-differs", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "第 2 行：⑳（红例账第 12 段）与 ⑦ 不同")
        # 补钉：三行都截成 8 段，--backfill 按 ⑤ 回账里取原行，补完与追加钉出来的逐字节相同
        all_old = edited_table(lambda fields, index: fields.__delitem__(slice(TABLE_OWN_COLUMNS, None)))
        write(TABLE, all_old)
        expect("backfill", ["--backfill"], 0, "补钉：3 行 8 段的旧行都按 ⑤ 取回账行接成 21 段")
        if open(table_path, encoding="utf-8").read() != pinned_text:
            wrong.append("backfill-restores-whole-rows")
            print("    backfill-restores-whole-rows 这一格判错：补完的表应当与追加钉出来的逐字节相同")  # gate-lint:detail
        cases += 1
        expect("backfill-again", ["--backfill"], 0, "没有要补的")
        expect("backfill-arguments", ["--backfill", ledger], 2, "--backfill 不收红例账")
        # ⑤ 的行号错位（Z1 那一行指到 tsv-2 那一行账）：取回的对不上，逐行报、不填，另外两行照补（开关 backfill-unverified 下判错）
        if "e165-selftest-red-examples.tsv:4\t" not in all_old:
            wrong.append("backfill-setup")
            print("    backfill-setup 这一格判错：Z1 那一行的 ⑤ 应当是 e165-selftest-red-examples.tsv:4")  # gate-lint:detail
        cases += 1
        write(TABLE, all_old.replace("e165-selftest-red-examples.tsv:4\t", "e165-selftest-red-examples.tsv:2\t"))
        mismatched = expect("backfill-mismatched-origin", ["--backfill"], 1, "补了 2 行（回读都在）；1 行取不到原行")
        if "取不到原行：第 4 行" not in mismatched or "账第 5 段与 ④" not in mismatched:
            wrong.append("backfill-mismatched-origin-detail")
            print(f"    backfill-mismatched-origin-detail 这一格判错：要逐行点名第 4 行、哪一段对不上，实测：{mismatched.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        write(TABLE, all_old.replace("e165-selftest-red-examples.tsv:4\t", "e165-gone-red-examples.tsv:4\t"))
        expect("backfill-ledger-missing", ["--backfill"], 1, "红例账 research/results/e165-gone-red-examples.tsv 不在")
        write(TABLE, all_old)
        current_digest = lambda path: "0" * 64
        expect("backfill-race", ["--backfill"], 1, "被别人改过")
        current_digest = original_digest
        if open(table_path, encoding="utf-8").read() != all_old:
            wrong.append("backfill-race-wrote")
            print("    backfill-race-wrote 这一格判错：复核判红之后表还是被写了")  # gate-lint:detail
        cases += 1
        write(TABLE, pinned_text)
        # 另一趟出了同样的红例：⑤ 不是它的行，只靠 ⑥ 认，名字取它自己那一趟的产物
        run_ledger = os.path.join(root, "research/results/e165-selftest-r1-red-examples.tsv")
        run_kill_bounds = os.path.join(root, "research/results/e165-selftest-r1-kill-bounds.out")
        write(os.path.relpath(run_ledger, root), "# 第二趟的账\n" + "".join("\t".join(fields) + "\n" for fields in ledger_rows))
        run_text = run_product(run_ledger)
        write(os.path.relpath(run_kill_bounds, root), run_text)
        both = ["--check", ledger, run_ledger]
        expect("check-run-names", both, 0, "2 份红例账 6 行都钉在")
        expect("check-run-names-kill-bounds-option", [*both, "--kill-bounds", run_kill_bounds], 2, "--check 不收 --kill-bounds")
        # 另一趟的账同一个输入、同一个变异，只是 ⑱（账第 10 段，首红与否）不同：追加另钉一行整行抄，之后 --check 才绿（开关 dedupe-by-input-variant 下两格判错）
        variant_ledger = os.path.join(root, "research/results/e165-selftest-r2-red-examples.tsv")
        variant_rows = [list(fields) for fields in ledger_rows]
        variant_rows[0][9] = "false"
        write(os.path.relpath(variant_ledger, root), "# 第三趟的账\n" + "".join("\t".join(fields) + "\n" for fields in variant_rows))
        variant_kill_bounds = os.path.join(root, "research/results/e165-selftest-r2-kill-bounds.out")
        write(os.path.relpath(variant_kill_bounds, root), run_product(variant_ledger))
        expect("pin-whole-row-variant", [variant_ledger], 0, "钉进 1 行（1 份账 4 行：2 行已钉")
        expect("check-whole-row-variant", ["--check", ledger, variant_ledger], 0, "2 份红例账 6 行都钉在")
        os.remove(variant_ledger)
        os.remove(variant_kill_bounds)
        write(TABLE, pinned_text)
        # 追加与 --check 同一套：那一趟时变异表第 2 行是「先删的一条」，之后被删、末尾加了一行；账里 tsv-3、tsv-4 要按那一趟的 title 钉成「红的一条」「红的两条」
        # （开关 append-names-from-today-table 下按今天的行号钉成「红的两条」「后加的一条」，append-pairing-optional 下产物不在、账 sha256 不对也照钉）
        deleted_ledger = os.path.join(root, "research/results/e165-selftest-deleted-red-examples.tsv")
        deleted_kill_bounds = os.path.join(root, "research/results/e165-selftest-deleted-kill-bounds.out")
        deleted_rows = [
            ["tsv-3", "3", "M", "2", "7", "counters", "+J4", "true", "true", "true", "cmd", state_line("M", 7, "shape=3c0", "returned:a"), state_line("M", 7, "shape=3c0", "returned:b")],
            ["tsv-4", "4", "M", "2", "8", "counters", "+J4", "true", "true", "true", "cmd", state_line("M", 8, "shape=3c1", "returned:a"), state_line("M", 8, "shape=3c1", "returned:c")],
        ]
        write(os.path.relpath(deleted_ledger, root), "# 删行之后的账\n" + "".join("\t".join(fields) + "\n" for fields in deleted_rows))
        write(CRATES_TABLE, crates_table + "后加的一条\tcrates/x.rs\tg\th\t-p x\tt\n")
        deleted_product = ("E7RESULT name=kill_bound item=tsv-3 table=crates/mutations.tsv table_row=3 outcome=red red_examples=1 title=红的一条 complete=yes\n"
                           "E7RESULT name=kill_bound item=tsv-4 table=crates/mutations.tsv table_row=4 outcome=red red_examples=1 title=红的两条 complete=yes\n"
                           f"E7RESULT name=verdict scope=segment2 variants=2 red=2 ledger_sha256={file_digest(deleted_ledger)}\n")
        expect("pin-kill-bounds-missing", [deleted_ledger], 1, "配的 kill-bounds 产物 research/results/e165-selftest-deleted-kill-bounds.out 不在")
        if open(table_path, encoding="utf-8").read() != pinned_text:
            wrong.append("pin-kill-bounds-missing-wrote")
            print("    pin-kill-bounds-missing-wrote 这一格判错：配不上产物判红之后表还是被写了")  # gate-lint:detail
        cases += 1
        write(os.path.relpath(deleted_kill_bounds, root), deleted_product.replace(f"ledger_sha256={file_digest(deleted_ledger)}", "ledger_sha256=" + "0" * 64))
        expect("pin-ledger-digest", [deleted_ledger], 1, "账与产物不是同一趟")
        # --kill-bounds 显式给的产物（第二段驱动收尾时这样调，文件名可以不配）过同样的四样：收尾行记的账不是这一份照样判红
        driver_product = os.path.join(root, "research/results/driver-output.out")
        write(os.path.relpath(driver_product, root), deleted_product.replace(f"ledger_sha256={file_digest(deleted_ledger)}", "ledger_sha256=" + "0" * 64))
        expect("pin-explicit-kill-bounds-digest", ["--kill-bounds", driver_product, deleted_ledger], 1, "账与产物不是同一趟")
        os.remove(driver_product)
        write(os.path.relpath(deleted_kill_bounds, root), deleted_product)
        expect("pin-deleted-row", [deleted_ledger], 0, "钉进 2 行")
        deleted_text = open(table_path, encoding="utf-8").read()
        if ("e165-selftest-deleted-red-examples.tsv:2\tcrates/mutations.tsv#红的一条\t" not in deleted_text
                or "e165-selftest-deleted-red-examples.tsv:3\tcrates/mutations.tsv#红的两条\t" not in deleted_text or "#后加的一条" in deleted_text):
            wrong.append("pin-deleted-row-names")
            print("    pin-deleted-row-names 这一格判错：tsv-3、tsv-4 要按那一趟的 title 钉成「红的一条」「红的两条」，不许出现今天按行号换出的「后加的一条」")  # gate-lint:detail
        cases += 1
        expect("check-deleted-row", ["--check", deleted_ledger], 0, "1 份红例账 2 行都钉在")
        os.remove(deleted_ledger)
        os.remove(deleted_kill_bounds)
        write(CRATES_TABLE, crates_table)
        write(TABLE, pinned_text)
        # 钉完之后别的会话删了变异表第 2 行、末尾加一行：今天的 tsv-2 是「红的两条」、tsv-3 是新加的那条，那一趟的名字照旧
        write(CRATES_TABLE, crates_table.replace("红的一条\tcrates/x.rs\ta\tb\t-p x\tt\n", "") + "后加的一条\tcrates/x.rs\tg\th\t-p x\tt\n")
        expect("check-shifted-table", both, 0, "2 份红例账 6 行都钉在")
        write(CRATES_TABLE, crates_table)
        # 真没钉全：那一趟给 tsv-3 的名字是表里没钉过的变异
        write(os.path.relpath(run_kill_bounds, root), run_text.replace("title=红的两条", "title=别的变异"))
        missing_text = expect("check-title-other-variant", both, 1, "6 行里 1 行没钉进")
        # 出路逐份给补钉命令（追加按文件名配同一趟的产物，不带 --kill-bounds），只点没钉全的那一份账
        if ("e165-pin-red-examples.py research/results/e165-selftest-r1-red-examples.tsv" not in missing_text
                or "research/results/e165-selftest-red-examples.tsv" in missing_text or "--kill-bounds" in missing_text):
            wrong.append("check-title-other-variant-howto")
            print(f"    check-title-other-variant-howto 这一格判错：出路要给 e165-pin-red-examples.py <没钉全的账>、不点钉全了的账、不带 --kill-bounds，实测：{missing_text.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        # 配对不上：title 缺了、产物不在、变体集合不同、收尾行记的账不是这一份、没收尾
        write(os.path.relpath(run_kill_bounds, root), run_text.replace(" title=红的两条", ""))
        expect("check-title-missing", both, 1, "item=tsv-3 table_row=3 那一行，或那一行没有 title=")
        os.remove(run_kill_bounds)
        expect("check-kill-bounds-missing", both, 1, "配的 kill-bounds 产物 research/results/e165-selftest-r1-kill-bounds.out 不在")
        write(os.path.relpath(run_kill_bounds, root),
              run_text + "E7RESULT name=kill_bound item=tsv-9 table=crates/mutations.tsv table_row=9 outcome=red red_examples=2 title=账里没有的 complete=yes\n")
        expect("check-variant-sets-differ", both, 1, "tsv-9：账里 0 行，产物的 red_examples= 是 2")
        write(os.path.relpath(run_kill_bounds, root), run_text.replace(f"ledger_sha256={file_digest(run_ledger)}", "ledger_sha256=" + "0" * 64))
        expect("check-ledger-digest", both, 1, "账与产物不是同一趟")
        write(os.path.relpath(run_kill_bounds, root), "".join(line + "\n" for line in run_text.split("\n") if line.startswith("E7RESULT name=kill_bound ")))
        expect("check-run-unfinished", both, 1, "没有收尾行 E7RESULT name=verdict")
        os.remove(run_kill_bounds)
        os.remove(run_ledger)
        conflicting = [list(fields) for fields in ledger_rows]
        conflicting[0][11] = state_line("L", 3, "shape=1c0", "returned:z")
        other_ledger = os.path.join(root, "research/results/e165-selftest-conflict-red-examples.tsv")
        write(os.path.relpath(other_ledger, root), "# 账头\n" + "\t".join(conflicting[0]) + "\n")
        other_kill_bounds = os.path.join(root, "research/results/e165-selftest-conflict-kill-bounds.out")
        write(os.path.relpath(other_kill_bounds, root),
              "E7RESULT name=kill_bound item=tsv-2 table=crates/mutations.tsv table_row=2 outcome=red red_examples=1 title=红的一条 complete=yes\n"
              f"E7RESULT name=verdict scope=segment2 variants=1 red=1 ledger_sha256={file_digest(other_ledger)}\n")
        expect("conflict", [other_ledger], 1, "原实现逐态行不同")
        os.remove(other_ledger)
        os.remove(other_kill_bounds)
        expect("replay-green", ["--replay", "--binary", stub, "--memory-cap", "none"], 0, "都与 ⑦ 逐字节相同")
        expect("replay-mismatch", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "行与 ⑦ 不同", {"STUB_MODE": "wrong"})
        expect("replay-fail", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "装置退 1", {"STUB_MODE": "fail"})
        expect("replay-device-without-flag", ["--replay"], 77, "还不认 --pinned-cases")
        # 进度行：装置跑得超过进度间隔（假时钟每读一次走 200 秒、间隔 300 秒；桩打一行后停 0.6 秒），跑着时至少打一行 E165_PINNED_PROGRESS，
        # 带挂钟与已打的结果行数；判定照常绿（开关 progress-silent 下一行都不打，这一格判错）
        fake_now, progress_sink = [0.0], io.StringIO()

        def fake_clock():
            fake_now[0] += 200.0
            return fake_now[0]

        saved_progress = (PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK)
        PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK = fake_clock, 0.05, progress_sink
        try:
            expect("replay-progress-green", ["--replay", "--binary", stub, "--memory-cap", "none"], 0, "都与 ⑦ 逐字节相同", {"STUB_MODE": "slow"})
        finally:
            PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK = saved_progress
        progress_lines = [line for line in progress_sink.getvalue().split("\n") if line.startswith("E165_PINNED_PROGRESS ")]
        if not progress_lines or not all(re.search(r"挂钟 \d+ 秒：.*装置已打 \d+/3 行结果", line) for line in progress_lines):
            wrong.append("replay-progress")
            print(f"    replay-progress 这一格判错：装置跑过进度间隔时要至少一行 E165_PINNED_PROGRESS（挂钟、已打 n/3 行结果），实测：{progress_sink.getvalue()[-300:]!r}")  # gate-lint:detail
        cases += 1
        # 经内存包装起、包装自己退 252（排不上）：判「没跑成」，出路照包装文件头，不报回归（开关 wrapper-exit-as-replay-red 下这一格判错）
        fake_wrapper =os.path.join(work, "fake-memory-cap-wrapper.sh")
        with open(fake_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 252（内存不够排不上）\nexit 252\n")
        saved_runner, MEMORY_CAP_RUNNER = MEMORY_CAP_RUNNER, fake_wrapper
        try:
            expect("replay-wrapper-exit", ["--replay", "--binary", stub, "--memory-cap", "1G"], 1, "没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 252")
        finally:
            MEMORY_CAP_RUNNER = saved_runner
        # 装置被信号停了（直接起时 kill -9 自己；经包装起时包装退 137，外面 kill -9 就是这样）：出路照包装文件头，不写修实现（开关 killed-as-regression 下这两格判错）
        killed_howto = "照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办"
        killed = expect("replay-killed", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "装置被信号停了（退 -9，信号 9）", {"STUB_MODE": "killed"})
        if killed_howto not in killed or "修实现" in killed:
            wrong.append("replay-killed-howto")
            print(f"    replay-killed-howto 这一格判错：出路要照包装文件头、不写修实现，实测：{killed.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        killing_wrapper = os.path.join(work, "killed-memory-cap-wrapper.sh")
        with open(killing_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 137（命令被外面 kill -9）\nexit 137\n")
        saved_runner, MEMORY_CAP_RUNNER = MEMORY_CAP_RUNNER, killing_wrapper
        try:
            expect("replay-wrapper-137", ["--replay", "--binary", stub, "--memory-cap", "1G"], 1, "装置被信号停了（退 137，信号 9）")
        finally:
            MEMORY_CAP_RUNNER = saved_runner
        write(CRATES_TABLE, crates_table.replace("红的两条", "改了名的"))
        expect("replay-variant-missing", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "不在它的表里了")
        write(CRATES_TABLE, crates_table)

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK:
        for item in ("pin-twice", "titles-unchecked", "no-recheck-before-write", "check-ignores-missing", "fingerprint-unchecked", "variant-unchecked", "replay-ignores-output",
                     "wrapper-exit-as-replay-red", "killed-as-regression", "names-from-today-table", "kill-bounds-optional", "title-missing-ignored",
                     "title-not-compared", "variant-sets-unchecked", "ledger-digest-unchecked",
                     "whole-row-unchecked", "old-rows-accepted", "copied-columns-unchecked", "dedupe-by-input-variant", "backfill-unverified",
                     "variant-by-line-number", "append-names-from-today-table", "append-pairing-optional", "progress-silent"):
            completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
                                       env=dict(os.environ, E165_PIN_RED_EXAMPLES_BREAK=item))
            if completed.returncode == 0:
                breaks_missed.append(item)
            else:
                breaks_proven += 1
    if wrong:
        report_red(f"e165-pin-red-examples 自证 {cases} 格里 {len(wrong)} 格判错：{' '.join(wrong)}", "按判错那几格的标签找 append / check / replay 里对应的一段修")
        return EXIT_RED
    if breaks_missed:
        report_red(f"弄坏开关打开之后自证照样全对：{' '.join(breaks_missed)}", "那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关")
        return EXIT_RED
    suffix = f"；弄坏开关 {breaks_proven} 个，每个都让至少一格判错" if not BREAK else "（开着弄坏开关，没重跑开关）"
    print(f"  ✓ e165-pin-red-examples 自证 {cases} 格都判对{suffix}")
    return EXIT_GREEN


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
