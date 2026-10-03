import hashlib, importlib.util, json, os, subprocess, sys, types
sys.dont_write_bytecode = True
root, home, driver_path, rustc_override, broken_text, source_read_path, sample_tools, sample_exclusions, sample_environment = sys.argv[1:10]
broken = set(filter(None, broken_text.split(",")))
howto_run = "主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步：--sync-tree --commit 到包着这一次暂存树的提交，再 --changed-since HEAD，跑完再跑这一道"
specification = importlib.util.spec_from_file_location("broad_mutation_driver_for_gate", driver_path)
driver = importlib.util.module_from_spec(specification)
try:
    specification.loader.exec_module(driver)
except Exception as error:  # 驱动被删、改坏
    print(f"  ✗ 读不进广谱变异的驱动 {driver_path}：{error}")
    print("     → 怎么办：它随仓走，被删了或改坏了从 git 找回来；导出范围、源码指纹、全局键与版本指纹的算法只认它一份，这一格不另写。")
    sys.exit(1)
source_path = os.path.join(home, "tree.source")
if "pending-unchecked" not in broken and os.path.exists(os.path.join(home, "tree.source.pending")):
    print(f"  ✗ {home} 里有 tree.source.pending：上一次 --sync-tree 没做完，库目录那一版判不了")
    print(f"     → 怎么办：{howto_run}（--sync-tree 重做一遍就把它收掉）")
    sys.exit(1)
try:
    with open(source_read_path, encoding="utf-8") as handle:
        source = json.load(handle)
    fingerprint = source["fingerprint"]
    recorded_paths = list(source["export_paths"])
except (OSError, ValueError, KeyError, TypeError) as error:
    print(f"  ✗ 读不出广谱变异库目录的版本指针 {source_path}：{error}")
    print(f"     → 怎么办：库目录换了地方就设 SINGLEFS_BROAD_MUTATION_HOME；还没建过就 {howto_run}")
    sys.exit(1)
version_directory = os.path.join(home, "versions", fingerprint)
problems = []
# 导出范围：驱动的 export_paths 在被判仓上现算（它读 REPOSITORY 下的登记表，这里把 REPOSITORY 换成被判仓）
if "export-range-unchecked" in broken:
    export_paths = recorded_paths
else:
    driver.REPOSITORY = root
    try:
        export_paths = driver.export_paths()
    except BaseException as error:  # 驱动拒绝时抛 Refusal 或 SystemExit
        print(f"  ✗ 在 {root} 上算不出导出范围（驱动的 export_paths）：{error}")
        print("     → 怎么办：看 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay.sh 那一行还在不在、成不成形；导出范围就取它。")
        sys.exit(1)
    if sorted(export_paths) != sorted(recorded_paths):
        problems.append(f"导出范围不同：库目录那一版记的是 {' '.join(recorded_paths)}，被判仓登记的是 {' '.join(export_paths)}")
# 被判那棵树在导出范围上的逐文件 sha256：读 git 索引（暂存区），不读工作区；忽略的目录名与符号链接照驱动的 manifest_of_tree
environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
ignored = set(driver.IGNORED_DIRECTORY_NAMES)
def ignored_path(path):
    return any(part in ignored for part in path.split("/")[:-1])
staged = {}
if "work-tree-read" in broken:
    for export_path in export_paths:
        full = os.path.join(root, export_path)
        if os.path.isfile(full):
            staged[export_path] = driver.file_digest(full)
        elif os.path.isdir(full):
            for relative, digest in driver.manifest_of_tree(full).items():
                staged[os.path.join(export_path, relative)] = digest
else:
    listed = subprocess.run(["git", "-C", root, "ls-files", "-s", "-z", "--", *export_paths], capture_output=True, env=environment)
    if listed.returncode != 0:
        print(f"  ✗ 读不了 {root} 的 git 索引（git ls-files 退 {listed.returncode}：{listed.stderr.decode(errors='replace').strip()[:200]}）")
        print("     → 怎么办：这一道判的是暂存树，被判的根要是 git 工作树（主工作区或 gate.sh --staged 的临时 worktree）。")
        sys.exit(1)
    objects = {}
    for record in listed.stdout.split(b"\0"):
        if not record:
            continue
        metadata, _, path = record.partition(b"\t")
        mode, object_name, stage_number = metadata.decode().split()
        name = os.fsdecode(path)
        if stage_number != "0" and "merge-conflict-unchecked" not in broken:
            print(f"  ✗ 索引里 {name} 在合并冲突里，暂存树判不了")
            print("     → 怎么办：先解决冲突并暂存，再按这一道的出路让驱动判这一次的暂存树。")
            sys.exit(1)
        if mode in ("120000", "160000") or ignored_path(name):
            continue
        objects[name] = object_name
    if objects:
        request = "".join(f"{object_name}\n" for object_name in objects.values()).encode()
        blobs = subprocess.run(["git", "-C", root, "cat-file", "--batch"], input=request, capture_output=True, env=environment)
        if blobs.returncode != 0:
            print(f"  ✗ git cat-file --batch 退 {blobs.returncode}，读不出索引里的文件内容")
            print("     → 怎么办：看 git 的报错（对象库坏了先 git fsck），修好再跑这一格。")
            sys.exit(1)
        output, position = blobs.stdout, 0
        for name, object_name in objects.items():
            end = output.index(b"\n", position)
            header = output[position:end].decode().split()
            size = int(header[2])
            staged[name] = hashlib.sha256(output[end + 1:end + 1 + size]).hexdigest()
            position = end + 1 + size + 1
if not staged:
    print(f"  ✗ 被判的暂存树在导出范围（{' '.join(export_paths)}）里一个文件都没有")
    print("     → 怎么办：看被判的根对不对（第一个参数）、导出范围登记得对不对；一个文件都没有就没有可判的变异。")
    sys.exit(1)
try:
    recorded = driver.read_manifest(os.path.join(version_directory, "manifest.tsv"))
except OSError as error:
    print(f"  ✗ 读不出库目录那一版的源码清单 {version_directory}/manifest.tsv：{error}")
    print(f"     → 怎么办：{howto_run}")
    sys.exit(1)
staged_source_fingerprint = driver.fingerprint_of_manifest(staged)
if "files-unchecked" not in broken:
    only_staged = sorted(set(staged) - set(recorded))
    only_recorded = sorted(set(recorded) - set(staged))
    differing = sorted(path for path in set(staged) & set(recorded) if staged[path] != recorded[path])
    if only_staged or only_recorded or differing:
        listing = [f"内容不同 {path}" for path in differing] + [f"暂存树有、那一版没有 {path}" for path in only_staged] + [f"那一版有、暂存树没有 {path}" for path in only_recorded]
        problems.append(f"源码与库目录那一版不同（{len(listing)} 个文件）：" + "；".join(listing[:20]) + ("；……" if len(listing) > 20 else ""))
    if not problems and "source-fingerprint-unchecked" not in broken and staged_source_fingerprint != source.get("source_fingerprint"):
        problems.append(f"逐文件相同而源码指纹不同（暂存树 {staged_source_fingerprint[:16]}，tree.source 记的 {str(source.get('source_fingerprint'))[:16]}）：manifest.tsv 与 tree.source 不是同一次写的")
toolchain_path = os.path.join(version_directory, "toolchain.txt")
current_toolchain = None
toolchain_matches = False
if "toolchain-unchecked" not in broken:
    if rustc_override:
        with open(rustc_override, encoding="utf-8") as handle:
            current_toolchain = handle.read().strip() + "\n"
    else:
        tree_directory = os.path.join(home, "tree")
        try:
            found = subprocess.run(["rustc", "-Vv"], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                                   cwd=tree_directory if os.path.isdir(tree_directory) else root)
            current_toolchain = found.stdout.strip() + "\n" if found.returncode == 0 and found.stdout.strip() else None
        except OSError:
            current_toolchain = None
        if current_toolchain is None:
            problems.append("起不了 rustc -Vv：此刻的工具链判不了")
    if current_toolchain is not None:
        if not os.path.isfile(toolchain_path):
            problems.append(f"那一版没记工具链（{toolchain_path} 不在）")
        else:
            with open(toolchain_path, encoding="utf-8") as handle:
                recorded_toolchain = handle.read()
            if recorded_toolchain != current_toolchain:
                problems.append(f"工具链不同：那一版记的是「{recorded_toolchain.splitlines()[0] if recorded_toolchain else '空'}」，此刻是「{current_toolchain.splitlines()[0]}」")
            else:
                toolchain_matches = True
# 全局键（E4）：那一版 --sync-tree 时记的全文与此刻现算的逐行相同；限时参数取那一版记的（门禁不知道主 agent 跑 ③ 给的限时参数，驱动真跑时自己核）
global_digest = source.get("global_key")
global_line_count = None
if toolchain_matches and not problems and "global-key-unchecked" not in broken:
    global_path = os.path.join(version_directory, "global-key.txt")
    if not global_digest or not os.path.isfile(global_path):
        problems.append(f"那一版没记全局键（tree.source 里{'有' if global_digest else '没有'} global_key，{global_path} {'在' if os.path.isfile(global_path) else '不在'}）：是全局键进版本指纹之前的驱动换的那一版")
    else:
        with open(global_path, encoding="utf-8") as handle:
            recorded_global = handle.read()
        if hashlib.sha256(recorded_global.encode()).hexdigest() != global_digest:
            problems.append(f"{global_path} 与 tree.source 记的 global_key 不是同一次写的")
        else:
            if sample_tools:
                with open(sample_tools, encoding="utf-8") as handle:
                    tools = [line.strip() for line in handle if line.strip()]
            else:
                tools = driver.tool_versions(types.SimpleNamespace(llvm_bin=None))
            limits = next((line for line in recorded_global.splitlines() if line.startswith("limits ")), "limits （那一版没记）")
            # 构建环境与驱动同一处算：库目录的 tree/（没有就是被判的根），此刻的环境；样本给了 .broad-sample/environment 的按它算（写死字面值的样本用）
            build_root = os.path.join(home, "tree") if os.path.isdir(os.path.join(home, "tree")) else root
            key_environment = os.environ
            if sample_environment:
                with open(sample_environment, encoding="utf-8") as handle:
                    key_environment = dict(line.rstrip("\n").replace("{root}", root).split("=", 1) for line in handle if "=" in line and not line.startswith("#"))
            try:
                build_environment = driver.build_environment_of(build_root, key_environment)
            except Exception as error:  # 驱动拒绝时抛 Refusal：配置文件读不了、RUSTC 的 -V 跑不出
                problems.append(f"算不出此刻的构建环境（驱动的 build_environment_of，在 {build_root} 上）：{error}")
                build_environment = None
            if build_environment is not None:
                current_global = driver.global_key_lines(driver.JUDGE_RULES_VERSION, current_toolchain, staged, build_environment, tools,
                                                         driver.exclusions_digest_of(sample_exclusions or driver.EXCLUSIONS_FILE), limits)
                if current_global != recorded_global:
                    recorded_lines, current_lines = recorded_global.splitlines(), current_global.splitlines()
                    changed = [f"那一版记的「{line}」" for line in recorded_lines if line not in current_lines] + [f"此刻是「{line}」" for line in current_lines if line not in recorded_lines]
                    problems.append(f"全局键与那一版记的不同（排除表、工具版本、构建环境、升档常量或判法版本变了，那一版的结论不按此刻的全局键重判）："
                                    + "；".join(changed[:8] or ["逐行相同的行一样、次序不同"]))
                else:
                    global_line_count = len(current_global.splitlines())
if (toolchain_matches and not problems and "files-unchecked" not in broken and "version-key-unchecked" not in broken
        and driver.version_key(home, staged_source_fingerprint, current_toolchain, global_digest) != fingerprint):
    problems.append(f"按驱动的 version_key 算出的版本指纹不是 tree.source 记的 {fingerprint[:16]}")
if problems:
    print(f"  ✗ 广谱变异库目录 {home} 此刻那一版（{fingerprint[:16]}，导出自 {str(source.get('commit'))[:12]}）不是被判的这棵暂存树：")  # gate-lint:summary
    for problem in problems:
        print(f"     {problem}")  # gate-lint:detail
    print(f"     → 怎么办：{howto_run}；暂存区之后又变了就重来一遍")
    sys.exit(1)
print(f"  ✓ 广谱变异库目录那一版（{fingerprint[:16]}，导出自 {str(source.get('commit'))[:12]}）就是被判的暂存树：导出范围 {' '.join(export_paths)} 里 {len(staged)} 个文件逐个相同，工具链相同"
      + (f"，全局键 {global_line_count} 行逐行相同" if global_line_count is not None else ""))
