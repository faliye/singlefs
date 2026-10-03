#!/usr/bin/env python3
"""T3 探针：用假清单、假一步驱动副本驱动，在各时刻 KILL，再重跑同一条命令，核四件事：已有定论的条有没有被重跑、被打断的条有没有被当成已完成、
有没有哪一步在带着别的条改动的树上跑、树最后是不是原样。用法：resume_probe.py <副本驱动路径> <草稿工作目录> [只跑的组名…]"""
import difflib, importlib.util, json, os, shutil, subprocess, sys

DRIVER, WORK = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
ONLY = set(sys.argv[3:])
HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("__broad_copy_for_probe__", DRIVER)
drv = importlib.util.module_from_spec(spec)
spec.loader.exec_module(drv)
FINAL = set(drv.FINAL_OUTCOMES)
CORE, CHECKER = "crates/singlefs-core/src/lib.rs", "crates/singlefs-checker/src/lib.rs"
SOURCES = {CORE: "".join(f"pub fn f{n}() -> u32 {{\n    {n}0\n}}\n" for n in range(1, 7)),
           CHECKER: "".join(f"pub fn f{n}() -> u32 {{\n    {n}0\n}}\n" for n in (7, 8)), "Cargo.toml": "[workspace]\n"}
PLAN = [("1", "one-test", "fail-nextest"), ("2", "one-build", "exit:101"), ("5", "two-test-debug", "exit:101"), ("7", "one-test", "exit:253")]
EXPECTED = {"1": "caught", "2": "unviable", "3": "missed", "4": "missed", "5": "caught", "6": "missed", "7": "timeout", "8": "missed"}


def mutants():
    found, sequence = [], 0
    for package, path in (("singlefs-core", CORE), ("singlefs-checker", CHECKER)):
        original = SOURCES[path]
        for index, line in enumerate(original.split("\n")):
            if not line.strip().isdigit():
                continue
            sequence += 1
            start, end, replacement = (index + 1, 5), (index + 1, len(line) + 1), f"MUT_{sequence}()"
            mutated = drv.replace_span(original, start, end, replacement + drv.MUTATION_MARKER)
            diff = "".join(difflib.unified_diff(original.splitlines(keepends=True), mutated.splitlines(keepends=True), f"a/{path}", f"b/{path}"))
            found.append({"package": package, "file": path, "name": f"{path}:{index + 1}:5: replace value with {replacement}",
                          "span": {"start": {"line": start[0], "column": start[1]}, "end": {"line": end[0], "column": end[1]}},
                          "replacement": replacement, "diff": diff})
    return found


class Case:
    def __init__(self, name, sources=None):
        self.name, self.work = name, os.path.join(WORK, name)
        shutil.rmtree(self.work, ignore_errors=True)
        self.home, self.source = os.path.join(self.work, "home"), os.path.join(self.work, "source")
        self.write_source(sources or SOURCES)
        with open(os.path.join(self.work, "mutants.json"), "w") as handle:
            json.dump(mutants(), handle)
        self.plan(PLAN)
        self.env = dict(os.environ, PROBE_WORK=self.work, BROAD_MUTATION_STEP_DRIVER=os.path.join(HERE, "fake_step.py"),
                        BROAD_MUTATION_LIST_DRIVER=os.path.join(HERE, "fake_list.py"), BROAD_MUTATION_TREE_SOURCE=self.source,
                        BROAD_MUTATION_ALLOW_VOLATILE_HOME="1")
        for name in ("SINGLEFS_HEAVY_TESTS", "SINGLEFS_BROAD_MUTATION_HOME", "BROAD_MUTATION_BREAK", "KP", "KP_SEQ", "KP_OUTCOME"):
            self.env.pop(name, None)

    def write_source(self, sources):
        shutil.rmtree(self.source, ignore_errors=True)
        for path, text in sources.items():
            os.makedirs(os.path.dirname(os.path.join(self.source, path)), exist_ok=True)
            with open(os.path.join(self.source, path), "w") as handle:
                handle.write(text)

    def plan(self, rows):
        with open(os.path.join(self.work, "plan.tsv"), "w") as handle:
            handle.write("".join("\t".join(row) + "\n" for row in rows))

    def run(self, *arguments, kp=None, extra_env=None):
        """kp = (点名, 序号, 结局或 None)：经 kp_driver.py 起；否则直接起副本。交 (退出码, 输出)。"""
        env = dict(self.env, **(extra_env or {}))
        command = [sys.executable, DRIVER]
        if kp:
            command = [sys.executable, os.path.join(HERE, "kp_driver.py"), DRIVER]
            env.update(KP=kp[0], KP_SEQ=kp[1])
            if kp[2]:
                env["KP_OUTCOME"] = kp[2]
        done = subprocess.run(command + ["--home", self.home, "--timeout-floor", "5", "--lock-wait", "30", *arguments],
                              capture_output=True, text=True, stdin=subprocess.DEVNULL, env=env)
        return done.returncode, done.stdout + done.stderr

    def take_calls(self):
        path, calls = os.path.join(self.work, "calls.log"), []
        if os.path.exists(path):
            calls = [tuple(line.rstrip("\n").split("\t")) for line in open(path) if line.strip()]
            os.remove(path)
        return calls

    def results_path(self):
        versions = os.path.join(self.home, "versions")
        return [os.path.join(versions, name, "results.tsv") for name in sorted(os.listdir(versions))]

    def rows(self):
        return [row for path in self.results_path() for row in drv.read_table(path, len(drv.RESULT_COLUMNS))[0]]

    def states(self):
        return {row[0]: row[3] for row in self.rows()}

    def tree_report(self):
        """(树里登记的文件是不是都与源相同, 树里多出来的文件)"""
        tree, same, extra = os.path.join(self.home, "tree"), True, []
        for root, _dirs, files in os.walk(tree):
            for name in files:
                relative = os.path.relpath(os.path.join(root, name), tree)
                if relative not in SOURCES:
                    extra.append(relative)
                elif open(os.path.join(root, name)).read() != SOURCES[relative]:
                    same = False
        return same, extra


def judge_resume(case, rc_killed, killed_calls):
    """KILL 之后：记下现场，重跑同一条命令，核四件事，打一行。"""
    before = case.rows()
    final_before = {row[0] for row in before if row[3] in FINAL}
    stage_one_before = {seq for seq, state in {row[0]: row[3] for row in before}.items() if state == drv.STAGE_ONE_ONLY}
    inflight = os.path.exists(os.path.join(case.home, "inflight.json"))
    rc, output = case.run()
    calls = case.take_calls()
    rerun_done = sorted({call[0] for call in calls} & final_before, key=int)
    stage_one_again = sorted({call[0] for call in calls if call[1].startswith("one-")} & stage_one_before, key=int)
    dirty = [call for call in calls if call[2] != ("-" if call[0] == "0" else call[0])]
    duplicates = sorted(seq for seq in EXPECTED if sum(1 for row in case.rows() if row[0] == seq and row[3] in FINAL) > 1)
    same, extra = case.tree_report()
    hit = bool(rerun_done or stage_one_again or dirty or duplicates or extra or not same or rc != 0 or case.states() != EXPECTED)
    print(f"KILL {case.name}: 被杀那趟退 {rc_killed}、它调了 {len(killed_calls)} 步；杀后已有定论 {sorted(final_before, key=int)}、只过第一段 {sorted(stage_one_before)}、"
          f"在途记录{'在' if inflight else '不在'}；重跑退 {rc}；定论被重跑 {rerun_done}；第一段被重跑 {stage_one_again}；在脏树上跑的步 {dirty}；"
          f"定论行重复的条 {duplicates}；终态{'对' if case.states() == EXPECTED else '不对 ' + str(sorted(case.states().items()))}；"
          f"树{'原样' if same else '不是原样'}、多出文件 {extra} ⇒ {'打中' if hit else '没打中'}", flush=True)
    return output


KILLS = [  # (名字, kp 或 None, 计划里的杀点 (序号, 步名) 或 None)
    ("k01-存了原件还没记在途", ("apply:after-original", "3", None), None),
    ("k02-记了在途还没改文件", ("apply:after-record", "3", None), None),
    ("k03-改了文件还没起步", ("apply:after-mutate", "3", None), None),
    ("k04-第一段编译中", None, ("3", "one-build")),
    ("k05-第一段测试中", None, ("3", "one-test")),
    ("k06-落stage-one-only行之前", ("record:before", "3", "stage-one-only"), None),
    ("k07-落行的write与fsync之间", ("record:between-write-fsync", "3", "stage-one-only"), None),
    ("k08-落stage-one-only行之后", ("record:after", "3", "stage-one-only"), None),
    ("k09-第二段编译中", None, ("3", "two-build-debug")),
    ("k10-第二段release测试中", None, ("3", "two-test-release")),
    ("k11-落missed行之前", ("record:before", "3", "missed"), None),
    ("k12-落missed行之后恢复原件之前", ("record:after", "3", "missed"), None),
    ("k13-落caught行之后恢复原件之前", ("record:after", "1", "caught"), None),
    ("k14-恢复时拷了临时件还没换上", ("restore:before-replace", "3", None), None),
    ("k15-恢复时换上了还没删在途", ("restore:after-replace", "3", None), None),
    ("k16-恢复时删了在途记录还没删原件", ("restore:after-remove-record", "3", None), None),
    ("k17-第一段基线中", None, ("0", "one-test")),
    ("k18-第二段基线中", None, ("0", "two-test-debug")),
]


def group_kills():
    for name, kp, plan_kill in KILLS:
        case = Case(name)
        case.run("--sync-tree")
        if plan_kill:
            case.plan(PLAN + [(plan_kill[0], plan_kill[1], "kill-parent")])
        rc, _output = case.run(kp=kp)
        killed_calls = case.take_calls()
        case.plan(PLAN)
        judge_resume(case, rc, killed_calls)
    # 连杀两次：第 3 条第一段测试中杀；重起时在开跑恢复「拷了临时件还没换上」处再杀；第三次照常跑
    case = Case("k19-连杀两次-第二次杀在开跑恢复里")
    case.run("--sync-tree")
    case.plan(PLAN + [("3", "one-test", "kill-parent")])
    case.run()
    case.take_calls()
    case.plan(PLAN)
    rc, _output = case.run(kp=("restore:before-replace", "startup", None))
    judge_resume(case, rc, case.take_calls())


def group_item_resume():
    """E1：点名跑被 KILL 之后重跑同一条命令。"""
    case = Case("e1-点名跑被杀后重跑同一条命令")
    case.run("--sync-tree")
    command = ["--item", "1", "--item", "2", "--item", "4", "--item", "6", "--skip-stage-two"]
    case.plan(PLAN + [("6", "one-test", "kill-parent")])
    rc1, _ = case.run(*command)
    case.take_calls()
    before = [row[0] + ":" + row[3] for row in case.rows()]
    case.plan(PLAN)
    rc2, _ = case.run(*command)
    calls = case.take_calls()
    print(f"E1 {case.name}: 命令「{' '.join(command)}」；被杀那趟退 {rc1}，杀后的行 {before}；重跑退 {rc2}，重跑里被调的序号 "
          f"{sorted({call[0] for call in calls}, key=int)}；重跑后的行 {[row[0] + ':' + row[3] for row in case.rows()]}", flush=True)


def summary_line(case):
    rc, output = case.run("--summary")
    return f"退 {rc} " + next((line for line in output.splitlines() if line.startswith("BROAD_MUTATION_SUMMARY")), "").split("total=")[-1]


def group_downgrade():
    """E2：点名重跑把已有定论盖成 stage-one-only / timeout；E11：点名 + --stage-two-only 给没跑过第一段的条记 missed。"""
    case = Case("e2-点名重跑盖掉定论")
    case.run("--sync-tree")
    case.run()
    case.take_calls()
    print(f"E2 全跑完：{summary_line(case)}")
    rc, _ = case.run("--item", "4", "--item", "5", "--skip-stage-two")
    case.take_calls()
    print(f"E2 再跑「--item 4 --item 5 --skip-stage-two」（第 4 条原是 missed、第 5 条原是第二段 caught）退 {rc}：{summary_line(case)}")
    rc, _ = case.run()
    calls = case.take_calls()
    print(f"E2 之后照常重跑退 {rc}，被调的 {sorted({(call[0], call[1]) for call in calls})}：{summary_line(case)}")
    case.plan(PLAN + [("1", "one-test", "exit:253")])
    rc, _ = case.run("--item", "1", "--skip-stage-two")
    case.take_calls()
    print(f"E2 第 1 条原是 caught，机器忙时点名重跑（假一步答 253）退 {rc}：{summary_line(case)}；第 1 条的行 {[row[3] for row in case.rows() if row[0] == '1']}")
    rc, _ = case.run()
    print(f"E2 之后照常重跑退 {rc}，被调的序号 {sorted({call[0] for call in case.take_calls()})}（timeout 是定论，不再跑）")
    case = Case("e11-点名加stage-two-only")
    case.run("--sync-tree")
    rc, _ = case.run("--item", "1", "--item", "3", "--stage-two-only")
    calls = case.take_calls()
    print(f"E11 新库上「--item 1 --item 3 --stage-two-only」退 {rc}；被调的步 {sorted({call[1] for call in calls})}；行 {[row[0] + ':' + row[3] + '@' + row[4] for row in case.rows()]}"
          f"（第 1 条照计划第一段会被抓到）")


def group_environment():
    """E3：基线绿过之后环境坏了（每条的编译都退 101 / 每条的测试都退 100）；E4：某一条的一步退认不出的码。"""
    for label, step, action in (("编译一律退 101（盘满这一类）", "one-build", "exit:101"), ("第一段测试一律退 100", "one-test", "fail-nextest")):
        case = Case("e3-" + step)
        case.run("--sync-tree")
        case.run("--item", "1", "--skip-stage-two")     # 基线在环境好的时候绿过
        case.take_calls()
        case.plan([("*", step, action)])
        rc, _ = case.run()
        calls = case.take_calls()
        print(f"E3 基线绿过之后{label}：照常跑退 {rc}，基线被重跑 {sum(1 for call in calls if call[0] == '0')} 步；{summary_line(case)}")
        case.plan(PLAN)
        rc, _ = case.run()
        print(f"E3 环境修好之后重跑同一条命令退 {rc}，被调 {len(case.take_calls())} 步；{summary_line(case)}")
    case = Case("e4-认不出的退出码")
    case.run("--sync-tree")
    case.plan(PLAN + [("3", "one-test", "exit:104")])
    for attempt in (1, 2, 3):
        rc, output = case.run()
        calls = case.take_calls()
        print(f"E4 第 3 条的 one-test 退 104，第 {attempt} 次跑：退 {rc}，被调的序号 {sorted({call[0] for call in calls}, key=int)}；{summary_line(case)}")
    same, extra = case.tree_report()
    print(f"E4 三次之后树{'原样' if same else '不是原样'}、多出文件 {extra}")


def group_keys():
    """E5：同一指纹下清单重新生成、次序变了；E7：结果表里留了半行。"""
    case = Case("e5-清单重生成次序变")
    case.run("--sync-tree")
    case.run()
    case.take_calls()
    print(f"E5 全跑完：{summary_line(case)}")
    version = os.path.dirname(case.results_path()[0])
    os.remove(os.path.join(version, "list.json"))      # 驱动第 751 行给的出路：删掉 list.json 再 --sync-tree
    reordered = list(reversed(mutants()[:6])) + mutants()[6:]    # 假设新一版 cargo-mutants 在同一份源码上给的次序不同
    with open(os.path.join(case.work, "mutants.json"), "w") as handle:
        json.dump(reordered, handle)
    rc, output = case.run("--sync-tree")
    items = json.load(open(os.path.join(version, "list.json")))
    names = {str(item["sequence"]): item["name"] for item in items}
    mismatched = sorted({row[0] for row in case.rows() if names.get(row[0]) != row[2]}, key=int)
    print(f"E5 删 list.json、清单次序变了之后 --sync-tree 退 {rc}（指纹目录还是 {os.path.basename(version)[:16]}）；{summary_line(case)}；"
          f"结果行的变异名与清单同序号的变异名对不上的序号 {mismatched}")
    rc, _ = case.run()
    print(f"E5 之后照常跑退 {rc}，被调 {len(case.take_calls())} 步")
    case = Case("e7-半行")
    case.run("--sync-tree")
    case.run()
    case.take_calls()
    with open(case.results_path()[0], "a") as handle:
        handle.write("4\tsinglefs-core\tx\tmis")          # 写到一半（盘满、断电）留下的半行
    print(f"E7 结果表末尾留半行：{summary_line(case)}")
    rc, _ = case.run("--item", "4", "--skip-stage-two")
    case.take_calls()
    print(f"E7 照汇总给的出路「--item 4」重跑退 {rc}：{summary_line(case)}")
    rc, _ = case.run()
    print(f"E7 照常重跑退 {rc}，被调 {len(case.take_calls())} 步：{summary_line(case)}")


def group_fake_driver():
    """E6：不带前缀、假一步指到 /bin/true，对一个还没有基线的库点名跑。"""
    case = Case("e6-假一步写库")
    case.run("--sync-tree")
    rc, output = case.run("--item", "7", "--skip-stage-two", extra_env={"BROAD_MUTATION_STEP_DRIVER": "/bin/true"})
    version = os.path.dirname(case.results_path()[0])
    baseline = [line.rstrip("\n").split("\t")[:2] for line in open(os.path.join(version, "baseline.tsv")) if not line.startswith("#")]
    print(f"E6 不带 SINGLEFS_HEAVY_TESTS、BROAD_MUTATION_STEP_DRIVER=/bin/true「--item 7 --skip-stage-two」退 {rc}；基线表 {baseline}；"
          f"结果行 {[row[0] + ':' + row[3] + ' 备注=' + row[10] for row in case.rows()]}")
    case.env["BROAD_MUTATION_BREAK"] = "stage-two-missed-as-caught"
    rc, _ = case.run("--item", "3")
    case.take_calls()
    print(f"E6 BROAD_MUTATION_BREAK=stage-two-missed-as-caught 进一次非自证的跑「--item 3」退 {rc}；第 3 条的行 {[row[3] + '@' + row[4] + ' 备注=' + row[10] for row in case.rows() if row[0] == '3']}")


def group_sync():
    """E8：--sync-tree 在「新树换上了、tree.source 还没写」处被杀；新提交相对旧提交只多文件 / 改了文件。"""
    for label, change in (("只多一个文件", {"crates/singlefs-harness/tests/new_case.rs": "#[test]\nfn new_case() { panic!() }\n"}),
                          ("改了一个已有文件", {CORE: SOURCES[CORE] + "pub fn f9() -> u32 {\n    90\n}\n"})):
        case = Case("e8-" + label)
        case.run("--sync-tree")
        old = json.load(open(os.path.join(case.home, "tree.source")))["fingerprint"][:16]
        case.write_source(dict(SOURCES, **change))
        rc1, _ = case.run("--sync-tree", kp=("sync:after-swap-in", "", None))
        rc2, output = case.run()
        calls = case.take_calls()
        now = json.load(open(os.path.join(case.home, "tree.source")))["fingerprint"][:16]
        same, extra = case.tree_report()
        print(f"E8 新提交{label}，--sync-tree 在新树换上之后被杀（退 {rc1}）；接着照常跑退 {rc2}、调了 {len(calls)} 步、落了 {len(case.rows())} 行；"
              f"tree.source 的指纹 {old} → {now}；树里多出文件 {extra}、登记的文件{'与旧源相同' if same else '与旧源不同'}")
    case = Case("e8-换树之前被杀")
    case.run("--sync-tree")
    rc1, _ = case.run("--sync-tree", kp=("sync:before-swap-in", "", None))
    rc2, output = case.run()
    print(f"E8 --sync-tree 在旧树挪走、新树还没换上处被杀（退 {rc1}）；接着照常跑退 {rc2}：{output.strip().splitlines()[0] if output.strip() else ''}")
    rc3, _ = case.run("--sync-tree")
    rc4, _ = case.run()
    print(f"E8 再 --sync-tree 退 {rc3}、照常跑退 {rc4}：{summary_line(case)}")


GROUPS = {"kills": group_kills, "item-resume": group_item_resume, "downgrade": group_downgrade, "environment": group_environment,
          "keys": group_keys, "fake-driver": group_fake_driver, "sync": group_sync}
if __name__ == "__main__":
    os.makedirs(WORK, exist_ok=True)
    for name, group in GROUPS.items():
        if not ONLY or name in ONLY:
            print(f"== {name}", flush=True)
            group()
