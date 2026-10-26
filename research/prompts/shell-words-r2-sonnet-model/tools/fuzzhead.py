"""fuzzhead.py <种子> <条数> <输出.json> : 随机拼 heredoc 前面的那一串（引号、转义、替换、算术、注释、进程替换的各种写法），最后接 bash <<'EOF' 喂 cargo test --all。
有的把 bash <<EOF 放进还没收尾的 $( / 反引号 / <( 里。真 bash 判这条 heredoc 到底有没有被 bash 读去跑，钩子在修补前后的树上各判一遍。"""
import random, json, sys
random.seed(int(sys.argv[1])); N = int(sys.argv[2])
atoms = ['echo "$(echo a)"; ', "echo 'it''s'; ", 'echo "x"; ', "x=$(echo 'a'); ", 'echo \\"; ', "echo $'a\\'b'; ", 'echo "a\\"b"; ', 'echo "(" ; ', 'echo ")" ; ', "echo '(' ; ", "echo ')' ; ", 'echo $((1+2)); ',
         'echo ${#x}; ', 'echo a#b; ', 'echo "#"; ', 'echo "$(echo "(")"; ', "echo `echo '\"'`; ", ': <(echo a); ', ': >(cat) </dev/null; ', 'echo "<("; ', 'echo "$(echo \')\')"; ', 'echo "`echo a`"; ',
         'echo "$(echo `echo b`)"; ', 'echo $(echo "a"); ', 'echo "$(echo a)" "b"; ', "echo \"it's\"; ", "echo 'say \"hi\"'; ", 'echo "a$(echo "b")c"; ', 'echo ${x:-"$(echo a)"}; ', "echo ${x:-'a b'}; ",
         'echo a # $(\n', 'echo "a" # "\n', 'f() { echo "$(date)"; }; ', 'case x in x) echo a;; esac; ', '[[ "$(echo a)" == a ]] && ', 'true && ', 'echo "\\$("; ', "echo \\' ; ", 'echo a\\\n', 'echo $(( (1+2)*3 )); ',
         'echo $( (echo a) ); ', 'echo "$( (echo a) )"; ', 'echo "${x#"a"}"; ', "echo ${x//'a'/b}; ", 'echo "a" > /dev/null; ', 'echo "$(echo a > /dev/null)"; ', 'echo 1>&2 2>/dev/null; ']
feed = ["bash <<'EOF'\ncargo test --all\nEOF", "sh <<EOF\ncargo test --all\nEOF", "bash -s <<'EOF'\ncargo test --all\nEOF", "cat <<'EOF'\ncargo test --all\nEOF"]
opens = [("x=$(", "\n)"), ('x="$(', '\n)"'), ("x=`", "\n`"), ("diff <(", "\n) /dev/null"), ('echo "$(echo "$(', '\n)")"'), ("echo $(echo \"$(", "\n)\")")]
cs, seen = [], set()
while len(cs) < N:
    k = random.randint(1, 4)
    head = "".join(random.choice(atoms) for _ in range(k))
    f = random.choice(feed)
    if random.random() < 0.35:
        o, c = random.choice(opens)
        cmd = head + o + f + c
    else:
        cmd = head + f
    if cmd in seen: continue
    seen.add(cmd); cs.append(dict(tag="head", cmd=cmd, hook="guard", agent="main"))
json.dump(cs, open(sys.argv[3], "w"), ensure_ascii=False); print(len(cs))
