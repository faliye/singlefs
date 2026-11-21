use super::*;

// ───────────────────────────── 一条流的一行（`name=flow`）与块键文件 ─────────────────────────────

pub(super) fn count_field(count: Option<u64>) -> String {
    count.map_or_else(|| BEYOND_U64.to_string(), |value| value.to_string())
}

fn log2_field(count: Option<u64>) -> String {
    count.map_or_else(
        || BEYOND_U64.to_string(),
        |value| format!("{:.4}", (value as f64).log2()),
    )
}

/// 一行里 `名=值` 的字段（空白隔开；值里可以再有 `=`，只在第一个 `=` 处切）。
pub(super) fn fields_of(body: &str) -> BTreeMap<String, String> {
    body.split_whitespace()
        .filter_map(|word| word.split_once('='))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

pub(super) fn required_field<'fields>(
    fields: &'fields BTreeMap<String, String>,
    name: &str,
) -> Result<&'fields str, String> {
    fields
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("缺字段 {name}"))
}

fn parse_count_field(text: &str) -> Result<Option<u64>, String> {
    if text == BEYOND_U64 {
        return Ok(None);
    }
    text.parse::<u64>()
        .map(Some)
        .map_err(|error| format!("{text:?} 不是整数也不是 {BEYOND_U64}：{error}"))
}

/// `name=flow` 那一行的内容（登记第六节 J1-甲）。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct FlowRow {
    pub(super) family: String,
    pub(super) tier: String,
    pub(super) label: String,
    pub(super) seed: Option<u64>,
    pub(super) ending: String,
    pub(super) writes: usize,
    pub(super) segments: usize,
    pub(super) longest_segment: usize,
    pub(super) two_state: Option<u64>,
    pub(super) full: Option<u64>,
    pub(super) in_place_subsets: Option<u64>,
    /// 没排计划（整条流态数装不进 u64、块数上界超过 [`MOST_BLOCKS_PLANNED_PER_FLOW`]，或写表里没有根槽写）是 `None`；
    /// 态数（`full`、`in_place_subsets`）照报，块数是态数除每块态数。
    pub(super) blocks_full: Option<usize>,
    pub(super) blocks_in_place_subsets: Option<usize>,
    pub(super) points: Option<usize>,
    /// 这条流在进程里排计划、取块键用的秒数（两种展开合计）。
    pub(super) keys_seconds: f64,
    /// 本地复算的两态闭式与 crates 的 `closed_form_state_count_of_write_index_segments` 是否相同；闭式算不出（有段不少于 64 写）时 `None`。
    pub(super) two_state_agrees_with_crates: Option<bool>,
}

impl FlowRow {
    pub(super) fn body(&self) -> String {
        let planned = |count: Option<usize>| {
            count.map_or_else(|| "not_planned".to_string(), |value| value.to_string())
        };
        format!(
            "name=flow family={} tier={} label={} seed={} ending={} writes={} segments={} longest_segment={} two_state={} log2_two_state={} full={} in_place_subsets={} blocks_full={} blocks_in_place_subsets={} points={} keys_seconds={:.1} two_state_agrees_with_crates={}",
            self.family,
            self.tier,
            self.label,
            self.seed.map_or_else(|| "none".to_string(), |seed| seed.to_string()),
            self.ending,
            self.writes,
            self.segments,
            self.longest_segment,
            count_field(self.two_state),
            log2_field(self.two_state),
            count_field(self.full),
            count_field(self.in_place_subsets),
            planned(self.blocks_full),
            planned(self.blocks_in_place_subsets),
            planned(self.points),
            self.keys_seconds,
            self.two_state_agrees_with_crates.map_or_else(|| "not_applicable".to_string(), |agrees| agrees.to_string()),
        )
    }

    /// [`Self::body`] 的逆。
    pub(super) fn from_body(body: &str) -> Result<Self, String> {
        let fields = fields_of(body);
        let usize_field = |name: &str| -> Result<usize, String> {
            required_field(&fields, name)?
                .parse::<usize>()
                .map_err(|error| format!("{name}：{error}"))
        };
        let planned = |name: &str| -> Result<Option<usize>, String> {
            match required_field(&fields, name)? {
                "not_planned" => Ok(None),
                text => text
                    .parse::<usize>()
                    .map(Some)
                    .map_err(|error| format!("{name}：{error}")),
            }
        };
        let seed = match required_field(&fields, "seed")? {
            "none" => None,
            text => Some(
                text.parse::<u64>()
                    .map_err(|error| format!("seed：{error}"))?,
            ),
        };
        Ok(Self {
            family: required_field(&fields, "family")?.to_string(),
            tier: required_field(&fields, "tier")?.to_string(),
            label: required_field(&fields, "label")?.to_string(),
            seed,
            ending: required_field(&fields, "ending")?.to_string(),
            writes: usize_field("writes")?,
            segments: usize_field("segments")?,
            longest_segment: usize_field("longest_segment")?,
            two_state: parse_count_field(required_field(&fields, "two_state")?)?,
            full: parse_count_field(required_field(&fields, "full")?)?,
            in_place_subsets: parse_count_field(required_field(&fields, "in_place_subsets")?)?,
            blocks_full: planned("blocks_full")?,
            blocks_in_place_subsets: planned("blocks_in_place_subsets")?,
            points: planned("points")?,
            keys_seconds: required_field(&fields, "keys_seconds")?
                .parse::<f64>()
                .map_err(|error| format!("keys_seconds：{error}"))?,
            two_state_agrees_with_crates: match required_field(
                &fields,
                "two_state_agrees_with_crates",
            )? {
                "not_applicable" => None,
                text => Some(
                    text.parse::<bool>()
                        .map_err(|error| format!("two_state_agrees_with_crates：{error}"))?,
                ),
            },
        })
    }

    /// 这条流在 `groups` 里的组名：流名去掉末尾的 `-seedNN`（同一族同一取样点的几个种子是一组）。
    pub(super) fn group(&self) -> String {
        group_of_the_label(&self.label)
    }
}

fn group_of_the_label(label: &str) -> String {
    match label.rsplit_once("-seed") {
        Some((group, index))
            if !index.is_empty() && index.chars().all(|digit| digit.is_ascii_digit()) =>
        {
            group.to_string()
        }
        Some(_) | None => label.to_string(),
    }
}

#[cfg(test)]
mod tests;
