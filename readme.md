# TUI Todo App

## Introduction

The TUI Todo App is a terminal-based todo list application built in Rust using the `tui` crate. It offers a rich, interactive experience for managing your tasks directly from the terminal.

__🤖 This project is created by ChatGPT 🤖__

## Usage

- Start the application by running `todo_app` from the terminal.
- Navigate through tasks with `jk`.
- Toggle task status with `space`.
- Add new tasks with `o`.
- Filter tasks with `/` and use `n` and `N` to navigate through results.
- Backup tasks with `b`.
- Reset tasks with `r`.

## Building and Installing

### Prerequisites

- __Rust__: Ensure you have Rust installed. You can get it from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
- __Cargo__: Comes with Rust, used for building the project.

### Build Manually

1. __Clone the Repository__:

    ```sh
    git clone https://github.com/longnghia/todo-tui.git
    cd todo-tui
    ```

2. __Build the Project__:

    To build the debug version of the application, run:

    ```sh
    cargo build
    ```

### Using `make` (Optional)

If you prefer, you can use the provided `Makefile` to build and install the binary:

1. __Run `make` to build and install__:

    ```sh
    make
    ```

2. __Run `make uninstall` to remove the binary__:

    ```sh
    make uninstall
    ```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For questions or issues, please open an issue in the [GitHub repository](https://github.com/longnghia/todo-tui) or contact me directly.
