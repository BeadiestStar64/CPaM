use std::collections::HashMap;

use std::path::Path;
use crate::cli::AddArgs;
use crate::config::CpamConfig;

pub fn execute(args: &AddArgs) {
    let toml_path = Path::new("cpam.toml");
    if !toml_path.exists() {
        eprintln!("cpam.toml がカレントディレクトリに存在しません。");
        return;
    }

    // 現在の設定を読み込む
    let mut config = match CpamConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("cpam.toml の読み込みに失敗: {}", e);
            return;
        }
    };

    // 依存関係を追加
    let mut dependencies = config.dependencies.unwrap_or_else(|| HashMap::new());
    let version = args.version.clone().unwrap_or_else(|| "*".to_string());
    dependencies.insert(args.name.clone(), version);
    config.dependencies = Some(dependencies);

    // 設定を保存
    if let Err(e) = config.save() {
        eprintln!("cpam.toml の更新に失敗: {}", e);
        return;
    }

    println!("依存ライブラリ '{}' を追加しました。", args.name);
}
