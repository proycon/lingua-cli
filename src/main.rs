use lingua::Language::{
    Arabic, Chinese, Czech, French, German, Polish, Portuguese, Romanian, Russian, Spanish,
    Swedish, Ukrainian,
};
use lingua::{LanguageDetector, LanguageDetectorBuilder};
use std::env;

fn main() {
    // Select languages by ISO 639-1 code.
    let languages = vec![
        French, German, Spanish, Portuguese, Russian, Swedish, Chinese, Polish, Czech, Romanian,
        Arabic, Ukrainian,
    ];
    let detector: LanguageDetector = LanguageDetectorBuilder::from_languages(&languages).build();
    let text: String = env::args()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    let results = detector.compute_language_confidence_values(&text);
    if !results.is_empty() {
        print!("{} {}\n", results[0].0.iso_code_639_1(), results[0].1);
    }
}
