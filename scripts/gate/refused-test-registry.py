#!/usr/bin/env python3
# admission: always 判的是此刻验证包的测试函数与拒算登记表，两边有一边改了都要现判
# run-condition: none 平常跑起 admission.py 的 test-functions-request 与 test-plan（只算键、查判定库，不编不跑测试），连不上判定库服务或算不出按本次未跑退 77；自证只读造出来的计划输出
"""拒算的测试函数必须登记：测试函数的单元键算不出（闭包读活树、读运行时拼出的仓根路径这类），按条复用那一趟照跑、不复用、不入库，
登记表 .claude/gate.d/refused-tests.tsv 一行一条这样的测试函数，写为什么非读活树不可。拒算的测试函数不在表里判红；表里登了而已经不拒算的判红（只缩不涨）。
门禁 code-source-discipline 的 refused-tests 格调它的 check；合入后验证之后主 agent 调它的 list，逐条判新出现的拒算该不该拒。

用法：
    refused-test-registry.py check [仓根] [--plan <文件>] [--admission <脚本>]
                                        核登记表与此刻的拒算清单逐条对上；核过打一句成功句（拒算几条、表几行）；有问题逐处列、退 1；
                                        清单取不到或不完整退 77（本次未跑，不是通过）
    refused-test-registry.py list [仓根] [--plan <文件>] [--admission <脚本>]
                                        打此刻的拒算清单，每条一行「已登记 / 未登记<TAB>测试函数全名<TAB>机器给的拒算理由」，另列表里已不拒算的行；不判红，退 0（取不到清单退 77）
    refused-test-registry.py --selftest   拿造出来的计划输出核每一条判法；REFUSED_TEST_REGISTRY_BREAK=<项> 时点名的格必须红

拒算清单的取法（仓根默认是这份脚本所在的仓）：照 .claude/gate.d/crash-point-replay.sh 的 crash-case-replay 格（cargo test --release -p singlefs-verification --lib --tests
包里的全部测试函数）同一套起法，python3 scripts/gate/admission.py test-functions-request crash-point-replay.sh <这份脚本所在的仓> 打出判定种类与交给 test-plan 的参数，再
python3 scripts/gate/admission.py test-plan <仓根> <种类> singlefs-verification <计划文件> <那些参数> --exclude-registered；它标准输出里「<名字><TAB>refused<TAB>…<TAB>理由」每行是一条拒算
（名字是 test-plan 第一列：测试目标根文件与限定名，同一个函数编进几个目标的另接 @<目标号>）。登记的崩溃枚举用例与放量分项按崩溃状态块判，不进这一份清单。
--plan 给的文件当 test-plan 的标准输出读（自证与门禁样本用）；--admission 换 admission.py（自证用假的）。
完整性：test-plan 要退 0 并打末行「PLAN judge=… reuse=… refused=R …」，R 要等于逐条列出的 refused 行数；退非 0（含 75 判定库服务不可用）、没有 PLAN 行、条数对不上，整份清单作废，退 77。
覆盖：只有验证包不带特性那一趟的测试函数；挂特性的测试文件（feature-gated-tests 格）、产品 GPU 包（product-gpu-real-card-tests 格）与别的包里的 #[test] 的拒算这里不取。

登记表写法在它自己的表头：一行「测试函数全名<TAB>为什么非读活树不可」，再加可选的一段 # 说明。理由至少 8 个字；同一个测试函数不登两行。
每加一行由主 agent 二次确认之后登记，实现员不登记（.claude/main-agent.md「合入后验证之后判新出现的拒算」）。

退出码：0 核过（list：列完）；1 有问题（逐处列）；2 用法错；77 拒算清单取不到或不完整，本次未跑。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是 selftest() 里那一格，自证打「✗ 自证 <格名>：…」）：
  REFUSED_TEST_REGISTRY_BREAK=unregistered-refusals-accepted  拒算的测试函数不在表里不判红
      「✗ 自证 refusal-without-row：」
  REFUSED_TEST_REGISTRY_BREAK=stale-rows-accepted  表里登了而已经不拒算的行不判红
      「✗ 自证 row-without-refusal：」
  REFUSED_TEST_REGISTRY_BREAK=reason-length-unchecked  不核理由的字数
      「✗ 自证 reason-too-short：」
  REFUSED_TEST_REGISTRY_BREAK=duplicate-rows-accepted  同一个测试函数登两行不判红
      「✗ 自证 duplicate-row：」
  REFUSED_TEST_REGISTRY_BREAK=count-gate-ignored  不核 PLAN 行报的拒算条数与逐条列出的 refused 行数是不是相等
      「✗ 自证 plan-count-mismatch：」
  REFUSED_TEST_REGISTRY_BREAK=unavailable-as-green  清单取不到、不完整当核过
      「✗ 自证 plan-count-mismatch：」、「✗ 自证 plan-without-summary：」、「✗ 自证 admission-failed：」、「✗ 自证 request-unreadable：」
"""
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("REFUSED_TEST_REGISTRY_BREAK", "")
REGISTRY_TABLE = os.path.join(".claude", "gate.d", "refused-tests.tsv")
ADMISSION_SCRIPT = os.path.join(REPOSITORY_ROOT, "scripts", "gate", "admission.py")
REQUEST_STAGE_NAME = "crash-point-replay.sh"   # test-functions-request 的阶段文件名：和 crash-case-replay 格同一个，判定种类随它
PLANNED_PACKAGE = "singlefs-verification"
COMMAND_TIMEOUT_SECONDS = 3600   # 推的：真仓上没量过全程
MINIMUM_REASON_CHARACTERS = 8
PLAN_COLUMNS = 7   # test-plan 逐条行：名字、决策、键、目标号、目标根文件、libtest 全名、原因
REFUSED_DECISION = "refused"
PLAN_SUMMARY = re.compile(r"^PLAN .*\brefused=(\d+)\b")
NOT_RUN_EXIT = 77


def broken(switch):
    return switch in BROKEN.split(",")


def read_registry(root):
    """→ ({测试函数: (行号, 理由)}, 问题)。"""
    rows, found = {}, []
    path = os.path.join(root, REGISTRY_TABLE)
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError as error:
        return {}, [f"读不了登记表 {REGISTRY_TABLE}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        where = f"{REGISTRY_TABLE}:{number}"
        fields = [field.strip() for field in line.split("\t#", 1)[0].split("\t")]
        if len(fields) != 2 or not fields[0]:
            found.append(f"{where} 是 {len(fields)} 段，要两段（测试函数全名、为什么非读活树不可，制表符分隔）")
            continue
        name, reason = fields
        if name in rows and not broken("duplicate-rows-accepted"):
            found.append(f"{where} {name} 登记了两次（第 {rows[name][0]} 行已经有）")
            continue
        rows.setdefault(name, (number, reason))
        if len(reason) < MINIMUM_REASON_CHARACTERS and not broken("reason-length-unchecked"):
            found.append(f"{where} {name} 的「为什么非读活树不可」只有 {len(reason)} 个字，至少 {MINIMUM_REASON_CHARACTERS} 个字")
    return rows, found


def parse_plan(text):
    """test-plan 的标准输出 → (拒算行 [(测试函数, 机器给的理由)], PLAN 行报的拒算条数或 None)。"""
    refused, declared = [], None
    for line in text.splitlines():
        fields = line.split("\t")
        if len(fields) >= PLAN_COLUMNS and fields[1] == REFUSED_DECISION:
            refused.append((fields[0], fields[PLAN_COLUMNS - 1].removeprefix("拒算：")))
            continue
        counted = PLAN_SUMMARY.match(line)
        if counted is not None:
            declared = int(counted.group(1))
    return refused, declared


def plan_problems(text):
    """清单作废的理由（空表示完整）。"""
    refused, declared = parse_plan(text)
    if declared is None:
        return ["输出里没有末行「PLAN judge=… reuse=… refused=R …」：test-plan 没打完"]
    if declared != len(refused) and not broken("count-gate-ignored"):
        return [f"PLAN 行报拒算 {declared} 条，逐条行列出 {len(refused)} 条"]
    return []


def run_command(command):
    """→ (退出码, 标准输出, 标准错误) 或 (None, 理由, "")。每条命令放进它自己的进程组，超时对整组发 KILL。"""
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    except OSError as error:
        return None, f"起不了 {' '.join(command)}：{error}", ""
    try:
        output, error_output = process.communicate(timeout=COMMAND_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate()
        return None, f"{' '.join(command)} 超过 {COMMAND_TIMEOUT_SECONDS} 秒没跑完，已停掉", ""
    return process.returncode, output, error_output


def tail_of(output, error_output):
    return (" ".join(error_output.strip().splitlines()[-3:]) or " ".join(output.strip().splitlines()[-3:]))[:300]


def obtain_plan(root, plan_file, admission):
    """→ (test-plan 的标准输出, 取不到的理由)。"""
    if plan_file is not None:
        try:
            with open(plan_file, encoding="utf-8") as handle:
                return handle.read(), None
        except OSError as error:
            return None, f"读不了计划输出文件 {plan_file}：{error}"
    code, output, error_output = run_command(["python3", admission, "test-functions-request", REQUEST_STAGE_NAME, REPOSITORY_ROOT])
    if code is None:
        return None, output
    kind, arguments = "", []
    for line in output.splitlines():
        tag, _, value = line.partition("\t")
        if tag == "kind":
            kind = value
        elif tag == "argument":
            arguments.append(value)
    if code != 0 or not kind or not arguments:
        return None, f"admission.py test-functions-request 没打出判定种类与参数（退 {code}）：{tail_of(output, error_output)}"
    scratch = tempfile.mkdtemp(prefix="refused-test-registry-")
    try:
        code, output, error_output = run_command(["python3", admission, "test-plan", root, kind, PLANNED_PACKAGE, os.path.join(scratch, "plan"),
                                                  *arguments, "--exclude-registered"])
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    if code is None:
        return None, output
    if code != 0:
        return None, f"admission.py test-plan 退 {code}（75 是判定库服务不可用）：{tail_of(output, error_output)}"
    return output, None


def judge(root, plan_file=None, admission=None):
    """→ (退出码, 问题清单, 登记表几行, 拒算几条, 清单取不到的理由, 拒算 {测试函数: 理由})。"""
    text, failure = obtain_plan(root, plan_file, admission or ADMISSION_SCRIPT)
    if failure is None:
        broken_plan = plan_problems(text)
        failure = "；".join(broken_plan) if broken_plan else None
    if failure is not None:
        if broken("unavailable-as-green"):
            return 0, [], 0, 0, None, {}
        return NOT_RUN_EXIT, [], 0, 0, failure, {}
    refused = {}
    for name, reason in parse_plan(text)[0]:
        refused.setdefault(name, reason)
    registry, found = read_registry(root)
    for name, reason in refused.items():
        if name not in registry and not broken("unregistered-refusals-accepted"):
            found.append(f"{name} 拒算（{reason}），{REGISTRY_TABLE} 里没有这一行")
    if not broken("stale-rows-accepted"):
        for name, (number, _reason) in registry.items():
            if name not in refused:
                found.append(f"{REGISTRY_TABLE}:{number} {name} 已经不拒算（或这条测试已经不在验证包里）")
    return (1 if found else 0), found, len(registry), len(refused), None, refused


def command_check(root, plan_file, admission):
    code, found, registered, refused_count, failure, _refused = judge(root, plan_file, admission)
    if code == NOT_RUN_EXIT:
        print(f"  ! 拒算清单取不到或不完整，这一格本次未跑（这不是通过）：{failure}")
        print("     → 怎么办：看上面那一句；判定库服务连不上就等 prune-server 回来再跑；"
              "清单不完整就单独跑 python3 scripts/gate/refused-test-registry.py list 看 admission.py test-plan 为什么没打完")
        return NOT_RUN_EXIT
    if found:
        for problem in found:
            print(f"  ✗ {problem}")  # gate-lint:detail
        print(f"  ✗ 拒算的测试函数与登记表 {REGISTRY_TABLE} 有 {len(found)} 处对不上（逐处列在上面）")  # gate-lint:summary
        print("     → 怎么办：新出现的拒算先改测试让单元键算得出（不读活树、不读运行时拼出的仓根路径，输入编进二进制或随测试带）；"
              "断言就是关于真仓、非读活树不可的，交主 agent 二次确认，确认了由主 agent 在登记表加一行「测试函数全名<TAB>为什么非读活树不可」（实现员不登记）；"
              "表里已经不拒算的行删掉；要看每条的状态跑 python3 scripts/gate/refused-test-registry.py list")
        return 1
    print(f"  ✓ 拒算的测试函数与登记表逐条对上：拒算 {refused_count} 条、登记表 {registered} 行，每一条拒算都有表行、每一行都还在拒算")
    return 0


def command_list(root, plan_file, admission):
    code, _found, registered, refused_count, failure, refused = judge(root, plan_file, admission)
    if code == NOT_RUN_EXIT:
        print(f"  ! 拒算清单取不到或不完整：{failure}")
        print("     → 怎么办：看上面那一句；判定库服务连不上就等 prune-server 回来再跑")
        return NOT_RUN_EXIT
    registry, _ = read_registry(root)
    for name, reason in sorted(refused.items()):
        print(f"{'已登记' if name in registry else '未登记'}\t{name}\t{reason}")
    for name, (number, _reason) in sorted(registry.items()):
        if name not in refused:
            print(f"已不拒算\t{name}\t{REGISTRY_TABLE}:{number} 这一行该删")
    print(f"  · 拒算 {refused_count} 条、登记表 {registered} 行；未登记的由主 agent 逐条二次确认该不该拒、确认了才登记（list 不判红，退 0）")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="refused-test-registry-selftest-")
    failures, cells = [], []
    try:
        def world(name, table_lines):
            root = os.path.join(work, name)
            os.makedirs(os.path.join(root, ".claude", "gate.d"))
            with open(os.path.join(root, REGISTRY_TABLE), "w", encoding="utf-8") as handle:
                handle.write("# 样本登记表\n" + "".join(line + "\n" for line in table_lines))
            return root

        def plan(refused_names, declared=None, summary=True):
            lines = [f"tests/plain.rs::passes_{index}\treuse\tkey{index}\tsample:test:plain\ttests/plain.rs\tpasses_{index}\t键相同" for index in range(3)]
            lines += [f"{name}\trefused\t-\tsample:test:plain\ttests/plain.rs\t{name.rsplit('::', 1)[-1]}\t拒算：闭包读运行期拼出的路径（样本）" for name in refused_names]
            if summary:
                lines.append(f"PLAN judge=0 reuse=3 refused={len(refused_names) if declared is None else declared} not-compiled=0 registered=0 ignored=0 library=/sample")
            return "\n".join(lines) + "\n"

        def plan_file(name, text):
            path = os.path.join(work, name + ".plan")
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)
            return path

        def expect(cell, root, text, wanted_code, wanted_text=None):
            cells.append(cell)
            code, found, _registered, _refused_count, failure, _refused = judge(root, plan_file(cell, text))
            shown = " ".join(found) + (failure or "")
            if code != wanted_code:
                failures.append((cell, f"应当退 {wanted_code}，实际退 {code}：{shown}"))
            elif wanted_text is not None and wanted_text not in shown:
                failures.append((cell, f"应当报一处含「{wanted_text}」的问题，实际 {shown}"))

        first, second = "tests/live_tree.rs::reads_the_live_tree", "tests/live_files.rs::reads_a_file_named_at_run_time"
        reason = "断言就是关于真仓里的登记表，要读活树"
        row_first, row_second = f"{first}\t{reason}", f"{second}\t{reason}"
        both = [row_first, row_second]
        expect("clean", world("clean", both), plan([first, second]), 0)
        expect("refusal-without-row", world("refusal-without-row", [row_first]), plan([first, second]), 1, f"{second} 拒算")
        expect("registered-turns-green", world("registered-turns-green", both), plan([first, second]), 0)
        expect("row-without-refusal", world("row-without-refusal", both), plan([first]), 1, f"{second} 已经不拒算")
        expect("reason-too-short", world("reason-too-short", [f"{first}\t读活树", row_second]), plan([first, second]), 1, "只有 3 个字")
        expect("duplicate-row", world("duplicate-row", both + [row_first]), plan([first, second]), 1, "登记了两次")
        expect("malformed-row", world("malformed-row", [first, row_second]), plan([first, second]), 1, "是 1 段，要两段")
        expect("plan-count-mismatch", world("plan-count-mismatch", both), plan([first, second], declared=3), NOT_RUN_EXIT, "PLAN 行报拒算 3 条，逐条行列出 2 条")
        expect("plan-without-summary", world("plan-without-summary", both), plan([first, second], summary=False), NOT_RUN_EXIT, "末行")

        def fake_admission(name, request_body, plan_exit, plan_text):
            """假的 admission.py：test-functions-request 打判定种类与参数，test-plan 退指定的码、打指定的计划输出。"""
            path = os.path.join(work, name + "-admission.py")
            with open(path, "w", encoding="utf-8") as handle:
                handle.write("import sys\n"
                             "if sys.argv[1] == 'test-functions-request':\n"
                             f"    sys.stdout.write({request_body!r})\n"
                             "    sys.exit(0)\n"
                             "if sys.argv[1] == 'test-plan' and '--exclude-registered' in sys.argv[2:] and sys.argv[4] == 'singlefs-verification':\n"
                             f"    sys.stdout.write({plan_text!r})\n"
                             "    sys.stderr.write('样本里的 test-plan 退非 0\\n')\n"
                             f"    sys.exit({plan_exit})\n"
                             "sys.exit(9)\n")
            return path

        request_text = "kind\tcrash-point-replay.sh:test-functions\nargument\t--profile\nargument\trelease\n"
        cells.append("admission-failed")
        code, _found, _registered, _count, failure, _refused = judge(world("admission-failed", both), None,
                                                                    fake_admission("failed", request_text, 75, plan([first, second])))
        if code != NOT_RUN_EXIT or "退 75" not in (failure or ""):
            failures.append(("admission-failed", f"test-plan 退 75 应当按本次未跑退 {NOT_RUN_EXIT} 并写出退出码，实际 {code} {failure}"))
        cells.append("request-unreadable")
        code, _found, _registered, _count, failure, _refused = judge(world("request-unreadable", both), None, fake_admission("unreadable", "", 0, plan([first, second])))
        if code != NOT_RUN_EXIT or "test-functions-request" not in (failure or ""):
            failures.append(("request-unreadable", f"test-functions-request 什么都没打应当按本次未跑退 {NOT_RUN_EXIT} 并点名它，实际 {code} {failure}"))
        cells.append("admission-printed")
        code, found, _registered, _count, failure, _refused = judge(world("admission-printed", both), None, fake_admission("printed", request_text, 0, plan([first, second])))
        if code != 0:
            failures.append(("admission-printed", f"admission.py 打出完整计划、表对得上，应当核过，实际 {code} {found} {failure}"))
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 judge() 那一条核法；REFUSED_TEST_REGISTRY_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ refused-test-registry 自证通过：对上的核过，拒算不在表里、表里已不拒算、理由太短、同一个测试函数登两行、一行不是两段各报出来，"
          f"登进表转绿，计划输出不完整（条数对不上、没有 PLAN 行）、test-plan 退 75、test-functions-request 没打出请求都按本次未跑退 {NOT_RUN_EXIT}，"
          f"admission.py 打出完整计划照核（查了 {len(cells)} 格）")
    return 0


def parse_arguments(arguments):
    """→ (子命令, 仓根, 计划输出文件或 None, admission.py 或 None) 或 None（用法错）。"""
    if not arguments or arguments[0] not in ("check", "list"):
        return None
    root, plan_file, admission, index = REPOSITORY_ROOT, None, None, 1
    root_given = False
    while index < len(arguments):
        argument = arguments[index]
        if argument in ("--plan", "--admission") and index + 1 < len(arguments):
            if argument == "--plan":
                plan_file = arguments[index + 1]
            else:
                admission = arguments[index + 1]
            index += 2
        elif not argument.startswith("-") and not root_given:
            root, root_given = os.path.abspath(argument), True
            index += 1
        else:
            return None
    return arguments[0], root, plan_file, admission


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    parsed = parse_arguments(arguments)
    if parsed is None:
        print(f"  ✗ 认不出的参数 {' '.join(arguments) or '（没给）'}")
        print("  → 怎么办：refused-test-registry.py check|list [仓根] [--plan <文件>] [--admission <脚本>] | --selftest")
        return 2
    command, root, plan_file, admission = parsed
    return (command_check if command == "check" else command_list)(root, plan_file, admission)


BREAK_VARIABLES = ("REFUSED_TEST_REGISTRY_BREAK",)


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在 --selftest 里认（文件头「弄坏开关」）：命令行不是单独一个 --selftest 而 BREAK_VARIABLES 里有一个非空，退 2 并点名它只在自证里认；在参数解析之前判。"""
    for name in BREAK_VARIABLES:
        if os.environ.get(name) and arguments != ["--selftest"]:
            print(f"  ✗ {name} 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证", file=sys.stderr)
            print(f"     → 怎么办：去掉 {name} 再跑；要证红就跑 {name}=<项> python3 scripts/gate/refused-test-registry.py --selftest", file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    refusal = refuse_break_outside_selftest(sys.argv[1:])
    if refusal:
        sys.exit(refusal)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
