"""样本的公共结构。{R}=喂钩子并起真 bash 的那棵树（桩树），{C}=仓副本（不在 git 仓里），{D}=草稿目录。"""
SAMPLES = []
_counter = {}

ROLE_NAMES = {"main": "主", "sub": "子(three-way-forward)", "editor": "子(implementation-writer，有 Edit)"}


def add(fam, cmd, roles=("sub", "main"), bg=False, cwd="C", setup="", stage=True, want=None, note="", prefix_variants=True, timeout=30, prefix_style="inline"):
    """每个角色一条样本。want=None：期望由真 bash 观测 + policy.py 算；want=[...] 或 []：手标（note 必须写依据）。
    主 agent 角色按需生成"不带前缀""带 SINGLEFS_HEAVY_TESTS=commit 前缀"两条。"""
    for role in roles:
        variants = [("", "")]
        if role == "main" and prefix_variants:
            variants = [("np", ""), ("pf", "SINGLEFS_HEAVY_TESTS=commit " if prefix_style == "inline" else "export SINGLEFS_HEAVY_TESTS=commit; ")]
        for tag, pre in variants:
            n = _counter.get(fam, 0) + 1
            _counter[fam] = n
            sid = f"{fam}-{n:03d}-{role}{tag}"
            SAMPLES.append({"id": sid, "fam": fam, "cmd": pre + cmd, "role": role, "bg": bg, "cwd": cwd, "setup": setup,
                            "stage": stage, "want": want, "note": note, "timeout": timeout})
