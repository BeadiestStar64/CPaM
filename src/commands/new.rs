use crate::cli::NewArgs;
use crate::config::{BuildConfig, CpamConfig, ProjectConfig};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

pub fn execute(args: &NewArgs) {
    // 完全に対話形式のプロジェクト作成プロセス
    let project_name = get_project_name(args.name.as_deref());
    let project_type = &args.project_type;
    let language = select_language(args.language.as_deref());
    let build_tool = select_build_tool(args.build_tool.as_deref());
    let compiler = select_compiler(args.compiler.as_deref(), &language);

    // 選択内容の表示と確認
    println!("\n===== プロジェクト設定の確認 =====");
    println!("- 名前: {}", project_name);
    println!("- 言語: {}", language);
    println!("- ビルドツール: {}", build_tool);
    println!("- コンパイラ: {}", compiler);
    println!("- 種別: {}", project_type);
    println!("===============================");
    print!("この設定でプロジェクトを作成しますか？ [Y/n]: ");
    io::stdout().flush().unwrap();

    let mut confirm = String::new();
    io::stdin().lock().read_line(&mut confirm).expect("入力の読み取りに失敗しました");
    if confirm.trim().to_lowercase() == "n" {
        println!("操作をキャンセルしました。もう一度実行して設定し直してください。");
        return;
    }

    // プロジェクトのルートディレクトリを作成
    let base_path = Path::new(&project_name);
    if base_path.exists() {
        println!("警告: '{}' ディレクトリが既に存在します。", project_name);
        print!("既存ディレクトリに上書きしますか？ [y/N]: ");
        io::stdout().flush().unwrap();

        let mut overwrite = String::new();
        io::stdin().lock().read_line(&mut overwrite).expect("入力の読み取りに失敗しました");
        if !overwrite.trim().to_lowercase().starts_with('y') {
            println!("操作をキャンセルしました。");
            return;
        }
    } else if let Err(e) = fs::create_dir(base_path) {
        println!("エラー: プロジェクトディレクトリの作成に失敗しました。");
        println!("詳細: {}", e);
        println!("別の名前でプロジェクトを作成するか、既存のディレクトリを確認してください。");
        return;
    }

    // 標準的なプロジェクトディレクトリ構造を作成
    println!("\nプロジェクト構造を作成中...");
    for dir in &["include", "src", "lib"] {
        let path = base_path.join(dir);
        if let Err(e) = fs::create_dir_all(&path) {
            println!("警告: {} ディレクトリの作成に失敗しました: {}", dir, e);
            println!("一部のディレクトリが作成できませんでしたが、続行します。");
        }
    }

    // 言語に応じた main ファイルを作成
    let (main_file_name, main_content) = match language.as_str() {
        "c" => (
            "main.c",
            "#include <stdio.h>\n\nint main() {\n    printf(\"Hello, World!\\n\");\n    return 0;\n}\n",
        ),
        "cuda" => (
            "main.cu",
            "#include <stdio.h>\n\n__global__ void hello() {\n    printf(\"Hello, World!\\n\");\n}\n\nint main() {\n    hello<<<1,1>>>();\n    cudaDeviceSynchronize();\n    return 0;\n}\n",
        ),
        _ => (
            "main.cpp",
            "#include <iostream>\n\nint main() {\n    std::cout << \"Hello, World!\" << std::endl;\n    return 0;\n}\n",
        ),
    };

    // src/main.xxx を作成
    let main_file_path = base_path.join("src").join(main_file_name);
    if let Err(e) = fs::write(&main_file_path, main_content) {
        println!("警告: メインソースファイルの作成に失敗しました: {}", e);
        println!("続行しますが、後でソースファイルを手動で作成する必要があります。");
    } else {
        println!("ソースファイル {} を作成しました", main_file_name);
    }

    // CMakeLists.txt の作成
    let cmake_language = match language.as_str() {
        "c" => "C",
        "cuda" => "CUDA",
        _ => "CXX",
    };

    // コンパイラが選択されている場合、CMakeに設定を追加
    let compiler_config = if compiler != "default" {
        match cmake_language {
            "C" => format!("\nset(CMAKE_C_COMPILER {})\n", compiler),
            "CUDA" => format!("\nset(CMAKE_CUDA_COMPILER {})\n", compiler),
            _ => format!("\nset(CMAKE_CXX_COMPILER {})\n", compiler),
        }
    } else {
        String::new()
    };

    let cmake_content = format!(
        "cmake_minimum_required(VERSION 3.10)\n\
         project({} LANGUAGES {})\n\
         set(CMAKE_CXX_STANDARD 17){}\n\n\
         add_executable({} src/{})\n",
        project_name, cmake_language, compiler_config, project_name, main_file_name
    );

    let cmake_path = base_path.join("CMakeLists.txt");
    if let Err(e) = fs::write(&cmake_path, cmake_content) {
        println!("警告: CMakeLists.txt の作成に失敗しました: {}", e);
        println!("続行しますが、後でCMakeLists.txtを手動で作成する必要があります。");
    } else {
        println!("CMakeLists.txt を作成しました");
    }

    // cpam.toml の設定ファイル作成
    let config = CpamConfig {
        project: Some(ProjectConfig {
            name: project_name.clone(),
            language: language.clone(),
            build_tool: build_tool.clone(),
            project_type: project_type.clone(),
        }),
        build: Some(BuildConfig {
            generator: Some(get_generator_for_build_tool(&build_tool)),
            build_dir: Some("build".to_string()),
            source_dir: Some(".".to_string()),
            options: if compiler != "default" {
                Some(vec![format!("-DCMAKE_{}_COMPILER={}",
                    if cmake_language == "CXX" { "CXX" } else { cmake_language },
                    compiler)])
            } else {
                None
            },
        }),
        dependencies: None,
    };

    let toml_str = match toml::to_string(&config) {
        Ok(str) => str,
        Err(e) => {
            println!("警告: 設定のシリアライズに失敗しました: {}", e);
            println!("基本的な設定ファイルを作成します。");
            format!(
                "[project]\nname = \"{}\"\nlanguage = \"{}\"\nbuild_tool = \"{}\"\nproject_type = \"{}\"\n\n[build]\ngenerator = \"{}\"\nbuild_dir = \"build\"\nsource_dir = \".\"\n",
                project_name, language, build_tool, project_type, get_generator_for_build_tool(&build_tool)
            )
        }
    };

    let cpam_toml_path = base_path.join("cpam.toml");
    if let Err(e) = fs::write(&cpam_toml_path, toml_str) {
        println!("警告: cpam.toml の作成に失敗しました: {}", e);
    } else {
        println!("cpam.toml 設定ファイルを作成しました");
    }

    println!("\n🎉 プロジェクト '{}' の作成が完了しました！", project_name);
    println!("\n開始方法:");
    println!("  cd {}", project_name);
    println!("  cpam build        # プロジェクトをビルド");
    println!("  cpam run          # プロジェクトを実行");
}

// プロジェクト名を取得または生成する関数
fn get_project_name(name_arg: Option<&str>) -> String {
    if let Some(name) = name_arg {
        return name.to_string();
    }

    println!("\n=== 新規プロジェクト作成 ===");
    println!("作成するプロジェクト名を入力してください。(これはフォルダ名になります!)");
    println!("GoogleのC++スタイルシートに従い、アンダースコアなしの単語をハイフン(-)で区切った名前を推奨します。");
    println!("例: hello-world, my-project");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut name = String::new();
    io::stdin().lock().read_line(&mut name).expect("入力の読み取りに失敗しました");
    let name = name.trim().to_string();

    if name.is_empty() {
        println!("プロジェクト名が空です。デフォルト名 'my-project' を使用します。");
        return "my-project".to_string();
    }

    // 入力された名前のバリデーション
    if name.contains("_") {
        println!("注意: アンダースコア(_)よりもハイフン(-)の使用を推奨します。");
    }

    if name.contains(" ") {
        println!("警告: スペースを含む名前はビルド時に問題が発生する可能性があります。");
        println!("スペースをハイフン(-)に置き換えますか？ [Y/n]: ");
        let mut replace = String::new();
        io::stdin().lock().read_line(&mut replace).expect("入力の読み取りに失敗しました");
        if replace.trim().to_lowercase() != "n" {
            return name.replace(" ", "-");
        }
    }

    name
}

// 言語を対話形式で選択する関数
fn select_language(language_arg: Option<&str>) -> String {
    if let Some(lang) = language_arg {
        return lang.to_string();
    }

    println!("\n使用する言語を選択してください:");
    println!("1. C++ (cpp) - オブジェクト指向プログラミングと標準ライブラリ");
    println!("2. C (c) - システムプログラミングに適した言語");
    println!("3. CUDA (cuda) - NVIDIA GPUプログラミング向け");
    print!("選択 [1-3] > ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).expect("入力の読み取りに失敗しました");
    let input = input.trim();

    match input {
        "2" => "c".to_string(),
        "3" => "cuda".to_string(),
        "c" => "c".to_string(),
        "cuda" => "cuda".to_string(),
        "cpp" | "c++" => "cpp".to_string(),
        _ => {
            if !input.is_empty() && input != "1" {
                println!("注意: 入力「{}」を認識できません。デフォルトの「C++」を使用します。", input);
            }
            "cpp".to_string()
        }
    }
}

// ビルドツールを対話形式で選択する関数
fn select_build_tool(build_tool_arg: Option<&str>) -> String {
    if let Some(tool) = build_tool_arg {
        return tool.to_string();
    }

    println!("\n使用するビルドシステムを選択してください:");
    println!("1. Make (make) - 最も広くサポートされているビルドツール");
    println!("2. Ninja (ninja) - 高速で効率的なビルドツール");

    // Windowsの場合はVisual Studioも選択肢に追加
    let is_windows = cfg!(target_os = "windows");
    if is_windows {
        println!("3. Visual Studio (vs) - WindowsでのC++開発に最適");
    }

    print!("選択 > ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).expect("入力の読み取りに失敗しました");
    let input = input.trim();

    match input {
        "2" | "ninja" => "ninja".to_string(),
        "3" | "vs" if is_windows => "vs".to_string(),
        "make" => "make".to_string(),
        _ => {
            if !input.is_empty() && input != "1" {
                println!("注意: 入力「{}」を認識できません。デフォルトの「make」を使用します。", input);
            }
            "make".to_string()
        }
    }
}

// コンパイラを検出・選択する関数
fn select_compiler(compiler_arg: Option<&str>, language: &str) -> String {
    if let Some(compiler) = compiler_arg {
        return compiler.to_string();
    }

    // まず利用可能なコンパイラを検出
    let compilers = detect_available_compilers(language);

    // 利用可能なコンパイラが1つだけの場合は選択をスキップ
    if compilers.len() == 1 {
        let compiler = &compilers[0];
        println!("\n検出されたコンパイラ: {} を使用します", compiler);
        return compiler.clone();
    }

    // 利用可能なコンパイラがない場合はデフォルト設定を使用
    if compilers.is_empty() {
        println!("\n警告: 使用可能なコンパイラが見つかりませんでした。");
        println!("システムデフォルトのコンパイラを使用します。");
        return "default".to_string();
    }

    // 複数のコンパイラが利用可能な場合は選択を促す
    println!("\n使用するコンパイラを選択してください:");
    for (i, compiler) in compilers.iter().enumerate() {
        println!("{}. {}", i + 1, compiler);
    }

    print!("選択 [1-{}] > ", compilers.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).expect("入力の読み取りに失敗しました");
    let input = input.trim();

    // 入力を数値として解釈
    if let Ok(num) = input.parse::<usize>() {
        if num >= 1 && num <= compilers.len() {
            return compilers[num - 1].clone();
        }
    }

    // 入力がコンパイラ名と一致する場合
    for compiler in &compilers {
        if input.to_lowercase() == compiler.to_lowercase() {
            return compiler.clone();
        }
    }

    // デフォルトのコンパイラを選択
    println!("注意: 入力「{}」を認識できません。最初のコンパイラを使用します。", input);
    compilers[0].clone()
}

// システムに利用可能なコンパイラを検出する関数
fn detect_available_compilers(language: &str) -> Vec<String> {
    let mut compilers = Vec::new();

    let compiler_commands = match language {
        "c" => vec!["gcc", "clang", "cc"],
        "cpp" => vec!["g++", "clang++", "c++"],
        "cuda" => vec!["nvcc"],
        _ => vec!["g++", "clang++"],
    };

    // Windowsの場合はMSVC系も追加
    if cfg!(target_os = "windows") {
        if language == "cpp" || language == "c" {
            compilers.push("cl.exe".to_string()); // Visual C++ compiler
        }
    }

    // コンパイラの存在を確認
    for cmd in compiler_commands {
        match Command::new(cmd).arg("--version").output() {
            Ok(_) => {
                compilers.push(cmd.to_string());
            }
            Err(_) => {
                // このコンパイラはインストールされていない
            }
        }
    }

    compilers
}

fn get_generator_for_build_tool(build_tool: &str) -> String {
    match build_tool.to_lowercase().as_str() {
        "ninja" => "Ninja".to_string(),
        "vs" => if cfg!(target_os = "windows") {
            "Visual Studio 17 2022".to_string()
        } else {
            "Unix Makefiles".to_string()
        },
        _ => "Unix Makefiles".to_string(), // デフォルトはMake
    }
}
