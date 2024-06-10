use clap::Parser;
use lingua::{DetectionResult, IsoCode639_1, Language, LanguageDetectorBuilder};
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
        help = "Use parallel computation, can only be used with per_line"
    )]
    parallel: bool,

    #[arg(short = 'L', long, help = "List supported languages")]
    list: bool,

    #[arg(
        short,
        long,
        help = "Show all confidence values (entire probability distribution), rather than just the winning score. Does not work with --multi"
    )]
    all: bool,

    #[arg(short = 'q', long, help = "Quick/low accuracy mode")]
    quick: bool,

    #[arg(
        short = 'm',
        long,
        help = "Classify multiple languages in mixed texts, will return matches along with UTF-8 byte offsets. Can not be combined with line mode."
    )]
    multi: bool,

    #[arg(
        short = 'c',
        long,
        help = "Confidence threshold, only output results with at least this confidence value (0.0-1.0)"
    )]
    confidence: Option<f64>,

    #[arg(
        short = 'M',
        long,
        help = "Minimum text length (without regard for whitespace, punctuation or numerals!). Shorter fragments will not be classified"
    )]
    minlength: Option<u8>,

    #[arg(short = 'd', long)]
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
        if args.minlength.is_none() || long_enough(&text, args.minlength.unwrap()) {
            if args.multi {
                let results = detector.detect_multiple_languages_of(&text);
                print_with_offset(&results, &text, &args.delimiter)
            } else {
                let results = detector.compute_language_confidence_values(text);
                print_confidence_values(&results, &args.delimiter, args.confidence, args.all);
            }
        } else {
            print!("unknown{}\n", &args.delimiter);
        }
    } else if args.per_line && args.parallel {
        let stdin = io::stdin();
        let lines: Vec<_> = stdin
            .lock()
            .lines()
            .filter_map(|x| {
                if let Ok(line) = x {
                    if args.minlength.is_some() && !long_enough(&line, args.minlength.unwrap()) {
                        None
                    } else {
                        Some(line)
                    }
                } else {
                    None
                }
            })
            .collect();
        let results = detector.compute_language_confidence_values_in_parallel(&lines);
        if args.minlength.is_some() {
            eprintln!("Note: Lines that do not match the minimum length will not be returned (disable parallel mode if you want to return them as 'unknown')")
        }
        for (line, results) in lines.iter().zip(results) {
            print_line_with_confidence_values(
                line,
                &results,
                &args.delimiter,
                args.confidence,
                args.all,
            );
        }
    } else if args.per_line {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                if args.minlength.is_none() || long_enough(&line, args.minlength.unwrap()) {
                    let results = detector.compute_language_confidence_values(&line);
                    print_line_with_confidence_values(
                        &line,
                        &results,
                        &args.delimiter,
                        args.confidence,
                        args.all,
                    );
                } else {
                    print!("unknown{}{}{}\n", &args.delimiter, &args.delimiter, line);
                }
            }
        }
    } else {
        let mut buf: Vec<u8> = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .expect("expected input via stdin");
        let text = String::from_utf8(buf).expect("Input should be valid utf-8");
        if args.minlength.is_none() || long_enough(&text, args.minlength.unwrap()) {
            if args.multi {
                let results = detector.detect_multiple_languages_of(&text);
                print_with_offset(&results, &text, &args.delimiter)
            } else {
                let results = detector.compute_language_confidence_values(text);
                print_confidence_values(&results, &args.delimiter, args.confidence, args.all);
            }
        }
    }
}

#[inline]
fn long_enough(line: &str, minlength: u8) -> bool {
    line.chars().filter(|c| c.is_alphabetic()).count() >= minlength as usize
}

fn print_confidence_values(
    results: &Vec<(Language, f64)>,
    delimiter: &str,
    confidence_threshold: Option<f64>,
    all: bool,
) {
    let mut found = false;
    for result in results {
        if confidence_threshold.is_some() && result.1 >= confidence_threshold.unwrap() {
            found = true;
            print!("{}{}{}\n", result.0.iso_code_639_1(), delimiter, result.1);
        }
        if !all {
            break;
        }
    }
    if !found {
        print!("unknown{}\n", delimiter);
    }
}

fn print_line_with_confidence_values(
    line: &str,
    results: &Vec<(Language, f64)>,
    delimiter: &str,
    confidence_threshold: Option<f64>,
    all: bool,
) {
    for result in results {
        if confidence_threshold.is_some() && result.1 >= confidence_threshold.unwrap() {
            print!(
                "{}{}{}{}{}\n",
                result.0.iso_code_639_1(),
                delimiter,
                result.1,
                delimiter,
                line
            );
        } else {
            print!("unknown{}{}{}\n", delimiter, delimiter, line);
        }
        if !all {
            break;
        }
    }
}

fn print_with_offset(results: &Vec<DetectionResult>, text: &str, delimiter: &str) {
    for result in results {
        print!(
            "{}{}{}{}{}{}{}\n",
            result.start_index(),
            delimiter,
            result.end_index(),
            delimiter,
            result.language().iso_code_639_1(),
            delimiter,
            &text[result.start_index()..result.end_index()],
        );
    }
}
