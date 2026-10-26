import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
CAP = cases_a.CAP; RB = cases_a.RB
# ---------- cargo 参数形态：像 --all 但不是的、点名非 checker 档包的，各写法 ----------
cargo_ok = [
 ('J01', 'cargo test --all-features -p singlefs-harness', ['full']),
 ('J02', 'cargo test --all-targets -p singlefs-harness', ['full']),
 ('J03', 'cargo test -p singlefs-harness -- --test-threads 1 --nocapture', ['ac_harness']),
 ('J04', 'cargo test --manifest-path scripts/experiment-apparatus/Cargo.toml --bin foo', ['full']),
 ('J05', 'cd crates/singlefs-core && {W}cargo test', ['full']),
 ('J06', 'cd crates/singlefs-harness && {W}cargo test --release', ['full']),
 ('J07', 'cargo test -p singlefs-format -p singlefs-core', ['ac_harness']),
 ('J08', 'cargo test --package=singlefs-core', ['ac_harness']),
 ('J09', 'cargo test -psinglefs-core', ['ac_harness']),
 ('J10', 'cargo test --release --no-fail-fast -p singlefs-core', ['ac_harness']),
 ('J11', 'cargo test -p singlefs-core --lib', ['ac_harness']),
 ('J12', 'cargo test -p singlefs-harness --test t -- --exact foo', ['ac_harness']),
 ('J13', 'cargo +nightly fmt --all', ['ac_nobuild']),
 ('J14', 'cargo test -p singlefs-core --features all', ['ac_harness']),
 ('J15', 'cargo test -p singlefs-checker', ['checker']),
]
for n, cmd, cites in cargo_ok:
    C(f'{n}-IW', IW, cmd.replace('{W}', CAP) if '{W}' in cmd else CAP + cmd, 0, 0, cites)
    C(f'{n}-M', M, cmd.replace('{W}', RB) if '{W}' in cmd else RB + cmd, 0, 0, cites)
cargo_bad = [
 ('J30', 'cargo test', ['full'], 'repo'),        # 工作区根裸跑
 ('J31', 'cargo +nightly test --all', ['full'], None),
 ('J32', 'cargo test --exclude singlefs-core --workspace', ['full'], None),
 ('J33', 'cargo test -p singlefs-checker-tier --no-fail-fast', ['checker'], None),
 ('J34', 'cargo test --package singlefs-checker-tier', ['checker'], None),
 ('J35', 'cargo test --package=singlefs-checker-tier --lib', ['checker'], None),
 ('J36', 'cargo test -psinglefs-checker-tier', ['checker'], None),
 ('J37', 'cargo test -p singlefs-core -p singlefs-checker-tier', ['checker'], None),
 ('J38', 'cargo test --release --all', ['full'], None),
 ('J39', 'cargo test --workspace --exclude singlefs-checker-tier', ['full'], None),
 ('J40', 'cargo nextest run -p singlefs-checker-tier', ['checker'], None),
 ('J41', 'cargo test -p singlefs-checker-tier -- --include-ignored', ['checker'], None),
]
for n, cmd, cites, cw in cargo_bad:
    opt = {'cwd': cw} if cw else {}
    C(f'{n}-IW', IW, CAP + cmd, 2, 0, cites + ['sub_refuse'], **opt)
    C(f'{n}-M', M, RB + cmd, 2, 0, cites + ['prefix_need'], **opt)
# 其它常见的无害命令
for ident in (M, IW):
    s = ident[:3]
    C(f'J50-{s}', ident, 'cargo --version; rustc --version; cargo -V', 0, 0, ['target'], cwd='repo')
    C(f'J51-{s}', ident, 'cargo locate-project --workspace', 0, 0, ['target'], cwd='repo')
    C(f'J52-{s}', ident, 'echo "it'"'"'s ok"; cargo fmt --check', 0, 0, ['redir'], cwd='repo')
    C(f'J53-{s}', ident, 'echo "$(echo "a$(echo b)c")" # cargo test --all', 0, 0, ['redir'], cwd='repo')
    C(f'J54-{s}', ident, 'while IFS= read -r l; do echo "$l"; done < <(ls)', 0, 0, ['b_loop_ok'], cwd='repo')
    C(f'J55-{s}', ident, 'ls -la \\\n  | head -3', 0, 0, ['redir'], cwd='repo')
    C(f'J56-{s}', ident, 'ls 2>&1 | tee {D}/ls.txt | head -2', 0, 0, ['redir'], cwd='repo')
    C(f'J57-{s}', ident, '[ -n "$x" ] && cargo fmt --all', 0, 0, ['target'], cwd='repo')
