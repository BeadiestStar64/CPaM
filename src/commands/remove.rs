use crate::cli::RemoveArgs;
use crate::config::CpamConfig;

pub fn execute(args: &RemoveArgs) {
    // 現在の設定を読み込む
    let mut config = match CpamConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("cpam.toml の読み込みに失敗: {}", e);
            return;
        }
    };

    // 依存関係から削除
    if let Some(dependencies) = &mut config.dependencies {
        if dependencies.remove(&args.name).is_none() {
            eprintln!("依存ライブラリ '{}' は設定に存在しません。", args.name);
            return;
        }
    } else {
        eprintln!("依存ライブラリ '{}' は設定に存在しません。", args.name);
        return;
    }

    // 設定を保存
    if let Err(e) = config.save() {
        eprintln!("cpam.toml の更新に失敗: {}", e);
        return;
    }

    println!("依存ライブラリ '{}' を削除しました。", args.name);
}
