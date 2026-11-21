"""task_server_selftest.py 拆出的一份：提交时拒、整包与未点名的跑法、确认单、长任务的各格（总说明在 task_server_selftest.py 文件头）。"""
import importlib.util
import os
import json
import re
import shlex
import time
import task_server_lib as lib
from task_server_selftest_fakes import (
    BLOCK_SCRIPT, MEBIBYTE, PLACEMENT_WAIT_SECONDS, POLL_SECONDS, SEEDED_ENDED_AT, SERVER, SESSION_A, SESSION_B, UNRECORDED_DEFAULT_PINNED_TEXT,
)
from task_server_selftest_world import (
    FAKE_COMMIT_GATE, FAKE_MERGE_ENTRY, FakeInbox, INBOX_TOKEN_SAMPLE, INBOX_WAIT_SECONDS, LONG_RUN_KIND, LONG_RUN_WHY_SAMPLE,
    RUN_LIMIT_SAMPLE_SECONDS, SCOPE_COMMIT, SCOPE_DAILY, SelftestFailure, attention_problems, describe, first_attempt, inbox_problems, outcome_head,
    read_text, start_inbox_server, wait_path, write_executable, write_fake_entry,
)


# 拆分族（一份脚本拆成入口加按拆法命名的部分文件）的认法只在门禁那一份库里写一份，这里按文件路径导入（本服务的 sys.path 不一定有那个目录）
SPLIT_FAMILY_LIBRARY_PATH = os.path.join(lib.REPOSITORY_ROOT, "scripts", "gate", "lib_lag_split_family.py")
_SPLIT_FAMILY_SPECIFICATION = importlib.util.spec_from_file_location("lib_lag_split_family_for_task_server", SPLIT_FAMILY_LIBRARY_PATH)
split_family = importlib.util.module_from_spec(_SPLIT_FAMILY_SPECIFICATION)
_SPLIT_FAMILY_SPECIFICATION.loader.exec_module(split_family)


CLIENT_REFUSED_EXIT = 3   # 客户端 task-server/task 的 REFUSED_EXIT（服务或客户端拒一条请求时退的码）；confirm-on-queued-task-accepted 按它判「别的会话确认仍被拒」

# temporary-service-own-server-compiles 现写的假 scripts/prune-server/prune-server-load-simulation.py（别的格不用它，所以不放进 World 的 fakes）
LOAD_SIMULATION_SCRIPT = "prune-server-load-simulation.py"
FAKE_LOAD_SIMULATION = r'''#!/usr/bin/env python3
"""task-server 自证的假 scripts/prune-server/prune-server-load-simulation.py：认 --selftest（后面的参数不看），当场退 0。"""
import sys

sys.exit(0 if sys.argv[1:2] == ["--selftest"] else 2)
'''


def cell_verdict_library_family_refused(world):
    """③ 带判定库的提交时只照登记表 .claude/gate.d/verdict-library-scripts.tsv 第三列判（task_server.py 文件头 ②「提交时拒」③）：第三列写「prune-server」的族照收、
    本机放不下就派到第二台跑完；写「无判定库：」的不算带判定库、照收；第三列写本地库目录的族，直接点到的与包在临时 prune-server 里的都当场拒
    （task submit 退 3、不给任务号，说明点名那一族与第三列、出路写接进 prune-server），一条都没收；登记表写坏了（不到三列）的判不了、照拒。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    registry = os.path.join(world.repo, SERVER.VERDICT_LIBRARY_REGISTRY)
    rows = ("# 自证的登记表：路径<TAB>自证命令<TAB>判定库放哪<TAB>复用三问格名\n"
            "scripts/selftest-served.sh\tbash scripts/selftest-served.sh --selftest\tprune-server（结论表 verdict-library/…）\treuse-three-questions\n"
            "scripts/selftest-local-library.sh\tbash scripts/selftest-local-library.sh --selftest\t<库根的 git common-dir>/selftest-verdicts/\treuse-three-questions\n"
            "scripts/selftest-no-library.sh\tbash scripts/selftest-no-library.sh --selftest\t无判定库：待立\treuse-three-questions\n")
    with open(registry, "w", encoding="utf-8") as handle:
        handle.write(rows)
    world.start_server()
    problems, accepted = [], []
    for name, word in (("第三列写 prune-server 的族", "scripts/selftest-served.sh"), ("第三列写「无判定库：」的", "scripts/selftest-no-library.sh")):
        try:
            task_id = world.submit(["bash", "-c", "exit 0", name, word], cwd=world.repo)
            accepted.append(task_id)
            summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
            if first_attempt(summary).get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
                problems.append(f"{name}应当照收、派第二台跑完，实际 {describe(summary)}")
        except SelftestFailure as failure:
            problems.append(f"{name}应当照收、派第二台跑完，实际 {failure}")
    local_library = "scripts/selftest-local-library.sh"
    wrapped = ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, "--server-binary", "/usr/bin/true", "--",
               "bash", "-c", "exit 0", "wrapped-local-library", local_library]
    for name, command in (("直接点到第三列写本地库目录的族", ["bash", "-c", "exit 0", "local-library", local_library]),
                          ("包在临时 prune-server 里点到那一族", wrapped)):
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        wanted = (local_library, "第三列", "接进 prune-server")
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in wanted):
            problems.append(f"{name}：应当退 3、不给任务号、说明点名 {local_library} 与第三列、出路写接进 prune-server，"
                            f"实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-400:]}」")
    with open(registry, "a", encoding="utf-8") as handle:
        handle.write("scripts/selftest-broken-row.sh\tbash scripts/selftest-broken-row.sh --selftest\n")
    completed = world.client(["submit", "--cwd", world.repo, "--", "bash", "-c", "exit 0", "broken-registry"])
    if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or "判不了" not in completed.stderr or "不到三列" not in completed.stderr:
        problems.append(f"登记表有一行不到三列：应当判不了、退 3、说明点名那一行，实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-400:]}」")
    with open(registry, "w", encoding="utf-8") as handle:
        handle.write(rows)
    listed = {item.get("id") for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", [])}
    if listed - set(accepted):
        problems.append(f"拒了的一条都不该进队，实际会话里另列着 {sorted(listed - set(accepted))}")
    if problems:
        raise SelftestFailure("带判定库的提交时照登记表第三列判：" + "；".join(problems), world.server_log_lines())
    return "第三列写 prune-server 的族与写「无判定库：」的照收、派到第二台；写本地库目录的族（直接点到、包在临时 prune-server 里）与登记表写坏的提交时拒"


def cell_temporary_service_wrapper_unwrapped(world):
    """临时 prune-server 的包装（python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <路径> -- <原命令>）：
    包着的照原命令算键、--server-binary 的路径不进键（两次包着的给的路径不同，命令键相同、第二次照第一次量到的 300M 定上限 300M）；
    键前面接 TEMPORARY_SERVICE_KEY_MARK，与不包的那一条不共用记录（不包的那次也量到 300M，包着的第一次没有自己的记录、取 4096M）；
    包装本身照旧整条执行（认登记表行同样照原命令认，由 verdict-library-family-refused 判）。"""
    scope = world.write_fake_scope(300 * MEBIBYTE)
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    inner = ["bash", "-c", "exit 0", "selftest-wrapped-marker", "fake-peer-run"]   # fake-peer-run：派到假第二台的也照真的那样在这棵树里起命令

    def wrapped(binary):
        return ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, "--server-binary", binary, "--", *inner]

    plain = world.finished(world.submit(inner, cwd=world.repo))
    first = world.finished(world.submit(wrapped("/usr/bin/true"), cwd=world.repo))
    second = world.finished(world.submit(wrapped("/bin/true"), cwd=world.repo))
    keys = [summary["task"].get("command_key") or "" for summary in (plain, first, second)]
    caps = [first_attempt(summary).get("memory_cap_text") for summary in (plain, first, second)]
    problems = []
    if keys[1] != keys[2] or keys[1] == keys[0] or not keys[1].startswith(lib.TEMPORARY_SERVICE_KEY_MARK):
        problems.append(f"两次包着的应当同一个命令键、以「{lib.TEMPORARY_SERVICE_KEY_MARK}」起头、与不包的不同，实际不包的「{keys[0]}」、包着的「{keys[1]}」与「{keys[2]}」")
    if caps[1:] != [UNRECORDED_DEFAULT_PINNED_TEXT, "300M"] or any(summary["task"].get("outcome") != "succeeded" for summary in (plain, first, second)):
        problems.append(f"包着的第一次没有自己的记录应当取 {UNRECORDED_DEFAULT_PINNED_TEXT}（不用不包的那次量到的 300M）、第二次照第一次的 300M 取 300M、都跑完，"
                        f"实际上限 {caps}、{describe(first)}、{describe(second)}")
    if not all(world.calls_of(summary["task"]["id"], "temporary-service") for summary in (first, second)):
        problems.append("包装本身应当照旧整条执行（假包装记一行），实际没起")
    if problems:
        raise SelftestFailure("临时 prune-server 的包装：" + "；".join(problems), world.server_log_lines())
    return (f"包着的照原命令算键、服务路径不进键、前面接「{lib.TEMPORARY_SERVICE_KEY_MARK}」：与不包的不共用记录（第一次 {UNRECORDED_DEFAULT_PINNED_TEXT}、"
            "第二次照第一次的 300M 取 300M），包装照旧整条执行")


WRAPPED_SELFTEST_SCRIPT = "scripts/selftest-wrapped-check.sh"   # temporary-service-heavy-refused：包装里照收的那条普通 --selftest 起的假脚本（相对假仓根）


def cell_temporary_service_heavy_refused(world):
    """④ 临时 prune-server 包装里会编的提交时拒（task_server.py 文件头 ②「提交时拒」④）：包装里套 cargo test（任务 4229 那一条的样子）
    当场拒（task submit 退 3、不给任务号，说明点名会编的 cargo、写明包装只给脚本自证用，出路写去掉包装重交），
    没进队；包装里普通的 --selftest 照收、包装照旧整条执行、跑完。"""
    world.resources(local=(30, 30))
    world.start_server()
    write_executable(os.path.join(world.repo, WRAPPED_SELFTEST_SCRIPT), "#!/usr/bin/env bash\nexit 0\n")

    def wrapped(*inner):
        return ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, "--server-binary", "/usr/bin/true", "--", *inner]

    problems = []
    refused_cases = (("包装里套 cargo test", wrapped("cargo", "test", "--release", "-p", "singlefs-verification", "--test", "selftest-target",
                                                 "fake-wrapper-exits=0"), "会编的 cargo"),)
    for name, command, wanted in refused_cases:
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        texts = (wanted, "只给脚本自证用", "去掉包装重交")
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in texts):
            problems.append(f"{name}：应当退 3、不给任务号、说明写「{'」「'.join(texts)}」，实际退 {completed.returncode}、"
                            f"stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    accepted = []
    try:
        task_id = world.submit(wrapped("bash", WRAPPED_SELFTEST_SCRIPT, "--selftest"), cwd=world.repo)
        accepted.append(task_id)
        summary = world.finished(task_id)
        if summary["task"].get("outcome") != "succeeded" or not world.calls_of(task_id, "temporary-service"):
            problems.append(f"包装里普通的 --selftest 应当照收、包装照旧整条执行（假包装记一行）、跑完，实际 {describe(summary)}、"
                            f"假包装记了 {len(world.calls_of(task_id, 'temporary-service'))} 行")
    except SelftestFailure as failure:
        problems.append(f"包装里普通的 --selftest 应当照收、跑完，实际 {failure}")
    listed = {item.get("id") for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", [])}
    if listed - set(accepted):
        problems.append(f"拒了的一条都不该进队，实际会话里另列着 {sorted(listed - set(accepted))}")
    if problems:
        raise SelftestFailure("临时 prune-server 包装里会编的提交时拒：" + "；".join(problems), world.server_log_lines())
    return (f"包装里套 cargo test 提交时退 3、说明点名会编的 cargo、出路写去掉包装重交；"
            f"包装里普通的 --selftest 照收、包装照旧整条执行、跑完（{len(refused_cases)} 条拒、{len(accepted)} 条收）")


WHOLE_WORKSPACE_MEMBER = "crates/selftest-member"   # whole-workspace-cargo-refused：假仓根虚拟清单里的成员（相对假仓根），自己有一份包清单
WHOLE_WORKSPACE_NEXT_TEXTS = ("-p <包>", "-p singlefs-verification", "merge-and-verify")   # 那一格拒绝说明的出路里要有的几样（task_server.py 的 WHOLE_WORKSPACE_NEXT_STEP）


def cell_whole_workspace_cargo_refused(world):
    """⑤ 不点名包的全仓编译提交时拒（task_server.py 文件头 ②「提交时拒」⑤；任务 8105）：假仓根写一份虚拟清单（只有 [workspace]、两个成员），在仓根交
    不带 -p 的 cargo build --offline --all-targets（8105 那一条）、不带 -p 的 cargo clippy --all-targets --all-features（8104 那一条）、带 --workspace 的 cargo clippy、
    带 -p 又带 --workspace 的 cargo test、bash -c 里不带 -p 的 cargo check，都当场拒（task submit 退 3、不给任务号，说明点名那条 cargo，出路写 -p <包>、
    -p singlefs-verification 与 merge-and-verify），一条都没进队；带 -p 的 cargo build、在成员目录里不带 -p 的 cargo test、不编的 cargo tree 照收、跑完。
    弄坏开关 whole-workspace-cargo-accepted 下拒的那几条照收，这一格红。"""
    world.resources(local=(30, 30))
    member = os.path.join(world.repo, WHOLE_WORKSPACE_MEMBER)
    os.makedirs(member, exist_ok=True)
    with open(os.path.join(world.repo, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write(f'[workspace]\nresolver = "2"\nmembers = ["{WHOLE_WORKSPACE_MEMBER}", "crates/selftest-other"]\n')
    with open(os.path.join(member, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write('[package]\nname = "selftest-member"\nversion = "0.0.0"\n')
    world.start_server()
    problems = []
    refused_cases = (("不带 -p 的 cargo build（任务 8105）", ["cargo", "build", "--offline", "--all-targets"], "cargo build 没点名包"),
                     ("不带 -p 的 cargo clippy（任务 8104）", ["cargo", "clippy", "--all-targets", "--all-features"], "cargo clippy 没点名包"),
                     ("带 --workspace 的 cargo clippy", ["cargo", "clippy", "--workspace", "--all-targets"], "cargo clippy 带了 --workspace"),
                     ("带 -p 又带 --workspace 的 cargo test", ["cargo", "test", "-p", "selftest-member", "--workspace"], "cargo test 带了 --workspace"),
                     ("bash -c 里不带 -p 的 cargo check", ["bash", "-c", "cargo check --release"], "cargo check 没点名包"))
    for name, command, wanted in refused_cases:
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        texts = (wanted, *WHOLE_WORKSPACE_NEXT_TEXTS)
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in texts):
            problems.append(f"{name}：应当退 3、不给任务号、说明写「{'」「'.join(texts)}」，实际退 {completed.returncode}、"
                            f"stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    accepted = []
    accepted_cases = (("带 -p 的 cargo build", world.repo, ["cargo", "build", "-p", "selftest-member", "fake-wrapper-exits=0"]),
                      ("在成员目录里不带 -p 的 cargo test", member, ["cargo", "test", "fake-wrapper-exits=0"]),
                      ("不编的 cargo tree", world.repo, ["cargo", "tree", "fake-wrapper-exits=0"]))
    for name, cwd, command in accepted_cases:
        try:
            task_id = world.submit(command, cwd=cwd)
            accepted.append(task_id)
            summary = world.finished(task_id)
            if summary["task"].get("outcome") != "succeeded":
                problems.append(f"{name}应当照收、跑完，实际 {describe(summary)}")
        except SelftestFailure as failure:
            problems.append(f"{name}应当照收、跑完，实际 {failure}")
    listed = {item.get("id") for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", [])}
    if listed - set(accepted):
        problems.append(f"拒了的一条都不该进队，实际会话里另列着 {sorted(listed - set(accepted))}")
    if problems:
        raise SelftestFailure("不点名包的全仓编译提交时拒：" + "；".join(problems), world.server_log_lines())
    return (f"虚拟清单的仓根上不点名包、带 --workspace 的 {len(refused_cases)} 条 cargo 提交时退 3、出路写点名要编的包；"
            f"点了包的、在成员目录里的、不编的 {len(accepted)} 条照收、跑完")


COPY_PROOF_BUILD = ["cargo", "build", "-p", "selftest-member", "fake-wrapper-exits=0"]   # copy-proof-* 两格：会编译的命令（假包装不真起）
COPY_PROOF_NO_BUILD = ["cargo", "fmt", "fake-wrapper-exits=0"]   # 不编译的命令
COPY_PROOF_ADVICE_TEXTS = ("另拷一份副本", "minimal-copy.py")   # 两格的拒绝说明里都要有的出路


def copy_proof_world(world, names):
    """copy-proof-* 两格共用：起服务，在这一格的假共享根下建 names 里每个名字的空目录当副本 → {名: 真路径}。"""
    world.resources(local=(30, 30))
    copies = {}
    for name in names:
        copies[name] = os.path.realpath(os.path.join(world.root, name))
        os.makedirs(copies[name], exist_ok=True)
    world.start_server()
    return copies


def copy_proof_try(world, what, command, cwd, wanted, problems):
    """交一条（只交、不确认，待确认的不进队）：wanted 是 None 要照收（退 0、打任务号），是几个词要拒（退 3、不给任务号、stderr 带这几个词）；
    不对的写进 problems。→ 收下的任务号，没收下交 None。"""
    completed = world.client(["submit", "--cwd", cwd, "--", *command])
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
    if wanted is None:
        if completed.returncode != 0 or not found:
            problems.append(f"{what}应当照收、退 0、打任务号，实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-400:]}」")
    elif completed.returncode != 3 or found or not all(word in completed.stderr for word in wanted):
        problems.append(f"{what}应当退 3、不给任务号、说明写「{'」「'.join(wanted)}」，实际退 {completed.returncode}、"
                        f"stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    return int(found.group(1)) if found else None


def copy_proof_cancel(world, accepted):
    """收下的待确认任务都取消（它们不进队，只在库里占着：后面的判看得见在排在跑待确认的），清空 accepted。"""
    for task_id in accepted:
        world.client(["cancel", str(task_id)])
    accepted.clear()


def split_family_pattern(entry, pattern):
    """脚本 entry（仓内相对路径）与它拆出的部分文件（同目录下按拆法命名的，认法 split_family.family_members）里，pattern 第一处命中；一处都没有交 None。"""
    directory = os.path.join(lib.REPOSITORY_ROOT, os.path.dirname(entry))
    candidates = [os.path.join(os.path.dirname(entry), name) for name in sorted(os.listdir(directory))]
    for member in split_family.family_members(entry, candidates):
        found = re.search(pattern, read_text(os.path.join(lib.REPOSITORY_ROOT, member)))
        if found:
            return found
    return None


def copy_proof_run(copy):
    """对 copy 的证红：scripts/mutation/prove-red.sh --copy <副本> <crate> <变异名>（在假仓根起）。"""
    return ["bash", lib.UNNAMED_RUN_PROVE_RED_ENTRY, "--copy", copy, "selftest-crate", "selftest-row"]


def cell_copy_proof_build_refused(world):
    """⑨ 证红进行中的副本里，会编译的命令提交时拒（task_server.py 文件头 ②「提交时拒」⑨；异常表 copy-under-proof）：副本根上留着还原日志 .prove-red-pending、
    持着 300 秒内刷新过的副本锁持有者记录 .prove-red.owner、或有对这份副本的证红任务在排在跑待确认，而会编译的 cargo（工作目录在副本里；cwd 在副本外、
    --manifest-path 指进副本；bash -c 里 cd 进副本）都退 3、不给任务号，说明点名标记或那条证红任务、出路写等它出结局与另拷一份副本；
    别的副本里的会编译的命令、同一份副本里不编译的 cargo fmt、持有者记录过期（超过 300 秒没刷新）而没有还原日志的副本、证红取消之后的同一份副本都照收。
    还核服务认的两个标记名与过期秒数与 prove-red.sh、apply-writer-patch.py 里写的同一个（两份脚本拆出的部分文件一并读：常量在哪一份里写都认）。
    弄坏开关 copy-proof-build-accepted 下拒的照收，copy-proof-every-copy-judged 下别的副本里的也拒，copy-proof-every-command-judged 下不编译的也拒，这一格红。"""
    copies = copy_proof_world(world, ("copy-one", "copy-two"))
    one, two = copies["copy-one"], copies["copy-two"]
    problems, accepted = [], []
    for relative, pattern, wanted in (("scripts/mutation/prove-red.sh", r'RESTORE_JOURNAL_NAME = "([^"]+)"', lib.COPY_PROOF_PENDING_NAME),
                                      ("scripts/gate/apply-writer-patch.py", r'COPY_OWNER_NAME = "([^"]+)"', lib.COPY_PROOF_OWNER_NAME),
                                      ("scripts/gate/apply-writer-patch.py", r"COPY_OWNER_STALE_SECONDS = (\d+)", str(lib.COPY_PROOF_OWNER_STALE_SECONDS))):
        found = split_family_pattern(relative, pattern)
        if not found or found.group(1) != wanted:
            problems.append(f"{relative} 里写的 {pattern} 应当是 {wanted}，实际 {found.group(1) if found else '读不到'}")
    pending, owner = os.path.join(one, lib.COPY_PROOF_PENDING_NAME), os.path.join(one, lib.COPY_PROOF_OWNER_NAME)
    open(pending, "w", encoding="utf-8").close()
    copy_proof_try(world, "留着还原日志的副本里会编译的命令", COPY_PROOF_BUILD, one, ("证红", lib.COPY_PROOF_PENDING_NAME, "重跑", *COPY_PROOF_ADVICE_TEXTS), problems)
    accepted.append(copy_proof_try(world, "留着还原日志的那份之外的副本里会编译的命令", COPY_PROOF_BUILD, two, None, problems))
    os.unlink(pending)
    open(owner, "w", encoding="utf-8").close()
    copy_proof_try(world, "持有者记录刚刷新过的副本里会编译的命令", COPY_PROOF_BUILD, one,
                   ("证红", lib.COPY_PROOF_OWNER_NAME, "task-server/task status", *COPY_PROOF_ADVICE_TEXTS), problems)
    stale = time.time() - 2 * lib.COPY_PROOF_OWNER_STALE_SECONDS
    os.utime(owner, (stale, stale))
    accepted.append(copy_proof_try(world, "持有者记录过期、没有还原日志的副本里会编译的命令", COPY_PROOF_BUILD, one, None, problems))
    os.unlink(owner)
    copy_proof_cancel(world, [task_id for task_id in accepted if task_id is not None])
    accepted.clear()
    proof = copy_proof_try(world, "对 copy-one 的证红（里面没有会编译的任务）", copy_proof_run(one), world.repo, None, problems)
    if proof is not None:
        wanted = ("证红", f"任务 {proof}", "task-server/task wait", *COPY_PROOF_ADVICE_TEXTS)
        copy_proof_try(world, "有证红任务的副本里会编译的命令", COPY_PROOF_BUILD, one, wanted, problems)
        copy_proof_try(world, "有证红任务的副本、cwd 在副本外而 --manifest-path 指进副本的命令", [*COPY_PROOF_BUILD[:4], "--manifest-path", os.path.join(one, "Cargo.toml")],
                       world.repo, wanted, problems)
        copy_proof_try(world, "有证红任务的副本、bash -c 里 cd 进副本再编译的命令", ["bash", "-c", f"cd {one} && cargo build -p selftest-member"], world.repo, wanted, problems)
        accepted.append(copy_proof_try(world, "别的副本里会编译的命令（有证红任务的是 copy-one）", COPY_PROOF_BUILD, two, None, problems))
        accepted.append(copy_proof_try(world, "有证红任务的副本里不编译的 cargo fmt", COPY_PROOF_NO_BUILD, one, None, problems))
        copy_proof_cancel(world, [task_id for task_id in accepted if task_id is not None])
        world.client(["cancel", str(proof)])
        accepted.append(copy_proof_try(world, "证红取消之后的同一份副本里会编译的命令", COPY_PROOF_BUILD, one, None, problems))
    if problems:
        raise SelftestFailure("证红进行中的副本里会编译的命令提交时拒：" + "；".join(problems), world.server_log_lines())
    return ("留着还原日志、持有者记录刚刷新过、有证红任务的副本里会编译的命令（含 cwd 在副本外、--manifest-path 指进副本、bash -c 里 cd 进副本）提交时退 3、点名标记或那条证红任务；"
            "别的副本、不编译的 cargo fmt、持有者记录过期的、证红取消之后的照收")


def cell_copy_proof_start_refused(world):
    """⑨ 对一份副本的证红开跑时副本里已有会编译的任务，提交时拒（task_server.py 文件头 ②「提交时拒」⑨；异常表 copy-under-proof）：copy-one 里有会编译的任务待确认时，
    对 copy-one 的证红退 3、不给任务号，说明点名那条任务、写会编译、出路写等它出结局与另拷一份副本；copy-two 里只有不编译的 cargo fmt、copy-three 里什么都没有时，
    对它们的证红照收；已经有证红任务的副本再交一条证红照收（副本锁自己拒第二趟）；那条会编译的任务取消之后，对 copy-one 的证红照收。
    弄坏开关 copy-proof-start-accepted 下拒的照收，copy-proof-every-copy-judged 下别的副本的证红也拒，copy-proof-every-command-judged 下不编译的任务也挡证红，这一格红。"""
    copies = copy_proof_world(world, ("copy-one", "copy-two", "copy-three"))
    one, two, three = copies["copy-one"], copies["copy-two"], copies["copy-three"]
    problems = []
    build = copy_proof_try(world, "copy-one 里会编译的命令（先到）", COPY_PROOF_BUILD, one, None, problems)
    no_build = copy_proof_try(world, "copy-two 里不编译的 cargo fmt", COPY_PROOF_NO_BUILD, two, None, problems)
    if build is not None:
        copy_proof_try(world, "对 copy-one 的证红（里面有会编译的任务）", copy_proof_run(one), world.repo,
                       (f"任务 {build}", "会编译", "task-server/task wait", *COPY_PROOF_ADVICE_TEXTS), problems)
    proofs = [copy_proof_try(world, "对 copy-two 的证红（里面只有不编译的任务）", copy_proof_run(two), world.repo, None, problems),
              copy_proof_try(world, "对 copy-three 的证红（里面没有任务）", copy_proof_run(three), world.repo, None, problems),
              copy_proof_try(world, "已经有证红任务的 copy-two 再交一条证红", copy_proof_run(two), world.repo, None, problems)]
    if build is not None:
        world.client(["cancel", str(build)])
        proofs.append(copy_proof_try(world, "会编译的任务取消之后对 copy-one 的证红", copy_proof_run(one), world.repo, None, problems))
    copy_proof_cancel(world, [task_id for task_id in proofs + [no_build] if task_id is not None])
    if problems:
        raise SelftestFailure("有会编译的任务的副本上证红提交时拒：" + "；".join(problems), world.server_log_lines())
    return ("副本里已有会编译的任务时对它的证红提交时退 3、点名那条任务；别的副本、只有不编译的任务的、已有证红任务的、会编译的任务取消之后的副本上的证红照收")


WHOLE_PACKAGE_COMMANDS = (   # ⑬ 整包测试四种形态（task_server.py 文件头 ②「提交时拒」⑥）：(说什么, 命令, 在哪起)
    ("cargo test -p 不点名测试目标", ["cargo", "test", "-p", "singlefs-verification"], "work"),
    ("cargo test --lib --tests", ["cargo", "test", "-p", "singlefs-verification", "--lib", "--tests"], "work"),
    ("crash-point-replay.sh --check 三格全点", ["bash", ".claude/gate.d/crash-point-replay.sh", "--check", "crash-case-replay,feature-gated-tests,product-gpu-real-card-tests"], "repo"),
    ("不给补丁目录的 merge-and-verify.sh", ["bash", "scripts/gate/merge-and-verify.sh"], "repo"))


def submit_raw(world, command, cwd, confirmation="", scope=SCOPE_DAILY, full_gate=None):
    """只交、不确认（交了的待确认、不进队、不跑）：→ CompletedProcess。full_gate 没给的，演提交时那一趟（scope 是 SCOPE_COMMIT 而命令是提交时门禁）带全量标识。"""
    if full_gate is None:
        full_gate = scope == SCOPE_COMMIT and lib.commit_gate_entry(list(command), cwd)
    words = ["submit", "--cwd", cwd] + (["--full-gate"] if full_gate else []) + (["--whole-package-confirmed", confirmation] if confirmation else []) + ["--", *command]
    return world.client(words, extra={"SINGLEFS_RUN_SCOPE": scope})


def submit_confirmed(world, command, cwd, confirmation, scope=SCOPE_DAILY):
    """带整包确认文件交一条、照确认单确认（同 World.submit），交任务号。"""
    completed = submit_raw(world, command, cwd, confirmation, scope)
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
    if completed.returncode != 0 or not found:
        raise SelftestFailure(f"带确认文件提交 {command} 没拿到任务号（退 {completed.returncode}）", [completed.stdout, completed.stderr] + world.server_log_lines())
    world.confirm_submitted(completed, extra={"SINGLEFS_RUN_SCOPE": scope})
    return int(found.group(1))


def cell_whole_package_unconfirmed_refused(world):
    """整包测试没带确认标识当场拒（task_server.py 文件头 ②「提交时拒」⑥；.claude/agent-common.md「日常开发的测试不跑整个包」）：四种形态各交一回，都退 3、
    stderr 写「整包测试没带确认标识」与 --whole-package-confirmed 那条出路；带 --workspace 的被 ⑤ 全仓编译先拒，同样退 3。
    弄坏开关 whole-package-unconfirmed-accepted 下四种都收下，这一格红。"""
    world.start_server()
    problems = []
    for what, command, where in WHOLE_PACKAGE_COMMANDS:
        completed = submit_raw(world, command, world.work if where == "work" else world.repo)
        if completed.returncode != 3 or "整包测试没带确认标识" not in completed.stderr or "--whole-package-confirmed" not in completed.stderr:
            problems.append(f"{what}没带确认标识应当退 3、stderr 写「整包测试没带确认标识」与出路，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
    workspace = submit_raw(world, ["cargo", "test", "--workspace"], world.work)
    if workspace.returncode != 3:
        problems.append(f"cargo test --workspace 应当退 3，实际退 {workspace.returncode}：「{workspace.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("整包测试没带确认标识：" + "；".join(problems), world.server_log_lines())
    return f"整包测试 {len(WHOLE_PACKAGE_COMMANDS)} 种形态没带确认标识都退 3，--workspace 的也退 3"


def cell_whole_package_named_accepted(world):
    """点名了测试目标的不拒（cargo test -p … --test <目标>、--lib <测试名>），提交时的门禁不拒（.claude/gate.d/crash-point-replay.sh 不带 --check、「为什么」写了提交前）：
    都照收（退 0、打任务号；只交不确认，不跑）。"""
    world.start_server()
    problems = []
    for what, command, where, scope in (("--test 点名", ["cargo", "test", "-p", "singlefs-verification", "--test", "selftest_target"], world.work, SCOPE_DAILY),
                                        ("--lib 带测试名", ["cargo", "test", "-p", "singlefs-verification", "--lib", "selftest_name"], world.work, SCOPE_DAILY),
                                        ("只带测试名", ["cargo", "test", "-p", "singlefs-verification", "selftest_name"], world.work, SCOPE_DAILY),
                                        ("--lib --tests 带测试名", ["cargo", "test", "-p", "singlefs-verification", "--lib", "--tests", "--", "--exact", "selftest_name"], world.work, SCOPE_DAILY),
                                        ("提交时的门禁", ["bash", ".claude/gate.d/crash-point-replay.sh"], world.repo, SCOPE_COMMIT)):
        completed = submit_raw(world, command, where, scope=scope)
        if completed.returncode != 0 or lib.SUBMIT_MARK not in completed.stdout:
            problems.append(f"{what}的应当照收、退 0，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("点名的与提交时的门禁不拒：" + "；".join(problems), world.server_log_lines())
    return "点名测试目标、带测试名的四种与提交时的门禁都照收"


WHOLE_PACKAGE_CONFIRMATION_TEXT = ("① 改到的文件与函数：task-server/task_server.py 的 judge_stall_stop 与 run_limit_accumulated\n"
                                   "② 引到它们的测试目标与函数：task_server_lib.py --selftest 的几格，grep 命中七处\n"
                                   "③ 为什么只点名②不够：自证造的整包确认，演三段都写全的一份\n")


def write_confirmation(world, name, text):
    path = os.path.join(world.root, name)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    return path


def cell_whole_package_confirmation_form(world):
    """整包确认文件只判形式（task_server_lib.py 的 whole_package_confirmation_problem）：缺 ③ 那一段的退 3、点名 ③；③ 不到 12 个字的退 3；相对路径退 3；
    三段写全的照收（退 0），task status 那一条有一行「整包已确认：<文件>」。"""
    world.start_server()
    command = ["cargo", "test", "-p", "singlefs-verification"]
    full = write_confirmation(world, "whole-package-full.md", WHOLE_PACKAGE_CONFIRMATION_TEXT)
    problems = []
    for what, path, text in (("缺 ③", write_confirmation(world, "whole-package-missing.md", WHOLE_PACKAGE_CONFIRMATION_TEXT.rsplit("③", 1)[0]), "③"),
                             ("③ 太短", write_confirmation(world, "whole-package-short.md", WHOLE_PACKAGE_CONFIRMATION_TEXT.rsplit("③", 1)[0] + "③ 太短\n"), "不到"),
                             ("相对路径", "whole-package-full.md", "不是绝对路径")):
        completed = submit_raw(world, command, world.work, confirmation=path)
        if completed.returncode != 3 or text not in completed.stderr:
            problems.append(f"确认文件{what}应当退 3、写「{text}」，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
    accepted = submit_raw(world, command, world.work, confirmation=full)
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", accepted.stdout)
    shown = world.client(["status", found.group(1)]) if found else None
    if accepted.returncode != 0 or shown is None or f"整包已确认：{full}" not in shown.stdout + shown.stderr:
        problems.append(f"三段写全的应当照收、status 有「整包已确认：{full}」，实际退 {accepted.returncode}、status「{(shown.stdout + shown.stderr).strip()[:300] if shown else ''}」")
    if problems:
        raise SelftestFailure("整包确认文件的形式：" + "；".join(problems), world.server_log_lines())
    return "缺 ③、③ 太短、相对路径都退 3；三段写全的照收，status 有整包已确认那一行"


def cell_whole_package_confirmed_still_reminded(world):
    """整包已确认的照样受两小时提醒（task_server.py 文件头 ②「提交时拒」⑥ 与 ⑦「跑满两小时」）：不给补丁目录的假 merge-and-verify.sh 带三段写全的确认文件交、跑 6 秒，满 3 秒记提醒、服务不停；
    照提醒 task confirm <号> --why 确认，status 有整包已确认与长跑已确认两行，照跑完。
    弄坏开关 whole-package-confirmed-exempts-two-hours 下不提醒，long-run-confirmed-flag-hidden 下没有长跑已确认那一行，这一格红。"""
    world.resources(local=(100, 100))
    release = os.path.join(world.root, "release-whole-package-reminded")
    entry = os.path.join(world.repo, FAKE_MERGE_ENTRY)
    os.makedirs(os.path.dirname(entry), exist_ok=True)
    write_executable(entry, f"#!/usr/bin/env bash\nfor round in $(seq 1 600); do [[ -e {shlex.quote(release)} ]] && exit 0; sleep 0.1; done; exit 9\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    full = write_confirmation(world, "whole-package-full.md", WHOLE_PACKAGE_CONFIRMATION_TEXT)
    task_id = submit_confirmed(world, ["bash", FAKE_MERGE_ENTRY], world.repo, full)
    waited = world.client(["wait", str(task_id)])
    problems, _ = attention_problems(waited, task_id, LONG_RUN_KIND, lib.ATTENTION_EXIT)
    confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
    shown = world.client(["status", str(task_id)])
    text = shown.stdout + shown.stderr
    if confirmed.returncode != 0:
        problems.append(f"task confirm <号> --why 应当退 0，实际退 {confirmed.returncode}：「{(confirmed.stdout + confirmed.stderr).strip()[-300:]}」")
    for line in (f"整包已确认：{full}", f"长跑已确认：{LONG_RUN_WHY_SAMPLE}"):
        if line not in text:
            problems.append(f"确认之后 status 应当有「{line}」，实际「{text.strip()[:400]}」")
    open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("whole_package_confirmed") != full or summary["task"].get("outcome") != "succeeded":
        problems.append(f"应当带整包确认文件、过了界照跑完，实际标识「{summary['task'].get('whole_package_confirmed')}」、{describe(summary)}")
    if problems:
        raise SelftestFailure("整包已确认照样受两小时提醒：" + "；".join(problems), world.server_log_lines())
    return f"整包已确认的满 {RUN_LIMIT_SAMPLE_SECONDS} 秒照样提醒、服务不停；确认之后两种标识都打、照跑完"


def cell_temporary_service_wrapper_compiles(world):
    """不带 --server-binary 的临时 prune-server 包装自己要在服务给的编译目录里现编 prune-server：外层这一条按会编的算，派本机、派第二台都给编译目录
    （同一条命令改前派本机不给、派第二台由 multi-host-run.sh 照例给，两台结果不同）；带了 --server-binary 的照旧不编、不给。"""
    world.resources(local=(30, 30), peer_down="自证：先只在本机跑")
    world.start_server()

    def wrapped(marker, *options):
        return ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, *options, "--",
                "bash", "-c", "exit 0", marker, "fake-peer-run"]

    local_bare = world.finished(world.submit(wrapped("wrapper-compiles-local"), cwd=world.repo))
    local_bound = world.finished(world.submit(wrapped("wrapper-compiles-bound", "--server-binary", "/usr/bin/true"), cwd=world.repo))
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    peer_bare = world.finished(world.submit(wrapped("wrapper-compiles-peer"), cwd=world.repo), timeout=PLACEMENT_WAIT_SECONDS)
    problems = []
    for name, summary, host, compiles in (("派本机、不带 --server-binary", local_bare, "local", True), ("派本机、带 --server-binary", local_bound, "local", False),
                                          ("派第二台、不带 --server-binary", peer_bare, "peer", True)):
        attempt = first_attempt(summary)
        build = attempt.get("build_directory") or ""
        if summary["task"].get("outcome") != "succeeded" or attempt.get("host") != host:
            problems.append(f"{name}：应当在{host}跑完，实际 {describe(summary)}、host {attempt.get('host')}")
        if bool(summary["task"].get("compiles")) != compiles or bool(build) != compiles:
            problems.append(f"{name}：compiles 应当是 {compiles}、尝试记着编译目录的应当是 {compiles}，实际 compiles {summary['task'].get('compiles')}、编译目录「{build}」")
    for name, summary, root in (("派本机", local_bare, world.builds), ("派第二台", peer_bare, world.peer_local)):
        directory = first_attempt(summary).get("build_directory") or ""
        if directory and not lib.inside(directory, root):
            problems.append(f"{name}给的编译目录应当在 {root} 下，实际 {directory}")
    if problems:
        raise SelftestFailure("临时 prune-server 包装现编：" + "；".join(problems), world.server_log_lines())
    return "不带 --server-binary 的包装派本机、派第二台都按会编的算、给编译目录；带 --server-binary 的不给"


def cell_temporary_service_own_server_compiles(world):
    """自带临时服务的两份脚本（task_server_lib.py 的 OWN_SERVER_SELFTESTS）单独的 --selftest 不带 --server-binary 与 PRUNE_SERVER_BINARY 时，自己在服务给的编译目录里
    现编 prune-server：外层这一条按会编的算，派本机、派第二台都给编译目录（同一条命令改前派本机不给、派第二台由 multi-host-run.sh 照例给，两台结果不同）；
    带了 --server-binary 的照旧不编、不给。命令前的赋值与任务环境里给了 PRUNE_SERVER_BINARY 的、--selftest-families、别的模式与别的脚本，认法是
    lib.temporary_service_builds_server 的纯函数，在这一格里直接核一张表。"""
    write_executable(os.path.join(world.repo, "scripts", "prune-server", LOAD_SIMULATION_SCRIPT), FAKE_LOAD_SIMULATION)
    world.resources(local=(30, 30), peer_down="自证：先只在本机跑")
    world.start_server()

    def own(script, mode, *options):
        return ["python3", f"scripts/prune-server/{script}", mode, *options]

    def submitted(script, *options):
        return world.submit(own(script, "--selftest", *options, "fake-peer-run"), cwd=world.repo)

    local_client = world.finished(submitted(lib.TEMPORARY_SERVICE_SCRIPT))
    local_load = world.finished(submitted(LOAD_SIMULATION_SCRIPT))
    local_bound = world.finished(submitted(lib.TEMPORARY_SERVICE_SCRIPT, "--server-binary", "/usr/bin/true"))
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    peer_client = world.finished(submitted(lib.TEMPORARY_SERVICE_SCRIPT, "on-the-peer"), timeout=PLACEMENT_WAIT_SECONDS)
    problems = []
    for name, summary, host, compiles in (("客户端自证派本机、不带 --server-binary", local_client, "local", True),
                                          ("压测脚本自证派本机、不带 --server-binary", local_load, "local", True),
                                          ("客户端自证派本机、带 --server-binary", local_bound, "local", False),
                                          ("客户端自证派第二台、不带 --server-binary", peer_client, "peer", True)):
        attempt = first_attempt(summary)
        build = attempt.get("build_directory") or ""
        if summary["task"].get("outcome") != "succeeded" or attempt.get("host") != host:
            problems.append(f"{name}：应当在{host}跑完，实际 {describe(summary)}、host {attempt.get('host')}")
        if bool(summary["task"].get("compiles")) != compiles or bool(build) != compiles:
            problems.append(f"{name}：compiles 应当是 {compiles}、尝试记着编译目录的应当是 {compiles}，实际 compiles {summary['task'].get('compiles')}、编译目录「{build}」")
    for name, summary, root in (("派本机", local_client, world.builds), ("派第二台", peer_client, world.peer_local)):
        directory = first_attempt(summary).get("build_directory") or ""
        if directory and not lib.inside(directory, root):
            problems.append(f"{name}给的编译目录应当在 {root} 下，实际 {directory}")
    binary_variable = lib.TEMPORARY_SERVICE_BINARY_VARIABLE
    table = (("客户端 --selftest", own(lib.TEMPORARY_SERVICE_SCRIPT, "--selftest"), {}, True),
             ("客户端 --selftest-families", own(lib.TEMPORARY_SERVICE_SCRIPT, "--selftest-families"), {}, True),
             ("压测脚本 --selftest", own(LOAD_SIMULATION_SCRIPT, "--selftest"), {}, True),
             ("客户端 --selftest 带 --server-binary", own(lib.TEMPORARY_SERVICE_SCRIPT, "--selftest", "--server-binary", "/usr/bin/true"), {}, False),
             ("压测脚本 --selftest 带 --server-binary", own(LOAD_SIMULATION_SCRIPT, "--selftest", "--server-binary", "/usr/bin/true"), {}, False),
             ("命令前的赋值给了 PRUNE_SERVER_BINARY", ["env", f"{binary_variable}=/usr/bin/true", *own(lib.TEMPORARY_SERVICE_SCRIPT, "--selftest")], {}, False),
             ("任务环境里给了 PRUNE_SERVER_BINARY", own(LOAD_SIMULATION_SCRIPT, "--selftest"), {binary_variable: "/usr/bin/true"}, False),
             ("压测脚本不是 --selftest（压测用法）", own(LOAD_SIMULATION_SCRIPT, "--count", "5"), {}, False),
             ("别的脚本的 --selftest", ["python3", "scripts/prune-server/other-script.py", "--selftest"], {}, False),
             ("包装不带 --server-binary（照旧）", own(lib.TEMPORARY_SERVICE_SCRIPT, lib.TEMPORARY_SERVICE_FLAG, "--", "true"), {}, True))
    for name, command, task_environment, expected in table:
        found = lib.temporary_service_builds_server(command, task_environment)
        if found != expected:
            problems.append(f"{name}：temporary_service_builds_server 应当交 {expected}，实际 {found}（{command}）")
    if problems:
        raise SelftestFailure("自带临时服务的自证现编：" + "；".join(problems), world.server_log_lines())
    return (f"客户端与压测脚本不带二进制的 --selftest 派本机、派第二台都按会编的算、给编译目录；带 --server-binary 或 PRUNE_SERVER_BINARY 的、"
            f"别的模式与别的脚本不给（纯函数表 {len(table)} 条对）")


BYPASS_WARNING_HEAD = "✗ 严厉警告："   # task_server.py 的 bypass_warning_lines 头一行的开头
BYPASS_NEXT_STEP_HEAD = "→ 怎么办："
# repository-target-bypass-warned 写进假仓根的几份入口脚本：命中的（缺省值退回裸写的 target/release/、$(dirname …)/.. 拼到仓根 target/debug/）、
# 只用服务给的 CARGO_TARGET_DIR 的，与自证那一支里造假路径的 python（--selftest 起的不读正文；名字以 cells 收尾的函数体不读，同 memory_peaks_lib.py 的 key_fold_cells）
BYPASS_SCRIPT_TEXT = ('#!/usr/bin/env bash\n# 自证样本：缺省取仓根 target/ 下的二进制（任务 7937–7944 那种写法）\n'
                      'BINARY="${SAMPLE_BINARY:-target/release/sample_tool}"\nFALLBACK="$(dirname "$0")/../target/debug/sample_tool"\necho "would run $BINARY or $FALLBACK"\n')
SERVICE_TARGET_SCRIPT_TEXT = '#!/usr/bin/env bash\n# 自证样本：只取服务给的编译目录\nBINARY="${CARGO_TARGET_DIR:-/nonexistent}/release/sample_tool"\necho "would run $BINARY"\n'
SELFTEST_SAMPLE_SCRIPT_TEXT = ('import sys\n\n\ndef key_fold_cells(repository):\n'
                               '    return [f"{repository}/target/debug/deps/journal_ring_fields-0123456789abcdef", "--exact", "ring"]\n\n\n'
                               'if __name__ == "__main__":\n    sys.exit(0)\n')


def cell_repository_target_bypass_warned(world):
    """提交时警告（task_server.py 文件头 ②「提交时警告」）：命令里的词落在仓根 target/ 下的、命令位置上起的本仓脚本正文里有指向仓根 target/ 的路径字面的，照收照跑，
    提交回复（客户端打到标准错误）、日志头、status（给号的与会话列表）三处都打「✗ 严厉警告：」那一行（点名会话号与任务号）与「→ 怎么办：」那一行；
    四种不算的（服务给的编译目录下、调用方给的 CARGO_TARGET_DIR 下、草稿目录里仓副本的 target/ 与在副本里起的脚本、自证那一支的假路径）都不打。
    弄坏开关 repository-target-bypass-unwarned 下不判，命中的那两条也不打。"""
    copy = os.path.join(world.work_root, "draft-bypass", "copy")   # 草稿目录里的仓副本：它的 target/ 与在它里面起的同一份脚本都不算
    for root, relative, text in ((world.repo, "scripts/long-run-bypass.sh", BYPASS_SCRIPT_TEXT), (world.repo, "scripts/service-target.sh", SERVICE_TARGET_SCRIPT_TEXT),
                                 (world.repo, "scripts/sample_lib.py", SELFTEST_SAMPLE_SCRIPT_TEXT), (copy, "scripts/long-run-bypass.sh", BYPASS_SCRIPT_TEXT)):
        os.makedirs(os.path.join(root, os.path.dirname(relative)), exist_ok=True)
        write_executable(os.path.join(root, relative), text)
    own_target = os.path.join(world.root, "own-target")
    world.start_server()
    hits = {"命令里的仓根 target/": (["env", f"SAMPLE_BINARY={world.repo}/target/debug/sample_tool", "bash", "-c", "exit 0"], world.repo),
            "入口脚本取仓根 target/": (["bash", "scripts/long-run-bypass.sh"], world.repo)}
    misses = {"服务给的编译目录": (["env", f"SAMPLE_BINARY={world.builds}/task-sample-1/target/release/sample_tool", "bash", "scripts/service-target.sh"], world.repo),
              "调用方给的 CARGO_TARGET_DIR": (["env", f"CARGO_TARGET_DIR={own_target}", f"SAMPLE_BINARY={own_target}/release/sample_tool", "bash", "-c", "exit 0"], world.repo),
              "草稿目录里的仓副本": (["env", f"SAMPLE_BINARY={copy}/target/release/sample_tool", "bash", "scripts/long-run-bypass.sh"], copy),
              "自证那一支的假路径（--selftest 起）": (["python3", "scripts/sample_lib.py", "--selftest"], world.repo),
              "自证那一支的假路径（cells 函数体）": (["python3", "scripts/sample_lib.py"], world.repo)}
    problems, submitted = [], {}
    for name, (command, cwd) in list(hits.items()) + list(misses.items()):
        completed = world.client(["submit", "--cwd", cwd, "--", *command])
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if completed.returncode != 0 or not found:
            problems.append(f"{name}：提交应当照收（退 0、打任务号），实际退 {completed.returncode}、{completed.stderr.strip()[-300:]}")
            continue
        world.confirm_submitted(completed)   # 照确认单确认（task_server.py 文件头 ⑦）：没确认的一直待确认、不进队；警告看的是提交回复的标准错误
        submitted[name] = (int(found.group(1)), completed.stderr)
    listed = world.client(["status"]).stdout
    for name, (task_id, stderr) in submitted.items():
        summary = world.finished(task_id)
        named = f"{BYPASS_WARNING_HEAD}会话 {SESSION_A} 交的任务 {task_id} "
        shown = world.client(["status", str(task_id)]).stdout
        log = world.client(["log", str(task_id), "--all"]).stdout.splitlines()
        head = log[1] if len(log) > 1 else ""
        places = {"提交回复的标准错误": stderr, "status <号>": shown, "会话的 status": listed, "日志头": head}
        if name in hits:
            if summary["task"].get("outcome") != "succeeded":
                problems.append(f"{name}：任务应当照跑（警告不拒），实际 {describe(summary)}")
            missing = [place for place, text in places.items() if named not in text]
            if missing or BYPASS_NEXT_STEP_HEAD not in stderr or not (len(log) > 2 and log[2].startswith(BYPASS_NEXT_STEP_HEAD)):
                problems.append(f"{name}（任务 {task_id}）：三处都应当打「{named}…」与「{BYPASS_NEXT_STEP_HEAD}…」，缺 {missing or '出路那一行'}；"
                                f"标准错误「{stderr.strip()[-300:]}」、日志头两行 {log[1:3]}")
        elif any("严厉警告" in text for text in (stderr, shown, "\n".join(log))) or f"交的任务 {task_id} " in listed:
            problems.append(f"{name}（任务 {task_id}）：不算绕过，不该打严厉警告，实际标准错误「{stderr.strip()[-300:]}」、status「{shown.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("提交时警告：" + "；".join(problems), world.server_log_lines())
    return (f"命中的 {len(hits)} 条照跑、提交回复、日志头、两种 status 都打了点名会话与任务号的严厉警告与出路；"
            f"不算的 {len(misses)} 条（服务给的编译目录、CARGO_TARGET_DIR、草稿里的仓副本、自证那一支两种）都不打")


CONFIRM_TIMEOUT_SAMPLE_SECONDS = "2"    # unconfirmed-cancelled：交了没确认满这么多秒服务取消（TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS）
CONFIRM_HOLD_SECONDS = 1.5         # submit-awaits-confirmation：待确认的那一条看这么久（十几轮排队）不该起
CONFIRM_SCOPE_SAMPLE = "种类=items; 跑什么=自证的确认单; 范围=2 项; 为什么=核确认单打不打范围声明"


def cell_submit_awaits_confirmation(world):
    """交任务的二次确认（task_server.py 文件头 ⑦）：交一条点了两项、带范围声明的，回话打确认单（命令、范围声明四段、项数 2、在排在跑 0 条）与 confirm-needed 一行（items=2 tasks=0），
    任务记 awaiting-confirmation、余量够也不起；同会话另交一条点了同一项、确认过的一直跑着；拿旧的两个数确认被拒（退 3）、打出服务此刻算的 tasks=1 与交叠的那一条的号；
    项数写错的同样拒；照服务算的两个数确认才进队、跑完。弄坏开关 confirm-skipped 下交了就进队、confirm-counts-unchecked 下旧数也收，这一格红。"""
    world.resources(local=(100, 100))
    world.start_server()
    problems = []
    submitted = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "confirm-sheet", "--item", "alpha", "--item", "beta"],
                             extra={"SINGLEFS_RUN_SCOPE": CONFIRM_SCOPE_SAMPLE})
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", submitted.stdout)
    task_id = int(found.group(1)) if found else 0
    needed = lib.confirm_needed_fields(submitted.stdout)
    if submitted.returncode != 0 or needed != (task_id, 2, 0):
        problems.append(f"交了应当退 0、打 confirm-needed task={task_id} items=2 tasks=0，实际退 {submitted.returncode}、读到 {needed}、「{submitted.stdout.strip()[-400:]}」")
    for wanted in ("确认单（任务", "命令：bash -c", "范围声明：种类=items", "项数 2：--item 2", "在排在跑 0 条"):
        if wanted not in submitted.stdout:
            problems.append(f"确认单里应当有「{wanted}」，实际「{submitted.stdout.strip()[-500:]}」")
    time.sleep(CONFIRM_HOLD_SECONDS)
    summary = world.summary(task_id) if task_id else {"task": {}, "attempts": []}
    if summary["task"].get("state") != lib.AWAITING_CONFIRMATION_STATE or summary["attempts"]:
        problems.append(f"没确认的应当一直待确认、一次都不起，实际 {describe(summary)}")
    release = os.path.join(world.root, "release-confirm-other")
    other = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release, "--item", "alpha"])
    world.running(other)
    stale = world.client(["confirm", str(task_id), "--items", "2", "--tasks", "0"])
    if stale.returncode != 3 or lib.confirm_needed_fields(stale.stdout) != (task_id, 2, 1) or f"{other}（都点了 --item alpha）" not in stale.stdout:
        problems.append(f"拿旧的条数确认应当被拒（退 3）、打服务此刻算的 items=2 tasks=1 与交叠的「{other}（都点了 --item alpha）」，"
                        f"实际退 {stale.returncode}、「{stale.stdout.strip()[-400:]}」「{stale.stderr.strip()[-200:]}」")
    wrong_items = world.client(["confirm", str(task_id), "--items", "3", "--tasks", "1"])
    if wrong_items.returncode != 3:
        problems.append(f"项数写错（3）应当被拒（退 3），实际退 {wrong_items.returncode}")
    confirmed = world.client(["confirm", str(task_id), "--items", "2", "--tasks", "1"])
    if confirmed.returncode != 0:
        problems.append(f"照服务算的两个数确认应当退 0，实际退 {confirmed.returncode}、「{confirmed.stderr.strip()[-300:]}」")
    done = world.finished(task_id) if task_id else {"task": {}}
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"确认之后应当进队、跑完，实际 {describe(done)}")
    open(release, "w").close()
    world.finished(other)
    if problems:
        raise SelftestFailure("交任务的二次确认：" + "；".join(problems), world.server_log_lines())
    return "交了待确认、不起；旧数与错的项数确认被拒并打出服务算的数与交叠的那一条；照服务算的数确认之后进队跑完"


def cell_unconfirmed_cancelled(world):
    """交了没确认（task_server.py 文件头 ⑦；异常表 unconfirmed）：满时限（自证里 2 秒）服务取消，结局 cancelled、说明以「没确认，已取消」起头、一次都没起；
    收件 socket 收到那一条结局，正文带「没确认，已取消」。弄坏开关 unconfirmed-kept 下一直待确认，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS": CONFIRM_TIMEOUT_SAMPLE_SECONDS})
    submitted = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "never-confirmed"], extra=inbox.environment())
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", submitted.stdout)
    if submitted.returncode != 0 or not found:
        raise SelftestFailure(f"交了应当退 0、打任务号，实际退 {submitted.returncode}", [submitted.stdout, submitted.stderr])
    task_id = int(found.group(1))
    summary = world.wait_for(task_id, lambda summary: summary["task"].get("state") == "finished", "没确认被服务取消", timeout=INBOX_WAIT_SECONDS)
    problems = []
    if summary["task"].get("outcome") != "cancelled" or not str(summary["task"].get("note") or "").startswith("没确认，已取消") or summary["attempts"]:
        problems.append(f"满时限没确认的应当记 cancelled、说明以「没确认，已取消」起头、一次都没起，实际 {describe(summary)}、说明「{summary['task'].get('note')}」")
    head = outcome_head(task_id, "cancelled")
    messages = inbox.wait_for(world, head)
    problems += inbox_problems(messages, head, "没确认被取消")
    if messages and "没确认，已取消" not in messages[0]["text"]:
        problems.append(f"推给会话的正文应当带「没确认，已取消」，实际「{messages[0]['text'][:300]}」")
    if problems:
        raise SelftestFailure("交了没确认：" + "；".join(problems), world.server_log_lines())
    return f"交了满 {CONFIRM_TIMEOUT_SAMPLE_SECONDS} 秒没确认：服务取消、推了「没确认，已取消」"


LONG_TASK_JUDGED = {lib.CONFIRM_RULE_SELFTEST_VARIABLE: ""}   # 判长短的几格起服务时写空串：交任务的二次确认只拦长任务，照真的判（小世界缺省写 all：不分长短一律待确认）
LONG_TASK_WHOLE_SCOPE = "种类=whole; 跑什么=自证造的整趟; 范围=全部; 为什么=自证：核不指定范围的整趟判成长任务"
LONG_TASK_RECORD_KEYS = {"over": "selftest-long-record-over", "edge": "selftest-long-record-edge", "short": "selftest-long-item-recorded"}   # 记过用时的几把命令键（long-task-by-record、long-task-unrecorded-test-or-item）
LONG_TASK_DIRECT_FAKE_ENTRIES = (("short-gate-cell", [".claude/gate.d/selftest-single-cell.sh"], ["--check", "one-cell"]),
                                 ("short-hook-swap", ["scripts/gate/hook-swap.py"], ["--selftest"]))   # short-task-queued-directly：假仓根里的门禁单格与脚本（正文都是 exit 0）


def judged_id(world, what, completed):
    """判长短的几格共用：从提交回复里取任务号；交不成（退非 0、没有任务号）判红。"""
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
    if completed.returncode != 0 or not found:
        raise SelftestFailure(f"{what}提交应当退 0、打任务号，实际退 {completed.returncode}", [completed.stdout, completed.stderr] + world.server_log_lines())
    return int(found.group(1))


def judged_submit(world, what, command, cwd=None, extra=None):
    """判长短的几格共用：只交、不照确认单确认 → (CompletedProcess, 任务号)。"""
    completed = world.client(["submit", "--cwd", cwd or world.work, "--", *command], extra=extra)
    return completed, judged_id(world, what, completed)


def judged_long_problems(world, what, completed, task_id, reason):
    """判成长任务的应当（task_server.py 文件头 ⑦「只有长任务要确认」）：打确认单与 confirm-needed 一行、确认单有「判成长任务：<依据>」、不打「直接进队：」，
    任务记待确认、一次都没起。核完取消它。→ 不对的那几句。"""
    problems = []
    if lib.confirm_needed_fields(completed.stdout) is None:
        problems.append(f"{what}应当判成长任务、打确认单与 confirm-needed 一行，实际「{completed.stdout.strip()[-400:]}」")
    if f"判成长任务：{reason}" not in completed.stdout:
        problems.append(f"{what}的确认单应当写「判成长任务：{reason}」，实际「{completed.stdout.strip()[-500:]}」")
    if lib.DIRECT_QUEUE_MARK in completed.stdout:
        problems.append(f"{what}是长任务、不该打「{lib.DIRECT_QUEUE_MARK}」，实际「{completed.stdout.strip()[-400:]}」")
    summary = world.summary(task_id)
    if summary["task"].get("state") != lib.AWAITING_CONFIRMATION_STATE or summary["attempts"]:
        problems.append(f"{what}应当待确认、一次都没起，实际状态 {summary['task'].get('state')}、{describe(summary)}")
    world.client(["cancel", str(task_id)])
    return problems


def judged_direct_problems(world, what, completed, task_id, reason):
    """没判成长任务的应当：不打确认单、打一行「直接进队：<依据>」，任务起了尝试、跑到收尾。不对的先取消、不等它。→ 不对的那几句。"""
    problems = []
    if lib.confirm_needed_fields(completed.stdout) is not None:
        problems.append(f"{what}不是长任务、不该打确认单，实际「{completed.stdout.strip()[-500:]}」")
    if f"{lib.DIRECT_QUEUE_MARK}{reason}" not in completed.stdout:
        problems.append(f"{what}应当打一行「{lib.DIRECT_QUEUE_MARK}{reason}」，实际「{completed.stdout.strip()[-500:]}」")
    if problems:
        world.client(["cancel", str(task_id)])
        return problems
    done = world.finished(task_id)
    if not done["attempts"]:
        problems.append(f"{what}直接进队应当起了尝试、跑到收尾，实际 {describe(done)}")
    return problems


def cell_long_task_by_record(world):
    """交任务的二次确认只拦长任务，按记过的用时判（task_server.py 文件头 ⑦ 的 long_task_judgement）：同一命令键库里近几次跑到退出的用时取最大，超过 1800 秒的判成长任务：
    较早一次 2400 秒、最近一次 120 秒的（取最大，不取最近一次）待确认、确认单写「判成长任务：同一命令键库里记过的用时近 2 次最长 2400 秒，超过 1800 秒」、一次都没起；
    对照：较早 600 秒、最近恰好 1800 秒的（超过才算）直接进队，提交输出打「直接进队：这把命令键记过的用时近 2 次最长 1800 秒，不超过 1800 秒」、跑完。
    弄坏开关 long-task-record-unread 下记过多久都不看，long-task-latest-only 下只看最近一次，short-task-held 下对照那一条也待确认，confirm-skipped 下待确认的那一条也进队，
    direct-reason-unreported 下不打「直接进队：」，long-task-reason-unsaid 下确认单不写依据，这一格红。"""
    over_command, edge_command = (["bash", "-c", "exit 0", name] for name in ("long-record-over", "long-record-edge"))
    for key, command, runs in ((LONG_TASK_RECORD_KEYS["over"], over_command, ((2400.0, SEEDED_ENDED_AT - 1000), (120.0, SEEDED_ENDED_AT))),
                               (LONG_TASK_RECORD_KEYS["edge"], edge_command, ((600.0, SEEDED_ENDED_AT - 1000), (float(lib.LONG_TASK_SECONDS), SEEDED_ENDED_AT)))):
        for seconds, ended_at in runs:
            world.seed_attempt({"command_key": key, "command": list(command), "outcome": "succeeded", "memory_peak": None, "seconds": seconds, "ended_at": ended_at})
    world.start_server(extra=LONG_TASK_JUDGED)
    completed, task_id = judged_submit(world, "记过 2400 秒与 120 秒的", over_command, extra={lib.KEY_VARIABLE: LONG_TASK_RECORD_KEYS["over"]})
    problems = judged_long_problems(world, "记过 2400 秒与 120 秒的", completed, task_id,
                                    lib.LONG_TASK_RECORD_REASON.format(count=2, seconds=2400.0, limit=lib.LONG_TASK_SECONDS))
    completed, task_id = judged_submit(world, "记过 600 秒与 1800 秒的", edge_command, extra={lib.KEY_VARIABLE: LONG_TASK_RECORD_KEYS["edge"]})
    problems += judged_direct_problems(world, "记过 600 秒与 1800 秒的", completed, task_id,
                                       lib.SHORT_TASK_RECORD_REASON.format(count=2, seconds=float(lib.LONG_TASK_SECONDS), limit=lib.LONG_TASK_SECONDS))
    if problems:
        raise SelftestFailure("按记过的用时判长任务：" + "；".join(problems), world.server_log_lines())
    return "记过的用时近几次最大 2400 秒的待确认、确认单写依据；最大恰好 1800 秒的直接进队、打「直接进队：」、跑完"


def cell_long_task_whole_run(world):
    """交任务的二次确认只拦长任务，判整趟（long_task_judgement）：不指定范围的整趟——范围声明种类是 whole、带 --full-gate、带 --whole-package-confirmed 三种各交一条——都判成长任务、
    待确认、确认单写「判成长任务：不指定范围的整趟（<命中的那一种>）」、一次都没起；对照：同一条 bash -c、范围声明种类是 items 的直接进队、跑完。
    弄坏开关 long-task-whole-unjudged 下三种都直接进队，short-task-held 下对照那一条也待确认，confirm-skipped 下三种也进队，direct-reason-unreported 下对照那一条不打「直接进队：」，
    long-task-reason-unsaid 下确认单不写依据，这一格红。"""
    world.start_server(extra=LONG_TASK_JUDGED)
    confirmation = write_confirmation(world, "long-task-whole.md", WHOLE_PACKAGE_CONFIRMATION_TEXT)
    problems = []
    for what, command, where, scope, confirmed, gate, mark in (
            ("范围声明种类是 whole 的", ["bash", "-c", "exit 0", "whole-kind"], world.work, LONG_TASK_WHOLE_SCOPE, "", False, "范围声明种类是 whole"),
            ("带 --full-gate 的", ["bash", FAKE_COMMIT_GATE], world.repo, SCOPE_DAILY, "", True, "带 --full-gate"),
            ("带 --whole-package-confirmed 的", ["bash", "scripts/crash/crash-point-replay.sh"], world.repo, SCOPE_DAILY, confirmation, False, "带 --whole-package-confirmed")):
        completed = submit_raw(world, command, where, confirmation=confirmed, scope=scope, full_gate=gate)
        task_id = judged_id(world, what, completed)
        problems += judged_long_problems(world, what, completed, task_id, lib.LONG_TASK_WHOLE_REASON.format(marks=mark))
    completed = submit_raw(world, ["bash", "-c", "exit 0", "whole-control"], world.work, scope=SCOPE_DAILY)
    problems += judged_direct_problems(world, "范围声明种类是 items 的对照", completed, judged_id(world, "对照", completed), lib.SHORT_TASK_UNRECORDED_REASON)
    if problems:
        raise SelftestFailure("不指定范围的整趟判长任务：" + "；".join(problems), world.server_log_lines())
    return "范围声明种类 whole、带 --full-gate、带 --whole-package-confirmed 的都待确认、确认单写依据；种类 items 的对照直接进队、跑完"


def cell_long_task_sharded(world):
    """交任务的二次确认只拦长任务，判分片（long_task_judgement）：服务要拆成分片的（假分片入口带 run）判成长任务、待确认、确认单写「判成长任务：服务要拆成分片（入口 <脚本>）」、一次都没起；
    对照：同一个入口带 --selftest（不分片）直接进队、跑完。
    弄坏开关 long-task-shard-unjudged 下分片的直接进队，sharding-unrecognized 下认不出分片的也直接进队，short-task-held 下对照那一条也待确认，confirm-skipped 下分片的也进队，
    direct-reason-unreported 下对照那一条不打「直接进队：」，long-task-reason-unsaid 下确认单不写依据，这一格红。"""
    world.start_server(extra=LONG_TASK_JUDGED)
    sharded = world.write_spec("spec-long-sharded.json", {"shards": [{}]})
    completed, task_id = judged_submit(world, "要分片的", sharded, cwd=world.repo)
    entry = os.path.realpath(os.path.join(world.repo, "fake-sharded.py"))
    problems = judged_long_problems(world, "要分片的", completed, task_id, lib.LONG_TASK_SHARD_REASON.format(entry=entry))
    completed, task_id = judged_submit(world, "不分片的对照", ["python3", "fake-sharded.py", "--selftest"], cwd=world.repo)
    problems += judged_direct_problems(world, "不分片的对照", completed, task_id, lib.SHORT_TASK_UNRECORDED_REASON)
    if problems:
        raise SelftestFailure("要分片的判长任务：" + "；".join(problems), world.server_log_lines())
    return "要分片的待确认、确认单写依据；同一个入口不分片的对照直接进队、跑完"


def cell_long_task_unrecorded_test_or_item(world):
    """交任务的二次确认只拦长任务，判没有用时记录的（long_task_judgement）：库里这把命令键没有用时记录、又是会编译的 cargo 测试（cargo test -p … --test …）或按项跑的门禁（命令里点了 --item）的，
    判成长任务、待确认、确认单写「判成长任务：没有用时记录、又是会编译的 cargo 测试」或「…按项跑的门禁（--item）」、一次都没起；
    对照：同样点了 --item、这把键记过 300 秒的直接进队、打「直接进队：这把命令键记过的用时近 1 次最长 300 秒，不超过 1800 秒」、跑完。
    弄坏开关 long-task-unrecorded-unjudged 下两种没记录的都直接进队，short-task-held 下对照那一条也待确认，confirm-skipped 下两种也进队，
    direct-reason-unreported 下对照那一条不打「直接进队：」，long-task-reason-unsaid 下确认单不写依据，这一格红。"""
    item_command = ["bash", "-c", "exit 0", "unrecorded-item", "--item", "alpha"]
    world.seed_attempt({"command_key": LONG_TASK_RECORD_KEYS["short"], "command": item_command, "outcome": "succeeded", "memory_peak": None, "seconds": 300.0})
    world.start_server(extra=LONG_TASK_JUDGED)
    problems = []
    completed = submit_raw(world, ["cargo", "test", "-p", "singlefs-verification", "--test", "selftest_target"], world.work)
    problems += judged_long_problems(world, "没有用时记录的 cargo 测试", completed, judged_id(world, "cargo 测试", completed),
                                     lib.LONG_TASK_UNRECORDED_REASON.format(what=lib.LONG_TASK_UNRECORDED_TESTS))
    completed, task_id = judged_submit(world, "没有用时记录、点了 --item 的", ["bash", "-c", "exit 0", "unrecorded-item-fresh", "--item", "alpha"])
    problems += judged_long_problems(world, "没有用时记录、点了 --item 的", completed, task_id, lib.LONG_TASK_UNRECORDED_REASON.format(what=lib.LONG_TASK_UNRECORDED_ITEMS))
    completed, task_id = judged_submit(world, "记过 300 秒、点了 --item 的对照", item_command, extra={lib.KEY_VARIABLE: LONG_TASK_RECORD_KEYS["short"]})
    problems += judged_direct_problems(world, "记过 300 秒、点了 --item 的对照", completed, task_id,
                                       lib.SHORT_TASK_RECORD_REASON.format(count=1, seconds=300.0, limit=lib.LONG_TASK_SECONDS))
    if problems:
        raise SelftestFailure("没有用时记录的 cargo 测试与按项跑的门禁判长任务：" + "；".join(problems), world.server_log_lines())
    return "没有用时记录的 cargo 测试、点了 --item 的待确认、确认单写依据；记过 300 秒的同样点 --item 的对照直接进队、跑完"


def cell_short_task_queued_directly(world):
    """交任务的二次确认只拦长任务，其余直接进队、不出确认单（long_task_judgement）：没有用时记录的 bash -c 小命令、假仓根里的门禁单格（--check）、scripts/gate/hook-swap.py、
    带 --selftest 的各交一条，都不打确认单、提交输出打「直接进队：没有用时记录，但不是会编译的 cargo 测试，也不是按项跑的门禁（--item）」、submit 那一行 state=queued、跑完 succeeded。
    弄坏开关 short-task-held 下都待确认，direct-reason-unreported 下不打「直接进队：」，这一格红。"""
    for relative, text in ((".claude/gate.d/selftest-single-cell.sh", "#!/usr/bin/env bash\nexit 0\n"), ("scripts/gate/hook-swap.py", "#!/usr/bin/env python3\n")):
        os.makedirs(os.path.dirname(os.path.join(world.repo, relative)), exist_ok=True)
        write_executable(os.path.join(world.repo, relative), text)
    world.start_server(extra=LONG_TASK_JUDGED)
    problems = []
    for what, command in (("bash -c 小命令", ["bash", "-c", "exit 0", "short-bash"]), ("带 --selftest 的", ["bash", "-c", "exit 0", "short-selftest", "--selftest"]),
                          ("门禁单格（--check）", ["bash", ".claude/gate.d/selftest-single-cell.sh", "--check", "one-cell"]),
                          ("scripts/gate/hook-swap.py", ["python3", "scripts/gate/hook-swap.py", "--selftest"])):
        completed, task_id = judged_submit(world, what, command, cwd=world.repo)
        if "state=queued" not in completed.stdout:
            problems.append(f"{what}直接进队的提交行应当写 state=queued，实际「{completed.stdout.strip()[-300:]}」")
        problems += judged_direct_problems(world, what, completed, task_id, lib.SHORT_TASK_UNRECORDED_REASON)
    if problems:
        raise SelftestFailure("不是长任务的直接进队：" + "；".join(problems), world.server_log_lines())
    return "bash -c 小命令、门禁单格、hook-swap.py、带 --selftest 的都不出确认单、打「直接进队：」、state=queued、跑完"


def cell_confirm_on_queued_task_accepted(world):
    """task confirm 对直接进队的任务照旧认得（task_server.py 的 confirm，文件头 ⑦）：在跑的、已跑完的直接进队任务，提交它的会话 confirm（给什么数都不核）回「已在队里，不用确认」、退 0；
    别的会话 confirm 仍退 3；对照：判成长任务的待确认任务，拿错的两个数确认仍退 3、打服务算的确认单。
    弄坏开关 confirm-direct-refused 下对直接进队的退 3，confirm-skipped 下待确认的那一条也进队、错的数也收，short-task-held 下直接进队的那一条待确认，这一格红。"""
    world.start_server(extra=LONG_TASK_JUDGED)
    release = os.path.join(world.root, "release-confirm-direct")
    completed, direct = judged_submit(world, "直接进队的", ["bash", "-c", BLOCK_SCRIPT, "block", release])
    if lib.confirm_needed_fields(completed.stdout) is not None:
        raise SelftestFailure("没有用时记录的 bash -c 小命令应当直接进队，实际打了确认单", [completed.stdout, completed.stderr])
    problems = []
    try:
        world.running(direct)
        for what, session, code in (("在跑的", SESSION_A, 0), ("别的会话对在跑的", SESSION_B, CLIENT_REFUSED_EXIT)):
            confirmed = world.client(["confirm", str(direct), "--items", "7", "--tasks", "9"], session=session)
            if confirmed.returncode != code or (code == 0 and "已在队里，不用确认" not in confirmed.stdout):
                problems.append(f"{what}直接进队的 confirm 应当退 {code}{'、回「已在队里，不用确认」' if code == 0 else ''}，"
                                f"实际退 {confirmed.returncode}：「{(confirmed.stdout + confirmed.stderr).strip()[-300:]}」")
    finally:
        open(release, "w").close()
    world.finished(direct)
    after = world.client(["confirm", str(direct), "--items", "0", "--tasks", "0"])
    if after.returncode != 0 or "已在队里，不用确认" not in after.stdout:
        problems.append(f"已跑完的直接进队任务 confirm 应当退 0、回「已在队里，不用确认」，实际退 {after.returncode}：「{(after.stdout + after.stderr).strip()[-300:]}」")
    completed, long_id = judged_submit(world, "待确认的对照", ["bash", "-c", "exit 0", "still-long", "--item", "alpha"])
    wrong = world.client(["confirm", str(long_id), "--items", "99", "--tasks", "99"])
    if wrong.returncode != CLIENT_REFUSED_EXIT or lib.confirm_needed_fields(wrong.stdout) is None:
        problems.append(f"待确认的任务拿错的两个数确认应当退 {CLIENT_REFUSED_EXIT}、打确认单，实际退 {wrong.returncode}：「{(wrong.stdout + wrong.stderr).strip()[-300:]}」")
    world.client(["cancel", str(long_id)])
    if problems:
        raise SelftestFailure("task confirm 对直接进队的任务：" + "；".join(problems), world.server_log_lines())
    return "直接进队的任务（在跑的、跑完的）confirm 回「已在队里，不用确认」、退 0，别的会话退 3；待确认的拿错的数照旧退 3"


CONFIRM_LOCK_PROBE_SECONDS = 5.0                   # concurrent-confirms-serialized：头一条停在核过数之后，第二条至多等这么久看它自己退没退


def cell_full_gate_flag(world):
    """全量门禁标识（task_server.py 的 full_gate_refusal；异常表 full-gate-not-commit-entry）：提交时门禁带 --full-gate 照收、status 打「全量门禁」；
    范围声明写了提交前而没带它的退 3、出路写 --full-gate；带它又带 --check 的、带在别的命令上的都退 3。
    弄坏开关 full-gate-any-command-accepted 下后两条照收，这一格红。"""
    world.start_server()
    problems = []
    gate = ["bash", FAKE_COMMIT_GATE]
    accepted = submit_raw(world, gate, world.repo, full_gate=True)
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", accepted.stdout)
    if accepted.returncode != 0 or not found or "全量门禁" not in world.client(["status", found.group(1)]).stdout:
        problems.append(f"提交时门禁带 --full-gate 应当照收、status 打「全量门禁」，实际退 {accepted.returncode}：「{accepted.stderr.strip()[-200:]}」")
    declared = submit_raw(world, gate, world.repo, scope=SCOPE_COMMIT, full_gate=False)
    if declared.returncode != 3 or "--full-gate" not in declared.stderr:
        problems.append(f"范围声明写了提交前而没带 --full-gate 应当退 3、出路写它，实际退 {declared.returncode}：「{declared.stderr.strip()[-200:]}」")
    undeclared = submit_raw(world, gate, world.repo, full_gate=False)
    if undeclared.returncode != 3 or "--full-gate" not in undeclared.stderr:
        problems.append(f"不指定范围的提交时门禁没带 --full-gate（范围声明没写提交前）也应当退 3、出路写它，实际退 {undeclared.returncode}：「{undeclared.stderr.strip()[-200:]}」")
    confirmed_only = submit_raw(world, ["bash", "scripts/crash/crash-amplification.sh"], world.repo, full_gate=False,
                                confirmation=write_confirmation(world, "amplification-confirmed.md", WHOLE_PACKAGE_CONFIRMATION_TEXT))
    if confirmed_only.returncode != 3 or "--full-gate" not in confirmed_only.stderr:
        problems.append(f"崩溃放量不带 --item、只带整包确认文件而没带 --full-gate 应当退 3、出路写它，实际退 {confirmed_only.returncode}：「{confirmed_only.stderr.strip()[-200:]}」")
    for what, command in (("带 --check", [*gate, "--check", "lkmm-cell"]), ("别的命令", ["bash", "-c", "exit 0"])):
        completed = submit_raw(world, command, world.repo, full_gate=True)
        if completed.returncode != 3 or "--full-gate" not in completed.stderr:
            problems.append(f"{what}带 --full-gate 应当退 3，实际退 {completed.returncode}：「{completed.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("全量门禁标识：" + "；".join(problems), world.server_log_lines())
    return "提交时门禁带全量标识照收；写了提交前没带的、带在别处的都退 3"


def cell_commit_gate_query_forms_accepted(world):
    """提交时门禁只查不跑的写法不是不指定范围的那一趟（task_server_lib.py 的 query_only_entry，commit_gate_entry 与 unscoped_run_entry 照它不判；
    与 unnamed_run_problem 放行的同一组写法）：scripts/gate/gate-staged.sh --selftest、门禁阶段 --list 与 --tool-versions 不带 --full-gate 照收（只交、不确认）；
    对照：gate-staged.sh 不带参数没带 --full-gate 退 3、出路写它；只查不跑的带 --full-gate 退 3（全量标识只给不指定范围的那一趟）。
    弄坏开关 commit-gate-query-forms-judged 下只查不跑的三条照「没带 --full-gate」退 3，这一格红。"""
    world.start_server()
    write_fake_entry(world, "scripts/gate/gate-staged.sh")
    problems = []
    staged = ["bash", "scripts/gate/gate-staged.sh"]
    for what, command in (("gate-staged.sh --selftest", [*staged, "--selftest"]), ("门禁阶段 --list", ["bash", FAKE_COMMIT_GATE, "--list"]),
                          ("门禁阶段 --tool-versions", ["bash", FAKE_COMMIT_GATE, "--tool-versions"])):
        accepted = submit_raw(world, command, world.repo, full_gate=False)
        if accepted.returncode != 0 or not re.search(rf"{lib.SUBMIT_MARK} task=\d+", accepted.stdout):
            problems.append(f"{what}不带 --full-gate 应当照收（只查不跑，不是不指定范围的那一趟），实际退 {accepted.returncode}：「{accepted.stderr.strip()[-200:]}」")
    whole = submit_raw(world, staged, world.repo, full_gate=False)
    if whole.returncode != 3 or "--full-gate" not in whole.stderr:
        problems.append(f"gate-staged.sh 不带参数没带 --full-gate 应当退 3、出路写它，实际退 {whole.returncode}：「{whole.stderr.strip()[-200:]}」")
    flagged = submit_raw(world, [*staged, "--selftest"], world.repo, full_gate=True)
    if flagged.returncode != 3 or "--full-gate" not in flagged.stderr:
        problems.append(f"gate-staged.sh --selftest 带 --full-gate 应当退 3（全量标识只给不指定范围的那一趟），实际退 {flagged.returncode}：「{flagged.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("提交时门禁只查不跑的写法：" + "；".join(problems), world.server_log_lines())
    return "gate-staged.sh --selftest、门禁阶段 --list 与 --tool-versions 不带 --full-gate 照收；不带参数的照拒、只查不跑的带全量标识照拒"


def cell_manual_submit_path(world):
    """人手交的认法（task_server.py 的 manual_refusal；异常表 manual-from-session、sessionless-unmarked）：没有会话带 --manual 的照收、记无会话、跑完；
    会话里带 --manual 的退 3；没有会话也没带 --manual 的，环境里填了收件地址也退 3、出路写 --manual。
    弄坏开关 manual-from-session-accepted、sessionless-unmarked-accepted 下对应那一条照收，这一格红。"""
    world.start_server()
    problems = []
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "manual"], session=None))
    if summary["task"].get("outcome") != "succeeded" or summary["task"].get("session") != "" or summary["task"].get("inbox"):
        problems.append(f"人手带 --manual 应当照收、记无会话、不记收件地址，实际 {describe(summary)}、会话 {summary['task'].get('session')!r}")
    from_session = world.client(["submit", "--cwd", world.work, "--manual", "--", "bash", "-c", "exit 0"])
    if from_session.returncode != 3 or "--manual" not in from_session.stderr:
        problems.append(f"会话里带 --manual 应当退 3，实际退 {from_session.returncode}：「{from_session.stderr.strip()[-200:]}」")
    unmarked = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0"], session=None,
                            extra={lib.INBOX_SOCKET_VARIABLE: "/nonexistent/inbox.sock", lib.INBOX_TOKEN_VARIABLE: INBOX_TOKEN_SAMPLE})
    if unmarked.returncode != 3 or "--manual" not in unmarked.stderr:
        problems.append(f"没有会话、环境里填了收件地址而没带 --manual 应当退 3、出路写 --manual，实际退 {unmarked.returncode}：「{unmarked.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("人手交的认法：" + "；".join(problems), world.server_log_lines())
    return "人手带 --manual 照收、记无会话；会话带它、没有会话又没带它的都退 3"


def cell_concurrent_confirms_serialized(world):
    """同一会话的确认串行（缺陷七；task_server.py 的 session_lock）：同会话两条待确认、都照确认单（在排在跑 0 条）确认。头一条核过数之后停在改状态之前
    （TASK_SERVER_SELFTEST_PAUSE_POINT=confirm-counted），第二条在 CONFIRM_LOCK_PROBE_SECONDS 秒里应当一直等着会话锁、不退；放开之后头一条退 0，
    第二条照头一条进队之后的条数（1 条）核、对不上退 3、打出服务算的条数 1。弄坏开关 confirm-counts-outside-session-lock 下第二条不等、照 0 条核过退 0，这一格红。"""
    pause = os.path.join(world.root, "pause-confirm")
    os.makedirs(pause)
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"confirm-counted:{pause}"})
    submitted = [world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", f"concurrent-{name}"]) for name in ("first", "second")]
    needed = [lib.confirm_needed_fields(completed.stdout) for completed in submitted]
    if any(fields is None for fields in needed):
        raise SelftestFailure("两条都应当待确认、打出确认单那一行", [completed.stdout + completed.stderr for completed in submitted])
    (first, first_items, first_tasks), (second, second_items, second_tasks) = needed
    open(os.path.join(pause, "arm-confirm-counted"), "w").close()
    leading = world.client(["confirm", str(first), "--items", str(first_items), "--tasks", str(first_tasks)], background=True)
    problems = []
    try:
        wait_path(world, os.path.join(pause, "reached-confirm-counted"), "头一条核过数、停在改状态之前")
        trailing = world.client(["confirm", str(second), "--items", str(second_items), "--tasks", str(second_tasks)], background=True)
        deadline = time.monotonic() + CONFIRM_LOCK_PROBE_SECONDS
        while time.monotonic() < deadline and trailing.poll() is None:
            time.sleep(POLL_SECONDS)
        early = trailing.poll() is not None
    finally:
        open(os.path.join(pause, "release-confirm-counted"), "w").close()
    leading_out, _ = leading.communicate(timeout=90)
    trailing_out, trailing_err = trailing.communicate(timeout=90)
    recounted = lib.confirm_needed_fields(trailing_out + "\n" + trailing_err)
    if early:
        problems.append(f"头一条拿着会话锁时第二条应当等着、不退，实际 {CONFIRM_LOCK_PROBE_SECONDS:.0f} 秒里退了 {trailing.returncode}")
    if leading.returncode != 0:
        problems.append(f"头一条应当确认成、退 0，实际退 {leading.returncode}：「{leading_out.strip()[-200:]}」")
    if trailing.returncode != 3 or recounted is None or recounted[2] != second_tasks + 1:
        problems.append(f"第二条应当照头一条进队之后的条数核：退 3、确认单那一行写 tasks={second_tasks + 1}，实际退 {trailing.returncode}、"
                        f"那一行 {recounted}（{(trailing_out + trailing_err).strip()[-300:]}）")
    world.resources(local=(30, 30))
    world.finished(first)
    world.client(["cancel", str(second)])
    if problems:
        raise SelftestFailure("同会话的确认串行：" + "；".join(problems), world.server_log_lines())
    return "同会话两条并发确认：第二条等头一条改完状态、照进队之后的条数核，对不上退 3"


WHOLE_PACKAGE_SINGLE_CELLS = (   # whole-package-single-cells-refused：(说什么, 命令, 该不该拒)，都在假仓根起
    ("单点 feature-gated-tests", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "feature-gated-tests"], True),
    ("单点 product-gpu-real-card-tests", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "product-gpu-real-card-tests"], True),
    ("单点 crash-case-replay 不带 --crash-cases-only", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "crash-case-replay"], True),
    ("merge-and-verify.sh 带 --all-packages", ["bash", "scripts/gate/merge-and-verify.sh", "--all-packages", "selftest-patch-directory"], True),
    ("--check crash-case-replay --crash-cases-only --item crash-case:x", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "crash-case-replay",
                                                                         "--crash-cases-only", "--item", "crash-case:x"], False))
FEATURE_GATED_CHECK = ["bash", lib.WHOLE_PACKAGE_CRASH_GATE, ".", "--check", "feature-gated-tests"]   # whole-package-feature-items-* 两格在假仓根起的那一格
FEATURE_ITEMS_ACCEPTED_CASES = (   # whole-package-feature-items-accepted：(说什么, 命令, 该不该拒)
    ("--check feature-gated-tests 点一个挂特性的文件", FEATURE_GATED_CHECK + ["--item", lib.CRASH_GATE_FEATURE_ITEM_PREFIX + "selftest_feature.rs"], False),
    ("--check feature-gated-tests 点库与一个挂特性的文件（--item= 写法）", FEATURE_GATED_CHECK + ["--item=" + lib.CRASH_GATE_FEATURE_ITEM_PREFIX + "lib",
                                                                                    "--item", lib.CRASH_GATE_FEATURE_ITEM_PREFIX + "selftest_feature.rs"], False))
FEATURE_ITEMS_OTHERS_REFUSED_CASES = (   # whole-package-feature-items-others-refused：(说什么, 命令, 该不该拒)
    ("--check feature-gated-tests 只点崩溃枚举用例", FEATURE_GATED_CHECK + ["--item", "crash-case:x"], True),
    ("--check feature-gated-tests,crash-case-replay --crash-cases-only 只点崩溃枚举用例",
     ["bash", lib.WHOLE_PACKAGE_CRASH_GATE, ".", "--check", "feature-gated-tests,crash-case-replay", "--crash-cases-only", "--item", "crash-case:x"], True))
UNSCOPED_RUN_CASES = (   # unscoped-runs-refused：(说什么, 命令, 范围声明, 该不该拒)，都在假仓根起
    ("提交时的门禁不带 --check、带全量标识", ["bash", ".claude/gate.d/lkmm.sh"], SCOPE_COMMIT, False),
    ("crash-amplification.sh 不带 --item、带全量标识", ["bash", "scripts/crash/crash-amplification.sh"], SCOPE_COMMIT, False),
    ("crash-amplification.sh 带 --item", ["bash", "scripts/crash/crash-amplification.sh", "--item", "selftest-part"], SCOPE_DAILY, False),
    ("scripts/crash/crash-point-replay.sh 不带 --item", ["bash", "scripts/crash/crash-point-replay.sh"], SCOPE_DAILY, True),
    ("replay.sh 不给实验号", ["bash", "scripts/experiment-apparatus/replay.sh"], SCOPE_DAILY, True),
    ("replay.sh 给了实验号", ["bash", "scripts/experiment-apparatus/replay.sh", "E1"], SCOPE_DAILY, False))


def submitted_refusal_problems(world, cases, refusal_text):
    """逐条只交不确认（submit_raw）：该拒的应当退 3、stderr 带 refusal_text 与 --whole-package-confirmed 那条出路；该收的退 0、打任务号（收下的随即取消）。→ 不对的那几句。"""
    problems = []
    for case in cases:
        what, command, scope, refused = case if len(case) == 4 else (case[0], case[1], SCOPE_DAILY, case[2])
        completed = submit_raw(world, command, world.repo, scope=scope)
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if found:
            world.client(["cancel", found.group(1)])
        if refused and (completed.returncode != 3 or refusal_text not in completed.stderr or "--whole-package-confirmed" not in completed.stderr):
            problems.append(f"{what}应当退 3、stderr 写「{refusal_text}」与出路，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
        elif not refused and (completed.returncode != 0 or not found):
            problems.append(f"{what}应当照收、退 0，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
    return problems


def cell_whole_package_single_cells_refused(world):
    """整包测试照命令实际跑什么判（task_server_lib.py 的 whole_package_script；.claude/agent-common.md「日常开发的测试不跑整个包」）：crash-point-replay.sh 的 --check
    单点 feature-gated-tests、单点 product-gpu-real-card-tests、单点 crash-case-replay 而不带 --crash-cases-only，merge-and-verify.sh 带 --all-packages，没带确认标识都退 3；
    --check crash-case-replay --crash-cases-only --item crash-case:x 照收。弄坏开关 whole-package-cells-all-three-only 下单点的三条照收，
    merge-all-packages-accepted 下带 --all-packages 的照收，这一格红。"""
    world.start_server()
    problems = submitted_refusal_problems(world, WHOLE_PACKAGE_SINGLE_CELLS, "整包测试没带确认标识")
    if problems:
        raise SelftestFailure("整包测试单点一格也算：" + "；".join(problems), world.server_log_lines())
    return f"{len(WHOLE_PACKAGE_SINGLE_CELLS) - 1} 种整包写法退 3，只判点名崩溃枚举用例的那一条照收"


def cell_whole_package_feature_items_accepted(world):
    """crash-point-replay.sh --check feature-gated-tests 点了 feature-gated:<文件名> 项的不是整包测试（lib.whole_package_script；那一格只编、只跑点名的挂特性文件）：
    FEATURE_ITEMS_ACCEPTED_CASES 两条（--item 与 --item= 两种写法）都照收、退 0。弄坏开关 whole-package-feature-items-uncounted 下照整包拒，这一格红。"""
    world.start_server()
    problems = submitted_refusal_problems(world, FEATURE_ITEMS_ACCEPTED_CASES, "整包测试没带确认标识")
    if problems:
        raise SelftestFailure("feature-gated-tests 点了挂特性的文件：" + "；".join(problems), world.server_log_lines())
    return f"--check feature-gated-tests 点了挂特性文件的 {len(FEATURE_ITEMS_ACCEPTED_CASES)} 种写法都照收"


def cell_whole_package_feature_items_others_refused(world):
    """crash-point-replay.sh --check 点了 feature-gated-tests、--item 只点了崩溃枚举用例的照旧是整包测试（那一格照跑整格）：FEATURE_ITEMS_OTHERS_REFUSED_CASES 两条都退 3、
    stderr 写「整包测试没带确认标识」与 --whole-package-confirmed 那条出路。弄坏开关 whole-package-any-item-exempts 下带了 --item 就照收，这一格红。"""
    world.start_server()
    problems = submitted_refusal_problems(world, FEATURE_ITEMS_OTHERS_REFUSED_CASES, "整包测试没带确认标识")
    if problems:
        raise SelftestFailure("feature-gated-tests 没点挂特性的文件：" + "；".join(problems), world.server_log_lines())
    return f"--check feature-gated-tests 只点崩溃枚举用例的 {len(FEATURE_ITEMS_OTHERS_REFUSED_CASES)} 种写法都退 3"


CARD_DECLARATION_SAMPLES = (   # card-declarations-refused：(脚本名, 文件头那一行)，各写一种卡的声明
    ("declared-card-lease.sh", f"{lib.CARD_DECLARATION_MARKS[0]} on-demand"),
    ("declared-gpu-cards.sh", f"{lib.CARD_DECLARATION_MARKS[1]} 2"),
    ("declared-gpu-arguments.sh", f"{lib.CARD_DECLARATION_MARKS[2]} *"),
)


def cell_card_declarations_refused(world):
    """入口文件头写了领卡、要卡的声明当场拒（task_server.py 文件头 ②「提交时拒」⑧；task_server_lib.py 的 card_declaration_problem）：假仓根下三份脚本
    各写一种（CARD_DECLARATION_SAMPLES），直接交与套在 bash -c 里交都退 3、不给任务号、stderr 写 lib.CARD_DECLARATION_REFUSAL、点名那份脚本与写了的那一种、
    出路写删掉那一行；同目录一份不写这几种的照收、跑完；客户端 lease take 认不出（退 2）。弄坏开关 card-declarations-accepted 下写了的照收，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    problems = []
    for name, header in CARD_DECLARATION_SAMPLES:
        write_executable(os.path.join(world.repo, name), f"#!/usr/bin/env bash\n{header}\nexit 0\n")
        for form, command in (("直接交", ["bash", name]), ("bash -c 里交", ["bash", "-c", f"bash {name}"])):
            completed = submit_raw(world, command, world.repo)
            found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
            if found:
                world.client(["cancel", found.group(1)])
            texts = (lib.CARD_DECLARATION_REFUSAL, name, header.split()[1], "删掉")
            if completed.returncode != 3 or found or not all(text in completed.stderr for text in texts):
                problems.append(f"{name}（{form}）应当退 3、不给任务号、stderr 写「{'」「'.join(texts)}」，实际退 {completed.returncode}、"
                                f"stderr「{completed.stderr.strip()[-300:]}」")
    write_executable(os.path.join(world.repo, "declared-nothing.sh"), "#!/usr/bin/env bash\nexit 0\n")
    try:
        plain = world.finished(world.submit(["bash", "declared-nothing.sh"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY}, cwd=world.repo))
        if plain["task"].get("outcome") != "succeeded":
            problems.append(f"不写卡的声明的那一份应当照收、跑完，实际 {describe(plain)}")
    except SelftestFailure as failure:
        problems.append(f"不写卡的声明的那一份应当照收，实际 {failure}")
    lease = world.client(["lease", "take"])
    if lease.returncode != lib.USAGE_EXIT:
        problems.append(f"客户端 lease take 应当认不出、退 {lib.USAGE_EXIT}，实际退 {lease.returncode}、「{(lease.stdout + lease.stderr).strip()[-200:]}」")
    if problems:
        raise SelftestFailure("卡的声明当场拒：" + "；".join(problems), world.server_log_lines())
    return f"{len(CARD_DECLARATION_SAMPLES)} 种卡的声明直接交与 bash -c 里交都退 3、写明卡归 GPU 服务；不写的照收；lease take 认不出"


def cell_unscoped_runs_refused(world):
    """不指定范围的跑法在提交之外交当场拒（task_server_lib.py 的 unscoped_run_entry 与 UNSCOPED_RUN_ENTRIES；.claude/main-agent.md「一轮怎么开、怎么收」）：
    UNSCOPED_RUN_CASES 里不带选条参数的都退 3、stderr 写「不指定范围的跑法没带确认标识」，带了选条参数的与提交时的门禁（崩溃放量入口在内，带全量标识）照收；
    不是提交时门禁的入口（scripts/crash/crash-point-replay.sh）不指定范围、带上整包确认文件照收；提交时门禁没带全量标识的拒法归 full-gate-flag 那一格。弄坏开关 unscoped-gate-accepted 下不带选条参数的照收，这一格红。"""
    world.start_server()
    problems = submitted_refusal_problems(world, UNSCOPED_RUN_CASES, "不指定范围的跑法没带确认标识")
    confirmed = submit_raw(world, ["bash", "scripts/crash/crash-point-replay.sh"], world.repo,
                           confirmation=write_confirmation(world, "unscoped-confirmed.md", WHOLE_PACKAGE_CONFIRMATION_TEXT))
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", confirmed.stdout)
    if found:
        world.client(["cancel", found.group(1)])
    if confirmed.returncode != 0 or not found:
        problems.append(f"不指定范围的那一条带上整包确认文件应当照收、退 0，实际退 {confirmed.returncode}：「{confirmed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("不指定范围的跑法：" + "；".join(problems), world.server_log_lines())
    return "不带选条参数的门禁与入口退 3，带了选条参数的、提交时的门禁、带了确认文件的照收"


UNNAMED_RUN_JUDGED = {lib.UNNAMED_RUN_SELFTEST_VARIABLE: ""}   # unnamed-* 几格起服务时写空串：提交时拒 ⑦ 照真的判（小世界缺省写 off）
UNNAMED_PATCH_DIRECTORY = "selftest-patch-directory"


def unnamed_run_problems(world, cases):
    """unnamed-* 几格共用（task_server.py 文件头 ②「提交时拒」⑦，判据 lib.unnamed_run_problem）：起照真的判 ⑦ 的服务，逐条只交不确认（submit_raw；
    cargo 与 bash -c 在这一格的工作目录起，同 whole-package-named-accepted，别的在假仓根起）。
    cases 是 (说什么, 命令, 拒绝说明里要有的改法；照收的写 None)：该拒的退 3、stderr 带 lib.UNNAMED_RUN_PROBLEM_HEAD 与那一句改法；该收的退 0、打任务号（随即取消）。→ 不对的那几句。"""
    world.start_server(extra=UNNAMED_RUN_JUDGED)
    problems = []
    for what, command, way_out in cases:
        completed = submit_raw(world, command, world.work if command[0] == "cargo" or command[:2] == ["bash", "-c"] else world.repo)
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if found:
            world.client(["cancel", found.group(1)])
        if way_out is not None and (completed.returncode != 3 or lib.UNNAMED_RUN_PROBLEM_HEAD not in completed.stderr or way_out not in completed.stderr):
            problems.append(f"{what}应当退 3、stderr 写「{lib.UNNAMED_RUN_PROBLEM_HEAD}」与改法「{way_out}」，实际退 {completed.returncode}：「{completed.stderr.strip()[-400:]}」")
        elif way_out is None and (completed.returncode != 0 or not found):
            problems.append(f"{what}应当照收、退 0，实际退 {completed.returncode}：「{completed.stderr.strip()[-400:]}」")
    return problems


def cell_unnamed_merge_unnamed_refused(world):
    """合入后验证不带 --item 当场拒（lib.unnamed_run_problem 的 UNNAMED_RUN_ITEM_ENTRIES）：给了补丁目录而没带 --item 的退 3、说明给出加 --item <包>:<目标根文件> 的样子。
    弄坏开关 unnamed-entry-item-unchecked、unnamed-run-refusal-skipped 下照收，这一格红。"""
    problems = unnamed_run_problems(world, (("合入后验证给了补丁目录、不带 --item", ["bash", FAKE_MERGE_ENTRY, UNNAMED_PATCH_DIRECTORY], "--item <包>:<目标根文件>"),))
    if problems:
        raise SelftestFailure("合入后验证不带 --item：" + "；".join(problems), world.server_log_lines())
    return "合入后验证不带 --item 退 3，说明给出加 --item 的样子"


def cell_unnamed_merge_named_accepted(world):
    """合入后验证带 --item 照收（--item <项> 与 --item=<项> 两种写法）。弄坏开关 unnamed-entry-item-uncounted 下带了也拒，这一格红。"""
    problems = unnamed_run_problems(world, (
        ("合入后验证带 --item", ["bash", FAKE_MERGE_ENTRY, "--skip-apply", "--item", "selftest-package:tests/selftest_target.rs", UNNAMED_PATCH_DIRECTORY], None),
        ("合入后验证带 --item=", ["bash", FAKE_MERGE_ENTRY, "--item=selftest-package:src/lib.rs", UNNAMED_PATCH_DIRECTORY], None)))
    if problems:
        raise SelftestFailure("合入后验证带 --item：" + "；".join(problems), world.server_log_lines())
    return "合入后验证带 --item 的两种写法都照收"


def cell_unnamed_cargo_target_refused(world):
    """cargo test 没点到 --exact 的单条测试函数当场拒（lib.unnamed_cargo_test_problem）：只点 --test 目标的、--lib 带筛选词的、bash -c 里只点目标的都退 3，
    说明给出 `-- <测试全名> --exact` 的样子。弄坏开关 unnamed-cargo-filter-accepted、unnamed-run-refusal-skipped 下照收，shell-inner-scope-unread 下 bash -c 那一条照收，这一格红。"""
    target = ["cargo", "test", "-p", "singlefs-verification", "--test", "selftest_target"]
    problems = unnamed_run_problems(world, (
        ("cargo test 只点 --test 目标", target, "-- <测试全名> --exact"),
        ("cargo test --lib 带筛选词不带 --exact", ["cargo", "test", "-p", "singlefs-verification", "--lib", "--", "selftest_name"], "-- selftest_name --exact"),
        ("bash -c 里 cargo test 只点 --test 目标", ["bash", "-c", shlex.join(target)], "-- <测试全名> --exact")))
    if problems:
        raise SelftestFailure("cargo test 没点到单条测试函数：" + "；".join(problems), world.server_log_lines())
    return "cargo test 只点目标、只给筛选词、bash -c 里只点目标的都退 3，说明给出 --exact 的样子"


def cell_unnamed_cargo_exact_accepted(world):
    """cargo test `--` 之后带 --exact 与测试全名的照收（两条全名一起点也算）；只编不跑的 --no-run 照收。弄坏开关 unnamed-cargo-exact-uncounted 下点名的也拒，这一格红。"""
    base = ["cargo", "test", "-p", "singlefs-verification", "--test", "selftest_target"]
    problems = unnamed_run_problems(world, (
        ("cargo test 点一条 --exact", base + ["--", "selftest_module::selftest_case", "--exact"], None),
        ("cargo test 点两条 --exact", base + ["--", "--exact", "selftest_module::replay_case", "selftest_module::merge_case"], None),
        ("cargo test --no-run", base + ["--no-run"], None)))
    if problems:
        raise SelftestFailure("cargo test 点名 --exact：" + "；".join(problems), world.server_log_lines())
    return "cargo test 点一条、两条 --exact 与 --no-run 都照收"


def cell_unnamed_prove_red_unnamed_refused(world):
    """证红不给变异名当场拒（lib.unnamed_run_problem 照 prove-red.sh 收参数的写法：--copy <副本> <crate> 之后至少一个变异名）：退 3、说明给出 <变异名…> 的样子。
    弄坏开关 unnamed-prove-red-unchecked、unnamed-run-refusal-skipped 下照收，这一格红。"""
    problems = unnamed_run_problems(world, (("证红不给变异名", ["bash", lib.UNNAMED_RUN_PROVE_RED_ENTRY, "--copy", world.repo, "singlefs-verification"], "<变异名…>"),))
    if problems:
        raise SelftestFailure("证红不给变异名：" + "；".join(problems), world.server_log_lines())
    return "证红不给变异名退 3，说明给出写变异名的样子"


def cell_unnamed_prove_red_named_accepted(world):
    """证红点名一个变异照收。弄坏开关 unnamed-prove-red-name-offset 下只点一个的算没点、拒，这一格红。"""
    problems = unnamed_run_problems(world, (("证红点名一个变异", ["bash", lib.UNNAMED_RUN_PROVE_RED_ENTRY, "--copy", world.repo, "singlefs-verification", "selftest-row"], None),))
    if problems:
        raise SelftestFailure("证红点名一个变异：" + "；".join(problems), world.server_log_lines())
    return "证红点名一个变异照收"


def cell_unnamed_exemptions_accepted(world):
    """两种例外不判点没点名：提交时的门禁不指定范围、带 --full-gate 照收；合入后验证不带 --item、带三段写全的 --whole-package-confirmed 照收；
    同一条合入后验证两样都不带的退 3（对照）。弄坏开关 unnamed-run-exemptions-ignored 下带了标识的也拒，这一格红。"""
    world.start_server(extra=UNNAMED_RUN_JUDGED)
    confirmation = write_confirmation(world, "unnamed-confirmed.md", WHOLE_PACKAGE_CONFIRMATION_TEXT)
    merge = ["bash", FAKE_MERGE_ENTRY, UNNAMED_PATCH_DIRECTORY]
    problems = []
    for what, completed, accepted in (
            ("提交时的门禁带 --full-gate", submit_raw(world, ["bash", ".claude/gate.d/crash-point-replay.sh"], world.repo, scope=SCOPE_COMMIT), True),
            ("合入后验证带 --whole-package-confirmed", submit_raw(world, merge, world.repo, confirmation=confirmation), True),
            ("合入后验证两样都不带", submit_raw(world, merge, world.repo), False)):
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if found:
            world.client(["cancel", found.group(1)])
        if accepted and (completed.returncode != 0 or not found):
            problems.append(f"{what}应当照收、退 0，实际退 {completed.returncode}：「{completed.stderr.strip()[-400:]}」")
        elif not accepted and (completed.returncode != 3 or lib.UNNAMED_RUN_PROBLEM_HEAD not in completed.stderr):
            problems.append(f"{what}应当退 3、写「{lib.UNNAMED_RUN_PROBLEM_HEAD}」，实际退 {completed.returncode}：「{completed.stderr.strip()[-400:]}」")
    if problems:
        raise SelftestFailure("点名的两种例外：" + "；".join(problems), world.server_log_lines())
    return "带 --full-gate 的提交时门禁与带整包确认文件的合入后验证照收，两样都不带的退 3"


def cell_unnamed_gate_cells_judged(world):
    """门禁阶段（lib.TEST_STARTING_GATE_CELLS）：--check 点了起测试的格（qemu-device-streams 的 device-streams）而没带 --item 的退 3、说明给出 --item 的样子；
    --item、--check 都不带的（doc-text 全部格）退 3；只点不起测试的格（crash-point-replay 的 crash-knobs）、带 --item 的照收。
    弄坏开关 unnamed-check-cells-accepted 下点起测试的格照收，unnamed-stage-whole-accepted 下两样都不带的照收，unnamed-run-refusal-skipped 下都照收，这一格红。"""
    problems = unnamed_run_problems(world, (
        ("--check 点起测试的格、不带 --item", ["bash", ".claude/gate.d/qemu-device-streams.sh", ".", "--check", "device-streams"], "--item <项>"),
        ("--item、--check 都不带", ["bash", ".claude/gate.d/doc-text.sh", "."], "--item <项>"),
        ("只点不起测试的格", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "crash-knobs"], None),
        ("带 --item", ["bash", ".claude/gate.d/qemu-device-streams.sh", ".", "--item", "selftest-mode"], None)))
    if problems:
        raise SelftestFailure("门禁阶段点没点名：" + "；".join(problems), world.server_log_lines())
    return "点起测试的格不带 --item、两样都不带的退 3，只点不起测试的格、带 --item 的照收"


UNNAMED_CRASH_GATE = ["bash", lib.WHOLE_PACKAGE_CRASH_GATE, "."]   # unnamed-crash-gate-* 两格在假仓根起的门禁 crash-point-replay


def cell_unnamed_crash_gate_whole_items_refused(world):
    """门禁 crash-point-replay 带了 --item 而 --check 没把起整个验证包的格排掉的当场拒（lib.crash_gate_whole_package_cells）：只点崩溃枚举用例、不带 --check 的
    （照旧九格，连按条复用那一趟）与带 --crash-cases-only 而不带 --check 的（产品 GPU 与挂特性两格照跑整格）都退 3，说明给出加 --check crash-case-replay --crash-cases-only 的样子。
    弄坏开关 unnamed-crash-gate-item-whole-accepted、unnamed-run-refusal-skipped 下照收，这一格红。"""
    problems = unnamed_run_problems(world, (
        ("点崩溃枚举用例、不带 --check", UNNAMED_CRASH_GATE + ["--item", "crash-case:x"], lib.CRASH_GATE_CASES_ONLY_FORM),
        ("带 --crash-cases-only、不带 --check", UNNAMED_CRASH_GATE + ["--crash-cases-only", "--item=crash-case:x"], lib.CRASH_GATE_CASES_ONLY_FORM)))
    if problems:
        raise SelftestFailure("门禁 crash-point-replay 带 --item 而没排掉整包的格：" + "；".join(problems), world.server_log_lines())
    return "门禁 crash-point-replay 点崩溃枚举用例而不带 --check 的两种写法都退 3，说明给出加 --check crash-case-replay --crash-cases-only 的样子"


def cell_unnamed_crash_gate_cases_only_accepted(world):
    """门禁 crash-point-replay 带了 --item 而起整个验证包的格都排掉了的照收：--check crash-case-replay --crash-cases-only 点崩溃枚举用例；
    不带 --check、只点挂特性的文件（只跑 feature-gated-tests 一格、只编点名的文件）。弄坏开关 unnamed-crash-gate-cases-only-uncounted 下前一条拒，这一格红。"""
    problems = unnamed_run_problems(world, (
        ("--check crash-case-replay --crash-cases-only 点崩溃枚举用例", UNNAMED_CRASH_GATE + ["--check", "crash-case-replay", "--crash-cases-only", "--item", "crash-case:x"], None),
        ("不带 --check、只点挂特性的文件", UNNAMED_CRASH_GATE + ["--item", lib.CRASH_GATE_FEATURE_ITEM_PREFIX + "selftest_feature.rs"], None)))
    if problems:
        raise SelftestFailure("门禁 crash-point-replay 只跑点名的项：" + "；".join(problems), world.server_log_lines())
    return "门禁 crash-point-replay 带 --crash-cases-only 点崩溃枚举用例、不带 --check 只点挂特性的文件都照收"


def cell_test_starting_cells_listed(world):
    """起测试的格表（lib.test_starting_gate_cells；手列，各道文件头的格名表不标哪一格起测试）里每一道的阶段文件在仓里、每个格名都在那一道文件头的格名表里
    （`# gate-cell:` 行，读法只用 scripts/gate/lib_stage_header.py 的 header_cells；那一道 --list 打的就是这张表，scripts/gate/gate-structure-check.py 判两边相同）。
    静态核，不起服务、不起门禁。弄坏开关 test-starting-cells-misnamed 下 lkmm 那一行的格名写错一个字，这一格红。"""
    header = lib.load_module("lib_stage_header", os.path.join(lib.REPOSITORY_ROOT, "scripts", "gate", "lib_stage_header.py"))
    table = lib.test_starting_gate_cells()
    problems = []
    for stage, cells in table.items():
        path = os.path.join(lib.REPOSITORY_ROOT, stage)
        if not os.path.isfile(path):
            problems.append(f"{stage} 不在仓里")
            continue
        listed = {name for _, name, _ in header.header_cells(path)}
        missing = [cell for cell in cells if cell not in listed]
        if missing:
            problems.append(f"{stage} 的格名表里没有 {'、'.join(missing)}（有 {'、'.join(sorted(listed))}）")
    if problems:
        raise SelftestFailure("起测试的格表与各道的格名表对不上（照那一道的 --list 改 task_server_lib.py 的 TEST_STARTING_GATE_CELLS）：" + "；".join(problems))
    return f"起测试的格表 {len(table)} 道、{sum(len(cells) for cells in table.values())} 格，格名都在各自那一道的格名表里"


def cell_confirm_sheet_reads_bash_c_inner_commands(world):
    """确认单读 bash -c 里命令位置上的脚本与参数（task_server_lib.py 的 shell_inner_commands、named_scope、crash_scope_text，task_server.py 的 scope_is_whole；合入后验证常被包在 bash -c 里，
    先现生成补丁再合入）：bash -c 里 merge-and-verify.sh 给了补丁目录的不是整包（scope_is_whole 假）、没给补丁目录的是整包（真）、带 --all-packages 的是整包；
    重定向的目标不当补丁目录（`merge-and-verify.sh > out.log` 仍是没给）；`cd … && crash-point-replay.sh --item crash-case:a --item crash-case:b --crash-expand-up-to 5` 点名 2 项（named_scope）、
    确认单「崩溃用例与域」写「用例 2 条（命令里点名）；域 --crash-expand-up-to 5」；不是 -c 的命令不钻（shell_inner_commands 交空表）。套在假调度器上，不起服务。
    弄坏开关 shell-inner-scope-unread 下 bash -c 里的范围一律认不出（项数 0、整包的也判成脚本自己定范围），这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = dict(os.environ), lib.REPOSITORY_ROOT
    fake.cargo_runs = lambda task: []
    problems = []

    def whole(command):
        return SERVER.Scheduler.scope_is_whole(fake, {"command": command, "cwd": lib.REPOSITORY_ROOT})

    patch_form = ["bash", "-c", "python3 scripts/gate/patch-of-the-current-tree.py --out p.patch > /dev/null; bash scripts/gate/merge-and-verify.sh p.patch 2>&1 | tail -n 5"]
    if whole(patch_form):
        problems.append("bash -c 里 merge-and-verify.sh 给了补丁目录：不是整包，实际判成整包")
    for what, command in (("bash -c 里 merge-and-verify.sh 没给补丁目录", ["bash", "-c", "bash scripts/gate/merge-and-verify.sh"]),
                          ("bash -c 里 merge-and-verify.sh 带 --all-packages", ["bash", "-c", "bash scripts/gate/merge-and-verify.sh p.patch --all-packages"]),
                          ("bash -c 里 merge-and-verify.sh 只有重定向的目标", ["bash", "-c", "bash scripts/gate/merge-and-verify.sh > out.log"])):
        if not whole(command):
            problems.append(f"{what}：应当是整包，实际判成脚本自己定范围")
    item_form = ["bash", "-c", "cd /somewhere && bash scripts/crash/crash-point-replay.sh --item crash-case:a --item crash-case:b --crash-expand-up-to 5"]
    named = lib.named_scope(item_form)
    crash_text = lib.crash_scope_text(item_form, lib.REPOSITORY_ROOT)
    if len(named) != 2 or crash_text != "用例 2 条（命令里点名）；域 --crash-expand-up-to 5":
        problems.append(f"bash -c 里 crash-point-replay.sh 点了 2 个 --item 与 --crash-expand-up-to 5：项数应当是 2、崩溃用例与域写「用例 2 条（命令里点名）；域 --crash-expand-up-to 5」，实际 {named}、「{crash_text}」")
    if lib.shell_inner_commands(["bash", "scripts/gate/merge-and-verify.sh", "p.patch"]):
        problems.append("不是 -c 的命令不钻 bash 的脚本：shell_inner_commands 应当交空表")
    if problems:
        raise SelftestFailure("确认单读 bash -c 里的合入后验证：" + "；".join(problems))
    return "bash -c 里的合入后验证：给了补丁目录的不是整包、没给与带 --all-packages 的是整包、重定向目标不当补丁；crash-point-replay.sh 点的 --item 与域读得到"
