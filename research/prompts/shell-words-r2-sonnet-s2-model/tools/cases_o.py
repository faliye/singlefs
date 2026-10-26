import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
# ---------- 命令位置之外出现重型名字：case 模式、for 列表、数组、测试表达式 ----------
words = [
 ('O01', 'case "$x" in qemu-system-*) echo q;; herd7) echo h;; *) echo o;; esac', 0, ['cmdpos','glob']),
 ('O02', 'case "$x" in a) echo a;; herd7) echo h;; esac', 0, ['cmdpos']),
 ('O03', 'case "$x" in\n  herd7) echo h ;;\nesac', 0, ['cmdpos']),
 ('O04', 'case "$x" in a|herd7) echo h;; esac', 0, ['cmdpos']),
 ('O05', 'case "$x" in a) :;; gate.sh) :;; esac', 0, ['cmdpos']),
 ('O06', 'case "$x" in a) :;; vm-bench.sh) :;; esac', 0, ['cmdpos']),
 ('O07', 'case "$x" in a) :;; qemu-system-x86_64) :;; esac', 0, ['cmdpos']),
 ('O08', 'case "$x" in herd7) echo h;; esac', 0, ['cmdpos']),
 ('O09', 'for t in herd7 gate.sh qemu-system-x86_64; do echo $t; done', 0, ['cmdpos']),
 ('O10', 'tools=(herd7 gate.sh qemu-system-x86_64); echo ${tools[@]}', 0, ['cmd_v']),
 ('O11', '[[ $x == herd7 ]] && echo y', 0, ['cmdpos']),
 ('O12', '[ "$tool" = "gate.sh" ] && echo y', 0, ['cmdpos']),
 ('O13', 'select t in herd7 qemu; do break; done < /dev/null', 0, ['cmdpos']),
 ('O14', 'hash herd7 2>/dev/null; whereis herd7; type -a herd7; ls $(which herd7 2>/dev/null)', 0, ['cmd_v']),
 ('O15', 'dpkg -l | grep -c herd7; apt list --installed 2>/dev/null | grep -c qemu-system', 0, ['cmdpos']),
 ('O16', 'man qemu-system-x86_64 2>/dev/null | head -1', 0, ['cmdpos']),
 ('O17', 'alias herd7=true', 0, ['cmdpos']),
 ('O18', 'declare -a t=(herd7 gate.sh)', 0, ['cmd_v']),
 ('O19', 'test -x /usr/bin/herd7 && echo y', 0, ['cmdpos']),
 ('O20', 'echo "do:" ; echo herd7 ; echo gate.sh', 0, ['cmdpos']),
 ('O21', 'qemu-system-x86_64 --version', 2, ['qemu']),
 ('O22', 'qemu-system-x86_64 -device help', 2, ['qemu']),
 ('O23', 'x=herd7; echo $x', 0, ['cmdpos']),
 ('O24', 'if [ -n "$HERD7" ]; then echo set; fi', 0, ['cmdpos']),
 ('O25', 'while read -r t; do echo "$t"; done <<< "herd7"', 0, ['cmdpos']),
 ('O26', 'printf "%s\\n" herd7 gate.sh | sort', 0, ['cmdpos']),
]
for n, cmd, e, cites in words:
    for ident in (M, IW):
        C(f'{n}-{ident[:3]}', ident, cmd, e, 0, cites, cwd='repo' if ident == M else 'copy')

# 同一类写法对检测器：case 的第二个及以后的模式、单独一行的模式
for w, cite in [('reboot', 'b_kill_power'), ('shutdown', 'b_kill_power'), ('halt', 'b_kill_power'), ('poweroff', 'b_kill_power'), ('killall', 'b_kill_pkill'), ('pkill', 'b_kill_pkill'), ('disown', 'b_detach'), ('coproc', 'b_detach'), ('claude', 'b_claude')]:
    C(f'O30-{w}', IW, f'case "$1" in a) :;; {w}) :;; esac', 0, 0, ['cmdpos', cite])
C('O31-reboot-ml', IW, 'case "$1" in\n  start) echo s;;\n  reboot) echo r;;\nesac', 0, 0, ['cmdpos','b_kill_power'])
# 对照：case 的臂里真起命令的照拒
C('O32-IW', IW, 'case "$1" in a) cargo test --all;; esac', 2, 0, ['full','sub_refuse'])
C('O33-IW', IW, 'case "$1" in a) echo ok;; b) herd7 foo.litmus;; esac', 2, 0, ['herd7','sub_refuse'])
C('O34-IW', IW, 'case "$1" in a) reboot;; esac', 0, 2, ['b_kill_power'])

# [[ ]] 里的 && 把后面的词当成新命令的开头
C('O40-IW', IW, '[[ -f README.md && herd7 == x ]] && echo y', 0, 0, ['cmdpos'])
C('O41-IW', IW, 'echo a | while read t; do case $t in a) :;; gate.sh) :;; esac; done', 0, 0, ['cmdpos'])

# 对照：case 的第一个模式写在 in 同一行时放行；写进脚本再起，脚本里读到的同一处照拒
C('O42-IW', IW, 'case "$x" in herd7) echo h;; esac', 0, 0, ['cmdpos'])
C('O43-IW', IW, 'case "$x" in "herd7") echo h;; a) :;; esac', 0, 0, ['cmdpos'])
C('O44-IW', IW, "cat > {D}/c44.sh <<'EOF'\ncase \"$1\" in\n  a) :;;\n  herd7) :;;\nesac\nEOF\nbash {D}/c44.sh a", 0, 0, ['cmdpos','script_read'])
C('O45-IW', IW, "cat > {D}/c45.sh <<'EOF'\ncase \"$1\" in\n  a) :;;\n  reboot) :;;\nesac\nEOF\nbash {D}/c45.sh a", 0, 0, ['cmdpos','script_read'])
