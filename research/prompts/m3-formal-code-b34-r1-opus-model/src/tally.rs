//! 计数：每张判过的镜像记一笔，按名字累计；违例与新旧不同各留第一处原文。
use std::collections::BTreeMap;

use crate::judge::Compared;

#[derive(Default)]
pub struct Tally {
    pub images: u64,
    /// 旧 checker 探针认出由记录施加出来的版本的镜像数、下一次挂载先施加的那一版的镜像数。
    pub images_with_versions_applied_only_by_records: u64,
    pub images_with_the_next_mount_version: u64,
    /// 新 checker 判红的镜像，按不变量。
    pub new_red: BTreeMap<String, u64>,
    pub new_red_first: BTreeMap<String, String>,
    /// 新旧不同：键「不变量 旧种类>新种类」。
    pub differences: BTreeMap<String, u64>,
    pub differences_first: BTreeMap<String, String>,
    /// I-5.4 在有两类版本的镜像上新 checker 的种类分布。
    pub i54_on_rv_images: BTreeMap<char, u64>,
    /// I-5.4 在全部镜像上新 checker 的种类分布。
    pub i54_all: BTreeMap<char, u64>,
    /// I-5.5 新 checker 的种类与原文头 24 字。
    pub i55_reasons: BTreeMap<String, u64>,
    /// 树表指针全零的根、记录（各张镜像累加）。
    pub all_zero_roots: u64,
    pub all_zero_records: u64,
}

impl Tally {
    pub fn note(&mut self, label: &str, compared: &Compared) {
        self.images += 1;
        if let Some(fl) = &compared.fl {
            for (name, fl_kind) in fl {
                let new_kind = &compared.new[name];
                if fl_kind.0 != new_kind.0 || (matches!(fl_kind.0, 'N' | 'P') && fl_kind.1 != new_kind.1) {
                    let key = format!("FL {name} {}>{}", new_kind.0, fl_kind.0);
                    *self.differences.entry(key.clone()).or_default() += 1;
                    self.differences_first.entry(key).or_insert_with(|| {
                        format!("{label} :: new=[{}] fl=[{}]", new_kind.1.chars().take(300).collect::<String>(), fl_kind.1.chars().take(300).collect::<String>())
                    });
                }
            }
        }
        let rv = !compared.versions_applied_only_by_records.is_empty();
        if rv {
            self.images_with_versions_applied_only_by_records += 1;
        }
        if compared.next_mount_version {
            self.images_with_the_next_mount_version += 1;
        }
        *self.i54_all.entry(compared.new.get("I-5.4").map_or('-', |k| k.0)).or_default() += 1;
        if let Some(k) = compared.new.get("I-5.5") {
            *self.i55_reasons.entry(format!("{}:{}", k.0, k.1.chars().take(24).collect::<String>())).or_default() += 1;
        }
        self.all_zero_roots += compared.all_zero_tree_table_pointers.0;
        self.all_zero_records += compared.all_zero_tree_table_pointers.1;
        if rv || compared.next_mount_version {
            let kind = compared.new.get("I-5.4").map_or('-', |k| k.0);
            *self.i54_on_rv_images.entry(kind).or_default() += 1;
        }
        for name in compared.new_violations() {
            *self.new_red.entry(name.to_string()).or_default() += 1;
            self.new_red_first.entry(name.to_string()).or_insert_with(|| {
                format!("{label} :: {}", compared.new[name].1.chars().take(400).collect::<String>())
            });
        }
        for (name, old, new) in compared.differences() {
            let key = format!("{name} {}>{}", old.0, new.0);
            *self.differences.entry(key.clone()).or_default() += 1;
            self.differences_first.entry(key).or_insert_with(|| {
                format!(
                    "{label} :: old=[{}] new=[{}]",
                    old.1.chars().take(300).collect::<String>(),
                    new.1.chars().take(300).collect::<String>()
                )
            });
        }
    }

    pub fn absorb(&mut self, other: Tally) {
        self.images += other.images;
        self.images_with_versions_applied_only_by_records += other.images_with_versions_applied_only_by_records;
        self.images_with_the_next_mount_version += other.images_with_the_next_mount_version;
        for (k, v) in other.new_red {
            *self.new_red.entry(k).or_default() += v;
        }
        for (k, v) in other.new_red_first {
            self.new_red_first.entry(k).or_insert(v);
        }
        for (k, v) in other.differences {
            *self.differences.entry(k).or_default() += v;
        }
        for (k, v) in other.differences_first {
            self.differences_first.entry(k).or_insert(v);
        }
        self.all_zero_roots += other.all_zero_roots;
        self.all_zero_records += other.all_zero_records;
        for (k, v) in other.i55_reasons {
            *self.i55_reasons.entry(k).or_default() += v;
        }
        for (k, v) in other.i54_all {
            *self.i54_all.entry(k).or_default() += v;
        }
        for (k, v) in other.i54_on_rv_images {
            *self.i54_on_rv_images.entry(k).or_default() += v;
        }
    }

    pub fn print(&self, tag: &str) {
        println!(
            "{tag} images={} i54_all={:?} rv_images={} next_mount_images={} i54_on_rv_images={:?} new_red={:?} differences={:?}",
            self.images,
            self.i54_all,
            self.images_with_versions_applied_only_by_records,
            self.images_with_the_next_mount_version,
            self.i54_on_rv_images,
            self.new_red,
            self.differences
        );
        println!("{tag}-Q i55_reasons={:?} all_zero_tree_table_pointer_roots={} records={}", self.i55_reasons, self.all_zero_roots, self.all_zero_records);
        for (k, v) in &self.new_red_first {
            println!("{tag}-RED-FIRST {k} {v}");
        }
        for (k, v) in &self.differences_first {
            println!("{tag}-DIFF-FIRST {k} {v}");
        }
    }
}
