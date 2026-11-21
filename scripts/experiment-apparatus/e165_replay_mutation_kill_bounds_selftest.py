#!/usr/bin/env python3
# admission: always 被 e165-replay-mutation-kill-bounds.sh 在 --selftest 时载入，自证只用假驱动与临时目录，上一次的结论不替这一次作保
# run-condition: none 被 E165 第二段驱动载入的自证模块；自证只在临时 prune-server 底下跑，起法在驱动文件头「用法」--selftest 那一条
"""E165 第二段驱动 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh 的自证（selftest）与它用的三份假 cargo（FAKE_CARGO、FAKE_CARGO_R3、FAKE_CARGO_R4）。
各格、弄坏开关下点名的格与退出码都在驱动文件头；驱动的各段在 e165_replay_mutation_kill_bounds_driver.py。
"""
import hashlib
import io
import json
import os
import re
import shutil
import signal
import subprocess
import tempfile
import time
from e165_replay_mutation_kill_bounds_driver import (  # noqa: E402
    CUSTOM_TABLE_PATH, DEFAULT_ROOT, DEVICE_SOURCE_PATH, EXIT_HALT_H1, EXIT_INCOMPLETE, EXIT_NOT_PROFILE_R2, EXIT_OK, EXIT_PIN_INCOMPLETE, EXIT_USAGE,
    EXIT_VOID_V7, ITEM_VERDICT_BREAK_MODES, JUDGE_FUNCTIONS, KILL_AFTER_VARIABLE, LIB_PATH, PROGRESS_INTERVAL_SECONDS, Progress, R2_BREAK_MODES,
    R3_BREAK_MODES, R3_REGISTERED, R4_BREAK_MODES, R4_GATE_R2_CONFIG_FIELD, R4_REGISTERED, R5_FILES_SHA256, R5_GATE_R2_PINS, R5_PINS, R5_TABLE_SHA256,
    RECOVERY_PATH, REGISTERED, SCRIPT_KEY, SELFTEST_JUDGE_FILE, SELFTEST_JUDGE_RESULT, SELFTEST_JUDGE_TEXT, SELF_PATH, SET_OUTPUT_ASIDE_VARIABLE, Setup,
    Stop, TABLE_PATH, TAGS_TABLE_PATH, WORK_DIRECTORY_NAME, compare_tag_sets, copy_root_entries, default_jobs_and_threads, environment_in_the_copy,
    item_requests_text, item_verdicts_command)
from e165_replay_mutation_kill_bounds_variant_judgement import (  # noqa: E402
    BREAK, CLAUSE_CRITERIA, DEVICE_PROFILE_ARGUMENTS, J6_ROWS, J6QN_ROWS, J7_ROWS, R3_BOUNDS_ARGUMENTS, R3_CRITERIA, R3_DEVICE_PROFILE_ARGUMENTS,
    R3_LEGAL_DOMAINS, R4_BOUNDS_ARGUMENTS, R4_CRITERIA, R4_DEVICE_PROFILE_ARGUMENTS, R4_GATE_R2_ARGUMENTS, R4_LEGAL_DOMAINS)



# ── 自证

FAKE_CARGO = r'''#!/usr/bin/env bash
# 假的 cargo：只认 build。副本的 recovery.rs 或 lib.rs 里有 BROKEN_SYNTAX 就「编不过」；否则「编」出一个 shell 脚本当二进制，
# 它照两份源文件里有没有 FLAG_RED 决定 L 域 N = 2 的 J4 违例数；有 FLAG_J6_TWO 时 N ≥ 2 的 J6_premise_holds 行见证文字变（违例数不变），
# 有 FLAG_J6QN_THREE 时 N ≥ 3 的 J6Qn_premise_fails 违例数变 1。二进制把每一次被调的参数记进 FAKE_BIN_LOG；带 --profile r2 才打 J6Qn 两行
# （FAKE_BIN_NO_R2_ROWS 设了时带了也不打：认不得 r2 档的装置）。
echo "$PWD" >> "$FAKE_CARGO_LOG"
source_file="crates/singlefs-core/src/recovery.rs crates/singlefs-replay-prefix/src/lib.rs"
if grep -q BROKEN_SYNTAX $source_file; then echo "error: could not compile" >&2; exit 101; fi
red=0; grep -q FLAG_RED $source_file && red=1
j6_two=0; grep -q FLAG_J6_TWO $source_file && j6_two=1
j6qn_three=0; grep -q FLAG_J6QN_THREE $source_file && j6qn_three=1
mkdir -p "$CARGO_TARGET_DIR/release"
cat > "$CARGO_TARGET_DIR/release/e165_replay_bounded_enumeration" <<BIN
#!/usr/bin/env bash
red=$red; j6_two=$j6_two; j6qn_three=$j6qn_three
echo "\$*" >> "\${FAKE_BIN_LOG:-/dev/null}"
profile=r1
if [ "\$1" = --profile ]; then profile="\$2"; shift 2; fi
r2_rows=0; [ "\$profile" = r2 ] && [ -z "\${FAKE_BIN_NO_R2_ROWS:-}" ] && r2_rows=1
state_line() { # n ordinal
  local counters=states
  if [ "\$red" = 1 ] && [ "\$1" = 2 ] && [ "\$2" = 1 ]; then counters=states,J4; fi
  echo "E7RESULT name=state domain=L ring_slots=64 n=\$1 ordinal=\$2 counters=\$counters state=fake\$2;first=returned"
}
case "\$1" in
  --bounds)
    sleep "\${FAKE_BIN_SLEEP:-0}"
    echo "E7RESULT name=config experiment=E165 bounds=segment2"
    for n in 1 2 3; do
      violations=0; [ "\$red" = 1 ] && [ "\$n" = 2 ] && violations=1
      witness=none; [ "\$j6_two" = 1 ] && [ "\$n" -ge 2 ] && witness=changed
      echo "E7RESULT name=states domain=L ring_slots=64 n=\$n states=2 panicked=0 failed=0"
      echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J4 violations=\$violations judged=2 witness=none"
      echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J6_premise_holds violations=0 judged=2 witness=\$witness"
      echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J6_premise_fails violations=0 judged=0 witness=none"
      if [ "\$r2_rows" = 1 ]; then
        qn=0; [ "\$j6qn_three" = 1 ] && [ "\$n" -ge 3 ] && qn=1
        echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J6Qn_premise_holds violations=0 judged=2 witness=none"
        echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J6Qn_premise_fails violations=\$qn judged=0 witness=none"
      fi
    done
    echo "E7RESULT name=verdict domain=L ring_slots=64 smallest_n=1 largest_n=3 states=6"
    echo "E7RESULT name=done emitted=21" ;;
  --list-states)
    n="\${2#L:}"; state_line "\$n" 0; state_line "\$n" 1 ;;
  --replay-state)
    rest="\${2#L:}"; state_line "\${rest%%:*}" "\${rest##*:}" ;;
esac
BIN
chmod +x "$CARGO_TARGET_DIR/release/e165_replay_bounded_enumeration"
'''


FAKE_CARGO_R3 = r'''#!/usr/bin/env python3
# 假的 cargo（r3 档自证用）：只认 build。副本的 recovery.rs 或 lib.rs 里有 BROKEN_SYNTAX 就「编不过」；否则「编」出一个 python 脚本当二进制，
# 它照两份源文件里的记号决定 --profile r3 --bounds segment1 --family root_choice 的输出：FLAG_KJ1（K 域 M = 1、U = 2 的 KJ1 违例数 3）、
# FLAG_KJ2（K 域 M = 1、U = 0 的 KJ2 违例数 2）、FLAG_KA_<n>（KA 的 KJ2 违例数 n，没有时 0）、FLAG_HALT（K 域 M = 1、U = 1 的 KJ0 违例 1、
# choose_root 交回空 1，落完行后退 1）、FLAG_OVER（K9 的所选根之上大于上限的态数 4）。二进制把每一次被调的参数记进 FAKE_BIN_LOG；
# 不带 --profile r3 就认不得 --family root_choice（退 2，同真装置）；FAKE_BIN_NO_R3_ROWS 设了时不打 criteria=root_choice 的判决行。
import os
import re
import sys

with open(os.environ["FAKE_CARGO_LOG"], "a") as log:
    log.write(os.getcwd() + "\n")
sources = ""
for path in ("crates/singlefs-core/src/recovery.rs", "crates/singlefs-replay-prefix/src/lib.rs"):
    with open(path, encoding="utf-8") as handle:
        sources += handle.read()
if "BROKEN_SYNTAX" in sources:
    print("error: could not compile", file=sys.stderr)
    sys.exit(101)
ka = re.search(r"FLAG_KA_(\d+)", sources)
flags = {"kj1": "FLAG_KJ1" in sources, "kj2": "FLAG_KJ2" in sources, "halt": "FLAG_HALT" in sources, "over": "FLAG_OVER" in sources,
         "ka": int(ka.group(1)) if ka else 0}
BINARY = """#!/usr/bin/env python3
import os
import sys

FLAGS = __FLAGS__
arguments = sys.argv[1:]
with open(os.environ.get('FAKE_BIN_LOG', os.devnull), 'a') as log:
    log.write(' '.join(arguments) + '\\n')
profile = None
rest = []
index = 0
while index < len(arguments):
    if arguments[index] == '--profile':
        profile = arguments[index + 1]
        index += 2
    else:
        rest.append(arguments[index])
        index += 1
if 'root_choice' in rest and profile != 'r3':
    print('  ✗ --family root_choice 只在 --profile r3 时有', file=sys.stderr)
    print('     → 怎么办：加 --profile r3 重跑，或换一个别的 --family', file=sys.stderr)
    sys.exit(2)
rows = ['E7RESULT name=config experiment=E165 bounds=segment1 family=root_choice profile=r3']
cells = [('K', 1, 0), ('K', 1, 1), ('K', 1, 2), ('K9', 1, 0), ('KW', 1, 0), ('KW', 1, 1)]


def state_row(domain, m, u, none, over):
    return (f'E7RESULT name=root_choice_states domain={domain} ring_slots=64 slots_per_region=8 mounts={m} user_publishes={u} states=10 '
            f'choose_root_returned_none={none} replay_failed=0 above_chosen_root_over_limit={over} above_chosen_root_at_limit=0 '
            f'above_chosen_root_max=1 panicked=0')


def violation_row(domain, m, u, criterion, count, judged, witness='none'):
    return (f'E7RESULT name=violations domain={domain} ring_slots=64 slots_per_region=8 mounts={m} user_publishes={u} '
            f'criterion={criterion} violations={count} judged={judged} witness={witness}')


for domain, m, u in cells:
    kj0 = kj1 = kj2 = kj3 = none = over = 0
    if FLAGS['kj1'] and (domain, m, u) == ('K', 1, 2):
        kj1 = 3
    if FLAGS['kj2'] and (domain, m, u) == ('K', 1, 0):
        kj2 = 2
    if FLAGS['halt'] and (domain, m, u) == ('K', 1, 1):
        kj0, none = 1, 1
    if FLAGS['over'] and domain == 'K9':
        over = 4
    rows.append(state_row(domain, m, u, none, over))
    for criterion, count in (('KJ0', kj0), ('KJ1', kj1), ('KJ2', kj2), ('KJ3', kj3)):
        rows.append(violation_row(domain, m, u, criterion, count, 10))
rows.append(state_row('KA', 0, 0, 1, 0))
rows.append(violation_row('KA', 0, 0, 'KJ0', 1, 125, 'choose_root_returned_none;ka_region0_slots012=[empty,empty,empty]'))
rows.append(violation_row('KA', 0, 0, 'KJ2', FLAGS['ka'], 124))
if not os.environ.get('FAKE_BIN_NO_R3_ROWS'):
    for domain in ('K', 'K9', 'KW', 'KA'):
        rows.append(f'E7RESULT name=verdict domain={domain} ring_slots=64 slots_per_region=8 largest_mounts=1 largest_user_publishes=2 criteria=root_choice states=10')
if FLAGS['halt']:
    rows.append('E7RESULT name=halt condition=TF9_K_域里_choose_root_交回空的态数_1')
rows.append(f"E7RESULT name=verdict scope=run bounds=segment1 family=root_choice states=60 halt_conditions={1 if FLAGS['halt'] else 0}")
rows.append(f'E7RESULT name=done emitted={len(rows) + 1}')
print('\\n'.join(rows))
sys.exit(1 if FLAGS['halt'] else 0)
"""
directory = os.path.join(os.environ["CARGO_TARGET_DIR"], "release")
os.makedirs(directory, exist_ok=True)
binary = os.path.join(directory, "e165_replay_bounded_enumeration")
with open(binary, "w", encoding="utf-8") as handle:
    handle.write(BINARY.replace("__FLAGS__", repr(flags)))
os.chmod(binary, 0o755)
'''


FAKE_CARGO_R4 = r'''#!/usr/bin/env python3
# 假的 cargo（r4 档自证用）：只认 build。副本的 recovery.rs 或 lib.rs 里有 BROKEN_SYNTAX 就「编不过」；否则「编」出一个 python 脚本当二进制，
# 它照两份源文件里的记号决定 --profile r4 --bounds segment1 --family root_choice_faults 的输出：FLAG_KJ1T（KF 域 M = 1、U = 2、被判时无故障、历史罩着的格 KJ1T 违例 3）、
# FLAG_KJ2（KF 域 M = 1、U = 0、被判时 RN、历史罩着的格 KJ2 违例 2）、FLAG_KA_<n>（KA 的 KJ2 违例数 n，没有时 0）、FLAG_KG_<列>（KG 两个格的读数换成登记表里那一列，
# 没有时取「今天的序」那一列）、FLAG_KG_BAD（KG 的读数不等于任何一列）、FLAG_HALT（KF 域 M = 1、U = 1 的 KJ0 违例 1、落完行后退 1）。
# 二进制把每一次被调的参数记进 FAKE_BIN_LOG；不带 --profile r4 就认不得 --family root_choice_faults（退 2，同真装置）；FAKE_BIN_NO_R4_ROWS 设了时不打 criteria=root_choice_faults 的判决行。
import os
import re
import sys

with open(os.environ["FAKE_CARGO_LOG"], "a") as log:
    log.write(os.getcwd() + "\n")
sources = ""
for path in ("crates/singlefs-core/src/recovery.rs", "crates/singlefs-replay-prefix/src/lib.rs"):
    with open(path, encoding="utf-8") as handle:
        sources += handle.read()
if "BROKEN_SYNTAX" in sources:
    print("error: could not compile", file=sys.stderr)
    sys.exit(101)
ka = re.search(r"FLAG_KA_(\d+)", sources)
kg = re.search(r"FLAG_KG_(v\d+)", sources)
flags = {"kj1t": "FLAG_KJ1T" in sources, "kj2": "FLAG_KJ2" in sources, "halt": "FLAG_HALT" in sources, "kg_bad": "FLAG_KG_BAD" in sources,
         "kg": kg.group(1) if kg else "today", "ka": int(ka.group(1)) if ka else 0}
BINARY = """#!/usr/bin/env python3
import os
import sys

FLAGS = __FLAGS__
KG_COLUMNS = {
    'today': {(1, 0): (0, 0, 0), (1, 2): (117, 117, 0)},
    'v1822': {(1, 0): (0, 0, 0), (1, 2): (117, 117, 3)},
    'v1823': {(1, 0): (0, 0, 0), (1, 2): (117, 117, 0)},
    'v1824': {(1, 0): (0, 0, 8), (1, 2): (468, 468, 351)},
    'v1825': {(1, 0): (0, 0, 0), (1, 2): (117, 117, 0)},
    'v1826': {(1, 0): (0, 0, 0), (1, 2): (200, 200, 5)},
}
arguments = sys.argv[1:]
with open(os.environ.get('FAKE_BIN_LOG', os.devnull), 'a') as log:
    log.write(' '.join(arguments) + '\\n')
profile = None
gate = None
rest = []
index = 0
while index < len(arguments):
    if arguments[index] == '--profile':
        profile = arguments[index + 1]
        index += 2
    elif arguments[index] == '--gate':
        gate = arguments[index + 1]
        index += 2
    else:
        rest.append(arguments[index])
        index += 1
if 'root_choice_faults' in rest and profile != 'r4':
    print('  ✗ --family root_choice_faults 只在 --profile r4 时有', file=sys.stderr)
    sys.exit(2)
rows = ['E7RESULT name=config experiment=E165 bounds=segment1 family=root_choice_faults profile=r4' + (' witness_gate=r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots' if gate == 'r2' else '')]
cells = [('KF', 1, 0), ('KF', 1, 1), ('KF', 1, 2), ('KF3', 1, 0), ('KF9', 1, 0), ('KFW', 1, 0), ('KG', 1, 0), ('KG', 1, 2)]
subcells = [('none', 'inside_the_promise'), ('RN', 'inside_the_promise'), ('RI', 'outside_the_promise')]


def state_row(domain, m, u, none):
    return (f'E7RESULT name=root_choice_fault_states domain={domain} ring_slots=64 slots_per_region=8 mounts={m} user_publishes={u} histories=10 '
            f'choose_root_returned_none_with_a_readable_root={none} replay_failed=0 panicked=0 kj1_and_kj1t_differing_states=0')


def violation_row(domain, m, u, fault, label, criterion, count):
    return (f'E7RESULT name=violations domain={domain} ring_slots=64 slots_per_region=8 mounts={m} user_publishes={u} judged_fault={fault} history={label} '
            f'criterion={criterion} states=10 violations={count} judged=10 witness=none')


def count_of(domain, m, u, fault, criterion):
    if FLAGS['kj1t'] and (domain, m, u, fault, criterion) == ('KF', 1, 2, 'none', 'KJ1T'):
        return 3
    if FLAGS['kj2'] and (domain, m, u, fault, criterion) == ('KF', 1, 0, 'RN', 'KJ2'):
        return 2
    if FLAGS['halt'] and (domain, m, u, fault, criterion) == ('KF', 1, 1, 'none', 'KJ0'):
        return 1
    if domain == 'KG' and fault == 'none' and criterion in ('KJ1', 'KJ1T', 'KJ2'):
        reading = KG_COLUMNS[FLAGS['kg']][(m, u)]
        if FLAGS['kg_bad']:
            reading = (reading[0] + 1, reading[1], reading[2])
        return reading[('KJ1', 'KJ1T', 'KJ2').index(criterion)]
    return 0


for domain, m, u in cells:
    rows.append(state_row(domain, m, u, 1 if FLAGS['halt'] and (domain, m, u) == ('KF', 1, 1) else 0))
    for fault, label in subcells:
        for criterion in ('KJ0', 'KJ1', 'KJ1T', 'KJ2', 'KJ3'):
            rows.append(violation_row(domain, m, u, fault, label, criterion, count_of(domain, m, u, fault, criterion)))
rows.append('E7RESULT name=root_choice_states domain=KA ring_slots=64 slots_per_region=8 mounts=0 user_publishes=0 states=125')
rows.append('E7RESULT name=violations domain=KA ring_slots=64 slots_per_region=8 mounts=0 user_publishes=0 criterion=KJ0 violations=1 judged=125 witness=choose_root_returned_none')
rows.append(f"E7RESULT name=violations domain=KA ring_slots=64 slots_per_region=8 mounts=0 user_publishes=0 criterion=KJ2 violations={FLAGS['ka']} judged=124 witness=none")
if not os.environ.get('FAKE_BIN_NO_R4_ROWS'):
    for domain in ('KF', 'KF3', 'KF9', 'KFW', 'KG'):
        rows.append(f'E7RESULT name=verdict domain={domain} ring_slots=64 slots_per_region=8 largest_mounts=1 largest_user_publishes=2 criteria=root_choice_faults states=10')
    rows.append('E7RESULT name=verdict domain=KA ring_slots=64 slots_per_region=8 largest_mounts=0 largest_user_publishes=0 criteria=root_choice states=125')
if FLAGS['halt']:
    rows.append('E7RESULT name=halt condition=TF9_KF_域里_replay_journal_报错的态数_1')
rows.append(f"E7RESULT name=verdict scope=run bounds=segment1 family=root_choice_faults states=60 halt_conditions={1 if FLAGS['halt'] else 0} profile=r4")
rows.append(f'E7RESULT name=done emitted={len(rows) + 1}')
print('\\n'.join(rows))
sys.exit(1 if FLAGS['halt'] else 0)
"""
directory = os.path.join(os.environ["CARGO_TARGET_DIR"], "release")
os.makedirs(directory, exist_ok=True)
binary = os.path.join(directory, "e165_replay_bounded_enumeration")
with open(binary, "w", encoding="utf-8") as handle:
    handle.write(BINARY.replace("__FLAGS__", repr(flags)))
os.chmod(binary, 0o755)
'''


def selftest():
    # 判定写进临时 prune-server：只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem），不在就退 2、一格都不跑
    code, output = item_verdicts_command(["selftest-service"])
    if code != 0:
        print(output.rstrip())
        print("  ✗ 自检没跑：判定写进临时 prune-server，自证只在临时服务底下跑")
        print("  → 怎么办：照文件头「用法」--selftest 那一条的包装写法起（python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash <本脚本> --selftest）")
        return EXIT_USAGE
    failures = []
    checked = 0
    selftest_started = time.monotonic()

    def say_progress(text):
        # 自证要跑一阵：每判完一项（与每段长的开跑前）打一行、立刻刷出，看日志的人知道走到哪了（文件头「用法」--selftest）
        print(f"E165_KILL_BOUNDS_SELFTEST_PROGRESS 挂钟 {time.monotonic() - selftest_started:.0f} 秒，判完 {checked} 项（不过 {len(failures)} 项）；{text}", flush=True)

    def expect(condition, what, fix):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append((what, fix))
        say_progress(f"刚判完的一项{'过了' if condition else '不过'}：{what[:120]}")

    def write(path, text):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)

    def read(path):
        if not os.path.exists(path):
            return ""
        with open(path, encoding="utf-8") as handle:
            return handle.read()

    # --jobs、--threads 的缺省：登记文件第一条非注释行的数是 N；什么都不给 1 × N，只给 --jobs J 是 J × (N ÷ J)，设了上限 7 取 7，两个都给原样
    registered = next((int(line.split()[0]) for line in read(os.path.join(DEFAULT_ROOT, ".claude", "heavy-thread-default")).splitlines()
                       if line.strip() and not line.lstrip().startswith("#")), None)
    defaults = (default_jobs_and_threads(None, None, {}), default_jobs_and_threads(4, None, {}), default_jobs_and_threads(None, None, {"SINGLEFS_THREAD_CAP": "7"}),
                default_jobs_and_threads(2, 3, {}), default_jobs_and_threads(None, 5, {}))
    expect(registered is not None and defaults == ((1, registered), (4, max(1, registered // 4)), (1, 7), (2, 3), (1, 5)),
           f"--jobs、--threads 的缺省：什么都不给要 1 × 登记的重活默认并行度（{registered}）、只给 --jobs 4 要 4 × 登记值 ÷ 4、上限 7 要 1 × 7、给了的原样（实际 {defaults}）",
           "看 default_jobs_and_threads 与 .claude/heavy-thread-default")

    # 真跑的按项判定请求里判法那一段（item_requests_text 经 judge_tokens 写）：三套各写判法模块里判一个变体的函数与写基线那一行的函数、judge-root 是脚本所在的仓，
    # 不写整份驱动（整份驱动的判法闭包连到选行、钉红例与自证读的整张表，表改一行每一项的键都变）；那几个函数要真在判法模块里。开关 judgewholedriver 下判法写回整份驱动
    judge_module = "scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py"
    wanted_judges = {"r2": ["judge_variant", "baseline_item_line"], "r3": ["judge_variant_r3", "baseline_item_line_r3"], "r4": ["judge_variant_r4", "baseline_item_line_r4"]}
    written_judges = {}
    for profile in wanted_judges:
        request = item_requests_text(Setup(DEFAULT_ROOT, False, profile), [{"name": "baseline"}], [])
        written_judges[profile] = [token for token in request.rstrip("\n").split("\t") if token.startswith(("judge:", "judge-root:"))]
    in_module = sorted({os.path.realpath(function.__code__.co_filename) for functions in JUDGE_FUNCTIONS.values() for function in functions})
    import e165_replay_mutation_kill_bounds_variant_judgement as judgement_module
    names_of_the_driver = [function.__name__ for profile in ("r2", "r3", "r4") for function in JUDGE_FUNCTIONS[profile]]
    expect(list(judgement_module.JUDGING_ENTRIES) == names_of_the_driver,
           f"判法模块登记的判法入口 JUDGING_ENTRIES 应当与驱动的 JUDGE_FUNCTIONS 按 r2、r3、r4 的次序同名，实际模块 {list(judgement_module.JUDGING_ENTRIES)}、驱动 {names_of_the_driver}",
           "看判法模块的 JUDGING_ENTRIES 与驱动的 JUDGE_FUNCTIONS：加、删、改名一个判法函数时两边一起改（算键库自证照模块那一行逐个收闭包）")
    expect(all(written_judges[profile] == [f"judge:{judge_module}::{name}" for name in names] + [f"judge-root:{DEFAULT_ROOT}"] for profile, names in wanted_judges.items())
           and in_module == [os.path.realpath(os.path.join(DEFAULT_ROOT, judge_module))],
           f"真跑的按项判定请求里判法应当是判法模块里判一个变体的函数与写基线那一行的函数（三套各两个）、judge-root 是脚本所在的仓，不是整份驱动，"
           f"实际 {written_judges}，函数所在的文件 {in_module}",
           "看 judge_tokens 与 JUDGE_FUNCTIONS：判法写 judge:<判法模块>::<函数>，函数要在判法模块里")

    # 副本的范围取共用库在根上现算的闭包（copy_root_entries）：假仓里 crates/demo 经 path 依赖仓根下的 served/、编译期读 .claude/heavy-thread-default
    # ⇒ 两样连同共用清单拷进副本，research/ 不拷；副本里起命令的多机配置取脚本所在的仓那一份（environment_in_the_copy：假仓 git init 过，没设取它的 multi-host.env，设了的照原样）
    with tempfile.TemporaryDirectory(prefix="e165-kill-bounds-copy-") as temporary:
        source = os.path.join(temporary, "root")
        for relative, text in (("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/demo"]\n'), ("Cargo.lock", "# lock\n"),
                               ("crates/demo/Cargo.toml", '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n'
                                                          '[dependencies]\nserved = { path = "../../served" }\n'),
                               ("crates/demo/src/lib.rs", 'pub const THREADS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.claude/heavy-thread-default"));\n'),
                               ("served/Cargo.toml", '[package]\nname = "served"\nversion = "0.1.0"\nedition = "2021"\n'), ("served/src/lib.rs", "pub fn served() {}\n"),
                               (".claude/heavy-thread-default", "7\n"), ("research/results/big.out", "r\n")):
            os.makedirs(os.path.dirname(os.path.join(source, relative)), exist_ok=True)
            write(os.path.join(source, relative), text)
        copy = os.path.join(temporary, "copy")
        try:
            copy_root_entries(source, copy)
            problem = None
        except Stop as stop:
            problem = str(stop)
        copied = sorted(os.path.relpath(os.path.join(directory, name), copy) for directory, _, names in os.walk(copy) for name in names)
        wanted = [".claude/heavy-thread-default", "Cargo.lock", "Cargo.toml", "crates/demo/Cargo.toml", "crates/demo/src/lib.rs", "served/Cargo.toml", "served/src/lib.rs"]
        expect(problem is None and copied == wanted,
               f"副本应当照共用库的闭包拷：应当是 {wanted}（path 依赖的 served/、编译期读的 .claude/heavy-thread-default 都在，research/ 不在），实际 {problem or copied}",
               "看 copy_root_entries 与 scripts/hosts/lib_repository_copy.py 的 copy_closure、copy_entries")
        git_made = subprocess.run(["git", "init", "-q", source], capture_output=True, stdin=subprocess.DEVNULL).returncode == 0
        wanted_configuration = os.path.join(os.path.realpath(source), "multi-host.env")
        unset = environment_in_the_copy(source, {}).get("SINGLEFS_MULTI_HOST_CONFIG")
        given = environment_in_the_copy(source, {"SINGLEFS_MULTI_HOST_CONFIG": "/given/multi-host.env"}).get("SINGLEFS_MULTI_HOST_CONFIG")
        expect(git_made and unset == wanted_configuration and given == "/given/multi-host.env",
               f"副本里起命令的多机配置应当取脚本所在的仓那一份：没设应当是 {wanted_configuration}、设了的照原样，实际 git 建仓 {git_made}、没设得 {unset}、设了得 {given}",
               "看 environment_in_the_copy 与 scripts/hosts/lib_repository_copy.py 的 environment_for_commands_in_the_copy")

    with tempfile.TemporaryDirectory(prefix="e165-kill-bounds-selftest-") as temporary:
        root = os.path.join(temporary, "root")
        os.makedirs(os.path.join(root, "crates/singlefs-core/src"))
        os.makedirs(os.path.join(root, "crates/singlefs-replay-prefix/src"))
        os.makedirs(os.path.join(root, "scripts/mutation/tables"))
        recovery = "fn replay() {\n    let alpha = 1;\n    let beta = 2;\n    let gamma = 3;\n}\n"
        library = "fn core_one() {\n    let delta = 4;\n    let epsilon = 5;\n}\n\nfn outside_registered() {\n    let zeta = 6;\n}\n"
        write(os.path.join(root, RECOVERY_PATH), recovery)
        write(os.path.join(root, LIB_PATH), library)
        write(os.path.join(root, "Cargo.toml"), "[workspace]\n")
        good_table = ("# 表头\n"
                      "红的\t" + RECOVERY_PATH + "\tlet alpha = 1;\tlet alpha = 9; // FLAG_RED\t-p x\tt\n"
                      "不红的\t" + RECOVERY_PATH + "\tlet beta = 2;\tlet beta = 7;\t-p x\tt\n"
                      "编不过的\t" + RECOVERY_PATH + "\tlet gamma = 3;\tBROKEN_SYNTAX\t-p x\tt\n"
                      "第二份源文件里红的\t" + LIB_PATH + "\tlet delta = 4;\tlet delta = 9; // FLAG_RED\t-p x\tt\n")
        good_custom = "Z1 自造的红\t" + LIB_PATH + "\tlet epsilon = 5;\tlet epsilon = 8; // FLAG_RED\n"
        # 假标签表：r3 今天的集合比登记时多一个（对不上）、r5 登记时那一版自己定的集合比登记的多一个（RH4）
        good_tags = ("# 自检用的假标签表\n"
                     "r2\told\tPD1\tyes\t见证\nr2\ttoday\tPD1\tyes\t见证\n"
                     "r3\told\tPD2\tyes\t见证\nr3\ttoday\tPD2,PD5\tyes\t见证\n"
                     "r4\told\tPD3\tyes\t见证\nr4\ttoday\tPD3\tyes\t见证\n"
                     "r5\told\tPD4,PD6\tyes\t见证\nr5\ttoday\tPD4\tyes\t见证\nr5\ttoday\tPD6\tno\t理由\n"
                     "Z1\told\tPD2\tyes\t见证\nZ1\ttoday\tPD2\tyes\t见证\n")
        registered_names = [[2, "红的", ["PD1"]], [3, "不红的", ["PD2"]], [4, "编不过的", ["PD3"]], [5, "第二份源文件里红的", ["PD4"]]]

        def register(names=None, library_text=None, custom_keys=("Z1",)):
            write(os.path.join(root, "selftest-registered.json"), json.dumps({
                "files_sha256": {RECOVERY_PATH: hashlib.sha256(recovery.encode()).hexdigest(), LIB_PATH: hashlib.sha256((library_text or library).encode()).hexdigest()},
                "functions": [[RECOVERY_PATH, "replay", 1, 5], [LIB_PATH, "core_one", 1, 4]],
                "names": names if names is not None else registered_names,
                "z_tags": {"Z1": ["PD2"]}, "custom_keys": list(custom_keys), "segment1_products": ["seg1.out"]}))

        write(os.path.join(root, TABLE_PATH), good_table)
        write(os.path.join(root, CUSTOM_TABLE_PATH), good_custom)
        write(os.path.join(root, TAGS_TABLE_PATH), good_tags)
        # 第一段 r2 档产物的样子：假二进制不带任何 FLAG、带 --profile r2 时 --bounds 打的每个 (域, N) 行
        write(os.path.join(root, "seg1.out"),
              "".join(f"E7RESULT name=states domain=L ring_slots=64 n={n} states=2 panicked=0 failed=0\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J4 violations=0 judged=2 witness=none\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J6_premise_holds violations=0 judged=2 witness=none\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J6_premise_fails violations=0 judged=0 witness=none\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J6Qn_premise_holds violations=0 judged=2 witness=none\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J6Qn_premise_fails violations=0 judged=0 witness=none\n"
                      for n in (1, 2, 3)))
        register()
        os.makedirs(os.path.join(root, ".claude/gate.d"))
        write(os.path.join(root, ".claude/gate.d/e165-pinned-red-examples.tsv"), "# 自检用的钉住的表\n")
        write(os.path.join(root, "fake-cargo"), FAKE_CARGO)
        os.chmod(os.path.join(root, "fake-cargo"), 0o755)
        # 自证起的那几趟按项判定的判法（judge_tokens）：判法闭包在假树上算，复用那六格的样板树是这棵树的拷贝，同样带着它
        os.makedirs(os.path.dirname(os.path.join(root, SELFTEST_JUDGE_FILE)), exist_ok=True)
        write(os.path.join(root, SELFTEST_JUDGE_FILE), SELFTEST_JUDGE_TEXT)
        subprocess.run(["git", "init", "-q", root], check=True)
        cargo_log = os.path.join(temporary, "cargo.log")
        binary_log = os.path.join(temporary, "binary.log")

        library_count = [0]

        def drive(arguments, broken="", extra=None, fake_root=None, library=None):
            # 弄坏开关：外面设了（BREAK，自证这个进程读到的）就每一趟都带上，再并上这一趟自己点名的（对照那几例）；外面没设只带这一趟自己的。
            # r2 那四项（R2_BREAK_MODES）与 r3 那四项（R3_BREAK_MODES）不在这里带：r2、r3 那几趟自己把外面给的原样传进去（r2_broken、r3_broken），别的几趟照常跑
            # 判定库：临时 prune-server 里每一趟一份新的命名空间（LONG_RUN_ITEMS_LIBRARY_DIRECTORY），续跑那几格给 library 让几趟共用一份；
            # 复用哨兵全当哨兵、不抽强制现判（编几次是定数）；假仓里没有真装置的源码，不给 LONG_RUN_ITEMS_SELFTEST_ENTRIES 的那几趟每一项都拒算、照跑
            library_count[0] += 1
            environment = dict(os.environ)
            environment["E165_KILL_BOUNDS_SELFTEST_ROOT"] = fake_root or root
            environment["FAKE_CARGO_LOG"] = cargo_log
            environment["FAKE_BIN_LOG"] = binary_log
            environment["E165_KILL_BOUNDS_BREAK"] = ",".join(sorted((BREAK - set(R2_BREAK_MODES) - set(R3_BREAK_MODES) - set(R4_BREAK_MODES)) | set(filter(None, broken.split(",")))))
            environment["LONG_RUN_ITEMS_LIBRARY_DIRECTORY"] = library or os.path.join(temporary, f"library-{library_count[0]}")
            environment["SINGLEFS_REUSE_SENTINEL_COUNT"] = "1000"
            for name in ("SINGLEFS_REUSE_SENTINEL_SEED", "LONG_RUN_ITEMS_SELFTEST_ENTRIES", "LONG_RUN_ITEMS_SELFTEST_FAULT", "LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE", KILL_AFTER_VARIABLE):
                environment.pop(name, None)
            environment.pop("SINGLEFS_THREAD_CAP", None)
            environment.pop("E165_KILL_BOUNDS_PROGRESS_INTERVAL", None)
            environment.pop("FAKE_BIN_SLEEP", None)
            environment.pop("FAKE_BIN_NO_R2_ROWS", None)
            environment.pop("FAKE_BIN_NO_R3_ROWS", None)
            environment.pop("FAKE_BIN_NO_R4_ROWS", None)
            environment.update(extra or {})
            # 几趟共用一份库的那几格（续跑、钉红例、改判法）重交同一条命令：已经存在的 --output 不收（文件头「用法」），
            # 驱动先把上一趟的产物与账挪成 .previous（只供自证的 SET_OUTPUT_ASIDE_VARIABLE）；别的格不设，--output、--ledger 已经存在的照拒
            environment.pop(SET_OUTPUT_ASIDE_VARIABLE, None)
            if library is not None:
                environment[SET_OUTPUT_ASIDE_VARIABLE] = "1"
            write(cargo_log, "")
            write(binary_log, "")
            completed = subprocess.run(["bash", SELF_PATH] + arguments, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            builds = [line for line in read(cargo_log).split("\n") if line]
            return completed, builds

        def paths(tag):
            return ["--scratch", os.path.join(temporary, "scratch-" + tag), "--output", os.path.join(temporary, tag + ".out"),
                    "--ledger", os.path.join(temporary, tag + ".ledger"), "--jobs", "2", "--threads", "1"]

        # 一、整轮：红、界内不红、无效各一条，自造一条红；收尾行齐。
        completed, builds = drive(paths("full"))
        output = read(os.path.join(temporary, "full.out"))
        ledger = read(os.path.join(temporary, "full.ledger"))
        last_line = output.rstrip("\n").split("\n")[-1]
        expect(completed.returncode == 0, f"整轮应当退 0，实际 {completed.returncode}：{completed.stdout[-400:]}{completed.stderr[-400:]}", "看 main_run 在假仓上卡在哪一步")
        expect(re.search(r"item=tsv-2 .* outcome=red first_clause_red=L:2:J4 .* legal_family_minimum=2 ", output) is not None, "红的那一条应当记 outcome=red、首红 L:2:J4", "看 compare_with_baseline 与 differences_at")
        expect(re.search(r"item=tsv-3 .* outcome=not_red_within_bounds .* not_red_domains=L\(≤3\) ", output) is not None, "不红的那一条应当记 not_red_within_bounds、上沿 3", "看判法模块 judge_variant 的结局分类")
        expect(re.search(r"item=tsv-4 .* outcome=invalid ", output) is not None, "编不过的那一条应当记 invalid", "看 prepare_and_build 的返回")
        expect(re.search(r"item=tsv-5 .* outcome=red first_clause_red=L:2:J4 ", output) is not None, "文件列是 lib.rs 的那一条应当改 lib.rs、记 outcome=red", "看 prepare_and_build 的 target_of")
        expect(re.search(r"item=Z1 .* outcome=red ", output) is not None, "自造的那一条（文件列 lib.rs）应当记 red", "看 load_items 读自造表")
        expect("已还原，基线与开头逐字相同" in completed.stdout and " name=restored " in output, "收尾要打「已还原，基线与开头逐字相同」并写 name=restored 行", "看 main_run 的收尾")
        expect(last_line == "E7RESULT name=done emitted=10", f"收尾行应当是 name=done emitted=10，实际 {last_line!r}", "emitted 数的是 E7RESULT 行，含 config 与 done")
        item_rows = [line for line in completed.stdout.split("\n") if line.startswith("E165_KILL_BOUNDS_ITEM ")]
        item_reds = {found.group(1): int(found.group(2)) for found in (re.match(r"E165_KILL_BOUNDS_ITEM item=(\S+) exit=0 judged=1 red=(\d) reused=0 seconds=\d+\.\d$", line) for line in item_rows) if found}
        stdout_last = completed.stdout.rstrip("\n").split("\n")[-1]
        expect(len(item_rows) == 6 and item_reds == {"baseline": 0, "tsv-2": 1, "tsv-3": 0, "tsv-4": 0, "tsv-5": 1, "Z1": 1}
               and re.match(r"E165_KILL_BOUNDS_TOTAL dispatched=6 collected=6 items=6 judged=6 red=3 reused=0 failed=0 seconds=\d+\.\d$", stdout_last) is not None,
               f"整轮每项一行应当带判了、红了、复用了多少与秒，末行总计派出 6 收回 6、红 3，实际 {item_rows}；末行 {stdout_last!r}",
               "看 ItemLines 与三套 main_run 里 work() 的 judged_item")
        expect(re.search(r"tsv-2\t2\tL\t2\t1\tcounters\t\+J4\ttrue\ttrue\ttrue\t", ledger) is not None, f"红例账应当有 tsv-2 在 L、N = 2、序号 1、+J4、两边单条重放都对上的一行，实际：{ledger[-300:]!r}", "看 first_differing_state 与 replayed_state_line")
        expect(len(builds) == 7, f"整轮应当编 7 次（基线 2、变体 5），实际 {len(builds)}", "每个变体一份副本、编一次")
        expect(not os.path.exists(os.path.join(temporary, "scratch-full", WORK_DIRECTORY_NAME, "tsv-2")), "变体的副本与编译目录做完要删掉", "看 restore_and_remove")
        pinned_table = os.path.join(root, ".claude/gate.d/e165-pinned-red-examples.tsv")
        ledger_examples = [line for line in ledger.split("\n") if line and not line.startswith("#") and line.split("\t")[4] != "none"]
        pinned_rows = [line for line in read(pinned_table).split("\n") if line and not line.startswith("#")]
        expect(ledger_examples and len(pinned_rows) == len(ledger_examples) and all(row.endswith("\t--profile r2") for row in pinned_rows)
               and f"钉红例：钉进 {len(ledger_examples)} 行、核了 {len(ledger_examples)} 行" in completed.stdout,
               f"整轮写完账与产物要当场钉进钉表、再核钉全：红例账里有 {len(ledger_examples)} 行反例，钉表应当有同样多行、档都是 --profile r2，"
               f"成功那句报钉进与核了几行；实际表里 {len(pinned_rows)} 行：{completed.stdout[-300:]}", "看 main_run 收尾的 pin_red_examples")
        # 一之二、--output 指到已经存在的文件（上面整轮写的那一份），--ledger 是新的：退 2、出路写全、一次不编，那一份一个字不动、账不写（开关 outputoverwritten 下照收、整份换掉）。
        # 一之三、--ledger 指到已经存在的文件（整轮写的那一份），--output 是新的：同样退 2、同一句出路、一次不编，账一个字不动、产物不写（开关 ledgeroverwritten 下照收）。
        # 开关开着时那一趟照新的一对钉、把整轮那一份换掉：两格判完把产物、账与钉表放回整轮写完时的样子，后面几格照整轮那一份比
        pinned_text = read(pinned_table)
        output_taken = paths("full")
        output_taken[output_taken.index("--ledger") + 1] = os.path.join(temporary, "output-taken.ledger")
        completed, builds = drive(output_taken)
        output_untouched = read(os.path.join(temporary, "full.out")) == output
        expect(completed.returncode == EXIT_USAGE and "--output full.out 已经存在" in completed.stdout and "换一个路径，或先挪走那一份" in completed.stdout
               and len(builds) == 0 and output_untouched and not os.path.exists(os.path.join(temporary, "output-taken.ledger")),
               f"--output 指到已经存在的文件应当退 2、出路写换一个路径或先挪走那一份、一次不编、那一份一个字不动，实际退 {completed.returncode}、编 {len(builds)} 次、"
               f"那一份{'没动' if output_untouched else '变了'}：{completed.stdout[-300:]}",
               "看 main 里三套整轮调 refuse_existing_output 的那几处")
        ledger_taken = paths("ledger-taken")
        ledger_taken[ledger_taken.index("--ledger") + 1] = os.path.join(temporary, "full.ledger")
        completed, builds = drive(ledger_taken)
        ledger_untouched = read(os.path.join(temporary, "full.ledger")) == ledger
        expect(completed.returncode == EXIT_USAGE and "--ledger full.ledger 已经存在" in completed.stdout and "换一个路径，或先挪走那一份" in completed.stdout
               and len(builds) == 0 and ledger_untouched and not os.path.exists(os.path.join(temporary, "ledger-taken.out")),
               f"--ledger 指到已经存在的文件应当退 2、出路写换一个路径或先挪走那一份、一次不编、那一份一个字不动，实际退 {completed.returncode}、编 {len(builds)} 次、"
               f"账{'没动' if ledger_untouched else '变了'}：{completed.stdout[-300:]}",
               "看 main 里不给 --profile 那一套调 refuse_existing_output 时交进去的 --ledger")
        write(os.path.join(temporary, "full.out"), output)
        write(os.path.join(temporary, "full.ledger"), ledger)
        write(pinned_table, pinned_text)

        # 二、锚点命中 0 次：自造表里的报「锚点腐化」退 3；crates 表里的让选出的行对不上登记（停机条款 H1）退 4；两样都一次不编、不写产物。
        write(os.path.join(root, CUSTOM_TABLE_PATH), "Z1 锚点坏的\t" + LIB_PATH + "\tlet omega = 4;\tlet omega = 8;\n")
        completed, builds = drive(paths("rot"))
        expect(completed.returncode == 3 and "锚点腐化" in completed.stdout + completed.stderr, f"锚点命中 0 次应当报「锚点腐化」退 3，实际退 {completed.returncode}", "看 check_the_anchors")
        expect(len(builds) == 0 and not os.path.exists(os.path.join(temporary, "rot.out")), f"锚点腐化时一次都不许编、不许写产物，实际编了 {len(builds)} 次", "check_the_anchors 要在建副本之前")
        completed, builds = drive(paths("rot-broken"), broken="noanchorcheck")
        expect(completed.returncode != 3 and len(builds) > 0, f"弄坏开关 noanchorcheck 开着时应当照跑（证明上一例的红来自锚点检查），实际退 {completed.returncode}、编 {len(builds)} 次", "看 BREAK 的 noanchorcheck")
        write(os.path.join(root, CUSTOM_TABLE_PATH), good_custom)
        write(os.path.join(root, TABLE_PATH), good_table.replace("let beta = 2;\tlet beta = 7;", "let omega = 4;\tlet omega = 7;"))
        completed, builds = drive(paths("rot-table"))
        expect(completed.returncode == 4 and "RF2" in completed.stdout and "anchor_hits_0" in completed.stdout and len(builds) == 0, f"crates 表里锚点命中 0 次应当选不到（RF2）退 4、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 select_by_name 与 load_items")
        write(os.path.join(root, TABLE_PATH), good_table)

        # 三、续跑（文件头「按项判定」）：判完 tsv-3 就被停下的一趟（只供自证的 E165_KILL_BOUNDS_SELFTEST_KILL_AFTER：record 之后整个进程被 SIGKILL），
        # 重交同一条命令只补没判的三项，库里复用的三项不编、各打一行 reused=1，产物行与整轮的逐字相同；再交一趟全部复用、一次不编、产物逐字相同。
        # 三趟共用一份判定库命名空间；假仓里的装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假树里的 Python 函数（不换就全部拒算、一项都不复用）。
        planted_code, planted = item_verdicts_command(["plant-fake-entries", root])
        resume_entries = os.path.join(temporary, "resume-entries.tsv")
        write(resume_entries, "".join(f"{name}\tscripts/experiment-apparatus/long_run_reuse_fake_entries.py::{'measure_beta' if name == 'baseline' else 'measure_alpha'}\n"
                                      for name in ("baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "Z1")))
        resume_extra = {"LONG_RUN_ITEMS_SELFTEST_ENTRIES": resume_entries}
        resume_library = os.path.join(temporary, "resume-library")
        resume_arguments = paths("resume")[:-4] + ["--jobs", "1", "--threads", "1"]
        resumed_path = os.path.join(temporary, "resume.out")
        completed, builds = drive(resume_arguments, library=resume_library, extra=dict(resume_extra, **{KILL_AFTER_VARIABLE: "tsv-3"}))
        expect(planted_code == 0 and completed.returncode == -signal.SIGKILL and len(builds) == 3,
               f"续跑：判完 tsv-3 就被停下的那一趟应当被 SIGKILL、只编了 3 次（开头基线、tsv-2、tsv-3），实际种假入口退 {planted_code}（{planted.strip()[-200:]}）、"
               f"退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-300:]}{completed.stderr[-300:]}",
               "看 ItemVerdicts.record 与只供自证的 E165_KILL_BOUNDS_SELFTEST_KILL_AFTER")
        completed, builds = drive(resume_arguments, library=resume_library, extra=resume_extra)
        resumed = read(resumed_path)
        expect(completed.returncode == 0, f"续跑应当退 0，实际 {completed.returncode}：{completed.stdout[-300:]}{completed.stderr[-300:]}", "看 ItemVerdicts 与 main_run")
        expect(len(builds) == 5, f"续跑应当只编 5 次（基线 2、没判的变体 3），实际 {len(builds)}", "判完的变体当场入库（ItemVerdicts.record），重交时 plan 答 reuse 的不编不跑")
        counts = {name: len(re.findall(rf"name=kill_bound item={name} .* complete=yes", resumed)) for name in ("baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "Z1")}
        expect(all(count == 1 for count in counts.values()), f"续跑之后每个变体恰好一行完整的，实际 {counts}", "看 judge_items_in_parallel 的 flush_in_order")
        expect(resumed.rstrip("\n").split("\n")[-1].startswith("E7RESULT name=done "), "续跑补齐之后要写收尾行", "看 main_run 的收尾")
        complete_lines = lambda text: sorted(line for line in text.replace("resume.", "full.").split("\n") if line.endswith("complete=yes"))
        expect(complete_lines(resumed) == complete_lines(output), "续跑出来的变体行（库里取回的与现判的）应当与整轮的逐字相同",
               "看 ItemVerdicts 的 value_text 与 reused_record：复跑命令那一段存成占位、取回时照这一趟填回；变体行里不许有随跑法变的量")
        resumed_items = [line for line in completed.stdout.split("\n") if line.startswith("E165_KILL_BOUNDS_ITEM ")]
        resumed_last = completed.stdout.rstrip("\n").split("\n")[-1]
        expect(any(line.startswith("E165_KILL_BOUNDS_ITEM item=baseline exit=0 judged=0 red=0 reused=1 ") for line in resumed_items)
               and any(line.startswith("E165_KILL_BOUNDS_ITEM item=tsv-2 exit=0 judged=0 red=1 reused=1 ") for line in resumed_items)
               and any(line.startswith("E165_KILL_BOUNDS_ITEM item=tsv-3 exit=0 judged=0 red=0 reused=1 ") for line in resumed_items) and len(resumed_items) == 6
               and re.match(r"E165_KILL_BOUNDS_TOTAL dispatched=3 collected=3 items=6 judged=3 red=3 reused=3 failed=0 ", resumed_last) is not None
               and "按项判定 现判 3、复用 3、拒算 0" in completed.stdout,
               f"续跑：库里复用的三项应当各打一行 reused=1（tsv-2 照库里那一行记 red=1），末行总计派出 3 收回 3、复用 3，成功句报现判 3、复用 3、拒算 0，"
               f"实际 {resumed_items}；末行 {resumed_last!r}", "看 ItemVerdicts.report_reused 与 finish 的 summary")
        pinned_after_resume = [line for line in read(pinned_table).split("\n") if line and not line.startswith("#")]
        expect(pinned_after_resume == pinned_rows and f"钉红例：钉进 0 行、核了 {len(ledger_examples)} 行" in completed.stdout,
               f"续跑补齐之后照样当场钉、当场核：同一批红例已钉，钉表应当不变、报钉进 0 行，实际 {len(pinned_after_resume)} 行：{completed.stdout[-300:]}",
               "看 main_run 收尾的 pin_red_examples：追加已钉的不重复")
        completed, builds = drive(resume_arguments, library=resume_library, extra=resume_extra)
        expect(completed.returncode == 0 and len(builds) == 0 and read(resumed_path) == resumed and "按项判定 现判 0、复用 6、拒算 0" in completed.stdout
               and "开头与收尾的基线都没编没跑" in completed.stdout,
               f"全部复用的一趟应当一次不编、产物与上一趟逐字相同、报现判 0、复用 6，实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-400:]}",
               "看 opening_baseline 一项都不跑的那一支与收尾行的基线行数")

        # 三之二、钉红例（文件头「钉红例」）：全部从库里复用的一趟（不编不跑变体）照样从头写产物与账、照钉照核。
        # 钉表清空，再造「账写完了、钉表没钉」（只供自证的 E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND：不追加）：当场 --check 判红、整趟退 8（开关 pincheckskipped 下退 0）
        write(pinned_table, "# 自检用的钉住的表\n")
        completed, builds = drive(resume_arguments, library=resume_library, extra=dict(resume_extra, E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND="1"))
        expect(completed.returncode == EXIT_PIN_INCOMPLETE and "钉红例：核钉全（--check）没过" in completed.stdout and "行没钉进" in completed.stdout and len(builds) == 0,
               f"账写完而钉表没钉时驱动当场核钉全应当判红、退 {EXIT_PIN_INCOMPLETE}、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-300:]}",
               "看 pin_red_examples 的 --check 那一步与它在 main_run 收尾的位置")
        completed, builds = drive(resume_arguments, library=resume_library, broken="pincheckskipped", extra=dict(resume_extra, E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND="1"))
        expect(completed.returncode == 0 and "核了 没核" in completed.stdout,
               f"弄坏开关 pincheckskipped 开着时应当照样退 0（证明上一例的红来自当场核钉全），实际退 {completed.returncode}：{completed.stdout[-300:]}", "看 BREAK 的 pincheckskipped")
        completed, builds = drive(resume_arguments, library=resume_library, extra=resume_extra)
        repinned = [line for line in read(pinned_table).split("\n") if line and not line.startswith("#")]
        # 钉表的出处 ⑤ 是「<红例账文件名>:<行号>」（e165-pin-red-examples.py 的 pinned_fields）：整轮钉的写 full.ledger，这一趟钉的写 resume.ledger；
        # 别的 21 段要与整轮钉的逐字相同，⑤ 要指到这一趟的账
        without_source = lambda rows: [row.split("\t")[:4] + row.split("\t")[5:] for row in rows]
        expect(completed.returncode == 0 and without_source(repinned) == without_source(pinned_rows) and len(builds) == 0
               and all(row.split("\t")[4].startswith("resume.ledger:") for row in repinned) and len(repinned) == len(pinned_rows)
               and f"钉进 {len(ledger_examples)} 行、核了 {len(ledger_examples)} 行" in completed.stdout,
               f"判过的一对重交同一条命令应当全部复用、照钉照核：钉回同样的 {len(pinned_rows)} 行（出处 ⑤ 换成这一趟的 resume.ledger、别的段与整轮钉的逐字相同）、一次不编，"
               f"实际退 {completed.returncode}、{len(repinned)} 行、编 {len(builds)} 次、出处 {[row.split(chr(9))[4] for row in repinned]}、"
               f"除出处外与整轮钉的{'相同' if without_source(repinned) == without_source(pinned_rows) else '不同'}：{completed.stdout[-300:]}",
               "看 main_run 收尾的 pin_red_examples 与 opening_baseline 一项都不跑的那一支")
        # 对照：弄坏开关 reuseignored（plan 答 reuse 的也照现判）下同一条命令每个变体都重编、复用哨兵判红；
        # recorddeferred（判完不当场入库）下被停下的那一趟一项都没入库，重交全部重编
        completed, builds = drive(resume_arguments, library=resume_library, broken="reuseignored", extra=resume_extra)
        expect(len(builds) == 7 and completed.returncode == EXIT_INCOMPLETE and "复用失灵" in completed.stdout,
               f"弄坏开关 reuseignored 开着时应当每个变体都重编、复用哨兵判红退 1（证明「续跑应当只编 5 次」「全部复用的一趟应当一次不编」来自库里复用），"
               f"实际退 {completed.returncode}、编 {len(builds)} 次", "看 BREAK 的 reuseignored")
        deferred_library = os.path.join(temporary, "deferred-library")
        drive(resume_arguments, library=deferred_library, broken="recorddeferred", extra=dict(resume_extra, **{KILL_AFTER_VARIABLE: "tsv-3"}))
        completed, builds = drive(resume_arguments, library=deferred_library, extra=resume_extra)
        expect(len(builds) == 7, f"弄坏开关 recorddeferred 开着时被停下的那一趟一项都没入库，重交应当全部重编（证明「续跑应当只编 5 次」来自判完当场入库），实际编 {len(builds)} 次",
               "看 BREAK 的 recorddeferred")
        # 改判法：假树里的判法（SELFTEST_JUDGE_FILE）改掉返回的那一行，库里六项都判过，同一条命令应当每一项都现判、一项不拒算；改回去，后面几格照旧。
        # 证明自证的键里判法的闭包在假树上算（不在脚本所在的仓上收整份驱动），判法也进键（开关 judgeoffthetree 下闭包换到假树的上一层，每一项拒算）
        judge_path = os.path.join(root, SELFTEST_JUDGE_FILE)
        write(judge_path, SELFTEST_JUDGE_TEXT.replace(SELFTEST_JUDGE_RESULT, '    return "red when any criterion row differs"\n'))
        completed, builds = drive(resume_arguments, library=resume_library, extra=resume_extra)
        write(judge_path, SELFTEST_JUDGE_TEXT)
        expect(completed.returncode == 0 and len(builds) == 7 and "按项判定 现判 6、复用 0、拒算 0" in completed.stdout,
               f"改了假树里的判法应当每一项都现判、一项不拒算（报现判 6、复用 0、拒算 0，编 7 次），实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-400:]}",
               "看 judge_tokens：自证起的那几趟 judge: 要指假树里的 SELFTEST_JUDGE_ENTRY、不带 judge-root:")

        # 三之三、留存产物不在（research/ 可以整个删掉）：V7 的比对本次未跑——逐项点名、不当绿也不判红，基线那一行写 not_run（开关 retainedmissingred 下读不到、退非 0）
        os.rename(os.path.join(root, "seg1.out"), os.path.join(temporary, "seg1.out.aside"))
        completed, builds = drive(paths("no-seg1"))
        os.rename(os.path.join(temporary, "seg1.out.aside"), os.path.join(root, "seg1.out"))
        no_seg1 = read(os.path.join(temporary, "no-seg1.out"))
        expect(completed.returncode == 0 and "本次未跑：作废条款 V7 的比对：留存产物" in completed.stdout and "seg1.out 不在" in completed.stdout
               and "本次未跑 1 项" in completed.stdout and " rows_found_verbatim_in_segment1_products=not_run " in no_seg1,
               f"r2：第一段产物不在时 V7 的比对应当逐项报本次未跑、照跑完退 0、基线那一行写 not_run，实际退 {completed.returncode}：{completed.stdout[-400:]}{completed.stderr[-200:]}",
               "看 retained_product_not_run 与 check_against_segment_one")

        # 三之四、复用那六格（long_run_item_verdicts.py 文件头「复用格」）：样板树是这时的假仓（表、自造表、登记值同整轮），项甲 tsv-2、项乙 基线，
        # 只点名 tsv-2（基线照 @after 进计划）；两项的装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假树里的两个 Python 函数。
        # 外面开着的按项判定那几项弄坏开关（ITEM_VERDICT_BREAK_MODES）原样带进去，别的不带；LONG_RUN_ITEMS_BREAK 随环境带进去。
        reuse_template = os.path.join(temporary, "reuse-template")
        shutil.copytree(root, reuse_template, symlinks=True)
        # 复用三问一格里同一条命令起四趟：已经存在的 --output 不收（文件头「用法」），驱动先把上一趟的产物与账挪走（只供自证的 SET_OUTPUT_ASIDE_VARIABLE）
        reuse_command = ["env", "E165_KILL_BOUNDS_SELFTEST_ROOT={tree}", "FAKE_CARGO_LOG=/dev/null", "FAKE_BIN_LOG=/dev/null", SET_OUTPUT_ASIDE_VARIABLE + "=1",
                         "E165_KILL_BOUNDS_BREAK=" + ",".join(sorted(BREAK & set(ITEM_VERDICT_BREAK_MODES))), "bash", SELF_PATH,
                         "--scratch", "{tree}/../scratch", "--output", "{tree}/../reuse-cells-kill-bounds.out", "--ledger", "{tree}/../reuse-cells-red-examples.tsv",
                         "--jobs", "1", "--threads", "1", "--item", "tsv-2"]
        say_progress("开跑复用那六格（long_run_item_verdicts.py reuse-cells，十一趟，跑完一并打出）")
        reuse_code, reuse_output = item_verdicts_command(["reuse-cells", "--script", SCRIPT_KEY, "--template", reuse_template, "--items", "tsv-2,baseline",
                                                          "--work", os.path.join(temporary, "reuse-cells"), "--green-exit", "0", "--", *reuse_command])
        print(reuse_output.rstrip())
        expect(reuse_code == 0 and "：6 格，红 0 格" in reuse_output,
               f"复用那六格（long_run_item_verdicts.py reuse-cells）应当全过，实际退 {reuse_code}：{reuse_output.strip()[-600:]}",
               "看上面 ✗ 那一格点名的条件、ItemVerdicts 的 plan / record / finish 与 item_requests_text 进键的那几样")

        # 四、--recheck 单条复跑与产物里的行相同；--list-items 列全、不编。
        completed, builds = drive(["--scratch", os.path.join(temporary, "scratch-recheck"), "--output", os.path.join(temporary, "full.out"),
                                   "--ledger", os.path.join(temporary, "full.ledger"), "--jobs", "1", "--threads", "1", "--recheck", "--item", "tsv-2"])
        expect(completed.returncode == 0 and "✓ tsv-2" in completed.stdout, f"--recheck 同一条应当逐字相同、退 0，实际退 {completed.returncode}：{completed.stdout[-300:]}", "看 main_run 的 recheck 分支")
        completed, builds = drive(["--list-items"])
        listed = [line for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 6 and len(builds) == 0, f"--list-items 应当列 6 个变体、不编，实际：{completed.stdout!r}", "看 main 的 list 分支")
        completed, builds = drive(paths("unknown-item") + ["--item", "tsv-99"])
        expect(completed.returncode == 2 and "不认识的变体：tsv-99；可点名的 6 项：baseline tsv-2 tsv-3 tsv-4 tsv-5 Z1" in completed.stdout and len(builds) == 0,
               f"--item 写了不认识的变体名应当退 2、列出可点名的 6 项、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-300:]}", "看 unknown_item")

        # 五、进度（假时钟）：差一秒到间隔不打；到了间隔打一行，点名在做的变体、它在哪一步、做完几个；再过一个间隔再打一行。
        clock = [1000.0]
        sink = io.StringIO()
        progress = Progress(3, PROGRESS_INTERVAL_SECONDS, clock=lambda: clock[0], sink=sink)
        progress.phase("tsv-9", "跑 --bounds segment2")
        clock[0] += PROGRESS_INTERVAL_SECONDS - 1
        early = progress.poll()
        clock[0] += 1
        first = progress.poll()
        progress.finished("tsv-9")
        clock[0] += PROGRESS_INTERVAL_SECONDS
        second = progress.poll()
        expect(early is None and first is not None and "tsv-9（跑 --bounds segment2，这一步 300 秒" in first and "做完 0/3 个" in first
               and second is not None and "做完 1/3 个" in second and sink.getvalue().count("E165_KILL_BOUNDS_PROGRESS ") == 2,
               f"假时钟上过了进度间隔（{PROGRESS_INTERVAL_SECONDS} 秒）应当各打一行、没到不打，实际：差一秒 {early!r}；到了 {first!r}；再过一个间隔 {second!r}",
               "看 Progress.due 与 Progress.poll")

        # 六、进度接在整轮上：假二进制每次跑 --bounds 慢 1 秒、间隔换成 0.2 秒，stderr 里要有点名在做的那个变体的进度行。
        progress_scratch = os.path.join(temporary, "scratch-progress")
        completed, builds = drive(["--scratch", progress_scratch, "--output", os.path.join(progress_scratch, "progress.out"),
                                   "--ledger", os.path.join(progress_scratch, "progress.ledger"), "--jobs", "1", "--threads", "1", "--item", "tsv-3"],
                                  broken=",".join(sorted(BREAK & {"progressnever"})), extra={"E165_KILL_BOUNDS_PROGRESS_INTERVAL": "0.2", "FAKE_BIN_SLEEP": "1"})
        progress_lines = [line for line in completed.stderr.split("\n") if line.startswith("E165_KILL_BOUNDS_PROGRESS ")]
        expect(completed.returncode == 0 and any("tsv-3（" in line for line in progress_lines),
               f"整轮里一个变体跑过进度间隔应当打出点名它的进度行，实际退 {completed.returncode}、进度行 {progress_lines[:3]!r}",
               "看 main_run 里 Progress 的 start、judge_built_variant 与判法模块 judge_variant 里的 mark")


        # 七、r1 第二段的选行（登记 5.2 R2 三格、文件列一格、标签比对两格、RH1 一格）：都只跑 --select-rows 或停在选行，一次不编；每一格配一个弄坏开关证明它会红。
        def select_run(tag, broken=""):
            target = os.path.join(temporary, tag + ".select.out")
            completed, builds = drive(["--select-rows", "--output", target], broken=broken)
            return completed, builds, read(target)

        verdict_of = lambda product: next((line for line in product.split("\n") if " name=verdict " in line), "")
        write(os.path.join(root, TABLE_PATH), good_table)
        completed, builds, product = select_run("sel-good")
        verdict = verdict_of(product)
        expect(completed.returncode == 0 and len(builds) == 0 and "registered_names=4 selected=4 not_selected=0 " in verdict and "custom_ok=1" in verdict,
               f"选行：假仓里 4 个名字都在、Z1 对得上，应当退 0、不编，实际退 {completed.returncode}、编 {len(builds)} 次：{verdict}", "看 build_row_selection")
        expect("today_equals_registered=3 today_differs_from_registered=1 differing_rows=3 differing_rows_with_extra_tags=3 differing_rows_with_missing_tags=none " in verdict
               and "rh4_rows=5 " in verdict and "registered_version_equals_today_by_the_same_method=2 " in verdict,
               f"选行：假标签表里 r3 今天多一个标签（对不上）、r5 登记时那一版自己定的集合多一个（RH4），总判行应当照实报，实际：{verdict}", "看 compare_tag_sets 与 build_row_selection 的计数")
        expect(re.search(r"name=row_selection row_kind=table item=tsv-5 mutation_name=第二份源文件里红的 .* file=crates/singlefs-replay-prefix/src/lib.rs anchor_hits=1 anchor_lines=2-2 function=core_one function_lines=1-4 selected=true ", product) is not None
               and re.search(r"row_kind=custom item=Z1 .* function=core_one .* selected=true ", product) is not None,
               "选行：文件列是 lib.rs 的那一行应当报它的文件、命中行与所在函数", "看 select_by_name 与 locate_in_functions")
        completed, builds = drive(["--select-rows", "--output", os.path.join(temporary, "sel-good.select.out")])
        expect(completed.returncode == 2 and read(os.path.join(temporary, "sel-good.select.out")) == product, f"选行：产物已存在时应当退 2、不覆盖，实际退 {completed.returncode}", "看 select_rows_action")
        # ① 名字被改掉、行号留着：按名字选应当报「选不到」并点名；弄坏开关 byrownumber 开着时按行号选到
        write(os.path.join(root, TABLE_PATH), good_table.replace("红的\t", "改了名的红的\t", 1))
        completed, builds, product = select_run("sel-renamed")
        expect(completed.returncode == 1 and len(builds) == 0 and "failed_condition=no_row_with_this_name" in product and "not_selected_rows=2 " in verdict_of(product),
               f"选行 ①：名字不在表里应当报 no_row_with_this_name、点名第 2 行、退 1、不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 select_by_name")
        completed, builds, product = select_run("sel-renamed-broken", broken="byrownumber")
        expect(completed.returncode == 0 and "not_selected=0 " in verdict_of(product), f"弄坏开关 byrownumber 开着时应当按行号选到（证明上一例的「选不到」来自按名字选），实际退 {completed.returncode}", "看 BREAK 的 byrownumber")
        # ② 同一个名字多一行
        write(os.path.join(root, TABLE_PATH), good_table + "不红的\t" + RECOVERY_PATH + "\tlet beta = 2;\tlet beta = 6;\t-p x\tt\n")
        completed, builds, product = select_run("sel-duplicate")
        expect(completed.returncode == 1 and len(builds) == 0 and "failed_condition=more_than_one_row_with_this_name" in product and "rows_with_this_name=2" in product,
               f"选行 ②：同名两行应当报 more_than_one_row_with_this_name、退 1、不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 select_by_name")
        completed, builds, product = select_run("sel-duplicate-broken", broken="nodupcheck")
        expect(completed.returncode == 0, f"弄坏开关 nodupcheck 开着时应当取第一行选到（证明上一例的红来自核同名多行），实际退 {completed.returncode}", "看 BREAK 的 nodupcheck")
        # ③ 某一行的原文挪到 9.2 的函数之外
        write(os.path.join(root, TABLE_PATH), good_table.replace("let delta = 4;\tlet delta = 9; // FLAG_RED", "let zeta = 6;\tlet zeta = 9; // FLAG_RED"))
        completed, builds, product = select_run("sel-outside")
        expect(completed.returncode == 1 and len(builds) == 0 and "failed_condition=outside_the_registered_functions" in product and "not_selected_rows=5 " in verdict_of(product),
               f"选行 ③：原文在 9.2 的函数之外应当报 outside_the_registered_functions、点名第 5 行、退 1、不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 locate_in_functions")
        completed, builds, product = select_run("sel-outside-broken", broken="nolocationcheck")
        expect(completed.returncode == 0, f"弄坏开关 nolocationcheck 开着时应当选到（证明上一例的红来自核位置），实际退 {completed.returncode}", "看 BREAK 的 nolocationcheck")
        # ④ 文件列不是 recovery.rs 或 lib.rs
        write(os.path.join(root, TABLE_PATH), good_table.replace("\t" + LIB_PATH + "\tlet delta", "\tcrates/other.rs\tlet delta"))
        completed, builds, product = select_run("sel-other-file")
        expect(completed.returncode == 1 and "failed_condition=file_column_is_not_a_registered_source" in product, f"选行 ④：文件列不是登记的两份应当报 file_column_is_not_a_registered_source，实际退 {completed.returncode}", "看 select_by_name")
        # ⑧ 登记名那一行不成形（cargo 参数那一段空着）：切表照 crates-mutation-rows.py 的 read_table，不收它；弄坏开关 owntablesplit 开着时照收
        write(os.path.join(root, TABLE_PATH), good_table.replace("\tlet alpha = 9; // FLAG_RED\t-p x\tt\n", "\tlet alpha = 9; // FLAG_RED\t\tt\n", 1))
        completed, builds, product = select_run("sel-malformed")
        expect(completed.returncode == 1 and len(builds) == 0 and "failed_condition=no_row_with_this_name" in product and "not_selected_rows=2 " in verdict_of(product),
               f"选行 ⑧：登记名那一行不成形（cargo 参数那一段空着）应当报 no_row_with_this_name、点名第 2 行、退 1、不编，实际退 {completed.returncode}、编 {len(builds)} 次",
               "看 mutation_table_rows（切表照 crates-mutation-rows.py 的 read_table）")
        completed, builds, product = select_run("sel-malformed-broken", broken="owntablesplit")
        expect(completed.returncode == 0, f"弄坏开关 owntablesplit 开着时应当照收那一行、选到（证明上一例的「选不到」来自 read_table 判它不成形），实际退 {completed.returncode}",
               "看 BREAK 的 owntablesplit")
        write(os.path.join(root, TABLE_PATH), good_table)
        # ⑤ 改坏时只写 recovery.rs、不看第二列：文件列是 lib.rs 的那一条由红转「锚点腐化」
        completed, builds = drive(paths("rec-only") + ["--item", "tsv-5"], broken="recoveryonly")
        expect(completed.returncode == 3 and "锚点腐化" in completed.stdout, f"弄坏开关 recoveryonly 开着时文件列是 lib.rs 的那一条应当由红转「锚点腐化」退 3（证明整轮那一格的红来自按第二列改文件），实际退 {completed.returncode}", "看 target_of 与 prepare_and_build")
        # ⑥ RH1：lib.rs 与登记的 sha256 不等：不选行、不编
        write(os.path.join(root, LIB_PATH), "// 多一行\n" + library)
        completed, builds, product = select_run("sel-sha")
        expect(completed.returncode == 4 and "停机条款 RH1" in completed.stdout and len(builds) == 0 and product == "", f"选行 ⑥：lib.rs 的 sha256 不等于登记值应当报停机条款 RH1 退 4、不写产物，实际退 {completed.returncode}", "看 check_the_registered_inputs")
        write(os.path.join(root, LIB_PATH), library)
        # ⑦ 标签比对（登记 5.2 R2 第二行）与登记的绝对数：23 个名字、Z 四行；第 516 行（{PD15}）与第 513 行（{PD4}）互换比判不等，同一行与自己比判相等
        by_row = {row: tags for row, _name, tags in REGISTERED["names"]}
        expect(by_row[516] == ["PD15"] and by_row[513] == ["PD4"] and compare_tag_sets(by_row[516], by_row[513])[0] is False and compare_tag_sets(by_row[516], by_row[516])[0] is True
               and compare_tag_sets(by_row[516], by_row[516] + ["PD1"]) == (False, ["PD1"], []) and compare_tag_sets(by_row[1004], by_row[1004][:1]) == (False, [], ["PD6"]),
               f"标签比对：登记里第 516 行应是 PD15、第 513 行应是 PD4，互换比判不等、自己比自己判相等，多一个报多出来的、少一个报少了的，实际 {by_row.get(516)} {by_row.get(513)}", "看 compare_tag_sets")
        expect(len(REGISTERED["names"]) == 23 and len({row for row, _name, _tags in REGISTERED["names"]}) == 23 and len(REGISTERED["z_tags"]) == 4 and len(REGISTERED["functions"]) == 9
               and [row for row, _name, _tags in REGISTERED["names"]] == [23, 24, 36, 226, 354, 355, 406, 414, 448, 512, 513, 514, 515, 516, 517, 518, 613, 614, 920, 921, 922, 1003, 1004],
               "登记写死的数：23 个名字（登记时的行号 23、24、36、…、1004）、r1 登记标签集合的 Z 四行、9.2 的 9 个函数", "看 REGISTERED")
        expect(REGISTERED["custom_keys"] == ["Z1", "Z2", "Z3", "Z4", "Z5"] and DEVICE_PROFILE_ARGUMENTS == ["--profile", "r2"]
               and J6QN_ROWS == ("J6Qn_premise_holds", "J6Qn_premise_fails")
               and J7_ROWS == ("J7a_instance_table", "J7a_allocation_record_tree_root", "J7a_filesystem_identifier", "J7b", "J7c")
               and set(J6_ROWS) | set(J6QN_ROWS) | set(J7_ROWS) <= CLAUSE_CRITERIA,
               "r2 登记写死的：Z 五行（custom_keys Z1–Z5）、档参数 --profile r2、9.4 第 3 条那七个判据名都在条款判据栏里", "看 REGISTERED、DEVICE_PROFILE_ARGUMENTS 与 CLAUSE_CRITERIA")

        # 八、r2 补的格（登记 r2 9.4 第 6 条）。外面打开的 R2_BREAK_MODES 原样传进这几趟（文件头「弄坏开关」）。
        r2_broken = ",".join(sorted(BREAK & set(R2_BREAK_MODES)))
        r2_table = ("# 表头\n"
                    "J6 二 J6Qn 三\t" + RECOVERY_PATH + "\tlet alpha = 1;\tlet alpha = 9; // FLAG_J6_TWO FLAG_J6QN_THREE\t-p x\tt\n"
                    "只有 J6Qn 三\t" + RECOVERY_PATH + "\tlet beta = 2;\tlet beta = 7; // FLAG_J6QN_THREE\t-p x\tt\n")
        write(os.path.join(root, TABLE_PATH), r2_table)
        write(os.path.join(root, CUSTOM_TABLE_PATH), good_custom)
        register(names=[[2, "J6 二 J6Qn 三", []], [3, "只有 J6Qn 三", []]])
        # ① J6 在 N = 2、J6Qn 在 N = 3 与基线不同 ⇒ (2, 3)；只有 J6Qn 不同 ⇒ J6 报 none；装置每一次被调都带 --profile r2
        completed, builds = drive(paths("r2"), broken=r2_broken)
        output = read(os.path.join(temporary, "r2.out"))
        calls = [line for line in read(binary_log).split("\n") if line]
        expect(completed.returncode == 0, f"r2 整轮应当退 0，实际 {completed.returncode}：{completed.stdout[-400:]}{completed.stderr[-300:]}", "看 main_run 与 check_the_profile_rows")
        expect(re.search(r"item=tsv-2 .* first_red_by_criterion=J6:L=2;J6Qn:L=3 ", output) is not None,
               f"r2：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同应当报 J6:L=2、J6Qn:L=3，实际 {re.findall(r'item=tsv-2 .*?(first_red_by_criterion=\S*)', output)}",
               "看 first_red_by_criterion：两族各自取各自那两行、比整行文字")
        expect(re.search(r"item=tsv-3 .* first_red_by_criterion=J6:L=none;J6Qn:L=3 ", output) is not None,
               f"r2：只有 J6Qn 不同的假变体应当报 J6:L=none、J6Qn:L=3，实际 {re.findall(r'item=tsv-3 .*?(first_red_by_criterion=\S*)', output)}", "看 first_red_by_criterion")
        expect(re.search(r"item=tsv-2 .* first_clause_red=L:3:J6Qn_premise_fails .* first_text_red_criteria=L:2:J6_premise_holds ", output) is not None,
               "r2：J6 只差见证文字时按违例数的首红看不见它（L:3 才红、红的是 J6Qn），按文字的首红在 L:2、点名 J6_premise_holds", "看 CLAUSE_CRITERIA 与 first_text_red_criteria")
        expect(calls and all(line.startswith("--profile r2 ") for line in calls) and any(line.startswith("--profile r2 --list-states ") for line in calls),
               f"r2 整轮里假装置每一次被调都应当带 --profile r2（含 --list-states），实际 {len(calls)} 次、前几次 {calls[:3]!r}", "看 Setup.wrap 与 DEVICE_PROFILE_ARGUMENTS")
        # ② 基线里缺 J6Qn_ 行（装置认不得 r2 档）⇒ 报「装置没带 --profile r2」退 7、变体一个不编、不写产物
        completed, builds = drive(paths("r2-no-rows"), broken=r2_broken, extra={"FAKE_BIN_NO_R2_ROWS": "1"})
        expect(completed.returncode == EXIT_NOT_PROFILE_R2 and "装置没带 --profile r2" in completed.stdout and len(builds) == 1
               and not os.path.exists(os.path.join(temporary, "r2-no-rows.out")),
               f"r2：基线里缺 J6Qn_ 行应当报「装置没带 --profile r2」退 7、变体一个不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_the_profile_rows 与它在 main_run 里的位置")
        # ③ 自造表 Z 五行：--list-items 列全；登记的 Z 行与表对不上时整轮停在 RH1、一次不编
        five_custom = "".join(f"Z{index} 自造第 {index} 条\t{path}\t{old}\t{old[:-1]} // 改\n" for index, (path, old) in enumerate(
            [(LIB_PATH, "let epsilon = 5;"), (RECOVERY_PATH, "let gamma = 3;"), (LIB_PATH, "let delta = 4;"), (LIB_PATH, "let zeta = 6;"), (RECOVERY_PATH, "let alpha = 1;")], 1))
        write(os.path.join(root, CUSTOM_TABLE_PATH), five_custom)
        completed, builds = drive(["--list-items"], broken=r2_broken)
        listed = [line.split("\t")[0] for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 8 and listed[-5:] == ["Z1", "Z2", "Z3", "Z4", "Z5"],
               f"r2：自造表 Z 五行时 --list-items 应当列 8 行、末五行是 Z1–Z5，实际 {len(listed)} 行：{listed}", "看 custom_rows")
        completed, builds = drive(paths("r2-keys"), broken=r2_broken)
        expect(completed.returncode == EXIT_HALT_H1 and "Z 行" in completed.stdout and len(builds) == 0,
               f"r2：登记的 Z 行（假仓里只有 Z1）与自造表的五行对不上应当停在 RH1 退 4、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_the_custom_keys")

        # 九、r3 补的格（登记 r3 9.3 第 6 条的四格，另加几格）。r3 有自己的假仓 root3（假 cargo 是 python，假装置认 --profile r3 --family root_choice）；
        # 外面打开的 R3_BREAK_MODES 原样传进这几趟（文件头「弄坏开关」）。
        r3_broken = ",".join(sorted(BREAK & set(R3_BREAK_MODES)))
        root3 = os.path.join(temporary, "root3")
        os.makedirs(os.path.join(root3, "crates/singlefs-core/src"))
        os.makedirs(os.path.join(root3, "crates/singlefs-replay-prefix/src"))
        recovery3 = "fn choose_root() {\n    let b1 = 1;\n    let b2 = 2;\n}\n"
        library3 = "fn choose_root_slot() {\n    let a1 = 1;\n    let a2 = 2;\n    let a3 = 3;\n    let a4 = 4;\n    let a5 = 5;\n}\n"
        write(os.path.join(root3, RECOVERY_PATH), recovery3)
        write(os.path.join(root3, LIB_PATH), library3)
        write(os.path.join(root3, "Cargo.toml"), "[workspace]\n")
        write(os.path.join(root3, "fake-cargo"), FAKE_CARGO_R3)
        os.chmod(os.path.join(root3, "fake-cargo"), 0o755)
        fake_names = [(1822, "平局取后者赢", ["inferred", 0], LIB_PATH, "let a1 = 1;", "let a1 = 9;"),
                      (1823, "比较不看实例", ["pinned", 18], LIB_PATH, "let a2 = 2;", "let a2 = 9; // FLAG_KJ1 FLAG_KJ2 FLAG_KA_18"),
                      (1824, "取第一个有效者", ["pinned", 52], LIB_PATH, "let a3 = 3;", "let a3 = 9; // FLAG_KA_52"),
                      (1825, "平局取低", ["pinned", 36], LIB_PATH, "let a4 = 4;", "let a4 = 9; // FLAG_KA_36"),
                      (1826, "实例代号为主", ["pinned", 18], LIB_PATH, "let a5 = 5;", "let a5 = 9; // FLAG_KA_18 FLAG_HALT FLAG_OVER"),
                      (1827, "胶水不带实例", ["inferred", 18], RECOVERY_PATH, "let b1 = 1;", "let b1 = 9; // FLAG_KA_18")]

        def table3(name_of=lambda row, name: name, new_of=lambda row, new: new):
            return "# 表头\n" + "".join(f"{name_of(row, name)}\t{path}\t{old}\t{new_of(row, new)}\t-p x\tt\n" for row, name, _pin, path, old, new in fake_names)

        def register3():
            write(os.path.join(root3, "selftest-registered-r3.json"), json.dumps({
                "files_sha256": {RECOVERY_PATH: hashlib.sha256(recovery3.encode()).hexdigest(), LIB_PATH: hashlib.sha256(library3.encode()).hexdigest()},
                "functions": [[LIB_PATH, "choose_root_slot", 1, 7], [RECOVERY_PATH, "choose_root", 1, 4]],
                "names": [[row, name, pin] for row, name, pin, _path, _old, _new in fake_names],
                "segment2_product": "seg2.out"}))

        def paths3(tag):
            return ["--profile", "r3", "--scratch", os.path.join(temporary, "scratch-" + tag), "--output", os.path.join(temporary, tag + ".out"),
                    "--jobs", "2", "--threads", "1"]

        write(os.path.join(root3, TABLE_PATH), table3())
        register3()
        # 第二段产物的样子：不带任何记号的假装置在 --profile r3 --bounds segment1 --family root_choice 上打的 E7RESULT 行
        build_environment = dict(os.environ, FAKE_CARGO_LOG=os.devnull, CARGO_TARGET_DIR=os.path.join(temporary, "target3"))
        subprocess.run([os.path.join(root3, "fake-cargo"), "build"], cwd=root3, env=build_environment, check=True)
        baseline_product = subprocess.run([os.path.join(temporary, "target3/release/e165_replay_bounded_enumeration"), "--profile", "r3", "--bounds", "segment1",
                                           "--family", "root_choice"], env=dict(os.environ, FAKE_BIN_LOG=os.devnull), stdout=subprocess.PIPE, text=True, check=True).stdout
        write(os.path.join(root3, "seg2.out"), baseline_product)
        # ① 名单：--list-items 列 7 行（基线 1、变体 6），变体起名 tsv-<今天的表行号>
        completed, builds = drive(["--profile", "r3", "--list-items"], broken=r3_broken, fake_root=root3)
        listed = [line.split("\t")[0] for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 7 and listed == ["baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "tsv-6", "tsv-7"] and len(builds) == 0,
               f"r3：--list-items 应当列 7 行（基线 1、变体 6）、不编，实际 {len(listed)} 行：{listed}", "看 load_items_r3")
        # ② 整轮：钉值都对得上，退 0；首红按 (M, U) 升序取第一个违例数与基线不同的格，KJ1 在 (1, 2)、KJ2 在 (1, 0)；装置每一次被调都带 --profile r3
        completed, builds = drive(paths3("r3"), broken=r3_broken, fake_root=root3)
        output = read(os.path.join(temporary, "r3.out"))
        calls = [line for line in read(binary_log).split("\n") if line]
        expect(completed.returncode == 0, f"r3 整轮应当退 0，实际 {completed.returncode}：{completed.stdout[-500:]}{completed.stderr[-300:]}", "看 main_run_r3 与 check_the_profile_rows_r3")
        expect(re.search(r"item=tsv-3 .* outcome=red red_legal_domains=K first_red_by_criterion=KJ1:K=M1U2,K9=none,KW=none;KJ2:K=M1U0,K9=none,KW=none;KJ0:K=none,K9=none,KW=none;KJ3:K=none,K9=none,KW=none ", output) is not None,
               f"r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同应当报 KJ1:K=M1U2、KJ2:K=M1U0，实际 {re.findall(r'item=tsv-3 .*?(first_red_by_criterion=\S*)', output)}",
               "看 r3_first_reds：每条判据各取各的格，按 (M, U) 升序")
        expect(re.search(r"item=tsv-4 .* outcome=not_red_within_bounds .* ka_kj2=52 ka_expected=52 ka_pin=match ka_inferred=none ", output) is not None
               and re.search(r"item=tsv-2 .* outcome=not_red_within_bounds .* ka_kj2=0 ka_expected=0 ka_pin=not_pinned ka_inferred=match ", output) is not None,
               "r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1822、1827）不钉，只报 ka_inferred", "看 r3_ka_pin 与 judge_variant_r3")
        expect(re.search(r"item=tsv-6 .* outcome=red_only_by_kj0_or_kj3 red_legal_domains=none first_red_by_criterion=KJ1:K=none,K9=none,KW=none;KJ2:K=none,K9=none,KW=none;KJ0:K=M1U1,K9=none,KW=none;KJ3:.* "
                         r"states_row_differences=K=M1U1:choose_root_returned_none;K9=M1U0:above_chosen_root_over_limit .* above_chosen_root_over_limit=K:0,K9:4,KW:0,KA:0 "
                         r"choose_root_returned_none=K:1,K9:0,KW:0,KA:1 .* halt_conditions=1 .* run_exit=1 run_finished=true ", output) is not None,
               f"r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红，应当照常落行、记 run_exit=1、run_finished=true、outcome=red_only_by_kj0_or_kj3，实际 {[line[:700] for line in output.split(chr(10)) if 'item=tsv-6 ' in line]}",
               "看 judge_variant_r3：装置退 1 但最后一行是 name=done 就是跑完了")
        expect(output.rstrip("\n").split("\n")[-1] == "E7RESULT name=done emitted=11" and "已还原，基线与开头逐字相同" in completed.stdout and len(builds) == 8,
               f"r3 整轮：收尾行应当是 name=done emitted=11、打「已还原，基线与开头逐字相同」、编 8 次（基线 2、变体 6），实际 {output.rstrip(chr(10)).split(chr(10))[-1]!r}、编 {len(builds)} 次", "看 main_run_r3 的收尾")
        expect(calls and all(line.startswith("--profile r3 --bounds segment1 --family root_choice") for line in calls),
               f"r3 整轮里假装置每一次被调都应当带 --profile r3（连同 --bounds segment1 --family root_choice），实际 {len(calls)} 次、前几次 {calls[:3]!r}", "看 Setup.wrap 与 bounds_arguments")
        # ③ KA 上的读数不等于钉值（1825 的假读数 35、钉 36）⇒ ka_pin=mismatch、整轮跑完退 1；弄坏开关 r3kapinunchecked 开着时退 0
        write(os.path.join(root3, TABLE_PATH), table3(new_of=lambda row, new: new.replace("FLAG_KA_36", "FLAG_KA_35")))
        completed, builds = drive(paths3("r3-pin"), broken=r3_broken, fake_root=root3)
        output = read(os.path.join(temporary, "r3-pin.out"))
        expect(completed.returncode == EXIT_INCOMPLETE and re.search(r"item=tsv-5 .* ka_kj2=35 ka_expected=36 ka_pin=mismatch ", output) is not None
               and "ka_pin_mismatches=1 " in output and "KA 钉值对不上" in completed.stdout,
               f"r3：假变体的 KA 读数不等于钉值应当报 ka_pin=mismatch 且整轮退 1（落完行与收尾再退），实际退 {completed.returncode}、{re.findall(r'item=tsv-5 .*?(ka_pin=\S*)', output)}",
               "看 r3_ka_pin 与 main_run_r3 的收尾")
        completed, builds = drive(paths3("r3-pin-broken"), broken=",".join(filter(None, [r3_broken, "r3kapinunchecked"])), fake_root=root3)
        expect(completed.returncode == EXIT_OK, f"弄坏开关 r3kapinunchecked 开着时同一趟应当退 0（证明上一例的退 1 来自核钉值），实际退 {completed.returncode}", "看 BREAK 的 r3kapinunchecked")
        # ④ 基线缺 criteria=root_choice 判决行 ⇒ 报「装置没带 --profile r3」退 7、变体一个不编、不写产物
        write(os.path.join(root3, TABLE_PATH), table3())
        completed, builds = drive(paths3("r3-no-rows"), broken=r3_broken, extra={"FAKE_BIN_NO_R3_ROWS": "1"}, fake_root=root3)
        expect(completed.returncode == EXIT_NOT_PROFILE_R2 and "装置没带 --profile r3" in completed.stdout and len(builds) == 1
               and not os.path.exists(os.path.join(temporary, "r3-no-rows.out")),
               f"r3：基线缺 criteria=root_choice 判决行应当报「装置没带 --profile r3」退 7、变体一个不编、不写产物，实际退 {completed.returncode}、编 {len(builds)} 次",
               "看 check_the_profile_rows_r3 与它在 main_run_r3 里的位置")
        # ⑤ 基线与第二段产物不同 ⇒ 作废条款 TV7，退 6、变体一个不编
        write(os.path.join(root3, "seg2.out"), baseline_product.replace("states=10", "states=11", 1))
        completed, builds = drive(paths3("r3-tv7"), broken=r3_broken, fake_root=root3)
        expect(completed.returncode == EXIT_VOID_V7 and "TV7" in completed.stdout and len(builds) == 1 and not os.path.exists(os.path.join(temporary, "r3-tv7.out")),
               f"r3：基线与第二段产物不逐行相同应当报作废条款 TV7 退 6、变体一个不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_against_segment_two_product")
        # ⑤之二 第二段产物不在（留存产物可以整个删掉）⇒ TV7 的比对本次未跑、逐项点名，照跑完；收尾判决行写 baseline_equals_segment2_product=not_run（开关 retainedmissingred 下报 TV8 退 6）
        os.remove(os.path.join(root3, "seg2.out"))
        completed, builds = drive(paths3("r3-no-seg2"), broken=r3_broken, fake_root=root3)
        r3_no_seg2 = read(os.path.join(temporary, "r3-no-seg2.out"))
        expect(completed.returncode != EXIT_VOID_V7 and "本次未跑：作废条款 TV7 的比对：留存产物" in completed.stdout and "seg2.out 不在" in completed.stdout
               and " baseline_equals_segment2_product=not_run" in r3_no_seg2 and " rows_identical_to_segment2_product=not_run " in r3_no_seg2,
               f"r3：第二段产物不在时 TV7 的比对应当逐项报本次未跑、照跑完、产物写 not_run，实际退 {completed.returncode}：{completed.stdout[-400:]}",
               "看 retained_product_not_run 与 check_against_segment_two_product")
        write(os.path.join(root3, "seg2.out"), baseline_product)
        # ⑥ 选行不满足（变异名在表里改了）⇒ 那一行记 outcome=not_compared、别的行照做、整轮退 1（登记 TH5）
        write(os.path.join(root3, TABLE_PATH), table3(name_of=lambda row, name: "改了名的" + name if row == 1823 else name))
        completed, builds = drive(paths3("r3-th5"), broken=r3_broken, fake_root=root3)
        output = read(os.path.join(temporary, "r3-th5.out"))
        expect(completed.returncode == EXIT_INCOMPLETE and re.search(r"item=tsv-registered-1823 .* outcome=not_compared failed_condition=no_row_with_this_name ", output) is not None
               and "ka_pin=match" in output and len(builds) == 7 and "not_compared=1 " in output,
               f"r3：变异名选不到的那一行应当记 not_compared、别的五行照做（编 7 次：基线 2、变体 5）、整轮退 1，实际退 {completed.returncode}、编 {len(builds)} 次", "看 load_items_r3 与 judge_variant_r3 的 TH5 一支")
        write(os.path.join(root3, TABLE_PATH), table3())
        # ⑦ r3 模式不认 --select-rows、--recheck、--measure 与 --ledger（退 2）；--profile 只认 r2、r3
        completed, builds = drive(["--profile", "r3", "--select-rows", "--output", os.path.join(temporary, "r3-select.out")], broken=r3_broken, fake_root=root3)
        expect(completed.returncode == EXIT_USAGE and len(builds) == 0, f"r3：--select-rows 应当退 2，实际退 {completed.returncode}", "看 main 的 r3 用法判断")
        completed, builds = drive(paths3("r3-ledger") + ["--ledger", os.path.join(temporary, "r3-ledger.ledger")], broken=r3_broken, fake_root=root3)
        expect(completed.returncode == EXIT_USAGE and len(builds) == 0, f"r3：给了 --ledger 应当退 2（红例账不落），实际退 {completed.returncode}", "看 main 的 r3 用法判断")
        completed, builds = drive(["--profile", "r5", "--list-items"], broken=r3_broken, fake_root=root3)
        expect(completed.returncode == EXIT_USAGE, f"--profile r5 应当退 2，实际退 {completed.returncode}", "看 main 的 --profile")
        # ⑧ 登记写死的 r3 数：六个名字（行号 1822–1827）、钉值 18 / 52 / 36 / 18、两个函数、档参数、界参数、TQ5 看的判据次序
        expect([row for row, _name, _pin in R3_REGISTERED["names"]] == [1822, 1823, 1824, 1825, 1826, 1827]
               and [pin for _row, _name, pin in R3_REGISTERED["names"]] == [("inferred", 0), ("pinned", 18), ("pinned", 52), ("pinned", 36), ("pinned", 18), ("inferred", 18)]
               and [name for _path, name, _first, _last in R3_REGISTERED["functions"]] == ["choose_root_slot", "choose_root"]
               and R3_DEVICE_PROFILE_ARGUMENTS == ["--profile", "r3"] and R3_BOUNDS_ARGUMENTS == ["--bounds", "segment1", "--family", "root_choice"]
               and R3_CRITERIA[:2] == ("KJ1", "KJ2") and R3_LEGAL_DOMAINS == ("K", "K9", "KW"),
               "r3 登记写死的：六个变异名（1822–1827）、KA 上的钉值（1823 → 18、1824 → 52、1825 → 36、1826 → 18，1822、1827 是推的 0、18）、两个函数、档与界参数、首红的判据次序",
               "看 R3_REGISTERED、R3_DEVICE_PROFILE_ARGUMENTS、R3_BOUNDS_ARGUMENTS")

        # 十、r4 补的格（登记 r4 9.3 第 6 条的五格，另加几格）。r4 有自己的假仓 root4（假 cargo 是 python，假装置认 --profile r4 --family root_choice_faults）；
        # 外面打开的 R4_BREAK_MODES 原样传进这几趟（文件头「弄坏开关」）。
        r4_broken = ",".join(sorted(BREAK & set(R4_BREAK_MODES)))
        root4 = os.path.join(temporary, "root4")
        os.makedirs(os.path.join(root4, "crates/singlefs-core/src"))
        os.makedirs(os.path.join(root4, "crates/singlefs-replay-prefix/src"))
        recovery4 = "fn choose_root() {\n    let b1 = 1;\n    let b2 = 2;\n}\n"
        library4 = "fn choose_root_slot() {\n    let a1 = 1;\n    let a2 = 2;\n    let a3 = 3;\n    let a4 = 4;\n    let a5 = 5;\n}\n"
        write(os.path.join(root4, RECOVERY_PATH), recovery4)
        write(os.path.join(root4, LIB_PATH), library4)
        write(os.path.join(root4, "Cargo.toml"), "[workspace]\n")
        write(os.path.join(root4, "fake-cargo"), FAKE_CARGO_R4)
        os.chmod(os.path.join(root4, "fake-cargo"), 0o755)
        fake_names4 = [(1821, "平局取后者赢", ["inferred", 0], LIB_PATH, "let a1 = 1;", "let a1 = 9; // FLAG_KG_v1822"),
                       (1822, "比较不看实例", ["pinned", 18], LIB_PATH, "let a2 = 2;", "let a2 = 9; // FLAG_KJ1T FLAG_KJ2 FLAG_KA_18"),
                       (1823, "取第一个有效者", ["pinned", 52], LIB_PATH, "let a3 = 3;", "let a3 = 9; // FLAG_KA_52 FLAG_KG_v1824"),
                       (1824, "平局取低", ["pinned", 36], LIB_PATH, "let a4 = 4;", "let a4 = 9; // FLAG_KA_36"),
                       (1825, "实例代号为主", ["pinned", 18], LIB_PATH, "let a5 = 5;", "let a5 = 9; // FLAG_KA_18 FLAG_HALT FLAG_KG_v1826"),
                       (1826, "胶水不带实例", ["inferred", 18], RECOVERY_PATH, "let b1 = 1;", "let b1 = 9; // FLAG_KA_18")]
        fake_kg_columns = {"today": {"1,0": [0, 0, 0], "1,2": [117, 117, 0]}, "v1822": {"1,0": [0, 0, 0], "1,2": [117, 117, 3]},
                           "v1823": {"1,0": [0, 0, 0], "1,2": [117, 117, 0]}, "v1824": {"1,0": [0, 0, 8], "1,2": [468, 468, 351]},
                           "v1825": {"1,0": [0, 0, 0], "1,2": [117, 117, 0]}, "v1826": {"1,0": [0, 0, 0], "1,2": [200, 200, 5]}}

        def table4(name_of=lambda row, name: name, new_of=lambda row, new: new):
            return "# 表头\n" + "".join(f"{name_of(row, name)}\t{path}\t{old}\t{new_of(row, new)}\t-p x\tt\n" for row, name, _pin, path, old, new in fake_names4)

        def register4():
            write(os.path.join(root4, "selftest-registered-r4.json"), json.dumps({
                "files_sha256": {RECOVERY_PATH: hashlib.sha256(recovery4.encode()).hexdigest(), LIB_PATH: hashlib.sha256(library4.encode()).hexdigest()},
                "functions": [[LIB_PATH, "choose_root_slot", 1, 7], [RECOVERY_PATH, "choose_root", 1, 4]],
                "names": [[row, name, pin] for row, name, pin, _path, _old, _new in fake_names4],
                "labels": ["v1822", "v1823", "v1824", "v1825", "v1826", "v1827"],
                "kg_columns": fake_kg_columns,
                "segment1_product": "seg1.out"}))

        def paths4(tag):
            return ["--profile", "r4", "--scratch", os.path.join(temporary, "scratch-" + tag), "--output", os.path.join(temporary, tag + ".out"),
                    "--jobs", "2", "--threads", "1"]

        write(os.path.join(root4, TABLE_PATH), table4())
        register4()
        # 第一段产物的样子：不带任何记号的假装置在 --profile r4 --bounds segment1 --family root_choice_faults 上打的 E7RESULT 行
        build_environment = dict(os.environ, FAKE_CARGO_LOG=os.devnull, CARGO_TARGET_DIR=os.path.join(temporary, "target4"))
        subprocess.run([os.path.join(root4, "fake-cargo"), "build"], cwd=root4, env=build_environment, check=True)
        baseline_product4 = subprocess.run([os.path.join(temporary, "target4/release/e165_replay_bounded_enumeration"), "--profile", "r4", "--bounds", "segment1",
                                            "--family", "root_choice_faults"], env=dict(os.environ, FAKE_BIN_LOG=os.devnull), stdout=subprocess.PIPE, text=True, check=True).stdout
        write(os.path.join(root4, "seg1.out"), baseline_product4)
        # ① 名单：--list-items 列 7 行（基线 1、变体 6），变体起名 tsv-<今天的表行号>
        completed, builds = drive(["--profile", "r4", "--list-items"], broken=r4_broken, fake_root=root4)
        listed = [line.split("\t")[0] for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 7 and listed == ["baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "tsv-6", "tsv-7"] and len(builds) == 0,
               f"r4：--list-items 应当列 7 行（基线 1、变体 6）、不编，实际 {len(listed)} 行：{listed}", "看 load_items_r4")
        # ② 整轮：钉值都对得上，退 0；首红按 (M, U, 被判时故障, 历史标签) 升序取第一个违例数与基线不同的子格，KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着)；装置每一次被调都带 --profile r4
        completed, builds = drive(paths4("r4"), broken=r4_broken, fake_root=root4)
        output = read(os.path.join(temporary, "r4.out"))
        calls = [line for line in read(binary_log).split("\n") if line]
        expect(completed.returncode == 0, f"r4 整轮应当退 0，实际 {completed.returncode}：{completed.stdout[-500:]}{completed.stderr[-300:]}", "看 main_run_r4 与 check_the_profile_rows_r4")
        none_text = "none,KF3=none,KF9=none,KFW=none"
        expect(re.search(r"item=tsv-3 .* outcome=red variant=v1823 red_legal_domains=KF differs_on=not_applicable first_red_by_criterion=KJ1:KF=" + none_text
                         + r";KJ1T:KF=M1U2/none/inside,KF3=none,KF9=none,KFW=none;KJ2:KF=M1U0/RN/inside,KF3=none,KF9=none,KFW=none;KJ0:KF=" + none_text, output) is not None,
               f"r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同应当报 KJ1T:KF=M1U2/none/inside、KJ2:KF=M1U0/RN/inside，实际 {re.findall(r'item=tsv-3 .*?(first_red_by_criterion=\S*)', output)}",
               "看 r4_first_reds：每条判据各取各的子格，按 (M, U, 被判时故障, 历史标签) 升序")
        expect(re.search(r"item=tsv-2 .* outcome=not_red_within_bounds variant=v1822 red_legal_domains=none differs_on=KG .* kg_pin=match ", output) is not None,
               f"r4：假变体只在 KG 与基线不同（读数换成它自己那一列）应当记 not_red_within_bounds、differs_on=KG、kg_pin=match，实际 {[line[:600] for line in output.split(chr(10)) if 'item=tsv-2 ' in line]}",
               "看 r4_differs_on 与 judge_variant_r4 的结局分类")
        expect(re.search(r"item=tsv-4 .* outcome=not_red_within_bounds variant=v1824 .* differs_on=KG\+KA .* ka_kj2=52 ka_expected=52 ka_pin=match ka_inferred=none kg_pin=match ", output) is not None
               and re.search(r"item=tsv-5 .* outcome=not_red_within_bounds variant=v1825 .* differs_on=KA .* ka_kj2=36 ka_expected=36 ka_pin=match ka_inferred=none kg_pin=match ", output) is not None
               and re.search(r"item=tsv-7 .* ka_kj2=18 ka_expected=18 ka_pin=not_pinned ka_inferred=match kg_pin=match ", output) is not None,
               "r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1827 不钉，只报 ka_inferred（v1827 的 KG 取 v1823 那一列）", "看 r4_ka_pin、r4_kg_pin 与 judge_variant_r4")
        expect(re.search(r"item=tsv-6 .* outcome=red_only_by_kj0_or_kj3 variant=v1826 red_legal_domains=none .* first_red_by_criterion=KJ1:KF=none,KF3=none,KF9=none,KFW=none;"
                         r"KJ1T:KF=none,KF3=none,KF9=none,KFW=none;KJ2:KF=none,KF3=none,KF9=none,KFW=none;KJ0:KF=M1U1/none/inside,.* "
                         r"choose_root_returned_none_with_a_readable_root=KF:1,.* halt_conditions=1 .* run_exit=1 run_finished=true ", output) is not None,
               f"r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红，应当照常落行、记 run_exit=1、run_finished=true、outcome=red_only_by_kj0_or_kj3，实际 {[line[:700] for line in output.split(chr(10)) if 'item=tsv-6 ' in line]}",
               "看 judge_variant_r4：装置退 1 但最后一行是 name=done 就是跑完了")
        expect(output.rstrip("\n").split("\n")[-1] == "E7RESULT name=done emitted=11" and "已还原，基线与开头逐字相同" in completed.stdout and len(builds) == 8,
               f"r4 整轮：收尾行应当是 name=done emitted=11、打「已还原，基线与开头逐字相同」、编 8 次（基线 2、变体 6），实际 {output.rstrip(chr(10)).split(chr(10))[-1]!r}、编 {len(builds)} 次", "看 main_run_r4 的收尾")
        expect(calls and all(line.startswith("--profile r4 --bounds segment1 --family root_choice_faults") for line in calls),
               f"r4 整轮里假装置每一次被调都应当带 --profile r4（连同 --bounds segment1 --family root_choice_faults），实际 {len(calls)} 次、前几次 {calls[:3]!r}", "看 Setup.wrap 与 bounds_arguments")
        # ③ 变体在 KG 上的读数不等于它那一列（FLAG_KG_BAD）⇒ kg_pin=mismatch、整轮跑完退 1；弄坏开关 r4kgpinunchecked 开着时退 0
        write(os.path.join(root4, TABLE_PATH), table4(new_of=lambda row, new: new.replace("FLAG_KA_36", "FLAG_KA_36 FLAG_KG_BAD")))
        completed, builds = drive(paths4("r4-pin"), broken=r4_broken, fake_root=root4)
        output = read(os.path.join(temporary, "r4-pin.out"))
        expect(completed.returncode == EXIT_INCOMPLETE and re.search(r"item=tsv-5 .* kg_pin=mismatch kg_mismatched_cells=M1U0:1/0/0!=0/0/0;M1U2:118/117/0!=117/117/0 ", output) is not None
               and "kg_pin_mismatches=1 " in output and "KG 钉值对不上" in completed.stdout,
               f"r4：假变体的 KG 读数不等于它那一列应当报 kg_pin=mismatch 且整轮退 1（落完行与收尾再退），实际退 {completed.returncode}、{re.findall(r'item=tsv-5 .*?(kg_pin=\S* kg_mismatched_cells=\S*)', output)}",
               "看 r4_kg_pin 与 main_run_r4 的收尾")
        completed, builds = drive(paths4("r4-pin-broken"), broken=",".join(filter(None, [r4_broken, "r4kgpinunchecked"])), fake_root=root4)
        expect(completed.returncode == EXIT_OK, f"弄坏开关 r4kgpinunchecked 开着时同一趟应当退 0（证明上一例的退 1 来自核 KG 钉值），实际退 {completed.returncode}", "看 BREAK 的 r4kgpinunchecked")
        # ④ 基线缺 criteria=root_choice_faults 判决行 ⇒ 报「装置没带 --profile r4」退 7、变体一个不编、不写产物
        write(os.path.join(root4, TABLE_PATH), table4())
        completed, builds = drive(paths4("r4-no-rows"), broken=r4_broken, extra={"FAKE_BIN_NO_R4_ROWS": "1"}, fake_root=root4)
        expect(completed.returncode == EXIT_NOT_PROFILE_R2 and "装置没带 --profile r4" in completed.stdout and len(builds) == 1
               and not os.path.exists(os.path.join(temporary, "r4-no-rows.out")),
               f"r4：基线缺 criteria=root_choice_faults 判决行应当报「装置没带 --profile r4」退 7、变体一个不编、不写产物，实际退 {completed.returncode}、编 {len(builds)} 次",
               "看 check_the_profile_rows_r4 与它在 main_run_r4 里的位置")
        # ⑤ 基线与第一段产物不同 ⇒ 作废条款 TV7，退 6、变体一个不编
        write(os.path.join(root4, "seg1.out"), baseline_product4.replace("states=10", "states=11", 1))
        completed, builds = drive(paths4("r4-tv7"), broken=r4_broken, fake_root=root4)
        expect(completed.returncode == EXIT_VOID_V7 and "TV7" in completed.stdout and len(builds) == 1 and not os.path.exists(os.path.join(temporary, "r4-tv7.out")),
               f"r4：基线与第一段产物不逐行相同应当报作废条款 TV7 退 6、变体一个不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_against_segment_one_product")
        # ⑤之二 第一段产物不在（留存产物可以整个删掉）⇒ TV7 的比对本次未跑、逐项点名，照跑完；收尾判决行写 baseline_equals_segment1_product=not_run（开关 retainedmissingred 下报 TV8 退 6）
        os.remove(os.path.join(root4, "seg1.out"))
        completed, builds = drive(paths4("r4-no-seg1"), broken=r4_broken, fake_root=root4)
        r4_no_seg1 = read(os.path.join(temporary, "r4-no-seg1.out"))
        expect(completed.returncode != EXIT_VOID_V7 and "本次未跑：作废条款 TV7 的比对：留存产物" in completed.stdout and "seg1.out 不在" in completed.stdout
               and " baseline_equals_segment1_product=not_run" in r4_no_seg1 and " rows_identical_to_segment1_product=not_run " in r4_no_seg1,
               f"r4：第一段产物不在时 TV7 的比对应当逐项报本次未跑、照跑完、产物写 not_run，实际退 {completed.returncode}：{completed.stdout[-400:]}",
               "看 retained_product_not_run 与 check_against_segment_one_product")
        write(os.path.join(root4, "seg1.out"), baseline_product4)
        # ⑥ 选行不满足（变异名在表里改了）⇒ 那一行记 outcome=not_compared、别的行照做、整轮退 1（登记 TH5）
        write(os.path.join(root4, TABLE_PATH), table4(name_of=lambda row, name: "改了名的" + name if row == 1822 else name))
        completed, builds = drive(paths4("r4-th5"), broken=r4_broken, fake_root=root4)
        output = read(os.path.join(temporary, "r4-th5.out"))
        expect(completed.returncode == EXIT_INCOMPLETE and re.search(r"item=tsv-registered-1822 .* outcome=not_compared failed_condition=no_row_with_this_name ", output) is not None
               and "kg_pin=match" in output and len(builds) == 7 and "not_compared=1 " in output,
               f"r4：变异名选不到的那一行应当记 not_compared、别的五行照做（编 7 次：基线 2、变体 5）、整轮退 1，实际退 {completed.returncode}、编 {len(builds)} 次", "看 load_items_r4 与 judge_variant_r4 的 TH5 一支")
        write(os.path.join(root4, TABLE_PATH), table4())
        # ⑦ r4 模式不认 --select-rows、--recheck、--measure 与 --ledger（退 2）；--profile 只认 r2、r3、r4
        completed, builds = drive(["--profile", "r4", "--select-rows", "--output", os.path.join(temporary, "r4-select.out")], broken=r4_broken, fake_root=root4)
        expect(completed.returncode == EXIT_USAGE and len(builds) == 0, f"r4：--select-rows 应当退 2，实际退 {completed.returncode}", "看 main 的 r4 用法判断")
        completed, builds = drive(paths4("r4-ledger") + ["--ledger", os.path.join(temporary, "r4-ledger.ledger")], broken=r4_broken, fake_root=root4)
        expect(completed.returncode == EXIT_USAGE and len(builds) == 0, f"r4：给了 --ledger 应当退 2（红例账不落），实际退 {completed.returncode}", "看 main 的 r4 用法判断")
        completed, builds = drive(["--profile", "r5", "--list-items"], broken=r4_broken, fake_root=root4)
        expect(completed.returncode == EXIT_USAGE, f"--profile r5 应当退 2，实际退 {completed.returncode}", "看 main 的 --profile")
        # ⑧ 登记写死的 r4 数：六个名字（行号 1821–1826）、钉值 18 / 52 / 36 / 18、两个函数、档参数、界参数、TQ9 看的判据次序、KG 钉值表八个格
        expect([row for row, _name, _pin in R4_REGISTERED["names"]] == [1821, 1822, 1823, 1824, 1825, 1826]
               and [pin for _row, _name, pin in R4_REGISTERED["names"]] == [("inferred", 0), ("pinned", 18), ("pinned", 52), ("pinned", 36), ("pinned", 18), ("inferred", 18)]
               and R4_REGISTERED["labels"] == ["v1822", "v1823", "v1824", "v1825", "v1826", "v1827"]
               and [name for _path, name, _first, _last in R4_REGISTERED["functions"]] == ["choose_root_slot", "choose_root"]
               and R4_DEVICE_PROFILE_ARGUMENTS == ["--profile", "r4"] and R4_BOUNDS_ARGUMENTS == ["--bounds", "segment1", "--family", "root_choice_faults"]
               and R4_CRITERIA[:3] == ("KJ1", "KJ1T", "KJ2") and R4_LEGAL_DOMAINS == ("KF", "KF3", "KF9", "KFW")
               and sorted(R4_REGISTERED["kg_columns"]) == ["today", "v1822", "v1823", "v1824", "v1825", "v1826"]
               and all(len(column) == 8 for column in R4_REGISTERED["kg_columns"].values())
               and R4_REGISTERED["kg_columns"]["v1826"]["2,3"] == [309609, 346005, 49680] and R4_REGISTERED["kg_columns"]["today"]["2,3"] == [265923, 284607, 0],
               "r4 登记写死的：六个变异名（今天的第 1821–1826 行，登记名 v1822–v1827）、KA 上的钉值（v1823 → 18、v1824 → 52、v1825 → 36、v1826 → 18，v1822、v1827 是推的 0、18）、"
               "两个函数、档与界参数、首红的判据次序、KG 上六种本地择根序每格的钉值（KG M = 2、U = 3 一格 v1826 是 309 609 / 346 005 / 49 680、今天的序是 265 923 / 284 607 / 0）",
               "看 R4_REGISTERED、R4_DEVICE_PROFILE_ARGUMENTS、R4_BOUNDS_ARGUMENTS、R4_KG_RULE_READINGS")

        # 十一、r5 补的格（登记 r5 9.3 第 3 条、修1.九乙第 4 条）。三个假仓的源文件都等于原来那套假钉值；r5 那一套（selftest-registered*-r5.json）的两份源文件 sha256 写成全 0，
        # 所以「给了 --pins r5」必须停在 RH1、一次不编，「不给」照常跑（上面几节）。弄坏开关 r5pinsignored 外面打开时这三格红；每格后面自己打开它跑一次对照。
        def r5_variant_of(path, **changes):
            with open(path, encoding="utf-8") as handle:
                registration = json.load(handle)
            registration.update(changes)
            return json.dumps(registration)

        zero_hashes = {RECOVERY_PATH: "0" * 64, LIB_PATH: "0" * 64}
        write(os.path.join(root, TABLE_PATH), r2_table)
        write(os.path.join(root, CUSTOM_TABLE_PATH), good_custom)
        register(names=[[2, "J6 二 J6Qn 三", []], [3, "只有 J6Qn 三", []]])
        write(os.path.join(root, "selftest-registered-r5.json"), r5_variant_of(os.path.join(root, "selftest-registered.json"), files_sha256=zero_hashes))
        write(os.path.join(root3, "selftest-registered-r3-r5.json"), r5_variant_of(os.path.join(root3, "selftest-registered-r3.json"), files_sha256=zero_hashes))
        write(os.path.join(root4, "selftest-registered-r4-r5.json"), r5_variant_of(os.path.join(root4, "selftest-registered-r4.json"), files_sha256=zero_hashes))
        for mode, mode_paths, mode_root in (("r2", paths("r5-pins-r2"), root), ("r3", paths3("r5-pins-r3"), root3), ("r4", paths4("r5-pins-r4"), root4)):
            output_path = mode_paths[mode_paths.index("--output") + 1]
            completed, builds = drive(mode_paths + ["--pins", "r5"], fake_root=mode_root)
            expect(completed.returncode == EXIT_HALT_H1 and "停机条款 RH1" in completed.stdout and len(builds) == 0 and not os.path.exists(output_path),
                   f"r5：给了 --pins r5 应当按 r5 钉值核对、在旧钉值的假仓上停在 RH1（{mode} 模式：退 4、一次不编、不写产物），实际退 {completed.returncode}、编 {len(builds)} 次",
                   "看 Setup：--pins r5 要选 R5_PINS（自证里是 selftest-registered*-r5.json）")
            completed, builds = drive([word.replace("r5-pins-", "r5-pins-ignored-") for word in mode_paths] + ["--pins", "r5"], broken="r5pinsignored", fake_root=mode_root)
            expect(completed.returncode != EXIT_HALT_H1 and len(builds) > 0,
                   f"弄坏开关 r5pinsignored 开着时同一趟（{mode} 模式）应当照旧钉值跑过开工核对、编得到（证明上一例的 RH1 来自选了 r5 那一套），实际退 {completed.returncode}、编 {len(builds)} 次",
                   "看 BREAK 的 r5pinsignored")
        completed, builds = drive(["--profile", "r4", "--pins", "r6", "--list-items"], fake_root=root4)
        expect(completed.returncode == EXIT_USAGE, f"r5：--pins 只认 r5，给 r6 应当退 2，实际退 {completed.returncode}", "看 main 的 --pins")
        # --gate r2：只在 --profile r4 --pins r5 同给时认，别的组合退 2、一次不编
        for tag, arguments in (("no-pins", paths4("r5-gate-no-pins") + ["--gate", "r2"]),
                               ("r3", paths3("r5-gate-r3") + ["--pins", "r5", "--gate", "r2"]),
                               ("r2", paths("r5-gate-r2") + ["--pins", "r5", "--gate", "r2"]),
                               ("r1", paths4("r5-gate-r1") + ["--pins", "r5", "--gate", "r1"])):
            completed, builds = drive(arguments, fake_root=root3 if tag == "r3" else root if tag == "r2" else root4)
            expect(completed.returncode == EXIT_USAGE and len(builds) == 0,
                   f"r5：--gate r2 只在 --profile r4 --pins r5 同给时认（{tag} 那一种组合应当退 2、一次不编），实际退 {completed.returncode}、编 {len(builds)} 次", "看 main 的 --gate 判断")
        # --gate r2 那一套的变异表 sha256 与假仓里的表对不上 ⇒ 停在 RH1、一次不编
        write(os.path.join(root4, "selftest-registered-r4-r5-gate-r2.json"),
              r5_variant_of(os.path.join(root4, "selftest-registered-r4.json"), table_sha256="0" * 64, segment1_product="seg5.out"))
        completed, builds = drive(paths4("r5-gate-table-mismatch") + ["--pins", "r5", "--gate", "r2"], fake_root=root4)
        expect(completed.returncode == EXIT_HALT_H1 and f"{TABLE_PATH} 的 sha256 不等于登记的 {'0' * 64}" in completed.stdout and len(builds) == 0,
               f"r5：--gate r2 那一套的变异表钉值与仓里的表对不上时应当停在 RH1、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_the_registered_inputs 的变异表一支")
        # --gate r2 整轮：假装置每一次被调都带 --gate r2、基线 config 行带 gate=r2、TV7 比 5-甲 那份产物（seg5.out）；产物每个 kill_bound 行与收尾判决行带 gate=r2
        write(os.path.join(root4, "selftest-registered-r4-r5-gate-r2.json"),
              r5_variant_of(os.path.join(root4, "selftest-registered-r4.json"), segment1_product="seg5.out"))
        os.makedirs(os.path.join(root4, os.path.dirname(DEVICE_SOURCE_PATH)), exist_ok=True)
        write(os.path.join(root4, DEVICE_SOURCE_PATH), "// 自检用的假装置源码\n")
        device_sha256 = hashlib.sha256("// 自检用的假装置源码\n".encode()).hexdigest()
        gate_product = subprocess.run([os.path.join(temporary, "target4/release/e165_replay_bounded_enumeration"), "--profile", "r4", "--bounds", "segment1",
                                       "--family", "root_choice_faults", "--gate", "r2"], env=dict(os.environ, FAKE_BIN_LOG=os.devnull), stdout=subprocess.PIPE, text=True, check=True).stdout
        write(os.path.join(root4, "seg5.out"), gate_product)
        completed, builds = drive(paths4("r5-gate") + ["--pins", "r5", "--gate", "r2"], fake_root=root4)
        output = read(os.path.join(temporary, "r5-gate.out"))
        calls = [line for line in read(binary_log).split("\n") if line]
        kill_lines = [line for line in output.split("\n") if line.startswith("E7RESULT name=kill_bound ")]
        verdict_lines = [line for line in output.split("\n") if line.startswith("E7RESULT name=verdict ")]
        config_lines = [line for line in output.split("\n") if line.startswith("E7RESULT name=config ")]
        expect(completed.returncode == EXIT_OK and len(builds) == 8 and calls
               and all(line.startswith("--profile r4 --bounds segment1 --family root_choice_faults --gate r2") for line in calls)
               and len(kill_lines) == 7 and all(" gate=r2 " in line for line in kill_lines)
               and len(verdict_lines) == 1 and verdict_lines[0].endswith(" gate=r2")
               and len(config_lines) == 1 and f" pins=r5 pins_registration=research/prompts/e165-r5-prereg.md gate=r2 device_sha256={device_sha256}" in config_lines[0]
               and "device_arguments=--profile+r4+--bounds+segment1+--family+root_choice_faults+--gate+r2 " in config_lines[0],
               f"r5：--gate r2 整轮应当退 0、假装置每一次被调都带 --gate r2、产物带 gate=r2（kill_bound 7 行、判决行、config 行带装置 sha256），实际退 {completed.returncode}、编 {len(builds)} 次、"
               f"前几次调用 {calls[:2]!r}、kill_bound {len(kill_lines)} 行、{completed.stdout[-300:]}",
               "看 device_bounds_arguments、check_the_gate_rows_r4、pins_config_text 与 judge_variant_r4 的 head")
        completed, builds = drive(paths4("r5-gate-ignored") + ["--pins", "r5", "--gate", "r2"], broken="r4gateignored", fake_root=root4)
        expect(completed.returncode == EXIT_NOT_PROFILE_R2 and "装置没带 --gate r2" in completed.stdout and len(builds) == 1
               and not os.path.exists(os.path.join(temporary, "r5-gate-ignored.out")),
               f"弄坏开关 r4gateignored 开着时同一趟应当报「装置没带 --gate r2」退 7、变体一个不编、不写产物（基线缺 R2 的闸字段），实际退 {completed.returncode}、编 {len(builds)} 次",
               "看 BREAK 的 r4gateignored 与 check_the_gate_rows_r4")
        # 登记写死的 r5 数：钉值取第一段根上的（9.3 第 1 条），名字、标签、KA / KG 钉值、custom_keys 同旧的；原来三套一个字不改（9.3 第 2 条）
        expect([row for row, _name, _tags in R5_PINS["r2"]["names"]] == [23, 24, 36, 224, 347, 348, 399, 407, 441, 505, 506, 507, 508, 509, 510, 511, 604, 605, 908, 909, 910, 990, 991]
               and [(name, tags) for _row, name, tags in R5_PINS["r2"]["names"]] == [(name, tags) for _row, name, tags in REGISTERED["names"]]
               and [(path, name) for path, name, _first, _last in R5_PINS["r2"]["functions"]] == [(path, name) for path, name, _first, _last in REGISTERED["functions"]]
               and R5_PINS["r2"]["functions"][-1] == (RECOVERY_PATH, "replay_journal", 3467, 3614) and R5_PINS["r2"]["custom_keys"] == ["Z1", "Z2", "Z3", "Z4", "Z5"]
               and R5_PINS["r2"]["segment1_products"] == ["research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-legal-family.out",
                                                          "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-adversarial-family.out"]
               and all(R5_PINS[mode]["files_sha256"][RECOVERY_PATH].startswith("4454f6d3") and R5_PINS[mode]["files_sha256"][LIB_PATH].startswith("9966f42f")
                       and R5_PINS[mode]["table_sha256"].startswith("bc2627f8") for mode in ("r2", "r3", "r4"))
               and [row for row, _name, _pin in R5_PINS["r3"]["names"]] == [1810, 1811, 1812, 1813, 1814, 1815] == [row for row, _name, _pin in R5_PINS["r4"]["names"]]
               and [(name, pin) for _row, name, pin in R5_PINS["r3"]["names"]] == [(name, pin) for _row, name, pin in R3_REGISTERED["names"]]
               and [(name, pin) for _row, name, pin in R5_PINS["r4"]["names"]] == [(name, pin) for _row, name, pin in R4_REGISTERED["names"]]
               and R5_PINS["r4"]["labels"] == R4_REGISTERED["labels"] and R5_PINS["r4"]["kg_columns"] == R4_REGISTERED["kg_columns"]
               and R5_PINS["r3"]["functions"] == R5_PINS["r4"]["functions"] == [(LIB_PATH, "choose_root_slot", 3980, 4019), (RECOVERY_PATH, "choose_root", 1268, 1305)]
               and R5_PINS["r3"]["segment2_product"] == "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-root-choice.out"
               and R5_PINS["r4"]["segment1_product"] == "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r4-root-choice-faults.out"
               and R5_GATE_R2_PINS["table_sha256"] == "c9a03eb4af85a8e4c7206dfa3609593390516b80ca52be7090aced5696e84046" != R5_TABLE_SHA256
               and R5_GATE_R2_PINS["segment1_product"] == "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults.out"
               and R5_GATE_R2_PINS["files_sha256"] == R5_FILES_SHA256 and R5_GATE_R2_PINS["functions"] == R5_PINS["r4"]["functions"] and R5_GATE_R2_PINS["names"] == R5_PINS["r4"]["names"]
               and R4_GATE_R2_ARGUMENTS == ["--gate", "r2"]
               and R4_GATE_R2_CONFIG_FIELD == ("witness_gate", "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots")
               and REGISTERED["files_sha256"][RECOVERY_PATH].startswith("3d79b99e") and R3_REGISTERED["files_sha256"][RECOVERY_PATH].startswith("9337dc1b")
               and R4_REGISTERED["files_sha256"][RECOVERY_PATH].startswith("264664af") and R4_REGISTERED["table_sha256"].startswith("db2981d0"),
               "r5 登记写死的：三个模式的 r5 钉值（第一段根上的 recovery.rs 4454f6d3…、lib.rs 9966f42f…、表 bc2627f8…，r2 的 23 个名字在根里第 23–991 行、九个函数，"
               "r3 / r4 的六个名字在第 1810–1815 行、两个函数，比对对象是第一段的重跑产物）、名字与钉值同旧的、--gate r2 那一套（5-甲 根上的表 c9a03eb4…、5-甲 产物名、装置 config 行的 witness_gate=r2_… 键值、两个源文件与函数区间同 r4 那一套）、原来三套的哈希没动",
               "看 R5_PINS、R5_GATE_R2_PINS、R4_GATE_R2_ARGUMENTS")

    for what, fix in failures:
        print(f"  ✗ 自检：{what}")
        print(f"  → 怎么办：{fix}")
    print(f"自检：查了 {checked} 项，{len(failures)} 项不过")
    return EXIT_INCOMPLETE if failures else EXIT_OK
