mod args;
mod kernel;
mod notebook;

use crate::args::Args;
use crate::kernel::KernelSpec;
use crate::notebook::{Notebook, NotebookCell, NotebookCellOutput, NotebookVersion};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use termimad::{FmtLine, MadSkin};

const FIRST_COLUMN_WIDTH: usize = 7;

struct Line<'a, 's>(usize, &'a MadSkin, FmtLine<'s>);

impl Display for Line<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.1.write_fmt_line(f, &self.2, Some(self.0), false)
    }
}

fn main() {
    let args = Args::parse();

    match args {
        Args {
            file: None,
            kernel: None,
            run: false,
            list_kernels: true,
        } => {
            for dir in KernelSpec::directories() {
                println!("{}:", dir.display());
                if let Ok(read_dir) = std::fs::read_dir(dir) {
                    for kernel in read_dir.filter_map(|d| d.ok()) {
                        let kfn = kernel.file_name();
                        let name = kfn.to_string_lossy();
                        let kernel = KernelSpec::find(&name);
                        println!("   \x1b[32m{}\x1b[0m ({})", name, kernel.display_name)
                    }
                }
            }
        }
        Args {
            list_kernels: true, ..
        } => {
            panic!("--list-kernels didn't expect that much arguments")
        }
        Args {
            file: Some(file_name),
            kernel,
            run,
            list_kernels: false,
        } => {
            let file = BufReader::new(File::open(&file_name).unwrap());
            let notebook: Notebook = serde_json::from_reader(file).unwrap();
            print_file(notebook, &file_name)
        }
        Args { file: None, .. } => {
            panic!("please specify a file or a subcommand")
        }
    }
}

fn print_file(notebook: Notebook, file_name: &Path) {
    assert_eq!(notebook.version, NotebookVersion::new(4, 5));

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
