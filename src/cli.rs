use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cpam")]
#[command(about = "CPaM: 対話型CMakeプロジェクト生成ツール", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 新規プロジェクトを作成する
    New(NewArgs),
    /// 依存ライブラリを追加する
    Add(AddArgs),
    /// 依存ライブラリを削除する
    Remove(RemoveArgs),
    /// プロジェクトをビルドする
    Build(BuildArgs),
    /// プロジェクトを実行する
    Run(RunArgs),
}

#[derive(Parser)]
pub struct NewArgs {
    /// プロジェクト種別: "bin" (バイナリ)または "lib" (ライブラリ)（デフォルトは "bin"）
    #[arg(long, default_value = "bin")]
    pub project_type: String,
    /// プロジェクト名（省略すると対話形式で入力します）
    pub name: Option<String>,
    /// 使用する言語: c, cpp, cuda（省略すると対話形式で選択します）
    #[arg(long)]
    pub language: Option<String>,
    /// 使用するビルドツール: make, ninja（省略すると対話形式で選択します）
    #[arg(long)]
    pub build_tool: Option<String>,
    /// 使用するコンパイラ（省略すると検出または対話形式で選択します）
    #[arg(long)]
    pub compiler: Option<String>,
}

#[derive(Parser)]
pub struct AddArgs {
    /// 追加する依存ライブラリ名
    pub name: String,
    /// バージョン（任意）
    #[arg(long)]
    pub version: Option<String>,
    /// 取得元（Git URLなど、任意）
    #[arg(long)]
    pub source: Option<String>,
}

#[derive(Parser)]
pub struct RemoveArgs {
    /// 削除する依存ライブラリ名
    pub name: String,
}

#[derive(Parser)]
pub struct BuildArgs {
    /// リリースモードでビルドする
    #[arg(long)]
    pub release: bool,
    /// ビルドディレクトリ名（デフォルトは "build"）
    #[arg(long, default_value = "build")]
    pub build_dir: String,
    /// CMakeジェネレーター（省略時はcpam.tomlから読み取り）
    #[arg(long)]
    pub generator: Option<String>,
}

#[derive(Parser)]
pub struct RunArgs {
    /// リリースモードで実行する
    #[arg(long)]
    pub release: bool,
}
