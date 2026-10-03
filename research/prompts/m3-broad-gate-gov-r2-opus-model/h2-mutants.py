#!/usr/bin/env python3
"""H2：在 base 树的硬链接拷贝里只换 research/scripts/broad-mutation.py，逐个造共用函数的写错版。
用法：mutants.py <base 树> <放拷贝的目录>；每个变体一棵树 <目录>/<名>，打「名 \t 换了哪一句」。旧串在驱动里必须恰好出现一次。"""
import os, shutil, subprocess, sys
base, out = sys.argv[1], sys.argv[2]
driver_rel = "research/scripts/broad-mutation.py"
text = open(os.path.join(base, driver_rel), encoding="utf-8").read()
M = [
 ("fop-content-blind", '    text = "".join(f"{manifest[path]}  {path}\\n" for path in sorted(manifest))\n',
                       '    text = "".join(f"{path}\\n" for path in sorted(manifest))\n'),
 ("fop-path-blind",    '    text = "".join(f"{manifest[path]}  {path}\\n" for path in sorted(manifest))\n',
                       '    text = "".join(f"{digest}\\n" for digest in sorted(manifest.values()))\n'),
 ("vk-no-toolchain",   '        return hashlib.sha256(f"{source_fingerprint}\\n{toolchain}judge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n".encode()).hexdigest()\n',
                       '        return hashlib.sha256(f"{source_fingerprint}\\njudge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n".encode()).hexdigest()\n'),
 ("vk-no-source",      '        return hashlib.sha256(f"{source_fingerprint}\\n{toolchain}judge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n".encode()).hexdigest()\n',
                       '        return hashlib.sha256(f"{toolchain}judge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n".encode()).hexdigest()\n'),
 ("vk-no-global",      '        return hashlib.sha256(f"{source_fingerprint}\\n{toolchain}judge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n".encode()).hexdigest()\n',
                       '        return hashlib.sha256(f"{source_fingerprint}\\n{toolchain}judge {JUDGE_RULES_VERSION}\\n".encode()).hexdigest()\n'),
 ("vk-no-judge",       '        return hashlib.sha256(f"{source_fingerprint}\\n{toolchain}judge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n".encode()).hexdigest()\n',
                       '        return hashlib.sha256(f"{source_fingerprint}\\n{toolchain}global {global_key_digest}\\n".encode()).hexdigest()\n'),
 ("gkl-no-rules",      '    lines = [f"rules {rules}", "toolchain " + " | ".join(toolchain.strip().splitlines())]\n',
                       '    lines = ["toolchain " + " | ".join(toolchain.strip().splitlines())]\n'),
 ("gkl-no-toolchain",  '    lines = [f"rules {rules}", "toolchain " + " | ".join(toolchain.strip().splitlines())]\n',
                       '    lines = [f"rules {rules}"]\n'),
 ("gkl-no-files",      '            lines.append(f"file {path} {manifest[path]}")\n',
                       '            pass\n'),
 ("gkl-no-env",        '            lines.append(f"env {name}={environment[name]}")\n',
                       '            pass\n'),
 ("gkl-no-tools",      '    lines += [f"tool {version}" for version in tools]\n',
                       '    lines += []\n'),
 ("gkl-no-exclusions", '    lines.append(f"exclusions {exclusions_digest}")\n',
                       '    pass\n'),
 ("gkl-no-limits",     '    lines.append(limits_line)\n',
                       '    pass\n'),
 ("exd-always-none",   '    return file_digest(path) if os.path.isfile(path) else "none"\n',
                       '    return "none"\n'),
 ("limits-constant",   '    return f"limits cpu={arguments.cpu_multiplier}x+{arguments.cpu_floor} wall={arguments.wall_multiplier}x+{arguments.wall_floor}"\n',
                       '    return "limits"\n'),
 ("export-no-cargo",   '    paths = [path for path in copied if not path.startswith("research/")] + [".cargo"]\n',
                       '    paths = [path for path in copied if not path.startswith("research/")]\n'),
]
for name, old, new in M:
    assert text.count(old) == 1, (name, text.count(old))
    tree = os.path.join(out, name)
    if os.path.exists(tree):
        shutil.rmtree(tree)
    subprocess.run(["cp", "-al", base, tree], check=True)
    target = os.path.join(tree, driver_rel)
    temporary = target + ".new"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))
    os.chmod(temporary, 0o755)
    os.replace(temporary, target)   # 新 inode，不动 base 那一份
    print(f"{name}\t{new.strip()[:110]}")
