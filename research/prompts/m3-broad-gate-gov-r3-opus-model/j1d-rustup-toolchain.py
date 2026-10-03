#!/usr/bin/env python3
"""攻方 J1-d：会话 shell 里设 RUSTUP_TOOLCHAIN（rustup 认它，优先于树里的 rust-toolchain 文件）时，
驱动记的工具链（toolchain_text：在树里起 rustc -Vv）变不变、--unit-environment 打不打它。只起 rustc -Vv，不编不跑。
用法：j1d-rustup-toolchain.py <驱动路径> <树根> <工具链名>"""
import importlib.util, os, sys
driver_path, root, toolchain = sys.argv[1:4]
spec = importlib.util.spec_from_file_location("d", driver_path); d = importlib.util.module_from_spec(spec); spec.loader.exec_module(d)
os.environ.pop("RUSTUP_TOOLCHAIN", None)
plain = d.toolchain_text(root).splitlines()[0]
os.environ["RUSTUP_TOOLCHAIN"] = toolchain
pinned = d.toolchain_text(root).splitlines()[0]
names = d.unit_environment_names(dict(os.environ), root)
print(f"不设 RUSTUP_TOOLCHAIN：{plain}")
print(f"RUSTUP_TOOLCHAIN={toolchain}：{pinned}")
print(f"工具链{'变了' if plain != pinned else '没变'}；--unit-environment 认的变量：{names or '没有'}；带进单元的固定五个：{' '.join(d.UNIT_CARRIED_VARIABLES)}")
