use crate::{assets::CHUNKS, translate_message};
#[test]
fn every_additional_catalog_key_resolves_to_its_explicit_locale() {
    for source in CHUNKS.iter().skip(14) {
        let catalog: serde_json::Value = serde_json::from_str(source).unwrap();
        for (key, translations) in catalog.as_object().unwrap() {
            if key == "_version" {
                continue;
            }
            for locale in ["en", "zh"] {
                assert_eq!(
                    translate_message(locale, key),
                    translations[locale].as_str().unwrap().replace("%{", "{"),
                    "unresolved {locale} catalog key {key}"
                );
            }
        }
    }
}
