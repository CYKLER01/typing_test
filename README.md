# Typing Test CLI

A terminal-based typing test application written in Rust.

## Features

*   **Two Game Modes:**
    *   **Words:** Type a specific number of words.
    *   **Time:** Type for a specific amount of time.
*   **Difficulty Levels:** Choose between Easy, Medium, and Hard word lists.
*   **Customizable Layout:** Select from different visual themes.
*   **Configuration Menu:** An interactive menu to easily change settings.
*   **Persistent Results:** Your WPM and accuracy are saved for each test configuration.
*   **Stats View:** View your saved stats in a table and a graph.

## Building and Running

### Building from source

1.  **Clone the repository:**
    ```bash
    git clone <repository-url>
    cd typing_test
    ```

2.  **Build the project:**
    ```bash
    cargo build --release
    ```

3.  **Run the application:**
    *   To start a typing test with the current settings:
        ```bash
        ./target/release/typing_test
        ```
    *   To open the settings menu:
        ```bash
        ./target/release/typing_test -m
        ```
    *   To see your saved stats:
        ```bash
        ./target/release/typing_test -s
        ```
    *   To see the help message:
        ```bash
        ./target/release/typing_test -h
        ```

### Installation

You can install the application to make it available system-wide.

1.  **Install the binary:**
    ```bash
    cargo install --path .
    ```
    This will install the binary to `~/.cargo/bin/typing_test`.

2.  **Ensure `~/.cargo/bin` is in your `PATH`:**
    Add the following line to your shell's configuration file (e.g., `~/.bashrc`, `~/.zshrc`):
    ```bash
    export PATH="$HOME/.cargo/bin:$PATH"
    ```

3.  **Run the application:**
    You can now run the application using the `typing_test` command:
    ```bash
    typing_test
    typing_test -m
    ```

4.  **(Optional) Rename the command:**
    If you want to use a different command, like `rusttt`, you can rename the binary or create a symbolic link. For example:
    ```bash
    mv ~/.cargo/bin/typing_test ~/.cargo/bin/rusttt
    ```
    Now you can run the application with:
    ```bash
    rusttt
    rusttt -m
    rusttt -s
    ```

## How to Play

*   The application will start in the game mode specified in your configuration.
*   Start typing the words displayed on the screen.
*   The text will change color to indicate correct and incorrect characters.
*   Press the `Spacebar` to move to the next word.
*   Press `Tab` to restart the test.
*   Press `Esc` to exit the test.

## Stats View

You can access the stats view by running the application with the `-s` or `--stats` flag.

```bash
    cargo run -- -s
    ```

In the stats view, you can:

*   Navigate between game modes using the `Up` and `Down` arrow keys.
*   Switch between a table and a graph display using the `t` and `g` keys.
*   Press `q` to quit the stats view.


## Settings Menu

You can access the settings menu by running the application with the `-m` or `--menu` flag.

```bash
    cargo run -- -m
    ```

In the menu, you can:

*   Navigate between options using the `Up` and `Down` arrow keys.
*   Change the values of the selected option using the `Left` and `Right` arrow keys.
*   Press `Enter` to save your changes.
*   Press `q` to quit the menu.

### Available Settings

*   **Game Mode:** `Words` or `Time`.
*   **Test Length (Words):** The number of words for the "Words" game mode.
*   **Time Limit (Seconds):** The duration for the "Time" game mode.
*   **Layout Theme:** `Default` or `Boxes`.
*   **Word List Difficulty:** `Easy`, `Medium`, or `Hard`.

## Configuration

The application saves your settings and test results in a `config.json` file. This file is located in the appropriate configuration directory for your operating system.

*   **Linux:** `~/.config/typing_test/config.json`
*   **macOS:** `~/Library/Application Support/com.gemini.typing_test/config.json`
*   **Windows:** `C:\Users\<YourUser>\AppData\Roaming\gemini\typing_test\config\config.json`

You can manually edit this file to change the color theme or other advanced settings.

```