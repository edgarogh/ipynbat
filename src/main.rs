mod kernel;

use serde::de::IgnoredAny;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use termimad::{FmtLine, MadSkin};

const FIRST_COLUMN_WIDTH: usize = 7;

#[derive(Debug, serde::Deserialize)]
struct Notebook {
    cells: Vec<NotebookCell>,
    metadata: NotebookMetadata,

    #[serde(flatten)]
    version: NotebookVersion,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "cell_type")]
enum NotebookCell {
    #[serde(rename = "code")]
    Code {
        id: String,
        source: Vec<String>,
        metadata: IgnoredAny,
        execution_count: Option<u32>,
        outputs: Vec<NotebookCellOutput>,
    },
    #[serde(rename = "markdown")]
    Markdown {
        id: String,
        source: Vec<String>,
        metadata: IgnoredAny,
    },
    #[serde(rename = "raw")]
    Raw {
        id: String,
        source: Vec<String>,
        metadata: IgnoredAny,
        // TODO
    },
}

impl NotebookCell {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Code { .. } => "Code",
            Self::Markdown { .. } => "MD",
            Self::Raw { .. } => "Raw",
        }
    }

    pub fn source(&self, language: &str) -> String {
        let mut ret = String::new();

        match self {
            Self::Markdown { source, .. } => ret.extend(source.iter().map(|s| s.as_str())),
            Self::Code { source, .. } => {
                ret.push_str("```");
                ret.push_str(language);
                ret.push('\n');
                ret.extend(source.iter().map(|s| s.as_str()));
                ret.push_str("\n```\n");
            }
            Self::Raw { source, .. } => ret.extend(source.iter().map(|s| s.as_str())), // TODO DRY
        }

        ret
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "output_type")]
enum NotebookCellOutput {
    #[serde(rename = "stream")]
    Stream { name: String, text: Vec<String> },

    #[serde(rename = "display_data")]
    DisplayData {
        data: HashMap<String, serde_json::Value>,
    },
}

#[derive(Debug, serde::Deserialize)]
struct NotebookMetadata {
    #[serde(rename = "kernelspec")]
    kernel_spec: NotebookKernelSpec,

    language_info: IgnoredAny,
}

#[derive(Debug, serde::Deserialize)]
struct NotebookKernelSpec {
    display_name: String,
    language: String,
    name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
struct NotebookVersion {
    #[serde(rename = "nbformat")]
    major: u32,
    #[serde(rename = "nbformat_minor")]
    minor: u32,
}

impl Display for NotebookVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

struct Line<'a, 's>(usize, &'a MadSkin, FmtLine<'s>);

impl Display for Line<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.1.write_fmt_line(f, &self.2, Some(self.0), false)
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    assert_eq!(args.len(), 2, "One argument expected");

    let file_name = Path::new(args.get(1).expect("expected one argument"));
    let file = BufReader::new(File::open(file_name).unwrap());

    let notebook: Notebook = serde_json::from_reader(file).unwrap();

    assert_eq!(notebook.version, NotebookVersion { major: 4, minor: 5 });

    // let kernel_spec = KernelSpec::find(&notebook.metadata.kernel_spec.name);

    let (width, height) = termimad::terminal_size();
    let (width, height) = (width as usize, height as usize);
    let width_minus_col = width - FIRST_COLUMN_WIDTH - 1;

    let make_separator = |vertical: char| {
        let mut ret = String::with_capacity(width);
        ret.push_str("\x1b[38;5;238m");

        ret.extend(
            std::iter::repeat('─')
                .take(FIRST_COLUMN_WIDTH)
                .chain(std::iter::once(vertical))
                .chain(std::iter::repeat('─').take(width_minus_col)),
        );

        ret.push_str("\x1b[0m");

        ret
    };

    let separator_top = make_separator('┬');
    let separator = make_separator('┼');
    let separator_bottom = make_separator('┴');

    // FIXME Pager::new().setup();

    println!("{}", &separator_top);
    println!(
        "{: ^0width$}\x1b[38;5;238m│\x1b[0m File: \x1b[1m{:?}\x1b[0m  Kernel: \x1b[1m{}\x1b[0m",
        "",
        file_name.file_name().unwrap_or("?".as_ref()),
        &notebook.metadata.kernel_spec.display_name,
        width = FIRST_COLUMN_WIDTH,
    );

    for cell in notebook.cells {
        println!("{}", separator);

        let mut cell_name = Some(cell.name());
        let source = cell.source("python");
        let md = termimad::term_text(&source);

        match cell {
            NotebookCell::Markdown { .. } => {
                for line in md.lines {
                    println!(
                        "{: ^0width$}\x1b[38;5;238m│\x1b[0m {}",
                        cell_name.take().unwrap_or_default(),
                        Line(width, &md.skin, line),
                        width = FIRST_COLUMN_WIDTH,
                    );
                }
            }
            NotebookCell::Code {
                execution_count,
                outputs,
                ..
            } => {
                for line in md.lines {
                    println!(
                        "{: ^0width$}\x1b[38;5;238m│\x1b[0m {}",
                        cell_name.take().unwrap_or_default(),
                        Line(width, &md.skin, line),
                        width = FIRST_COLUMN_WIDTH,
                    );
                }

                let mut execution_count = Some(
                    execution_count
                        .map(|s| format!("[{}]", s))
                        .unwrap_or_else(|| String::from("[*]")),
                );

                for output in outputs {
                    println!(
                        "{: ^0width$}\x1b[38;5;238m├{}",
                        "",
                        std::iter::repeat('╌')
                            .take(width_minus_col)
                            .collect::<String>(),
                        width = FIRST_COLUMN_WIDTH,
                    );

                    match output {
                        NotebookCellOutput::Stream { text, .. } => {
                            for line in text.join("").lines() {
                                println!(
                                    "{: ^0width$}\x1b[38;5;238m│\x1b[0m {}",
                                    execution_count.take().as_deref().unwrap_or_default(),
                                    line,
                                    width = FIRST_COLUMN_WIDTH,
                                );
                            }
                        }
                        NotebookCellOutput::DisplayData { data } => {
                            for (mime, value) in data {
                                if mime.starts_with("image/") {
                                    let image = base64::decode(
                                        value.as_str().unwrap_or_default().trim_end(),
                                    )
                                    .unwrap();

                                    let image = image::load_from_memory(&image).unwrap();
                                    let image = viuer::resize(
                                        &image,
                                        Some((width_minus_col - 2) as _),
                                        Some((height / 2) as _),
                                    );

                                    let conf = viuer::Config {
                                        transparent: true,
                                        absolute_offset: false,
                                        x: (FIRST_COLUMN_WIDTH + 2) as _,
                                        y: 0,
                                        width: Some((width_minus_col - 2) as _),
                                        height: None,
                                        ..Default::default()
                                    };

                                    viuer::print(&image, &conf).unwrap();
                                } else if mime == "text/plain" {
                                    let default_vec = Vec::new();
                                    let text = value.as_array().unwrap_or(&default_vec);
                                    let text = text
                                        .iter()
                                        .filter_map(|value| value.as_str())
                                        .collect::<String>()
                                        .replace("\n", "\\n");

                                    println!(
                                        "{: ^0width$}\x1b[38;5;238m│ \x1b[3m{}\x1b[0m",
                                        execution_count.as_deref().take().unwrap_or_default(),
                                        text,
                                        width = FIRST_COLUMN_WIDTH,
                                    );
                                } else {
                                    println!(
                                        "{: ^0width$}\x1b[38;5;238m│ \x1b[93mUnsupported data output\x1b[0m",
                                        execution_count.as_deref().take().unwrap_or_default(),
                                        width = FIRST_COLUMN_WIDTH,
                                    );
                                }
                            }
                            // TODO
                        }
                    }
                }
            }
            NotebookCell::Raw { .. } => (),
        }
    }

    println!("{}", separator_bottom);
}
