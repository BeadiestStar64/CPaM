use crate::cli::BuildArgs;
use crate::config::CpamConfig;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

pub fn execute(args: &BuildArgs) {
    // 設定を読み込む
    let config = match CpamConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("cpam.toml の読み込みに失敗: {}", e);
            eprintln!("対話モードでビルド設定を入力してください。");

            // 対話的に設定を行う
            let _generator = prompt_for_generator();

            println!("デフォルトビルド設定で続行します。");
            CpamConfig::default()
        }
    };

    // ビルドディレクトリ
    let build_dir = &args.build_dir;
    if let Err(e) = fs::create_dir_all(build_dir) {
        eprintln!("ビルドディレクトリの作成に失敗: {}", e);
        return;
    }

    // ジェネレータ（CMakeのビルドシステム）を決定
    let generator = match &args.generator {
        Some(gen) => gen.clone(),
        None => config.get_cmake_generator(),
    };

    // ソースディレクトリを決定
    let source_dir = if let Some(build) = &config.build {
        build.source_dir.clone().unwrap_or_else(|| ".".to_string())
    } else {
        ".".to_string()
    };

    // CMakeの構成
    println!("CMakeを設定: ジェネレータ={}", generator);
    let mut cmake_config = Command::new("cmake");
    cmake_config.args(&["-S", &source_dir, "-B", build_dir, "-G", &generator]);

    // ビルドタイプを設定
    if args.release {
        cmake_config.arg("-DCMAKE_BUILD_TYPE=Release");
    } else {
        cmake_config.arg("-DCMAKE_BUILD_TYPE=Debug");
    }

    // 追加オプションがあれば設定
    if let Some(build) = &config.build {
        if let Some(options) = &build.options {
            for option in options {
                cmake_config.arg(option);
            }
        }
    }

    let status = cmake_config.status();
    match status {
        Ok(s) if s.success() => {
            println!("CMakeの設定に成功しました。");
        },
        Ok(s) => {
            eprintln!("CMake構成が失敗 (exit code: {})", s);
            return;
        }
        Err(e) => {
            eprintln!("cmake コマンドの実行に失敗: {}", e);
            return;
        }
    }

    // ビルド実行
    println!("ビルドを実行中...");
    let mut build_cmd = Command::new("cmake");
    build_cmd.args(&["--build", build_dir]);

    if args.release {
        build_cmd.args(&["--config", "Release"]);
    } else {
        build_cmd.args(&["--config", "Debug"]);
    }

    let status = build_cmd.status();
    match status {
        Ok(s) if s.success() => println!("ビルドに成功しました。"),
        Ok(s) => eprintln!("ビルドが失敗 (exit code: {})", s),
        Err(e) => eprintln!("ビルドコマンドの実行に失敗: {}", e),
    }
}

fn prompt_for_generator() -> String {
    println!("CMakeジェネレーターを選択してください:");
    println!("1. Unix Makefiles");
    println!("2. Ninja");
    println!("3. Visual Studio");
    println!("4. Xcode");
    print!("選択 [1-4]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("入力の読み取りに失敗しました");

    match input.trim() {
        "2" => "Ninja".to_string(),
        "3" => "Visual Studio 17 2022".to_string(),
        "4" => "Xcode".to_string(),
        _ => "Unix Makefiles".to_string(),
    }
}
