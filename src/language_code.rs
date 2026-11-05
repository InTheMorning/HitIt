//! The language code of a tag file (ADR 0080 Decision 13).
//!
//! An RSS `<language>` value is a BCP 47 style tag, for example `en-US`. The
//! `TLAN` frame holds an ISO 639-2 code of three letters. This module maps
//! the primary subtag to that code.
//!
//! The table gives the ISO 639-2/T code of each ISO 639-1 code. The source
//! is the `iso_639-2.json` file of the Debian `iso-codes` package, the
//! `alpha_3` field of each entry that has an `alpha_2` field. The table has
//! 183 entries.

/// Each ISO 639-1 code with its ISO 639-2/T code, sorted by the first code.
pub(crate) const ISO_639_1_TO_639_2_T: [(&str, &str); 183] = [
    ("aa", "aar"),
    ("ab", "abk"),
    ("ae", "ave"),
    ("af", "afr"),
    ("ak", "aka"),
    ("am", "amh"),
    ("an", "arg"),
    ("ar", "ara"),
    ("as", "asm"),
    ("av", "ava"),
    ("ay", "aym"),
    ("az", "aze"),
    ("ba", "bak"),
    ("be", "bel"),
    ("bg", "bul"),
    ("bi", "bis"),
    ("bm", "bam"),
    ("bn", "ben"),
    ("bo", "bod"),
    ("br", "bre"),
    ("bs", "bos"),
    ("ca", "cat"),
    ("ce", "che"),
    ("ch", "cha"),
    ("co", "cos"),
    ("cr", "cre"),
    ("cs", "ces"),
    ("cu", "chu"),
    ("cv", "chv"),
    ("cy", "cym"),
    ("da", "dan"),
    ("de", "deu"),
    ("dv", "div"),
    ("dz", "dzo"),
    ("ee", "ewe"),
    ("el", "ell"),
    ("en", "eng"),
    ("eo", "epo"),
    ("es", "spa"),
    ("et", "est"),
    ("eu", "eus"),
    ("fa", "fas"),
    ("ff", "ful"),
    ("fi", "fin"),
    ("fj", "fij"),
    ("fo", "fao"),
    ("fr", "fra"),
    ("fy", "fry"),
    ("ga", "gle"),
    ("gd", "gla"),
    ("gl", "glg"),
    ("gn", "grn"),
    ("gu", "guj"),
    ("gv", "glv"),
    ("ha", "hau"),
    ("he", "heb"),
    ("hi", "hin"),
    ("ho", "hmo"),
    ("hr", "hrv"),
    ("ht", "hat"),
    ("hu", "hun"),
    ("hy", "hye"),
    ("hz", "her"),
    ("ia", "ina"),
    ("id", "ind"),
    ("ie", "ile"),
    ("ig", "ibo"),
    ("ii", "iii"),
    ("ik", "ipk"),
    ("io", "ido"),
    ("is", "isl"),
    ("it", "ita"),
    ("iu", "iku"),
    ("ja", "jpn"),
    ("jv", "jav"),
    ("ka", "kat"),
    ("kg", "kon"),
    ("ki", "kik"),
    ("kj", "kua"),
    ("kk", "kaz"),
    ("kl", "kal"),
    ("km", "khm"),
    ("kn", "kan"),
    ("ko", "kor"),
    ("kr", "kau"),
    ("ks", "kas"),
    ("ku", "kur"),
    ("kv", "kom"),
    ("kw", "cor"),
    ("ky", "kir"),
    ("la", "lat"),
    ("lb", "ltz"),
    ("lg", "lug"),
    ("li", "lim"),
    ("ln", "lin"),
    ("lo", "lao"),
    ("lt", "lit"),
    ("lu", "lub"),
    ("lv", "lav"),
    ("mg", "mlg"),
    ("mh", "mah"),
    ("mi", "mri"),
    ("mk", "mkd"),
    ("ml", "mal"),
    ("mn", "mon"),
    ("mr", "mar"),
    ("ms", "msa"),
    ("mt", "mlt"),
    ("my", "mya"),
    ("na", "nau"),
    ("nb", "nob"),
    ("nd", "nde"),
    ("ne", "nep"),
    ("ng", "ndo"),
    ("nl", "nld"),
    ("nn", "nno"),
    ("no", "nor"),
    ("nr", "nbl"),
    ("nv", "nav"),
    ("ny", "nya"),
    ("oc", "oci"),
    ("oj", "oji"),
    ("om", "orm"),
    ("or", "ori"),
    ("os", "oss"),
    ("pa", "pan"),
    ("pi", "pli"),
    ("pl", "pol"),
    ("ps", "pus"),
    ("pt", "por"),
    ("qu", "que"),
    ("rm", "roh"),
    ("rn", "run"),
    ("ro", "ron"),
    ("ru", "rus"),
    ("rw", "kin"),
    ("sa", "san"),
    ("sc", "srd"),
    ("sd", "snd"),
    ("se", "sme"),
    ("sg", "sag"),
    ("si", "sin"),
    ("sk", "slk"),
    ("sl", "slv"),
    ("sm", "smo"),
    ("sn", "sna"),
    ("so", "som"),
    ("sq", "sqi"),
    ("sr", "srp"),
    ("ss", "ssw"),
    ("st", "sot"),
    ("su", "sun"),
    ("sv", "swe"),
    ("sw", "swa"),
    ("ta", "tam"),
    ("te", "tel"),
    ("tg", "tgk"),
    ("th", "tha"),
    ("ti", "tir"),
    ("tk", "tuk"),
    ("tl", "tgl"),
    ("tn", "tsn"),
    ("to", "ton"),
    ("tr", "tur"),
    ("ts", "tso"),
    ("tt", "tat"),
    ("tw", "twi"),
    ("ty", "tah"),
    ("ug", "uig"),
    ("uk", "ukr"),
    ("ur", "urd"),
    ("uz", "uzb"),
    ("ve", "ven"),
    ("vi", "vie"),
    ("vo", "vol"),
    ("wa", "wln"),
    ("wo", "wol"),
    ("xh", "xho"),
    ("yi", "yid"),
    ("yo", "yor"),
    ("za", "zha"),
    ("zh", "zho"),
    ("zu", "zul"),
];

/// The three-letter code for an RSS language value (ADR 0080 Decision 13).
///
/// The function trims and lowers the value, then takes the subtag before `-`
/// or `_`. A two-letter subtag gives its ISO 639-2/T code. A subtag of three
/// letters stays as it is. Each other value gives `None`.
pub(crate) fn language_code_for_tag(value: &str) -> Option<String> {
    let lowered = value.trim().to_lowercase();
    let subtag = lowered.split(['-', '_']).next().unwrap_or_default();
    if !subtag.chars().all(|ch| ch.is_ascii_lowercase()) {
        return None;
    }
    match subtag.len() {
        2 => ISO_639_1_TO_639_2_T
            .binary_search_by_key(&subtag, |(short, _)| short)
            .ok()
            .map(|index| ISO_639_1_TO_639_2_T[index].1.to_owned()),
        3 => Some(subtag.to_owned()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_0080_language_code_maps_a_two_letter_subtag() {
        for value in ["en", "EN", " en ", "en-us", "en_GB", "en-US-x-test"] {
            assert_eq!(
                language_code_for_tag(value).as_deref(),
                Some("eng"),
                "{value}"
            );
        }
        assert_eq!(language_code_for_tag("de").as_deref(), Some("deu"));
        assert_eq!(language_code_for_tag("zh-Hans").as_deref(), Some("zho"));
    }

    #[test]
    fn adr_0080_language_code_keeps_a_three_letter_subtag() {
        assert_eq!(language_code_for_tag("fra").as_deref(), Some("fra"));
        assert_eq!(language_code_for_tag("FRA-CA").as_deref(), Some("fra"));
    }

    #[test]
    fn adr_0080_language_code_gives_none_for_other_values() {
        for value in ["", "   ", "english", "xx", "e", "1a", "-us", "e1"] {
            // `xx` has two letters and no row in the table.
            assert_eq!(language_code_for_tag(value), None, "{value}");
        }
    }

    /// The table has each ISO 639-1 code once, in sorted order, with one
    /// three-letter value. The edition of `iso-codes` that the table comes
    /// from has 183 codes.
    #[test]
    fn adr_0080_language_table_has_each_iso_639_1_code() {
        assert_eq!(ISO_639_1_TO_639_2_T.len(), 183);
        for window in ISO_639_1_TO_639_2_T.windows(2) {
            assert!(window[0].0 < window[1].0, "{window:?}");
        }
        for (short, long) in ISO_639_1_TO_639_2_T {
            assert_eq!(short.len(), 2, "{short}");
            assert!(short.chars().all(|ch| ch.is_ascii_lowercase()), "{short}");
            assert_eq!(long.len(), 3, "{long}");
            assert!(long.chars().all(|ch| ch.is_ascii_lowercase()), "{long}");
        }
        let lookup = |short: &str| {
            ISO_639_1_TO_639_2_T
                .iter()
                .find(|(code, _)| *code == short)
                .map(|(_, long)| *long)
        };
        assert_eq!(lookup("cs"), Some("ces"));
        assert_eq!(lookup("nl"), Some("nld"));
        assert_eq!(lookup("fr"), Some("fra"));
        assert_eq!(lookup("zh"), Some("zho"));
    }
}
