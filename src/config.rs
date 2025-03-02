use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::io::{self, Error, ErrorKind};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CpamConfig {
    pub project: Option<ProjectConfig>,
    pub build: Option<BuildConfig>,
    pub dependencies: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ProjectConfig {
    pub name: String,
    pub language: String,
    pub build_tool: String,
    pub project_type: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BuildConfig {
    pub generator: Option<String>,
    pub source_dir: Option<String>,
    pub build_dir: Option<String>,
    pub options: Option<Vec<String>>,
}

impl CpamConfig {
    pub fn load() -> io::Result<Self> {
        let toml_path = Path::new("cpam.toml");
        if !toml_path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "cpam.toml がカレントディレクトリに存在しません。"));
        }

        let toml_str = fs::read_to_string(toml_path)?;
        match toml::from_str(&toml_str) {
            Ok(config) => Ok(config),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("cpam.toml の解析に失敗: {}", e))),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let toml_str = toml::to_string(self)
            .map_err(|e| Error::new(ErrorKind::InvalidData, format!("設定のシリアライズに失敗: {}", e)))?;

        fs::write("cpam.toml", toml_str)
    }

    pub fn get_cmake_generator(&self) -> String {
        if let Some(build) = &self.build {
            if let Some(generator) = &build.generator {
                return generator.clone();
            }
        }

        if let Some(project) = &self.project {
            match project.build_tool.as_str() {
                "ninja" => "Ninja".to_string(),
                "make" => "Unix Makefiles".to_string(),
                "vs" => if cfg!(target_os = "windows") {
                    "Visual Studio 17 2022".to_string()
                } else {
                    "Unix Makefiles".to_string()
                },
                _ => "Unix Makefiles".to_string(),
            }
        } else {
            "Unix Makefiles".to_string()
        }
    }

    // 実行ファイルの配置場所を判断するヘルパーメソッド
    pub fn _uses_subdirectories_for_executables(&self) -> bool {
        let generator = self.get_cmake_generator();
        generator.contains("Visual Studio") || generator.contains("Xcode")
    }
}
