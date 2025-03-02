# CPaM

![image](https://github.com/user-attachments/assets/a89ac192-42d9-4bdd-8326-f22e3ecb548d)

CPaM (C Package Manager) is a tool that automates the CMake build process and simplifies dependency management for C/C++/CUDA projects. It handles complex CMake configurations with simple commands, providing an environment that allows developers to focus on code.

> [!CAUTION]
> We have confirmed that it does not work properly with CUDA at this time. Please wait a while for the fix...

## Features

- Automation of CMake build processes
- Easy package dependency management
- Project template generation
- Multi-platform support (Windows, Linux, macOS)
- Rich customization options

## Installation

1. Download the latest executable from the [releases page](https://github.com/BeadiestStar64/CPaM/releases) and place the exe file in a directory of your choice.

2. Add the directory containing the CPaM executable to your PATH environment variable:

   - For Windows: System Properties → Environment Variables → Path → Edit → New
   - For Linux/macOS: Add `export PATH=$PATH:/path/to/cpam` to your `.bashrc` or `.zshrc`

3. Run `cpam --version` in your terminal to verify that it's correctly installed.

## Usage

CPaM provides various subcommands:

```
cpam <subcommand> [options]
```

## Subcommands

### cpam new

Creates a new C/C++ project.

```
cpam new <project-name> [options]
```

Options:

- `--type <type>`: Project type (executable, library)
- `--cpp`: Create as a C++ project (C is default)
- `--standard <std>`: Language standard to use (e.g., c11, c++17)

Example:

```
cpam new my-awesome-app --type executable --cpp --standard c++17
```

### cpam build

Builds the project.

```
cpam build [options]
```

Options:

- `--debug`: Debug build
- `--release`: Release build (default)
- `--clean`: Clean build

### cpam add

Adds a dependency package to the project.

```
cpam add <package-name> [version]
```

Example:

```
cpam add libcurl 7.80.0
```

### cpam run

Runs the built program.

```
cpam run [args...]
```

## Project Structure

Basic structure of a project created with CPaM:

```
my-project/
├── CMakeLists.txt    # Auto-generated CMake file
├── cpam.toml         # CPaM configuration file
├── src/              # Source files
│   └── main.c/cpp
├── include/          # Header files
├── tests/            # Test code
└── build/            # Build artifacts (included in .gitignore)
```

## Customization

Edit the `cpam.toml` file to customize your project settings:

```toml
[project]
name = "my-project"
version = "0.1.0"
authors = ["Your Name <your.email@example.com>"]
standard = "c17"

[dependencies]
libcurl = "7.80.0"
zlib = "1.2.11"
```

## Contributing

1. Fork this repository
2. Create a feature branch (`git checkout -b amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push the branch (`git push origin amazing-feature`)
5. Open a Pull Request

## License

Distributed under the Apache 2.0 License. See the [LICENSE](LICENSE) file for more details.

## Contact

If you have questions or feedback, please create an Issue.