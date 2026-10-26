import sys, subprocess, os, json
sys.path.insert(0, os.path.join(os.environ["SW_ROOT"], "tree", ".claude", "hooks"))
import lib_shell_words as L
cases=[ "${a:-{b,c}}", "${a:-b,c}", "x${a}{b,c}", "{a,b}${v}", "$v{a,b}", "\\${a,b}", "\"$\"{a,b}", "${v}{1..3}", "a{1..3}b", "{01..03}", "{1..10..3}", "{1..3..0}", "{5..1}", "{a..e..2}", "{a..Z}", "{A..e}", "{-2..2}", "{1..a}", "{a,b{c,d}}", "{a,b}{1,2}", "{,a}", "{a,}", "{a}", "{}", "{a,b", "a,b}", "{a,{b,c}", "{a,b}}", "{{a,b}", "{1..3}{a..c}", "{x..z}{1,2}", "\\{a,b}", "{a\\,b}", "{a,b\\}", "'{a,b}'", "\"{a,b}\"", "{a,'b,c'}", "{\"a,b\"}", "x{'',a}", "{1..65}", "{1..64}", "{1..2}{1..2}{1..2}{1..2}{1..2}{1..2}{1..2}", "{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}", "{00..3}", "{1..03}", "{1.5..3}", "{1..3.5}", "{ 1..3}", "{1.. 3}", "{a..a}", "{3..3}", "{1..-2}", "{-1..-3}", "{a,b}=c", "x={a,b}", "{a,b}c=d", "[[{a,b}", "{a,b}()", "~{a,b}", "{~,a}", "{a,b}/{c,d}", "$((1+{1,2}))", "$(echo {a,b})", "`echo {a,b}`", "{$(echo x),b}", "{$(echo x,y),b}", "a{b,c}$(echo {1,2})d", "{a,b}$(echo x)", "{@,%}", "{a-b,c}", "{!,x}", "}{a,b}{", "{a,b}{", "{{a,b}}", "{{1..2},{a..b}}", "{a..c}{1..2}", "{1,2}{3,4}{5,6}", "{ls,-l}", "{cargo,test}", "{cargo,echo}", "-{a,b}", "--{x,y}=1","{a,b}--","{,}x","x{,}","{,}","{,,}","{,a,}", "a{,}b{,}", "{1..3}\\", "{a..c}$v", "$v{1..3}", "{a,b}$", "$"+"{a,b}", "$,{a,b}", "{$,a}", "{a,$}", "{a,${v}}", "{${v},b}", "${v}${v}{a,b}"]
bad=0
for w in cases:
    script="set -f\nv=VV\nprintf '[%s]' "+w+"\necho\n"
    p=subprocess.run(["bash","-c",script],capture_output=True,text=True)
    if p.returncode!=0:
        print("BASHERR",w,p.stderr.strip()[:60]); continue
    out=p.stdout.rstrip("\n")
    bw=[] if out=="[]" and False else (out[1:-1].split("][") if out.startswith("[") else None)
    hw=[t for op,t in L.shell_tokens(w) if op is False]
    # normalise hook words: replace ${v} and $v
    hw2=[x.replace("${v}","VV") for x in hw]
    # bash drops empty words from unquoted expansions; mimic when word has no quotes
    if hw2!=bw:
        bad+=1; print("DIFF",json.dumps(w,ensure_ascii=False),"bash=",bw,"hook=",hw)
print("cases",len(cases),"diffs",bad)
