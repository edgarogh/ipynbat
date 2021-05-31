use serde::de::IgnoredAny;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, serde::Deserialize)]
pub struct Notebook {
    pub cells: Vec<NotebookCell>,
    pub metadata: NotebookMetadata,

    #[serde(flatten)]
    pub version: NotebookVersion,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "cell_type")]
pub enum NotebookCell {
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
pub enum NotebookCellOutput {
    #[serde(rename = "stream")]
    Stream { name: String, text: Vec<String> },

    #[serde(rename = "display_data")]
    DisplayData {
        data: HashMap<String, serde_json::Value>,
    },
}

#[derive(Debug, serde::Deserialize)]
pub struct NotebookMetadata {
    #[serde(rename = "kernelspec")]
    pub kernel_spec: NotebookKernelSpec,

    pub language_info: IgnoredAny,
}

#[derive(Debug, serde::Deserialize)]
pub struct NotebookKernelSpec {
    pub display_name: String,
    pub language: String,
    pub name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct NotebookVersion {
    #[serde(rename = "nbformat")]
    pub major: u32,

    #[serde(rename = "nbformat_minor")]
    pub minor: u32,
}

impl NotebookVersion {
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

impl Display for NotebookVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
