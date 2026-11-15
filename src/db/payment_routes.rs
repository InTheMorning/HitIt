//! The stored payment route of each track and feed (ADR 0076 Decision 9,
//! packet 003).
//!
//! Migration 17 adds `tracks.payment_routes_json` and
//! `feeds.payment_routes_json`. Each column holds the canonical
//! `api::PaymentRoute` list of the raw `podcast:value` block in the same row.
//! The stored route of a track is its own column, else the column of its feed.
//!
//! The playlist RSS check and the subscribe persist step refresh a column when
//! they write a raw block. A `MusicIndex` route goes into the track column only
//! when the track has no stored route. It never replaces a stored route.

#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

use crate::api::PaymentRoute;
use crate::rss::value_routes::{canonical_routes_json, is_lightning_route};

/// The read contract of `feeds` and `tracks` after migration 17: the
/// version-16 columns, then the column that migration 17 appends.
pub(super) const EXTENDED_COLUMNS: &[(&str, &[&str])] = &[
    (
        "feeds",
        &[
            "id",
            "feed_url",
            "feed_guid",
            "title",
            "link",
            "language",
            "description",
            "podcast_medium",
            "album_image_href",
            "album_image_mime",
            "people_json",
            "podcast_value_json",
            "is_subscribed",
            "last_fetched_at",
            "extra_json",
            "musicindex_updated_at",
            "album_artist",
            "payment_routes_json",
        ],
    ),
    (
        "tracks",
        &[
            "id",
            "feed_id",
            "item_guid",
            "enclosure_url",
            "enclosure_type",
            "link",
            "pub_date",
            "track_title",
            "artist_name",
            "album_title",
            "album_artist_name",
            "disc_number",
            "track_number",
            "duration_seconds",
            "itunes_duration_raw",
            "itunes_explicit",
            "track_image_href",
            "track_image_mime",
            "people_json",
            "item_value_json",
            "is_in_library",
            "extra_json",
            "removed_from_feed_at",
            "removed_from_feed_confirmed_at",
            "payment_routes_json",
        ],
    ),
];

const DDL: &str = r"
ALTER TABLE feeds ADD COLUMN payment_routes_json TEXT NULL;
ALTER TABLE tracks ADD COLUMN payment_routes_json TEXT NULL;
";

/// The result of the canonical route fill of migration 17.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RouteFill {
    /// Feed rows whose raw block converted.
    pub(crate) feeds_filled: usize,
    /// Feed rows with a raw block that did not convert. Their column stays null.
    pub(crate) feeds_unconverted: usize,
    /// Track rows whose raw block converted.
    pub(crate) tracks_filled: usize,
    /// Track rows with a raw block that did not convert. Their column stays null.
    pub(crate) tracks_unconverted: usize,
}

/// Migration 17 (ADR 0016 registry, ADR 0076 packet 003).
pub(super) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)
        .context("Add the payment route columns")?;
    let fill = fill_payment_routes(conn)?;
    // The migration registry returns no value, so the counts go to the
    // error stream with the other startup messages.
    eprintln!(
        "Migration 17 stored the payment route of {} feeds and {} tracks from their RSS value blocks. The value blocks of {} feeds and {} tracks did not convert, and their stored route stays empty.",
        fill.feeds_filled, fill.tracks_filled, fill.feeds_unconverted, fill.tracks_unconverted
    );
    Ok(())
}

/// Fill each empty `payment_routes_json` column from the raw block of its
/// row. A row whose block does not convert stays null and is counted.
pub(crate) fn fill_payment_routes(conn: &Connection) -> Result<RouteFill> {
    let (feeds_filled, feeds_unconverted) = fill_table(conn, "feeds", "podcast_value_json")?;
    let (tracks_filled, tracks_unconverted) = fill_table(conn, "tracks", "item_value_json")?;
    Ok(RouteFill {
        feeds_filled,
        feeds_unconverted,
        tracks_filled,
        tracks_unconverted,
    })
}

fn fill_table(conn: &Connection, table: &str, raw_column: &str) -> Result<(usize, usize)> {
    let rows = conn
        .prepare(&format!(
            "SELECT id, {raw_column} FROM {table}
             WHERE {raw_column} IS NOT NULL AND payment_routes_json IS NULL
             ORDER BY id"
        ))?
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .with_context(|| format!("read {table}.{raw_column}"))?;
    let mut filled = 0;
    let mut unconverted = 0;
    for (id, raw) in rows {
        match canonical_routes_json(Some(&raw)) {
            Some(routes) => {
                conn.execute(
                    &format!("UPDATE {table} SET payment_routes_json = ?2 WHERE id = ?1"),
                    params![id, routes],
                )
                .with_context(|| format!("write {table}.payment_routes_json"))?;
                filled += 1;
            }
            None => unconverted += 1,
        }
    }
    Ok((filled, unconverted))
}

/// The result of migration 26.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LightningOnly {
    /// Rows whose stored route changed to the route of their raw block.
    pub(crate) from_block: usize,
    /// Rows with no raw block that converts whose stored route lost a recipient.
    pub(crate) without_block: usize,
}

/// Migration 26 (ADR 0016 registry, ADR 0099 Decision 5).
pub(super) fn apply_lightning_only(conn: &Connection) -> Result<()> {
    let result = keep_lightning_routes(conn)?;
    eprintln!(
        "Migration 26 calculated the payment route of {} rows again from their RSS value blocks. It removed the recipients that are not Lightning from {} rows with no value block.",
        result.from_block, result.without_block
    );
    Ok(())
}

/// Make each stored route hold only Lightning routes (ADR 0099 Decision 5).
///
/// A row with a raw block that converts gets the route of that block again.
/// Each other row loses each recipient that is not a Lightning recipient,
/// and a route with no recipient left becomes null. A stored route that
/// does not decode stays as it is.
pub(crate) fn keep_lightning_routes(conn: &Connection) -> Result<LightningOnly> {
    let mut result = LightningOnly::default();
    for (table, raw_column) in [
        ("feeds", "podcast_value_json"),
        ("tracks", "item_value_json"),
    ] {
        let rows = conn
            .prepare(&format!(
                "SELECT id, {raw_column}, payment_routes_json FROM {table}
                 WHERE {raw_column} IS NOT NULL OR payment_routes_json IS NOT NULL
                 ORDER BY id"
            ))?
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .with_context(|| format!("read {table}.payment_routes_json"))?;
        for (id, raw, stored) in rows {
            let from_block = raw
                .as_deref()
                .and_then(|raw| canonical_routes_json(Some(raw)));
            let has_block = from_block.is_some();
            let routes = if has_block {
                from_block
            } else {
                let Some(Ok(routes)) = stored
                    .as_deref()
                    .map(serde_json::from_str::<Vec<PaymentRoute>>)
                else {
                    continue;
                };
                let kept = routes
                    .into_iter()
                    .filter(is_lightning_route)
                    .collect::<Vec<_>>();
                if kept.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&kept).context("encode the payment route")?)
                }
            };
            if routes == stored {
                continue;
            }
            conn.execute(
                &format!("UPDATE {table} SET payment_routes_json = ?2 WHERE id = ?1"),
                params![id, routes],
            )
            .with_context(|| format!("write {table}.payment_routes_json"))?;
            if has_block {
                result.from_block += 1;
            } else {
                result.without_block += 1;
            }
        }
    }
    Ok(result)
}

/// The stored route of one track: its own column, else the column of its
/// feed. `None` means that the database has no route for the track.
///
/// # Errors
///
/// Returns an error when the rows cannot be read or a column is not a route
/// list.
pub(crate) fn stored_route(conn: &Connection, track_id: i64) -> Result<Option<Vec<PaymentRoute>>> {
    let columns = conn
        .query_row(
            "SELECT t.payment_routes_json, f.payment_routes_json
             FROM tracks t JOIN feeds f ON f.id = t.feed_id
             WHERE t.id = ?1",
            [track_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .optional()
        .context("read the stored payment route")?;
    let Some((track, feed)) = columns else {
        return Ok(None);
    };
    track
        .or(feed)
        .map(|json| {
            serde_json::from_str::<Vec<PaymentRoute>>(&json)
                .context("decode the stored payment route")
        })
        .transpose()
}

/// Store a `MusicIndex` route for a track that has no stored route. The
/// function changes nothing when the track or its feed has a stored route,
/// or when the list is empty. It returns whether it wrote the column.
///
/// # Errors
///
/// Returns an error when the update fails.
pub(crate) fn store_musicindex_route(
    conn: &Connection,
    track_id: i64,
    routes: &[PaymentRoute],
) -> Result<bool> {
    if routes.is_empty() {
        return Ok(false);
    }
    let json = serde_json::to_string(routes).context("encode the Music Index payment route")?;
    let changed = conn
        .execute(
            "UPDATE tracks SET payment_routes_json = ?2
             WHERE id = ?1
               AND payment_routes_json IS NULL
               AND (SELECT f.payment_routes_json FROM feeds f WHERE f.id = tracks.feed_id) IS NULL",
            params![track_id, json],
        )
        .context("store the Music Index payment route")?;
    Ok(changed > 0)
}

/// The route of one file write (ADR 0076 Decision 9). The function reads the
/// stored route. When the database has none, it stores the `MusicIndex`
/// route first and then reads the stored value again.
///
/// # Errors
///
/// Returns an error when the stored route cannot be read or written.
pub(crate) fn route_for_write(
    conn: &Connection,
    track_id: i64,
    musicindex: Option<&[PaymentRoute]>,
) -> Result<Option<Vec<PaymentRoute>>> {
    if let Some(stored) = stored_route(conn, track_id)? {
        return Ok(Some(stored));
    }
    match musicindex {
        Some(routes) if store_musicindex_route(conn, track_id, routes)? => {
            stored_route(conn, track_id)
        }
        _ => Ok(None),
    }
}

/// Refresh the canonical column of a feed after a write of its raw block.
///
/// A converted feed route also clears the track columns of the feed that
/// have no raw block of their own. Such a column can hold only a `MusicIndex`
/// route, and the RSS feed route replaces it.
///
/// # Errors
///
/// Returns an error when an update fails.
pub(crate) fn refresh_feed_route(conn: &Connection, feed_id: i64, raw: Option<&str>) -> Result<()> {
    let routes = canonical_routes_json(raw);
    conn.execute(
        "UPDATE feeds SET payment_routes_json = ?2 WHERE id = ?1",
        params![feed_id, routes],
    )
    .context("write feeds.payment_routes_json")?;
    if routes.is_some() {
        conn.execute(
            "UPDATE tracks SET payment_routes_json = NULL
             WHERE feed_id = ?1 AND item_value_json IS NULL AND payment_routes_json IS NOT NULL",
            [feed_id],
        )
        .context("clear Music Index track routes")?;
    }
    Ok(())
}

/// Refresh the canonical column of a track after a write of its raw block.
///
/// # Errors
///
/// Returns an error when the update fails.
pub(crate) fn refresh_track_route(
    conn: &Connection,
    track_id: i64,
    raw: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE tracks SET payment_routes_json = ?2 WHERE id = ?1",
        params![track_id, canonical_routes_json(raw)],
    )
    .context("write tracks.payment_routes_json")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{inspect_schema, upgrades, SchemaCompatibility, CURRENT_VERSION, MIGRATIONS};

    const TWO_RECIPIENTS: &str = r#"{"value":null,"attrs":{"type":"lightning"},"children":{"valueRecipient":[{"attrs":{"name":"Band","type":"lnaddress","address":"a","split":"90"}},{"attrs":{"name":"App","type":"lnaddress","address":"b","split":"10"}}]}}"#;
    const NO_RECIPIENT: &str = r#"{"value":null,"attrs":{"type":"lightning"},"children":{}}"#;
    const BROKEN: &str =
        r#"{"children":{"valueRecipient":[{"attrs":{"type":"lnaddress","split":"most"}}]}}"#;

    fn version_16_with_blocks() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, 16).unwrap();
        conn.execute(
            "INSERT INTO feeds(id, feed_url, feed_guid, title, podcast_value_json) VALUES
                 (1, 'https://example.test/one.xml', 'f1', 'One', ?1),
                 (2, 'https://example.test/two.xml', 'f2', 'Two', ?2),
                 (3, 'https://example.test/three.xml', 'f3', 'Three', NULL)",
            params![TWO_RECIPIENTS, BROKEN],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tracks(id, feed_id, item_guid, track_title, item_value_json) VALUES
                 (1, 1, 'a', 'A', ?1),
                 (2, 1, 'b', 'B', NULL),
                 (3, 2, 'c', 'C', ?2),
                 (4, 3, 'd', 'D', ?3)",
            params![NO_RECIPIENT, BROKEN, TWO_RECIPIENTS],
        )
        .unwrap();
        conn
    }

    fn column(conn: &Connection, table: &str, id: i64) -> Option<String> {
        conn.query_row(
            &format!("SELECT payment_routes_json FROM {table} WHERE id = ?1"),
            [id],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn route(name: &str, address: &str, split: f64) -> PaymentRoute {
        PaymentRoute {
            recipient_name: Some(name.into()),
            route_type: Some("node".into()),
            split: Some(split),
            fee: Some(false),
            address: Some(address.into()),
            ..PaymentRoute::default()
        }
    }

    /// R3-02: a version 16 database with raw blocks migrates to version 17
    /// with filled canonical columns. A row that does not convert stays null
    /// and is counted.
    #[test]
    fn adr_0076_route_readiness_version_16_migrates_to_17_with_routes() {
        let conn = version_16_with_blocks();
        assert_eq!(
            inspect_schema(&conn).unwrap(),
            SchemaCompatibility::UpgradeRequired {
                applied: 16,
                current: MIGRATIONS.len()
            }
        );
        let retained = upgrades::retained_digest(&conn).unwrap();

        crate::db::migrate_schema_to(&conn, 17).unwrap();

        const { assert!(CURRENT_VERSION >= 17) };
        // Migration 18 (ADR 0082) follows, so version 17 is not current.
        assert_eq!(
            inspect_schema(&conn).unwrap(),
            SchemaCompatibility::UpgradeRequired {
                applied: 17,
                current: MIGRATIONS.len()
            }
        );
        upgrades::verify_target(&conn, 17).unwrap();
        assert_eq!(upgrades::retained_digest(&conn).unwrap(), retained);
        let name: String = conn
            .query_row(
                "SELECT name FROM schema_migrations WHERE version = 17",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "stored_payment_routes");

        let feed_one: Vec<PaymentRoute> =
            serde_json::from_str(&column(&conn, "feeds", 1).unwrap()).unwrap();
        assert_eq!(feed_one.len(), 2);
        assert_eq!(feed_one[0].recipient_name.as_deref(), Some("Band"));
        assert_eq!(feed_one[1].split, Some(10.0));
        assert_eq!(column(&conn, "feeds", 2), None, "a broken block stays null");
        assert_eq!(column(&conn, "feeds", 3), None, "no block stays null");
        assert_eq!(column(&conn, "tracks", 1).as_deref(), Some("[]"));
        assert_eq!(column(&conn, "tracks", 2), None);
        assert_eq!(
            column(&conn, "tracks", 3),
            None,
            "a broken block stays null"
        );
        assert!(column(&conn, "tracks", 4).is_some());

        // The fill of a migrated database finds nothing more to fill, and it
        // counts the two rows that do not convert.
        assert_eq!(
            fill_payment_routes(&conn).unwrap(),
            RouteFill {
                feeds_filled: 0,
                feeds_unconverted: 1,
                tracks_filled: 0,
                tracks_unconverted: 1,
            }
        );
        // The stored route of track 2 is the route of its feed.
        assert_eq!(stored_route(&conn, 2).unwrap().unwrap().len(), 2);
        assert_eq!(stored_route(&conn, 1).unwrap().unwrap().len(), 0);
        assert!(stored_route(&conn, 3).unwrap().is_none());
    }

    #[test]
    fn adr_0076_route_readiness_fill_counts_rows_before_migration_17_records() {
        let conn = version_16_with_blocks();
        conn.execute_batch(DDL).unwrap();
        assert_eq!(
            fill_payment_routes(&conn).unwrap(),
            RouteFill {
                feeds_filled: 1,
                feeds_unconverted: 1,
                tracks_filled: 2,
                tracks_unconverted: 1,
            }
        );
    }

    #[test]
    fn adr_0076_route_readiness_failed_migration_17_leaves_version_16() {
        let conn = version_16_with_blocks();
        let result = crate::db::migrate_schema_with(&conn, 17, |version, reached| {
            anyhow::ensure!(
                version != 17 || reached != crate::db::MigrationBoundary::AfterRecord,
                "fixture stop after migration 17"
            );
            Ok(())
        });
        assert!(result.is_err());
        upgrades::verify_target(&conn, 16).unwrap();
    }

    #[test]
    fn adr_0076_route_readiness_musicindex_route_never_replaces_a_stored_route() {
        let conn = version_16_with_blocks();
        crate::db::migrate_schema_to(&conn, 17).unwrap();
        let musicindex = [route("Other", "z", 100.0)];

        // Track 2 has the route of its feed, and track 1 has an empty route.
        for track_id in [1, 2] {
            let before = stored_route(&conn, track_id).unwrap();
            let written = route_for_write(&conn, track_id, Some(&musicindex)).unwrap();
            assert_eq!(
                serde_json::to_string(&written).unwrap(),
                serde_json::to_string(&before).unwrap()
            );
        }
        // Track 3 has no stored route, so the MusicIndex route is stored.
        let written = route_for_write(&conn, 3, Some(&musicindex))
            .unwrap()
            .unwrap();
        assert_eq!(written[0].recipient_name.as_deref(), Some("Other"));
        assert!(column(&conn, "tracks", 3).is_some());
        // An empty MusicIndex route is not stored.
        conn.execute(
            "UPDATE tracks SET payment_routes_json = NULL WHERE id = 3",
            [],
        )
        .unwrap();
        assert!(route_for_write(&conn, 3, Some(&[])).unwrap().is_none());
        assert_eq!(column(&conn, "tracks", 3), None);
    }

    #[test]
    fn adr_0076_route_readiness_feed_route_replaces_musicindex_track_route() {
        let conn = version_16_with_blocks();
        crate::db::migrate_schema_to(&conn, 17).unwrap();
        assert!(store_musicindex_route(&conn, 3, &[route("Other", "z", 100.0)]).unwrap());
        // Track 3 has a raw block of its own, so a feed refresh keeps its
        // column. Its block is broken, so the column holds the MusicIndex
        // route.
        refresh_feed_route(&conn, 2, Some(TWO_RECIPIENTS)).unwrap();
        assert!(column(&conn, "tracks", 3).is_some());
        conn.execute("UPDATE tracks SET item_value_json = NULL WHERE id = 3", [])
            .unwrap();
        refresh_feed_route(&conn, 2, Some(TWO_RECIPIENTS)).unwrap();
        assert_eq!(column(&conn, "tracks", 3), None);
        assert_eq!(stored_route(&conn, 3).unwrap().unwrap().len(), 2);
        refresh_track_route(&conn, 3, Some(NO_RECIPIENT)).unwrap();
        assert_eq!(column(&conn, "tracks", 3).as_deref(), Some("[]"));
    }
    const NODE: &str = "03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a";

    fn version_25_with_feed_and_tracks() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, 25).unwrap();
        conn.execute(
            "INSERT INTO feeds(id, feed_url, feed_guid, title) VALUES
                 (1, 'https://example.test/one.xml', 'f1', 'One')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tracks(id, feed_id, item_guid, track_title) VALUES
                 (1, 1, 'a', 'A'), (2, 1, 'b', 'B'), (3, 1, 'c', 'C')",
            [],
        )
        .unwrap();
        conn
    }

    fn types(conn: &Connection, table: &str, id: i64) -> Vec<Option<String>> {
        serde_json::from_str::<Vec<PaymentRoute>>(&column(conn, table, id).unwrap())
            .unwrap()
            .into_iter()
            .map(|route| route.route_type)
            .collect()
    }

    /// ADR 0099 Decision 5, class 1: a stored Lightning block with a
    /// recipient that is not a Lightning recipient gets its route again.
    #[test]
    fn adr_0099_migration_26_calculates_the_route_of_a_raw_block_again() {
        let conn = version_25_with_feed_and_tracks();
        let block = format!(
            r#"{{"attrs":{{"type":"lightning"}},"children":{{"valueRecipient":[{{"attrs":{{"name":"Band","type":"node","address":"{NODE}","split":"90"}}}},{{"attrs":{{"name":"Short","type":"node","address":"abc","split":"10"}}}}]}}}}"#
        );
        let usdc = r#"{"attrs":{"type":"usdc"},"children":{"valueRecipient":[{"attrs":{"name":"Band","type":"address","address":"0xabc","split":"100"}}]}}"#;
        let old = serde_json::to_string(&[route("Band", NODE, 90.0), route("Short", "abc", 10.0)])
            .unwrap();
        conn.execute(
            "UPDATE feeds SET podcast_value_json = ?1, payment_routes_json = ?2 WHERE id = 1",
            params![block, old],
        )
        .unwrap();
        conn.execute(
            "UPDATE tracks SET item_value_json = ?1, payment_routes_json = ?2 WHERE id = 1",
            params![usdc, old],
        )
        .unwrap();

        crate::db::migrate_schema_to(&conn, 26).unwrap();

        upgrades::verify_target(&conn, 26).unwrap();
        let feed: Vec<PaymentRoute> =
            serde_json::from_str(&column(&conn, "feeds", 1).unwrap()).unwrap();
        assert_eq!(feed.len(), 1);
        assert_eq!(feed[0].address.as_deref(), Some(NODE));
        assert_eq!(column(&conn, "tracks", 1).as_deref(), Some("[]"));
    }

    /// ADR 0099 Decision 5, class 2: a route with no raw block loses each
    /// recipient that is not a Lightning recipient.
    #[test]
    fn adr_0099_migration_26_removes_other_recipients_from_a_route_with_no_block() {
        let conn = version_25_with_feed_and_tracks();
        let address = PaymentRoute {
            route_type: Some("address".into()),
            ..route("Chain", "0xabc", 50.0)
        };
        let lnaddress = PaymentRoute {
            route_type: Some("lnaddress".into()),
            ..route("Band", "band@example.test", 50.0)
        };
        let set = |id: i64, routes: &[PaymentRoute]| {
            conn.execute(
                "UPDATE tracks SET payment_routes_json = ?2 WHERE id = ?1",
                params![id, serde_json::to_string(routes).unwrap()],
            )
            .unwrap();
        };
        set(
            1,
            &[
                address.clone(),
                lnaddress.clone(),
                route("Node", NODE, 50.0),
            ],
        );
        set(2, std::slice::from_ref(&address));
        let unchanged = serde_json::to_string(&[lnaddress]).unwrap();
        set(
            3,
            &serde_json::from_str::<Vec<PaymentRoute>>(&unchanged).unwrap(),
        );
        // A raw block that does not convert gives no route, so the stored
        // route of such a row is a MusicIndex route.
        conn.execute(
            "UPDATE tracks SET item_value_json = ?1 WHERE id = 2",
            [BROKEN],
        )
        .unwrap();

        let result = keep_lightning_routes(&conn).unwrap();

        assert_eq!(
            result,
            LightningOnly {
                from_block: 0,
                without_block: 2
            }
        );
        assert_eq!(
            types(&conn, "tracks", 1),
            [Some("lnaddress".to_owned()), Some("node".to_owned())]
        );
        assert_eq!(column(&conn, "tracks", 2), None);
        assert_eq!(column(&conn, "tracks", 3), Some(unchanged));
    }
}
