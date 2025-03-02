use crate::cli::RunArgs;
use crate::commands::build::execute as execute_build;
use crate::cli::BuildArgs;
use crate::config::CpamConfig;
use std::path::Path;
use std::process::Command;
use std::io::{self, Write};

pub fn execute(args: &RunArgs) {
    // 設定を読み込むか確認
    let config = match CpamConfig::load() {
        Ok(config) => {
            // 念のためプロジェクト情報があるか確認
            if config.project.is_none() {
                println!("プロジェクト情報が見つかりません。実行する前に確認が必要です。");
                if !confirm_continue() {
                    return;
                }
            }
            config
        },
        Err(_) => {
            println!("cpam.toml が見つからないか、読み込めません。");
            println!("ビルドと実行を続行しますか？");
            if !confirm_continue() {
                return;
            }
            CpamConfig::default()
        }
    };

    // まずビルドを実行
    let build_args = BuildArgs {
        release: args.release,
        build_dir: "build".to_string(),
        generator: None,
    };
    execute_build(&build_args);

    // プロジェクト名を取得
    let project_name = if let Some(project) = &config.project {
        project.name.clone()
    } else {
        // cpam.tomlが無いか不完全な場合は実行ファイル名を尋ねる
        println!("実行ファイル名を入力してください:");
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("入力の読み取りに失敗しました");
        input.trim().to_string()
    };

    // ビルドディレクトリを取得
    let build_dir = if let Some(build) = &config.build {
        build.build_dir.clone().unwrap_or_else(|| "build".to_string())
    } else {
        "build".to_string()
    };

    // 使用しているジェネレーターを取得
    let generator = config.get_cmake_generator();

    // 実行可能ファイルのパスを構築（ビルドシステムによって配置場所が異なる）
    let executable_path = get_executable_path(&build_dir, &project_name, &generator, args.release);

    println!("実行ファイルを探索中: {}", executable_path);

    // 実行可能ファイルが存在するか確認
    if !Path::new(&executable_path).exists() {
        println!("最初のパスで実行ファイルが見つかりません。代替パスを試します...");

        // 代替パスを試す
        let alt_paths = get_alternative_executable_paths(&build_dir, &project_name, args.release);
        let mut executable_found = false;

        for alt_path in &alt_paths {
            println!("代替パスを確認中: {}", alt_path);
            if Path::new(alt_path).exists() {
                println!("実行ファイルが見つかりました: {}", alt_path);
                executable_found = true;

                println!("実行ファイルを起動: {}", alt_path);
                let status = Command::new(alt_path).status();
                match status {
                    Ok(s) if s.success() => println!("プログラムは正常に実行されました。"),
                    Ok(s) => eprintln!("実行ファイルが異常終了 (exit code: {})", s),
                    Err(e) => eprintln!("実行ファイルの起動に失敗: {}", e),
                }

                break;
            }
        }

        if !executable_found {
            eprintln!("実行ファイルが見つかりません。ビルドが正常に完了したか確認してください。");
            eprintln!("検索したパス:");
            eprintln!("- {}", executable_path);
            for path in &alt_paths {
                eprintln!("- {}", path);
            }
        }

        return;
    }

    println!("実行ファイルを起動: {}", executable_path);
    let status = Command::new(&executable_path).status();
    match status {
        Ok(s) if s.success() => println!("プログラムは正常に実行されました。"),
        Ok(s) => eprintln!("実行ファイルが異常終了 (exit code: {})", s),
        Err(e) => eprintln!("実行ファイルの起動に失敗: {}", e),
    }
}

// ジェネレータとビルドタイプに基づいて実行ファイルのパスを取得
fn get_executable_path(build_dir: &str, project_name: &str, generator: &str, is_release: bool) -> String {
    if cfg!(target_os = "windows") {
        // Windowsでの処理
        match generator {
            // Ninja または Makefilesの場合、実行ファイルはビルドディレクトリの直下
            g if g.contains("Ninja") || g.contains("Makefiles") => {
                format!("{}/{}.exe", build_dir, project_name)
            },
            // Visual Studio の場合はサブディレクトリに出力される
            g if g.contains("Visual Studio") => {
                if is_release {
                    format!("{}/Release/{}.exe", build_dir, project_name)
                } else {
                    format!("{}/Debug/{}.exe", build_dir, project_name)
                }
            },
            // その他のジェネレータの場合はデフォルトパス
            _ => format!("{}/{}.exe", build_dir, project_name)
        }
    } else {
        // Unix系OSの場合
        format!("{}/{}", build_dir, project_name)
    }
}

// 代替の実行ファイルパスのリストを取得
fn get_alternative_executable_paths(build_dir: &str, project_name: &str, _is_release: bool) -> Vec<String> {
    let mut paths = Vec::new();

    if cfg!(target_os = "windows") {
        // Windows環境での一般的なパターン
        paths.push(format!("{}/{}.exe", build_dir, project_name));
        paths.push(format!("{}/Debug/{}.exe", build_dir, project_name));
        paths.push(format!("{}/Release/{}.exe", build_dir, project_name));
        paths.push(format!("{}/{}/Debug/{}.exe", build_dir, project_name, project_name));
        paths.push(format!("{}/{}/Release/{}.exe", build_dir, project_name, project_name));
    } else {
        // Unix環境での一般的なパターン
        paths.push(format!("{}/{}", build_dir, project_name));
        paths.push(format!("{}/Debug/{}", build_dir, project_name));
        paths.push(format!("{}/Release/{}", build_dir, project_name));
    }

    paths
}

fn confirm_continue() -> bool {
    print!("続行しますか？ [y/N]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("入力の読み取りに失敗しました");
    input.trim().to_lowercase().starts_with('y')
}
