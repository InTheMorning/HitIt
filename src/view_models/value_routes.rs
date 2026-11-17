//! The value routes look of musicindex ADR 0082, for the open value routes
//! cell of the track detail compare grid (R-ROUTES-8 item 1).
//!
//! The model gives the summary bar (R-ROUTES-1), the names line (R-ROUTES-2),
//! the panel with its groups and its fee line (R-ROUTES-3), the rows
//! (R-ROUTES-4) and the open content of a read-only row (R-ROUTES-5 item 4).
//! The percentages and the short address come from the shared crate
//! `musicindex-value` (musicindex ADR 0075 R-VALUE-4, ADR 0076 R-ADDRESS-2).
//! The site function `shareParts` of `search.html` is the reference.
//!
//! This app reads one list of Lightning routes (`HitIt` ADR 0099), not
//! `value_blocks`. The cell is one block with no block label, so R-ROUTES-6
//! does not apply (R-ROUTES-6 item 3).

#![warn(clippy::pedantic)]

use crate::api::PaymentRoute;

/// R-ROUTES-1 item 4: the bar names at most this number of segments.
const SHARE_PARTS_MAX: usize = 20;
/// R-ROUTES-1 item 4: the rest that "Other" can hold, in percent.
const SHARE_OTHER_MAX: f64 = 10.0;
/// R-ROUTES-1 item 4: a recipient with this percentage or more gets a name.
const SHARE_SMALL: f64 = 5.0;
/// R-ROUTES-2 item 2: the names line names this number of segments.
const SHARE_NAMED: usize = 3;

/// The separator of the names line and of the detail line (R-ROUTES-2 item 2).
pub(crate) const PART_SEPARATOR: &str = " · ";

/// The text of a section with no recipient (R-ROUTES-3 item 8).
pub(crate) const NONE_LISTED: &str = "None listed.";

/// The look of one segment of the summary bar (R-ROUTES-1 item 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShareSegmentKind {
    /// A recipient that is not a fee: the share color.
    Split,
    /// A fee recipient: stripes of the share color and the share track color.
    Fee,
    /// The recipients that are not named: the muted text color.
    Other,
}

/// One segment of the summary bar.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ShareSegment {
    pub kind: ShareSegmentKind,
    /// The percentage of the segment. The width of the segment is in
    /// proportion to it (R-ROUTES-1 item 2).
    pub percent: f64,
    /// The hover text (R-ROUTES-1 item 7).
    pub tooltip: String,
}

/// One row of a group of the panel (R-ROUTES-4).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ValueRouteRow {
    /// The position of the route in the feed. The key of the open state uses it.
    pub index: usize,
    /// R-ROUTES-4 item 2: the name, or the address when the route has no name.
    pub name: String,
    /// R-ROUTES-4 item 3: the fill of the split bar, from 0 to 1.
    pub meter_fraction: f32,
    /// R-ROUTES-4 item 4: the percentage with one decimal place, "fee" for a fee.
    pub share_text: String,
    /// The short address of ADR 0076 under the name (operator choice of
    /// 2026-11-17: the address has its own line at all widths).
    pub short_address: Option<String>,
    /// R-ROUTES-5 item 4: the full address of the open content and of the
    /// hover text of the short address.
    pub full_address: Option<String>,
    /// R-ROUTES-5 item 4: the full address of the open content, with a break
    /// point after "@" and after each dot only (operator choice of 2026-11-17).
    pub open_address: Option<String>,
    /// R-ROUTES-5 item 4: the labeled lines of the open content under the
    /// address (operator choice of 2026-11-17).
    pub details: Vec<ValueRouteDetail>,
    /// The accessible name of the Copy button.
    pub copy_label: String,
}

/// One labeled line of the open content of a row (R-ROUTES-5 item 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueRouteDetail {
    pub label: &'static str,
    pub value: String,
}

/// A zero-width space. The text wrapper of `GPUI` can break a line before it.
const BREAK_POINT: char = '\u{200b}';

/// One group of the panel (R-ROUTES-3 item 2).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ValueRouteGroup {
    /// R-ROUTES-3 item 3: the heading, in capitals.
    pub heading: &'static str,
    /// R-ROUTES-3 item 5: the text of an empty group.
    pub empty_text: &'static str,
    pub rows: Vec<ValueRouteRow>,
}

/// The value routes look of one open cell.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ValueRoutesView {
    /// R-ROUTES-1: the segments. An empty list gives no bar.
    pub segments: Vec<ShareSegment>,
    /// R-ROUTES-1 item 8: the accessible name of the bar.
    pub bar_label: String,
    /// R-ROUTES-2: the parts of the names line. A part moves as one unit.
    pub names: Vec<String>,
    /// R-ROUTES-3: "Recipients", then "Fees". Empty when the cell has no route.
    pub groups: Vec<ValueRouteGroup>,
    /// R-ROUTES-3 item 6: the fee line of ADR 0075 R-VALUE-5 item 7.
    pub fee_line: Option<String>,
    /// R-ROUTES-3 item 8: "None listed." when the cell has no route.
    pub empty_text: Option<&'static str>,
}

/// One recipient with a percentage above 0, in the sequence of the bar.
#[derive(Debug, Clone)]
struct SharePart {
    name: String,
    percent: f64,
    share: String,
    fee: bool,
}

impl SharePart {
    /// The text of R-ROUTES-1 item 7 for one recipient.
    fn detail(&self) -> String {
        let fee = if self.fee { ", fee" } else { "" };
        format!(
            "{} {} (share {}{fee})",
            self.name,
            percent_text(Some(self.percent)),
            self.share
        )
    }
}

/// Parse the raw cell value. A value that is not a route list gives `None`,
/// and the cell then shows the raw text.
#[must_use]
pub(crate) fn value_routes_view(raw_value: &str) -> Option<ValueRoutesView> {
    let routes = crate::metadata::parse_value_routes(raw_value)?;
    Some(view_for_routes(&routes))
}

#[must_use]
pub(crate) fn view_for_routes(routes: &[PaymentRoute]) -> ValueRoutesView {
    if routes.is_empty() {
        return ValueRoutesView {
            segments: Vec::new(),
            bar_label: String::new(),
            names: Vec::new(),
            groups: Vec::new(),
            fee_line: None,
            empty_text: Some(NONE_LISTED),
        };
    }
    let shares: Vec<Option<f64>> = routes.iter().map(|route| route.split).collect();
    let percents = musicindex_value::percentages(&shares);
    let total = shares_total(&shares);
    let (segments, bar_label, names) = summary_bar(routes, &percents);
    let groups = [
        ("RECIPIENTS", false, "No recipients."),
        ("FEES", true, "No fees."),
    ]
    .into_iter()
    .map(|(heading, fee, empty_text)| ValueRouteGroup {
        heading,
        empty_text,
        rows: split_order(routes, fee)
            .into_iter()
            .map(|index| route_row(index, &routes[index], percents[index], total))
            .collect(),
    })
    .collect();
    ValueRoutesView {
        segments,
        bar_label,
        names,
        groups,
        fee_line: fee_line(routes),
        empty_text: None,
    }
}

/// R-ROUTES-1 items 2 and 6: the width of each segment in a bar of `width`
/// with a `gap` between two segments. Each width is in proportion to the
/// percentage, and no width is below `minimum`. The other segments share the
/// width that the minimum takes.
#[must_use]
pub(crate) fn segment_widths(percents: &[f64], width: f32, gap: f32, minimum: f32) -> Vec<f32> {
    #[expect(clippy::cast_precision_loss, reason = "a bar has at most 21 segments")]
    let gaps = gap * percents.len().saturating_sub(1) as f32;
    let available = f64::from((width - gaps).max(0.0));
    let minimum = f64::from(minimum);
    let mut fixed = vec![false; percents.len()];
    loop {
        let fixed_width: f64 = fixed.iter().filter(|value| **value).map(|_| minimum).sum();
        let free_total: f64 = percents
            .iter()
            .zip(&fixed)
            .filter(|(_, fixed)| !**fixed)
            .map(|(percent, _)| *percent)
            .sum();
        let scale = if free_total > 0.0 {
            (available - fixed_width).max(0.0) / free_total
        } else {
            0.0
        };
        let mut changed = false;
        for (index, percent) in percents.iter().enumerate() {
            if !fixed[index] && percent * scale < minimum {
                fixed[index] = true;
                changed = true;
            }
        }
        if !changed {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a width in pixels fits in f32"
            )]
            return percents
                .iter()
                .zip(&fixed)
                .map(|(percent, fixed)| {
                    if *fixed {
                        minimum as f32
                    } else {
                        (percent * scale) as f32
                    }
                })
                .collect();
        }
    }
}

/// ADR 0075 R-VALUE-5 item 1: one decimal place, then " %". No percentage
/// gives "—". The space is a no-break space, as in the site.
#[must_use]
pub(crate) fn percent_text(percent: Option<f64>) -> String {
    percent.map_or_else(|| "—".to_string(), |value| format!("{value:.1}\u{a0}%"))
}

fn is_fee(route: &PaymentRoute) -> bool {
    route.fee.unwrap_or(false)
}

/// The name of a route: its name, then its address, then "Unnamed recipient".
fn route_name(route: &PaymentRoute) -> String {
    [route.recipient_name.as_deref(), route.address.as_deref()]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|text| !text.is_empty())
        .unwrap_or("Unnamed recipient")
        .to_string()
}

/// The raw share as the feed states it. `f64` display has no ".0".
fn raw_share(route: &PaymentRoute) -> String {
    route
        .split
        .map(|split| split.to_string())
        .unwrap_or_default()
}

fn address(route: &PaymentRoute) -> Option<&str> {
    route
        .address
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

/// R-ROUTES-1 and R-ROUTES-2: the segments, the accessible name and the
/// names line. The site function `shareParts` is the reference.
fn summary_bar(
    routes: &[PaymentRoute],
    percents: &[Option<f64>],
) -> (Vec<ShareSegment>, String, Vec<String>) {
    // R-ROUTES-1 items 2 and 3: no segment for 0 % or no percentage. The
    // recipients that are not fees come first, the largest first.
    let mut parts: Vec<SharePart> = routes
        .iter()
        .zip(percents)
        .filter_map(|(route, percent)| {
            percent
                .filter(|value| *value > 0.0)
                .map(|percent| SharePart {
                    name: route_name(route),
                    percent,
                    share: raw_share(route),
                    fee: is_fee(route),
                })
        })
        .collect();
    parts.sort_by(|a, b| {
        a.fee
            .cmp(&b.fee)
            .then_with(|| b.percent.total_cmp(&a.percent))
    });
    if parts.is_empty() {
        return (Vec::new(), String::new(), Vec::new());
    }

    // R-ROUTES-1 item 4: name the largest recipients until the bar has 20
    // named segments, or until the rest is 10 % or less and the next
    // recipient has less than 5 %.
    let mut by_size: Vec<usize> = (0..parts.len()).collect();
    by_size.sort_by(|&a, &b| parts[b].percent.total_cmp(&parts[a].percent));
    let mut named = vec![false; parts.len()];
    let mut named_count = 0;
    let mut rest: f64 = parts.iter().map(|part| part.percent).sum();
    for index in by_size {
        let percent = parts[index].percent;
        if named_count == SHARE_PARTS_MAX || (rest <= SHARE_OTHER_MAX && percent < SHARE_SMALL) {
            break;
        }
        named[index] = true;
        named_count += 1;
        rest -= percent;
    }
    // "Other" with one recipient hides one name for no space.
    if parts.len() - named_count == 1 {
        named.fill(true);
    }

    let mut segments: Vec<ShareSegment> = parts
        .iter()
        .zip(&named)
        .filter(|(_, named)| **named)
        .map(|(part, _)| ShareSegment {
            kind: if part.fee {
                ShareSegmentKind::Fee
            } else {
                ShareSegmentKind::Split
            },
            percent: part.percent,
            tooltip: part.detail(),
        })
        .collect();
    let others: Vec<&SharePart> = parts
        .iter()
        .zip(&named)
        .filter(|(_, named)| !**named)
        .map(|(part, _)| part)
        .collect();
    if !others.is_empty() {
        segments.push(ShareSegment {
            kind: ShareSegmentKind::Other,
            percent: others.iter().map(|part| part.percent).sum(),
            tooltip: others
                .iter()
                .map(|part| part.detail())
                .collect::<Vec<_>>()
                .join(", "),
        });
    }

    // R-ROUTES-1 item 8: each recipient with its percentage, and "fee".
    let bar_label = format!(
        "Shares: {}",
        parts
            .iter()
            .map(|part| {
                let fee = if part.fee { " (fee)" } else { "" };
                format!("{} {}{fee}", part.name, percent_text(Some(part.percent)))
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

    let legend = names_line(&parts, &named);
    (segments, bar_label, legend)
}

/// R-ROUTES-2 items 2 and 3: the parts of the names line.
fn names_line(parts: &[SharePart], named: &[bool]) -> Vec<String> {
    // R-ROUTES-2 item 2: the 3 largest named segments, in the sequence of the bar.
    let mut largest: Vec<usize> = (0..parts.len()).filter(|&index| named[index]).collect();
    largest.sort_by(|&a, &b| parts[b].percent.total_cmp(&parts[a].percent));
    largest.truncate(SHARE_NAMED);
    largest.sort_unstable();
    let mut legend: Vec<String> = largest
        .iter()
        .map(|&index| {
            let part = &parts[index];
            format!(
                "{} {}",
                names_line_name(&part.name),
                percent_text(Some(part.percent))
            )
        })
        .collect();
    // R-ROUTES-2 item 3: "N more" and the total percentage of the rest.
    let hidden = parts.len() - largest.len();
    if hidden > 0 {
        let total: f64 = parts.iter().map(|part| part.percent).sum();
        let shown: f64 = largest.iter().map(|&index| parts[index].percent).sum();
        legend.push(format!(
            "{hidden} more {}",
            percent_text(Some(total - shown))
        ));
    }
    legend
}

/// R-ROUTES-2 item 4: a Lightning address shows only the part before "@".
fn names_line_name(name: &str) -> &str {
    name.split_once('@').map_or(name, |(user, _)| user)
}

/// R-ROUTES-3 item 4: the routes of one group, the largest share first. Rows
/// with the same share keep the sequence of the feed. A route with no valid
/// share counts as 0.
fn split_order(routes: &[PaymentRoute], fee: bool) -> Vec<usize> {
    let amount =
        |route: &PaymentRoute| route.split.filter(|value| value.is_finite()).unwrap_or(0.0);
    let mut indexes: Vec<usize> = routes
        .iter()
        .enumerate()
        .filter(|(_, route)| is_fee(route) == fee)
        .map(|(index, _)| index)
        .collect();
    // `sort_by` is stable, so equal shares keep the feed sequence.
    indexes.sort_by(|&a, &b| amount(&routes[b]).total_cmp(&amount(&routes[a])));
    indexes
}

/// R-ROUTES-4 and R-ROUTES-5 item 4: one row and its open content.
fn route_row(
    index: usize,
    route: &PaymentRoute,
    percent: Option<f64>,
    total: f64,
) -> ValueRouteRow {
    let name = route_name(route);
    let fee = is_fee(route);
    let share_text = format!("{}{}", percent_text(percent), if fee { " fee" } else { "" });
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a fraction from 0 to 1 fits in f32"
    )]
    let meter_fraction = (percent.unwrap_or(0.0).clamp(0.0, 100.0) / 100.0) as f32;
    let full_address = address(route).map(str::to_string);
    let short_address = full_address.as_deref().map(musicindex_value::short_address);
    ValueRouteRow {
        index,
        copy_label: format!("Copy the address of {name}"),
        name,
        meter_fraction,
        share_text,
        short_address,
        open_address: full_address.as_deref().map(address_with_break_points),
        full_address,
        details: details(route, percent, total),
    }
}

/// The sum of the shares that `musicindex_value::percentages` uses: each
/// finite share that is not negative.
fn shares_total(shares: &[Option<f64>]) -> f64 {
    shares
        .iter()
        .filter_map(|share| share.filter(|n| n.is_finite() && *n >= 0.0))
        .sum()
}

/// The open address can break after "@" and after each dot, and not inside a
/// word (operator choice of 2026-11-17).
fn address_with_break_points(address: &str) -> String {
    let mut text = String::with_capacity(address.len() + 8);
    for character in address.chars() {
        text.push(character);
        if matches!(character, '@' | '.') {
            text.push(BREAK_POINT);
        }
    }
    text
}

/// R-ROUTES-5 item 4 (operator choice of 2026-11-17): the labeled lines of the
/// open content. "Type" is the name of the route type. "Shares" is the share of
/// the route and the total that the percentage uses. The custom key and the
/// custom value show when the route states them. The open content does not show
/// the percentage, and it does not show "split".
fn details(route: &PaymentRoute, percent: Option<f64>, total: f64) -> Vec<ValueRouteDetail> {
    let stated = |value: &Option<String>| {
        value
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string)
    };
    let route_type = match stated(&route.route_type).as_deref() {
        Some("lnaddress") => "Lightning address".to_string(),
        Some("node") => "Lightning node".to_string(),
        Some(other) => other.to_string(),
        None => "Not stated".to_string(),
    };
    let shares = match route.split {
        Some(share) if percent.is_some() && share.is_finite() && share >= 0.0 => {
            format!("{share} of {total}")
        }
        _ => "Not stated".to_string(),
    };
    [
        Some(("Type", route_type)),
        Some(("Shares", shares)),
        stated(&route.custom_key).map(|key| ("Custom key", key)),
        stated(&route.custom_value).map(|value| ("Custom value", value)),
    ]
    .into_iter()
    .flatten()
    .map(|(label, value)| ValueRouteDetail { label, value })
    .collect()
}

/// ADR 0075 R-VALUE-5 item 7: a line names each fee, and says that it comes
/// out of the payment. The site text is at `search.html` `feeNoteHtml`.
fn fee_line(routes: &[PaymentRoute]) -> Option<String> {
    let names: Vec<String> = routes
        .iter()
        .filter(|route| is_fee(route))
        .map(|route| {
            route
                .recipient_name
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .unwrap_or("Unnamed recipient")
                .to_string()
        })
        .collect();
    let (last, first) = names.split_last()?;
    if first.is_empty() {
        return Some(format!(
            "The fee to {last} comes out of the payment, not on top of it."
        ));
    }
    Some(format!(
        "The fees to {} and {last} come out of the payment, not on top of it.",
        first.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(name: &str, split: f64, fee: bool) -> PaymentRoute {
        PaymentRoute {
            recipient_name: Some(name.to_string()),
            route_type: Some("node".to_string()),
            split: Some(split),
            fee: Some(fee),
            address: Some(format!("{name}-address-0123456789abcdef")),
            custom_key: None,
            custom_value: None,
        }
    }

    fn kinds(view: &ValueRoutesView) -> Vec<ShareSegmentKind> {
        view.segments.iter().map(|segment| segment.kind).collect()
    }

    /// R-ROUTES-1 item 3: the recipients that are not fees come first, the
    /// largest first, then the fees, the largest first.
    #[test]
    fn r_routes_1_segments_put_fees_last_and_the_largest_first() {
        let view = view_for_routes(&[
            route("Fee", 10.0, true),
            route("Small", 20.0, false),
            route("Large", 70.0, false),
        ]);
        assert_eq!(
            kinds(&view),
            [
                ShareSegmentKind::Split,
                ShareSegmentKind::Split,
                ShareSegmentKind::Fee
            ]
        );
        let percents: Vec<f64> = view.segments.iter().map(|s| s.percent).collect();
        assert_eq!(percents, [70.0, 20.0, 10.0]);
        assert_eq!(view.segments[2].tooltip, "Fee 10.0\u{a0}% (share 10, fee)");
    }

    /// R-ROUTES-1 item 4: the small recipients join "Other" when the rest is
    /// 10 % or less and the next recipient has less than 5 %.
    #[test]
    fn r_routes_1_small_recipients_join_other() {
        let mut routes = vec![route("Main", 92.0, false)];
        for index in 0..4 {
            routes.push(route(&format!("Tiny{index}"), 2.0, false));
        }
        let view = view_for_routes(&routes);
        assert_eq!(
            kinds(&view),
            [ShareSegmentKind::Split, ShareSegmentKind::Other]
        );
        assert!((view.segments[1].percent - 8.0).abs() < 1e-9);
        assert!(view.segments[1]
            .tooltip
            .starts_with("Tiny0 2.0\u{a0}% (share 2), Tiny1"));
    }

    /// R-ROUTES-1 item 4: when only one recipient is not named, it gets its
    /// own segment and the bar has no "Other".
    #[test]
    fn r_routes_1_one_left_over_recipient_keeps_its_own_segment() {
        let view = view_for_routes(&[route("Main", 96.0, false), route("Tiny", 4.0, false)]);
        assert_eq!(
            kinds(&view),
            [ShareSegmentKind::Split, ShareSegmentKind::Split]
        );
    }

    /// R-ROUTES-1 item 4: the bar names at most 20 segments.
    #[test]
    fn r_routes_1_the_bar_names_at_most_twenty_segments() {
        let routes: Vec<PaymentRoute> = (0..25)
            .map(|index| route(&format!("R{index}"), 4.0, false))
            .collect();
        let view = view_for_routes(&routes);
        assert_eq!(view.segments.len(), 21);
        assert_eq!(view.segments[20].kind, ShareSegmentKind::Other);
    }

    /// R-ROUTES-1 items 1 and 2: no segment for 0 % or for no percentage,
    /// and no bar when no recipient has a percentage above 0.
    #[test]
    fn r_routes_1_zero_and_missing_shares_have_no_segment() {
        let mut empty = route("None", 0.0, false);
        empty.split = None;
        let view = view_for_routes(&[route("Zero", 0.0, false), empty.clone()]);
        assert!(view.segments.is_empty());
        assert!(view.names.is_empty());
        let view = view_for_routes(&[route("Some", 5.0, false), route("Zero", 0.0, false)]);
        assert_eq!(view.segments.len(), 1);
    }

    /// R-ROUTES-1 item 8: the accessible name lists each recipient with its
    /// percentage and "fee".
    #[test]
    fn r_routes_1_bar_label_lists_each_recipient() {
        let view = view_for_routes(&[route("A", 75.0, false), route("B", 25.0, true)]);
        assert_eq!(view.bar_label, "Shares: A 75.0\u{a0}%, B 25.0\u{a0}% (fee)");
    }

    /// R-ROUTES-2 items 2 and 3: the example of the ADR.
    #[test]
    fn r_routes_2_names_line_names_three_and_n_more() {
        let view = view_for_routes(&[
            route("Music Side Project", 40.0, false),
            route("HeyCitizenNode", 25.0, false),
            route("aaa", 15.0, false),
            route("bbb", 10.0, false),
            route("ccc", 6.0, false),
            route("ddd", 4.0, false),
        ]);
        assert_eq!(
            view.names.join(PART_SEPARATOR),
            "Music Side Project 40.0\u{a0}% · HeyCitizenNode 25.0\u{a0}% · aaa 15.0\u{a0}% · \
             3 more 20.0\u{a0}%"
        );
    }

    /// R-ROUTES-2 item 4: a Lightning address shows only the part before "@".
    #[test]
    fn r_routes_2_names_line_shortens_a_lightning_address() {
        let mut lightning = route("x", 50.0, false);
        lightning.recipient_name = None;
        lightning.address = Some("chadf@getalby.com".to_string());
        let view = view_for_routes(&[lightning, route("Band", 50.0, false)]);
        assert_eq!(view.names, ["chadf 50.0\u{a0}%", "Band 50.0\u{a0}%"]);
    }

    /// R-ROUTES-3 items 2, 4 and 5: two groups. The rows are the largest share
    /// first, and equal shares keep the feed sequence.
    #[test]
    fn r_routes_3_groups_sort_rows_by_share() {
        let view = view_for_routes(&[
            route("First", 10.0, false),
            route("Big", 60.0, false),
            route("Second", 10.0, false),
        ]);
        assert_eq!(view.groups.len(), 2);
        assert_eq!(view.groups[0].heading, "RECIPIENTS");
        let names: Vec<&str> = view.groups[0]
            .rows
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(names, ["Big", "First", "Second"]);
        assert_eq!(view.groups[1].heading, "FEES");
        assert!(view.groups[1].rows.is_empty());
        assert_eq!(view.groups[1].empty_text, "No fees.");
        assert_eq!(view.fee_line, None);
    }

    /// R-ROUTES-3 item 6: the fee line of ADR 0075 R-VALUE-5 item 7.
    #[test]
    fn r_routes_3_fee_line_names_each_fee() {
        let one = view_for_routes(&[route("Band", 95.0, false), route("App", 5.0, true)]);
        assert_eq!(
            one.fee_line.as_deref(),
            Some("The fee to App comes out of the payment, not on top of it.")
        );
        let three = view_for_routes(&[
            route("A", 1.0, true),
            route("B", 1.0, true),
            route("C", 1.0, true),
        ]);
        assert_eq!(
            three.fee_line.as_deref(),
            Some("The fees to A, B and C come out of the payment, not on top of it.")
        );
        assert_eq!(three.groups[0].empty_text, "No recipients.");
        assert!(three.groups[0].rows.is_empty());
    }

    /// R-ROUTES-3 item 8: a section with no recipient shows "None listed.".
    #[test]
    fn r_routes_3_no_route_gives_none_listed() {
        let view = value_routes_view("[]").expect("a route list");
        assert_eq!(view.empty_text, Some("None listed."));
        assert!(view.groups.is_empty());
        assert!(value_routes_view("not json").is_none());
    }

    /// R-ROUTES-4: the name, the split bar, the percentage with one decimal
    /// place and "fee", and the short address of ADR 0076.
    #[test]
    fn r_routes_4_row_values() {
        let mut no_share = route("Nobody", 0.0, false);
        no_share.split = None;
        let mut no_name = route("x", 1.0, true);
        no_name.recipient_name = None;
        no_name.address = Some("me@example.com".to_string());
        let view = view_for_routes(&[route("Band", 2.0, false), no_name, no_share]);
        let band = &view.groups[0].rows[0];
        assert_eq!(band.share_text, "66.7\u{a0}%");
        assert!((band.meter_fraction - 0.666_666_7).abs() < 1e-6);
        assert_eq!(
            band.short_address.as_deref(),
            Some("Band-addre\u{2026}89abcdef")
        );
        assert_eq!(
            band.full_address.as_deref(),
            Some("Band-address-0123456789abcdef")
        );
        let nobody = &view.groups[0].rows[1];
        assert_eq!(nobody.share_text, "—");
        assert!(nobody.meter_fraction.abs() < f32::EPSILON);
        let fee = &view.groups[1].rows[0];
        assert_eq!(fee.name, "me@example.com");
        assert_eq!(fee.share_text, "33.3\u{a0}% fee");
        // ADR 0076 R-ADDRESS-2: a Lightning address stays in full.
        assert_eq!(fee.short_address.as_deref(), Some("me@example.com"));
    }

    /// R-ROUTES-5 item 4: the labeled lines of the open content (operator
    /// choice of 2026-11-17).
    #[test]
    fn r_routes_5_open_content_gives_labeled_lines() {
        let mut keyed = route("Keyed", 90.0, false);
        keyed.custom_key = Some("696969".to_string());
        keyed.custom_value = Some("abc".to_string());
        let mut lightning = route("Lightning", 7.5, false);
        lightning.route_type = Some("lnaddress".to_string());
        let mut other = route("Other", 2.5, true);
        other.route_type = Some("keysend".to_string());
        let mut bare = route("Bare", 0.0, false);
        bare.route_type = None;
        bare.split = None;
        let view = view_for_routes(&[keyed, lightning, other, bare]);
        let lines = |rows: &[ValueRouteRow], name: &str| -> Vec<(&'static str, String)> {
            rows.iter()
                .find(|row| row.name == name)
                .expect("row")
                .details
                .iter()
                .map(|detail| (detail.label, detail.value.clone()))
                .collect()
        };
        let recipients = &view.groups[0].rows;
        assert_eq!(
            lines(recipients, "Keyed"),
            [
                ("Type", "Lightning node".to_string()),
                ("Shares", "90 of 100".to_string()),
                ("Custom key", "696969".to_string()),
                ("Custom value", "abc".to_string()),
            ]
        );
        assert_eq!(
            lines(recipients, "Lightning"),
            [
                ("Type", "Lightning address".to_string()),
                ("Shares", "7.5 of 100".to_string()),
            ]
        );
        assert_eq!(
            lines(&view.groups[1].rows, "Other"),
            [
                ("Type", "keysend".to_string()),
                ("Shares", "2.5 of 100".to_string()),
            ]
        );
        assert_eq!(
            lines(recipients, "Bare"),
            [
                ("Type", "Not stated".to_string()),
                ("Shares", "Not stated".to_string()),
            ]
        );
        // No line gives the percentage or "split".
        for row in view.groups.iter().flat_map(|group| &group.rows) {
            for detail in &row.details {
                assert!(!detail.value.contains('%'), "{detail:?}");
                assert!(!detail.value.contains("split"), "{detail:?}");
            }
        }
        assert_eq!(recipients[0].copy_label, "Copy the address of Keyed");
    }

    /// The open address breaks only after "@" and after each dot.
    #[test]
    fn r_routes_5_open_address_breaks_after_at_and_dots() {
        let mut lightning = route("Lightning", 1.0, false);
        lightning.address = Some("chadf@getalby.com".to_string());
        let view = view_for_routes(&[lightning]);
        let row = &view.groups[0].rows[0];
        assert_eq!(
            row.open_address.as_deref(),
            Some("chadf@\u{200b}getalby.\u{200b}com")
        );
        assert_eq!(row.full_address.as_deref(), Some("chadf@getalby.com"));
    }

    /// R-ROUTES-1 item 6: the widths fill the bar, and no segment is below 3 px.
    #[test]
    fn r_routes_1_segment_widths_keep_the_minimum() {
        let widths = segment_widths(&[50.0, 50.0], 160.0, 2.0, 3.0);
        assert_eq!(widths, [79.0, 79.0]);
        let widths = segment_widths(&[98.0, 1.0, 1.0], 160.0, 2.0, 3.0);
        assert!((widths[1] - 3.0).abs() < 1e-4 && (widths[2] - 3.0).abs() < 1e-4);
        let total: f32 = widths.iter().sum::<f32>() + 4.0;
        assert!((total - 160.0).abs() < 1e-3, "{widths:?}");
    }

    /// ADR 0075 R-VALUE-5 item 2: one recipient gets 100 %, whatever its share.
    #[test]
    fn one_recipient_gets_one_hundred_percent() {
        let view = view_for_routes(&[route("Solo", 0.0, false)]);
        assert_eq!(view.groups[0].rows[0].share_text, "100.0\u{a0}%");
        assert_eq!(view.names, ["Solo 100.0\u{a0}%"]);
    }
}
