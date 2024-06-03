use clap::Parser;
use lingua::{IsoCode639_1, Language, LanguageDetectorBuilder};
use std::io::{self, BufRead, Read};
use std::str::FromStr;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// List of iso-639-1 language codes
    #[arg(
        short,
        long,
        help = "comma seperated list of iso-639-1 codes of languages to detect, if not specified, all supported language will be used. Setting this improves accuracy and resource usage.",
        num_args = 1,
        value_delimiter = ',',
        required = false
    )]
    languages: Vec<String>,

    #[arg(
        short = 'n',
        long,
        help = "Classify language per line, this only works if text is not supplied directly as an argument"
    )]
    per_line: bool,

    #[arg(
        short = 'p',
        long,
        help = "Use parallel computation, can only be used with per_line mode"
    )]
    parallel: bool,

    #[arg(short = 'L', long, help = "List supported languages")]
    list: bool,

    #[arg(short = 'q', long, help = "Quick/low accuracy mode")]
    quick: bool,

    #[arg(short = 'm', long)]
    minimum_relative_distance: Option<f64>,

    #[arg(short = 'p', help = "preload models")]
    preload: bool,

    #[arg(short, long, default_value = "\t")]
    delimiter: String,

    #[arg(required = false)]
    text: Vec<String>,
}

fn main() {
    let args = Args::parse();
    if args.list {
        let mut languages: Vec<Language> = Language::all().into_iter().collect();
        languages.sort();
        for language in languages {
            println!("{} - {}", language.iso_code_639_1(), language);
        }
        std::process::exit(0);
    }

    // Select languages by ISO 639-1 code.
    let languages: Vec<_> = args
        .languages
        .iter()
        .map(|lang| {
            IsoCode639_1::from_str(lang.as_str())
                .expect("Supported iso639-1 language code expected")
        })
        .collect();
    let mut builder: LanguageDetectorBuilder = if languages.is_empty() {
        LanguageDetectorBuilder::from_all_languages()
    } else {
        LanguageDetectorBuilder::from_iso_codes_639_1(&languages)
    };
    if args.quick {
        builder.with_low_accuracy_mode();
    }
    if args.preload {
        builder.with_preloaded_language_models();
    }
    if let Some(minimum_relative_distance) = args.minimum_relative_distance {
        builder.with_minimum_relative_distance(minimum_relative_distance);
    }
    let detector = builder.build();

    if !args.text.is_empty() {
        //text provided as arguments
        let text: String = args.text.join(" ");
        let results = detector.compute_language_confidence_values(text);
        if !results.is_empty() {
            print!(
                "{}{}{}\n",
                results[0].0.iso_code_639_1(),
                args.delimiter,
                results[0].1
            );
        }
    } else if args.per_line && args.parallel {
        let stdin = io::stdin();
        let lines: Vec<_> = stdin.lock().lines().filter_map(|x| x.ok()).collect();
        let results = detector.compute_language_confidence_values_in_parallel(&lines);
        for (line, result) in lines.iter().zip(results) {
            print!(
                "{}{}{}{}{}\n",
                result[0].0.iso_code_639_1(),
                args.delimiter,
                result[0].1,
                args.delimiter,
                line
            );
        }
    } else if args.per_line {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                let results = detector.compute_language_confidence_values(&line);
                if !results.is_empty() {
                    print!(
                        "{}{}{}{}{}\n",
                        results[0].0.iso_code_639_1(),
                        args.delimiter,
                        results[0].1,
                        args.delimiter,
                        line
                    );
                }
            }
        }
    } else {
        let mut buf: Vec<u8> = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .expect("expected input via stdin");
        let text = String::from_utf8(buf).expect("Input should be valid utf-8");
        let results = detector.compute_language_confidence_values(text);
        if !results.is_empty() {
            print!(
                "{}{}{}\n",
                results[0].0.iso_code_639_1(),
                args.delimiter,
                results[0].1
            );
        }
    }
}
