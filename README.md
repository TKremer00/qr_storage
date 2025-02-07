# QR Video converter

This is a rust project that converts a file to a video of qr code, and can decode that video to a file again.

## Prerequisites
Before you begin, ensure you have Rust installed on your system.

### Installing Rust
To install Rust, run the following command:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, restart your terminal and verify the installation:

```sh
rustc --version
cargo --version
```

For more details, visit the official Rust website: [Rust Installation Guide](https://www.rust-lang.org/tools/install).

## Building the Project

### 1. Clone the Repository

```sh
git clone <REPO>
cd your-rust-project
```

### 2. Build the project

```sh
cargo b
```

### 3. Run the project

```sh
cargo r
```

## Create a release build

```sh
cargo build --release
```
The executable will be located in the target/release/ directory.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
