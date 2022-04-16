use lingua::{LanguageDetector, LanguageDetectorBuilder, Language, IsoCode639_1};
use std::env;

fn main() {
    // Select languages by ISO 639-1 code.
    let detector: LanguageDetector = LanguageDetectorBuilder::from_iso_codes_639_1(&[
        IsoCode639_1::FR,
        IsoCode639_1::ES,
        IsoCode639_1::FR,
        IsoCode639_1::PT,
        IsoCode639_1::RU,
        IsoCode639_1::IT,
        IsoCode639_1::ZH,
        IsoCode639_1::DE
    ]).build();
    let text: String = env::args().map(|x| x.to_string()).collect::<Vec<String>>().join(" ");
    let results = detector.compute_language_confidence_values(&text);
    if !results.is_empty() {
        print!("{} {}\n", results[0].0.iso_code_639_1(), results[0].1);
    }
}
