//! The canonical payment route of one raw `podcast:value` block
//! (ADR 0076 Decision 9, packet 003). ADR 0099 keeps only Lightning routes.
//!
//! The database keeps the raw block in `tracks.item_value_json` and
//! `feeds.podcast_value_json`, in the JSON form of
//! `helpers::value_block_json`. This module converts that block to the
//! `api::PaymentRoute` list that the file frame `TXXX:Podcast Value
//! Routes` holds. Migration 17, the playlist RSS check and the subscribe
//! persist step use this one converter.

#![warn(clippy::pedantic)]

use serde_json::Value;

use crate::api::PaymentRoute;

/// Converts one raw `podcast:value` block to its payment route list.
///
/// Each Lightning `podcast:valueRecipient` gives one route: `name`, `type`,
/// `address`, `split`, `fee`, `customKey` and `customValue`. A block without
/// such a recipient gives an empty list. The `fee` value follows musicindex
/// ADR 0086 R-FEED-4 item 5: no `fee` value removes a recipient.
///
/// ADR 0099 Decision 2: a block that is not a Lightning block gives an empty
/// list, and a recipient that is not a Lightning recipient gives no route.
///
/// # Errors
///
/// Returns a reason when the block is not a JSON object, when its recipient
/// list is not an array, or when a `split` value has no valid form.
pub(crate) fn payment_routes_from_value_block(raw: &str) -> Result<Vec<PaymentRoute>, String> {
    let value: Value =
        serde_json::from_str(raw).map_err(|error| format!("block is not JSON: {error}"))?;
    let Some(block) = value.as_object() else {
        return Err("block is not a JSON object".to_owned());
    };
    let recipients = match block.get("children").and_then(|children| {
        children
            .as_object()
            .and_then(|children| children.get("valueRecipient"))
    }) {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(recipients)) => recipients,
        Some(_) => return Err("valueRecipient is not an array".to_owned()),
    };
    let block_type = block
        .get("attrs")
        .and_then(|attrs| attrs.get("type"))
        .and_then(Value::as_str);
    if !is_lightning_block_type(block_type) {
        return Ok(Vec::new());
    }
    recipients
        .iter()
        .filter(|recipient| is_lightning_recipient(recipient))
        .map(recipient_route)
        .collect()
}

/// Tells if a `podcast:value` block type is Lightning (ADR 0099 Decision 1).
/// A block with no type is a Lightning block.
#[must_use]
pub(crate) fn is_lightning_block_type(block_type: Option<&str>) -> bool {
    let block_type = block_type.map(str::trim).unwrap_or_default();
    block_type.is_empty() || block_type.eq_ignore_ascii_case("lightning")
}

/// Tells if a recipient is a Lightning recipient (ADR 0099 Decision 2): the
/// type is `lnaddress`, or the type is `node` and the address is a public key
/// of 66 hexadecimal characters.
fn is_lightning_recipient(recipient: &Value) -> bool {
    let attr = |name: &str| {
        recipient
            .get("attrs")
            .and_then(|attrs| attrs.get(name))
            .and_then(Value::as_str)
    };
    is_lightning_type_and_address(attr("type"), attr("address"))
}

/// Tells if a stored route is a Lightning route (ADR 0099 Decision 2).
#[must_use]
pub(crate) fn is_lightning_route(route: &PaymentRoute) -> bool {
    is_lightning_type_and_address(route.route_type.as_deref(), route.address.as_deref())
}

fn is_lightning_type_and_address(route_type: Option<&str>, address: Option<&str>) -> bool {
    let route_type = route_type.map(str::trim).unwrap_or_default();
    if route_type.eq_ignore_ascii_case("lnaddress") {
        return true;
    }
    let address = address.map(str::trim).unwrap_or_default();
    route_type.eq_ignore_ascii_case("node")
        && address.len() == 66
        && address.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// The canonical JSON of one raw block, for a `payment_routes_json` column.
/// A missing block or a block that does not convert gives `None`.
#[must_use]
pub(crate) fn canonical_routes_json(raw: Option<&str>) -> Option<String> {
    let routes = payment_routes_from_value_block(raw?).ok()?;
    serde_json::to_string(&routes).ok()
}

fn recipient_route(recipient: &Value) -> Result<PaymentRoute, String> {
    let attrs = recipient.get("attrs").unwrap_or(&Value::Null);
    let text = |name: &str| -> Option<String> {
        attrs
            .get(name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    let split = text("split")
        .map(|split| {
            split
                .parse::<f64>()
                .ok()
                .filter(|split| split.is_finite())
                .ok_or_else(|| format!("split `{split}` is not a number"))
        })
        .transpose()?;
    let (fee, warning) = parse_fee(text("fee").as_deref());
    if let Some(warning) = warning {
        eprintln!("warning: value recipient {:?}: {warning}", text("name"));
    }
    Ok(PaymentRoute {
        recipient_name: text("name"),
        route_type: text("type").map(|route_type| route_type.to_ascii_lowercase()),
        split,
        fee: Some(fee),
        address: text("address"),
        custom_key: text("customKey"),
        custom_value: text("customValue"),
    })
}

/// The parsed form of a `fee` attribute (musicindex ADR 0086 R-FEED-4 item
/// 5), with a warning for a value that the rule does not name.
///
/// `true`, `1` and `yes`, in any case, mean a fee. `false`, `0`, `no`, an
/// empty value and a missing attribute mean not a fee. Each other value means
/// not a fee and gives a warning. No value removes the recipient.
fn parse_fee(fee: Option<&str>) -> (bool, Option<String>) {
    let Some(fee) = fee.map(str::trim).filter(|fee| !fee.is_empty()) else {
        return (false, None);
    };
    let is = |names: [&str; 3]| names.iter().any(|name| fee.eq_ignore_ascii_case(name));
    if is(["true", "1", "yes"]) {
        (true, None)
    } else if is(["false", "0", "no"]) {
        (false, None)
    } else {
        (
            false,
            Some(format!(
                "fee `{fee}` is not a known value; read as not a fee"
            )),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A block in the form that `helpers::value_block_json` writes for a
    /// recorded two-recipient `podcast:value` element.
    const RECORDED_BLOCK: &str = r#"{"value":null,"attrs":{"type":"lightning","method":"keysend","suggested":"0.00000005000"},"children":{"valueRecipient":[{"value":null,"attrs":{"name":"The Band","type":"node","address":"03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a","split":"95"},"children":{}},{"value":null,"attrs":{"name":"Podcastindex.org","type":"node","address":"03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a","split":"5","customKey":"696969","customValue":"eChoVKtO1KujpAA5HCoB","fee":"true"},"children":{}}]}}"#;

    /// R3-01: a recorded block converts to the expected list, and a block
    /// without a recipient converts to an empty list.
    #[test]
    fn adr_0076_route_readiness_value_block_converts_to_payment_routes() {
        let routes = payment_routes_from_value_block(RECORDED_BLOCK).unwrap();
        assert_eq!(routes.len(), 2);
        let band = &routes[0];
        assert_eq!(band.recipient_name.as_deref(), Some("The Band"));
        assert_eq!(band.route_type.as_deref(), Some("node"));
        assert_eq!(band.split, Some(95.0));
        assert_eq!(band.fee, Some(false));
        assert_eq!(
            band.address.as_deref(),
            Some("03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a")
        );
        assert_eq!(band.custom_key, None);
        assert_eq!(band.custom_value, None);
        let index = &routes[1];
        assert_eq!(index.recipient_name.as_deref(), Some("Podcastindex.org"));
        assert_eq!(index.split, Some(5.0));
        assert_eq!(index.fee, Some(true));
        assert_eq!(index.custom_key.as_deref(), Some("696969"));
        assert_eq!(index.custom_value.as_deref(), Some("eChoVKtO1KujpAA5HCoB"));

        for empty in [
            r#"{"value":null,"attrs":{"type":"lightning"},"children":{}}"#,
            r#"{"value":null,"attrs":{"type":"lightning"},"children":{"valueRecipient":[]}}"#,
            r#"{"attrs":{}}"#,
        ] {
            assert_eq!(payment_routes_from_value_block(empty).unwrap().len(), 0);
            assert_eq!(canonical_routes_json(Some(empty)).as_deref(), Some("[]"));
        }
    }

    #[test]
    fn adr_0076_route_readiness_invalid_value_block_does_not_convert() {
        for invalid in [
            "not json",
            "[]",
            r#"{"children":{"valueRecipient":{}}}"#,
            r#"{"children":{"valueRecipient":[{"attrs":{"type":"lnaddress","split":"most"}}]}}"#,
        ] {
            assert!(
                payment_routes_from_value_block(invalid).is_err(),
                "{invalid}"
            );
            assert_eq!(canonical_routes_json(Some(invalid)), None);
        }
        assert_eq!(canonical_routes_json(None), None);
    }

    /// ADR 0086 R-FEED-4 item 5: each `fee` value gives a fee or not a fee,
    /// and an unknown value gives a warning.
    #[test]
    fn adr_0086_fee_values_give_a_fee_or_not_a_fee() {
        for fee in ["true", "TRUE", "True", "1", "yes", "YES", " yes "] {
            assert_eq!(parse_fee(Some(fee)), (true, None), "{fee}");
        }
        for fee in ["false", "FALSE", "0", "no", "No", "", "  "] {
            assert_eq!(parse_fee(Some(fee)), (false, None), "{fee:?}");
        }
        assert_eq!(parse_fee(None), (false, None));
        for fee in ["maybe", "2", "y", "on"] {
            let (is_fee, warning) = parse_fee(Some(fee));
            assert!(!is_fee, "{fee}");
            assert!(
                warning.is_some_and(|warning| warning.contains(fee)),
                "{fee}"
            );
        }
    }

    /// ADR 0086 R-FEED-4 item 5: a recipient with an empty or unknown `fee`
    /// stays in the route list as not a fee.
    #[test]
    fn adr_0086_unusual_fee_keeps_the_recipient() {
        let raw = r#"{"attrs":{"type":"lightning"},"children":{"valueRecipient":[{"attrs":{"name":"Empty","type":"lnaddress","address":"a@b.c","split":"50","fee":""}},{"attrs":{"name":"Unknown","type":"lnaddress","address":"d@e.f","split":"40","fee":"maybe"}},{"attrs":{"name":"Yes","type":"lnaddress","address":"g@h.i","split":"10","fee":"Yes"}}]}}"#;
        let routes = payment_routes_from_value_block(raw).unwrap();
        let fees = routes
            .iter()
            .map(|route| (route.recipient_name.as_deref(), route.fee))
            .collect::<Vec<_>>();
        assert_eq!(
            fees,
            [
                (Some("Empty"), Some(false)),
                (Some("Unknown"), Some(false)),
                (Some("Yes"), Some(true)),
            ]
        );
        assert!(canonical_routes_json(Some(raw)).is_some());
    }

    const NODE: &str = "03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a";

    fn block(block_type: Option<&str>, recipients: &[(Option<&str>, &str)]) -> String {
        let recipients = recipients
            .iter()
            .map(|(route_type, address)| {
                let mut attrs = serde_json::json!({"address": address, "split": "50"});
                if let Some(route_type) = route_type {
                    attrs["type"] = serde_json::json!(route_type);
                }
                serde_json::json!({"attrs": attrs})
            })
            .collect::<Vec<_>>();
        let mut attrs = serde_json::json!({});
        if let Some(block_type) = block_type {
            attrs["type"] = serde_json::json!(block_type);
        }
        serde_json::json!({"attrs": attrs, "children": {"valueRecipient": recipients}}).to_string()
    }

    #[test]
    fn adr_0099_block_that_is_not_lightning_gives_no_route() {
        let recipients = [
            (Some("node"), NODE),
            (Some("lnaddress"), "band@example.com"),
        ];
        for lightning in [None, Some("lightning"), Some(" Lightning "), Some("")] {
            let routes = payment_routes_from_value_block(&block(lightning, &recipients)).unwrap();
            assert_eq!(routes.len(), 2, "{lightning:?}");
        }
        for other in ["usdc", "bitcoin", "HBD"] {
            let raw = block(Some(other), &recipients);
            assert_eq!(payment_routes_from_value_block(&raw).unwrap().len(), 0);
            assert_eq!(canonical_routes_json(Some(&raw)).as_deref(), Some("[]"));
        }
    }

    #[test]
    fn adr_0099_recipient_that_is_not_lightning_gives_no_route() {
        let short = &NODE[..64];
        let not_hex = NODE.replace('a', "g");
        let raw = block(
            Some("lightning"),
            &[
                (Some("Node"), NODE),
                (Some("lnaddress"), "band@example.com"),
                (
                    Some("address"),
                    "0x52908400098527886E0F7030069857D2E4169EE7",
                ),
                (Some("node"), short),
                (Some("node"), &not_hex),
                (Some("keysend"), NODE),
                (None, NODE),
            ],
        );
        let routes = payment_routes_from_value_block(&raw).unwrap();
        let kept = routes
            .iter()
            .map(|route| (route.route_type.as_deref(), route.address.as_deref()))
            .collect::<Vec<_>>();
        assert_eq!(
            kept,
            [
                (Some("node"), Some(NODE)),
                (Some("lnaddress"), Some("band@example.com")),
            ]
        );
    }

    /// A recipient that gives no route cannot make the block invalid.
    #[test]
    fn adr_0099_discarded_recipient_with_an_invalid_split_does_not_fail_the_block() {
        let raw = r#"{"attrs":{"type":"lightning"},"children":{"valueRecipient":[{"attrs":{"type":"address","address":"0xabc","split":"most"}}]}}"#;
        assert_eq!(payment_routes_from_value_block(raw).unwrap().len(), 0);
    }
}
