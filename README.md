# Lingua-cli

This is a small command-line tool for language detection, it is a simple
wrapper around the [lingua-rs](https://github.com/pemistahl/lingua-rs/) library
for Rust, read there for extensive documentation. A distinguishing feature is
that this library works better for short texts thanmany other libraries

## Installation

Ensure you have Rust's package manager `cargo`, then download, isntall and compile `lingua-cli` in one go as follows:

``$ cargo install lingua-cli``

## Usage

Pass text as parameter

``$ lingua-cli bonjour à tous``

Pass text via standard input:

``$ echo "bonjour à tous" | lingua-cli``

Constrain the languages you want to detect using `-l` with iso-639-1 languages
codes. Constraining the list improves accuracy. Do `-L` to see a list of
supported languages.

``$ echo "bonjour à tous" | lingua-cli -l "fr,de,es,nl,en"``

To classify input line-by-line, pass ``-n``.

``$ echo -e "bonjour à tous\nhola a todos\nhallo allemaal" | lingua-cli -n -l "fr,de,es,nl,en"``

```
fr      0.9069164472389637      bonjour à tous
es      0.918273871035807       hola a todos
nl      0.988293648761749       hallo allemaal
```

Output is TSV and consists of an iso-639-1 language code, confidence score, and in line-by-line mode, a copy of the line.
