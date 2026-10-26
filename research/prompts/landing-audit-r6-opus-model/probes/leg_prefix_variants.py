"""K2 交回闸 ② / 派发闸 ⑰ 共用的认法（lib_three_way_legs.leg_report_mentions_in_trees）在没被量过的几种写法上认出哪一棵。
造一个指向 m5-dev 的符号链接放在草稿目录里（只建链接，不碰 m5-dev），别的写法都是字符串。
用法：python3 -I leg_prefix_variants.py <钩子目录> <仓根> <草稿目录>"""
import os, sys
hooks, root, scratch = sys.argv[1:4]
sys.path.insert(0, hooks)
import lib_three_way_legs as legs
work = "/home/fy5090/.local/share/singlefs/work"
link = os.path.join(scratch, "m5-dev-link")
if not os.path.islink(link):
    os.symlink(os.path.join(work, "m5-dev"), link)
name = "research/prompts/x-r1-opus-output.md"
cases = [
    ("m5-dev 绝对路径（对照）", f"报告：{work}/m5-dev/{name}"),
    ("m5-dev 路径里带 /./", f"报告：{work}/m5-dev/./{name}"),
    ("m5-dev 路径末尾多一个 /", f"报告：{work}/m5-dev//{name}"),
    ("经符号链接写的 m5-dev", f"报告：{link}/{name}"),
    ("$SINGLEFS_WORK_ROOT/m5-dev（lib_work_root 认这种写法）", "报告：$SINGLEFS_WORK_ROOT/m5-dev/" + name),
    ("~/.local/share/singlefs/work/m5-dev", f"报告：~/.local/share/singlefs/work/m5-dev/{name}"),
    ("仓根路径里带 /./", f"报告：{root}/./{name}"),
    ("工作区相对路径写法", f"报告：{name}"),
    ("<副本>/ 占位起头", f"报告：<副本>/{name}"),
    ("草稿目录名起头的相对写法 m5-dev/", f"报告：m5-dev/{name}"),
    ("Markdown 链接 [报告](…/m5-dev/…)", f"[报告]({work}/m5-dev/{name})"),
    ("file:// 写法", f"报告 file://{work}/m5-dev/{name}"),
    ("仓根绝对路径（对照）", f"报告：{root}/{name}"),
    ("没有前缀（对照）", f"报告：{name}"),
]
trees, error = legs.registered_trees(root)
print(f"工作树：{trees}；取列表的错误：{error}")
for label, text in cases:
    found = legs.leg_report_mentions_in_trees(text, root)
    print(f"{label}：{'认出 ' + '、'.join(tree for _, tree in found) if found else '没认出'}")
os.unlink(link)
