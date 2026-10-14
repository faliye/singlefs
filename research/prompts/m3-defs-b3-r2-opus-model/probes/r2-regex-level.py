# R2 正则层核对：把门禁 code-tooling 的 heavy-thread-defaults 格里几条正则原样抄过来（抄的那几行在冻结副本该文件第 1936、1940-1951、1966-1970 行一带，
# 跑之前核冻结副本那份文件的 sha256 与抄时一致），对 gate-samples/code-tooling/r2-thread-gaps 里几行判「认不认」。不跑整格（门禁格子 agent 不起）。
# 用法：python3 -I r2-regex-level.py <冻结副本>；每行 name=r2-regex case=<格> hit=<0|1>
import hashlib, re, sys
frozen = sys.argv[1]
PINNED = "a9a97e797929e1155dec4202644794f60cb9b793afe95a212e53c7b7de112792"
actual = hashlib.sha256(open(frozen + "/.claude/gate.d/code-tooling.sh", "rb").read()).hexdigest()
print(f"name=r2-regex gate-file-sha256-matches={int(actual == PINNED)}")
THREAD_VARIABLE = r"[A-Z0-9_]*(?:THREADS?|THREAD_CAP|JOBS|WORKERS)[A-Z0-9_]*"
FORMS = [r"\bnproc\b", r"\bcpu_count\s*\(", r"\bsched_getaffinity\b", r"\bavailable_parallelism\b", r"\bavailable_threads\b",
         r"\bmin\s*\(\s*16\b|\.min\(\s*16\s*\)",
         r"\$\{" + THREAD_VARIABLE + r":-[0-9]+\}"
         r"|[\"']--(?:threads|jobs|workers|lanes)[\"'][^\n]*?\bdefault\s*=\s*[0-9]+"
         r"|\b(?=[A-Z0-9_]*(?:DEFAULT|WHEN_UNSET|BY_PROCESSOR))(?=[A-Z0-9_]*(?:THREADS|JOBS|WORKERS))[A-Z0-9_]+\s*(?::\s*\w+\s*)?=\s*[0-9]+\b"
         r"|[\"'](?:threads|jobs|workers)[\"']\s*:\s*[0-9]+\b"]
LITERAL_FALLBACK = re.compile(r"unwrap_or\(\s*[1-9][0-9]*\s*\)|Err\(_\)\s*=>\s*[1-9][0-9]*\b|\bor\s+[1-9][0-9]*\b")
CAPPED = re.compile(r"capped\.sh(?P<close>[\"']?)(?:\s*,\s*[\"']?|\s+)[0-9]+\b")
OWN_VARIABLE_SHELL = re.compile(r"\$\{?(" + THREAD_VARIABLE + r")")
SHELL_ASSIGNMENT = re.compile(r"^[ \t]*(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?(?P<name>" + THREAD_VARIABLE + r")=(?P<rhs>.*)$", re.M)
CAP_OR_REGISTRY = re.compile(r"SINGLEFS_THREAD_CAP|heavy_thread_default|default_heavy_threads|heavy-threads\.sh|heavy_threads|capped\.sh")
KEPT = re.compile(r"nproc|.*capped\.sh|[0-9]+|--(?:threads|jobs|workers|lanes)|threads|jobs|workers|" + THREAD_VARIABLE)
def any_form(line):
    return int(any(re.search(form, line) for form in FORMS) or bool(LITERAL_FALLBACK.search(line)) or bool(CAPPED.search(line)))
for label, line in [("env-prefix-assignment", "SAMPLE_DEVICE_THREADS=8 cargo run -q --release --bin sample-device"),
                    ("cargo-dash-j", "cargo build -j 8"),
                    ("test-threads-literal", "cargo test -- --test-threads 8"),
                    ("colon-equals-default", ': "${CARGO_BUILD_JOBS:=8}"'),
                    ("control-colon-dash-default", 'jobs="${CARGO_BUILD_JOBS:-8}"'),
                    ("real-replay-e161-pin", 'E161_THREADS=10 bash scripts/hosts/run-with-memory-cap.sh "$REPLAY_MEMORY_CAP" cargo run -q --release -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split -- feasibility')]:
    print(f"name=r2-regex case=thread-default-forms:{label} hit={any_form(line)}")
# 命令前缀赋值被当成文件自己的局部变量（SHELL_ASSIGNMENT 认它、右边不读自己）
found = SHELL_ASSIGNMENT.search("SAMPLE_DEVICE_THREADS=8 cargo run -q --release --bin sample-device")
print(f"name=r2-regex case=env-prefix-counted-as-shell-local hit={int(bool(found) and found.group('name') not in found.group('rhs'))}")
# 「读自己的线程变量」那一路的 break 条件：行首 #[cfg(test)]，不看它后面是不是 mod
print(f"name=r2-regex case=cfg-test-on-single-use-breaks hit={int('#[cfg(test)]'.startswith('#[cfg(test)]'))}")
# 字符串 "scripts/hosts/capped.sh" 被 kept 留下，CAP_OR_REGISTRY 认它
literal = "scripts/hosts/capped.sh"
print(f"name=r2-regex case=capped-path-string-kept hit={int(KEPT.fullmatch(literal) is not None)}")
print(f"name=r2-regex case=capped-path-string-counts-as-fallback hit={int(bool(CAP_OR_REGISTRY.search(literal)))}")
