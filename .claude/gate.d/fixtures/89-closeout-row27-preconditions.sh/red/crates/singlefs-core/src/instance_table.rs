// 样本：行回收进来了——实例表开始删行，C333 那一笔从此走得到。
fn append(rows: &mut Vec<InstanceRow>) {
    rows.push(row);
}

fn reclaim(rows: &mut Vec<InstanceRow>, dropped: InstanceGeneration) {
    rows.retain(|row| row.instance != dropped);
}
