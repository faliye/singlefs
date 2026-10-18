//! 输出行：一行一项、次序确定，行格式与 `scripts/gate/lib_function_units.py` 同一份——
//!   单元行 `<种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹>`（数据单元的种类是 `data`，限定名是数据文件从仓根起的路径，文件:行是读它的那一处）；
//!   键行 `key\t<sha256>`；拒算行 `refused\t<形态>\t<文件>:<行>\t<原样片段>`。

use std::collections::BTreeSet;

use crate::closure::{ClosureOutcome, DataUnit};
use crate::conditional_compilation::{ConditionalCompilationSet, Evaluation};
use crate::rust_items::cleaned_fragment;
use crate::sha256::Sha256Digest;
use crate::units::{ReadOrigin, Refusal, RefusalForm, SourceRead, UnitIndex, WgslUnitIndex};
use crate::workspace::{SourceLanguage, Workspace};

/// 一行加上排序用的位置（文件、起止行），排好之后丢掉位置。
struct PlacedLine {
    file: String,
    start_line: u32,
    end_line: u32,
    text: String,
}

fn sorted_texts(mut placed: Vec<PlacedLine>) -> Vec<String> {
    placed.sort_by(|left, right| {
        (&left.file, left.start_line, left.end_line, &left.text).cmp(&(
            &right.file,
            right.start_line,
            right.end_line,
            &right.text,
        ))
    });
    placed.into_iter().map(|line| line.text).collect()
}

fn unit_placed_line(workspace: &Workspace, unit_index: UnitIndex) -> PlacedLine {
    let unit = &workspace.units[unit_index.0];
    let file = workspace.files[unit.file.0].relative_path.clone();
    PlacedLine {
        text: format!(
            "{}\t{}\t{file}:{}-{}\t{}",
            unit.kind.output_name(),
            unit.qualified_name,
            unit.start_line,
            unit.end_line,
            unit.fingerprint.hexadecimal()
        ),
        file,
        start_line: unit.start_line,
        end_line: unit.end_line,
    }
}

fn wgsl_qualified_name(workspace: &Workspace, wgsl_index: WgslUnitIndex) -> String {
    let wgsl_unit = &workspace.wgsl_units[wgsl_index.0];
    let file = &workspace.files[wgsl_unit.file.0];
    let file_name = file
        .absolute_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    format!(
        "{}::{file_name}::{}",
        workspace.crates[file.crate_index.0].crate_identifier, wgsl_unit.name
    )
}

fn wgsl_placed_line(workspace: &Workspace, wgsl_index: WgslUnitIndex) -> PlacedLine {
    let wgsl_unit = &workspace.wgsl_units[wgsl_index.0];
    let file = workspace.files[wgsl_unit.file.0].relative_path.clone();
    PlacedLine {
        text: format!(
            "{}\t{}\t{file}:{}-{}\t{}",
            wgsl_unit.kind.output_name(),
            wgsl_qualified_name(workspace, wgsl_index),
            wgsl_unit.start_line,
            wgsl_unit.end_line,
            wgsl_unit.fingerprint.hexadecimal()
        ),
        file,
        start_line: wgsl_unit.start_line,
        end_line: wgsl_unit.end_line,
    }
}

fn data_placed_line(workspace: &Workspace, data_unit: &DataUnit) -> PlacedLine {
    let site = &workspace.units[data_unit.site.0];
    let file = workspace.files[site.file.0].relative_path.clone();
    PlacedLine {
        text: format!(
            "data\t{}\t{file}:{}-{}\t{}",
            data_unit.path_display,
            data_unit.site_line,
            data_unit.site_line,
            data_unit.digest.hexadecimal()
        ),
        file,
        start_line: data_unit.site_line,
        end_line: data_unit.site_line,
    }
}

/// 拒算行。
#[must_use]
pub fn refusal_line(workspace: &Workspace, refusal: &Refusal) -> String {
    format!(
        "refused\t{}\t{}:{}\t{}",
        refusal.form.output_name(),
        workspace.files[refusal.file.0].relative_path,
        refusal.line,
        cleaned_fragment(&refusal.fragment)
    )
}

/// 键行。
#[must_use]
pub fn key_line(key: &Sha256Digest) -> String {
    format!("key\t{}", key.hexadecimal())
}

fn sorted_refusal_lines(workspace: &Workspace, refusals: &BTreeSet<Refusal>) -> Vec<String> {
    let placed = refusals
        .iter()
        .map(|refusal| PlacedLine {
            file: workspace.files[refusal.file.0].relative_path.clone(),
            start_line: refusal.line,
            end_line: refusal.line,
            text: refusal_line(workspace, refusal),
        })
        .collect();
    sorted_texts(placed)
}

/// 切分阶段就看得出的全部拒算（`dump` 列的那一份）：每份文件切不全的、每个单元自己带的、数据文件读不了的，
/// 以及只按 rustc 自带名字认不出的 cfg 谓词（命令行给了 `--cfg` 的那几个名字在 `closure` / `key` 里照认）。
#[must_use]
pub fn parse_time_refusals(workspace: &Workspace) -> BTreeSet<Refusal> {
    let recognition = ConditionalCompilationSet::for_recognition_only();
    let mut refusals = BTreeSet::new();
    for file in &workspace.files {
        refusals.extend(file.file_level_refusals.iter().cloned());
    }
    for unit in &workspace.units {
        refusals.extend(unit.refusals.iter().cloned());
        for predicate in &unit.conditions {
            if recognition.evaluate(predicate) == Evaluation::Unknown {
                refusals.insert(Refusal {
                    file: unit.file,
                    line: unit.start_line,
                    form: RefusalForm::UnrecognizedConditionalCompilationPredicate,
                    fragment: crate::conditional_compilation::rendered_predicate(predicate),
                });
            }
        }
        for read in &unit.reads {
            if let SourceRead::File {
                absolute_path,
                line,
                digest: None,
                origin: ReadOrigin::Included | ReadOrigin::Runtime,
            } = read
            {
                refusals.insert(Refusal {
                    file: unit.file,
                    line: *line,
                    form: RefusalForm::DataFileUnreadable,
                    fragment: workspace.display_path(absolute_path),
                });
            }
        }
    }
    refusals
}

/// `dump` 的全部行：每个 Rust 单元、文档测试单元、WGSL 单元、数据单元（读 `.wgsl` 的不算，那几份按 WGSL 单元列），再接全部拒算行。
#[must_use]
pub fn dump_lines(workspace: &Workspace) -> Vec<String> {
    let mut placed: Vec<PlacedLine> = (0..workspace.units.len())
        .map(|position| unit_placed_line(workspace, UnitIndex(position)))
        .collect();
    placed.extend(
        (0..workspace.wgsl_units.len())
            .map(|position| wgsl_placed_line(workspace, WgslUnitIndex(position))),
    );
    for (position, unit) in workspace.units.iter().enumerate() {
        for read in &unit.reads {
            let SourceRead::File {
                absolute_path,
                line,
                digest: Some(digest),
                origin: ReadOrigin::Included | ReadOrigin::Runtime,
            } = read
            else {
                continue;
            };
            let is_wgsl = workspace
                .indexes
                .files_by_path
                .get(absolute_path)
                .is_some_and(|file| workspace.files[file.0].language == SourceLanguage::Wgsl);
            if is_wgsl {
                continue;
            }
            placed.push(data_placed_line(
                workspace,
                &DataUnit {
                    site: UnitIndex(position),
                    path_display: workspace.display_path(absolute_path),
                    site_line: *line,
                    digest: *digest,
                },
            ));
        }
    }
    let mut lines = sorted_texts(placed);
    lines.extend(sorted_refusal_lines(
        workspace,
        &parse_time_refusals(workspace),
    ));
    lines
}

/// `closure` 的全部行：闭包里的单元、WGSL 单元与数据单元（组成清单，只当索引），再接拒算行；没有拒算时最后一行是键行。
#[must_use]
pub fn closure_lines(
    workspace: &Workspace,
    outcome: &ClosureOutcome,
    key: Option<&Sha256Digest>,
) -> Vec<String> {
    let mut placed: Vec<PlacedLine> = outcome
        .units
        .iter()
        .map(|unit_index| unit_placed_line(workspace, *unit_index))
        .collect();
    placed.extend(
        outcome
            .wgsl_units
            .iter()
            .map(|wgsl_index| wgsl_placed_line(workspace, *wgsl_index)),
    );
    placed.extend(
        outcome
            .data_units
            .iter()
            .map(|data_unit| data_placed_line(workspace, data_unit)),
    );
    let mut lines = sorted_texts(placed);
    let refusals: BTreeSet<Refusal> = outcome.refusals.iter().cloned().collect();
    lines.extend(sorted_refusal_lines(workspace, &refusals));
    if let Some(key) = key {
        lines.push(key_line(key));
    }
    lines
}

/// 拒算行（`key` 拒算时只打这几行）。
#[must_use]
pub fn refusal_lines(workspace: &Workspace, refusals: &[Refusal]) -> Vec<String> {
    let set: BTreeSet<Refusal> = refusals.iter().cloned().collect();
    sorted_refusal_lines(workspace, &set)
}
