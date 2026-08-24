use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tokio::io::AsyncReadExt;

use crate::error::{AppError, Result};

#[derive(Debug, Default)]
pub struct InputReader {
    stdin_used: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum JsonKind {
    Array,
    Object,
}

impl InputReader {
    pub async fn optional_text(
        &mut self,
        inline: &Option<String>,
        file: &Option<PathBuf>,
        label: &'static str,
    ) -> Result<Option<String>> {
        if inline.is_some() && file.is_some() {
            return Err(AppError::Input(format!(
                "{label} 不能同时使用文本参数和文件参数"
            )));
        }

        match (inline, file) {
            (Some(value), None) => Ok(Some(value.clone())),
            (None, Some(path)) => self.read_text_file(path, label).await.map(Some),
            (None, None) => Ok(None),
            (Some(_), Some(_)) => unreachable!("checked above"),
        }
    }

    pub async fn required_text(
        &mut self,
        inline: &Option<String>,
        file: &Option<PathBuf>,
        label: &'static str,
    ) -> Result<String> {
        self.optional_text(inline, file, label)
            .await?
            .ok_or_else(|| AppError::Input(format!("必须提供 {label}")))
    }

    pub async fn optional_json(
        &mut self,
        inline: &Option<String>,
        file: &Option<PathBuf>,
        label: &'static str,
        expected_kind: JsonKind,
    ) -> Result<Option<Value>> {
        let Some(raw) = self.optional_text(inline, file, label).await? else {
            return Ok(None);
        };

        let value: Value = serde_json::from_str(&raw)
            .map_err(|_| AppError::Input(format!("{label} 不是有效 JSON")))?;
        let kind_matches = match expected_kind {
            JsonKind::Array => value.is_array(),
            JsonKind::Object => value.is_object(),
        };
        if !kind_matches {
            let expected_name = match expected_kind {
                JsonKind::Array => "JSON 数组",
                JsonKind::Object => "JSON 对象",
            };
            return Err(AppError::Input(format!("{label} 必须是 {expected_name}")));
        }

        Ok(Some(value))
    }

    async fn read_text_file(&mut self, path: &Path, label: &'static str) -> Result<String> {
        if path.as_os_str() == OsStr::new("-") {
            if self.stdin_used {
                return Err(AppError::StdinAlreadyUsed);
            }
            self.stdin_used = true;

            let mut bytes = Vec::new();
            tokio::io::stdin()
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| AppError::StdinRead)?;
            return String::from_utf8(bytes).map_err(|_| AppError::InputNotUtf8 {
                label,
                path: PathBuf::from("-"),
            });
        }

        let bytes = tokio::fs::read(path)
            .await
            .map_err(|_| AppError::InputRead {
                label,
                path: path.to_path_buf(),
            })?;
        String::from_utf8(bytes).map_err(|_| AppError::InputNotUtf8 {
            label,
            path: path.to_path_buf(),
        })
    }
}

pub fn validate_local_file(path: &Path) -> Result<()> {
    if path.as_os_str() == OsStr::new("-") {
        return Err(AppError::Input(
            "媒体文件不支持使用 - 从标准输入读取".to_owned(),
        ));
    }

    let metadata = std::fs::metadata(path).map_err(|_| AppError::InvalidLocalFile {
        path: path.to_path_buf(),
    })?;
    if !metadata.is_file() {
        return Err(AppError::InvalidLocalFile {
            path: path.to_path_buf(),
        });
    }

    Ok(())
}
