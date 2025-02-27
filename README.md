# Altius Benchtools

This is a collection of tools for Altius benchmarking, featuring a `profiler` for RPC server execution tracing and a `transaction_generator` for Ethereum test cases.

> **Tip:** Start by running the example in `examples/how_to_use_profiler.rs` to see the profiler in action and understand its different usage patterns.

<br>

## 1. Profiler

A tool for tracing and profiling RPC server execution with detailed timing and event tracking capabilities.

### Features

- Task timing with start/end markers
- Multi-threaded profiling support
- Event annotation with notes and descriptions
- JSON and ZIP output formats
- Special handling for transaction and commit events

### Usage

```rust
// Start timing a task
profiler::start("task_name");

// ... your code here ...

// Add notes to the current task
profiler::note_str("task_name", "key", "value");

// ... your code here ...

// End timing a task
profiler::end("task_name");

// Export results
profiler::dump_json("output.json");
// or
profiler::dump_zip("output");
```

### Multi-threaded Usage

The profiler supports concurrent operations across multiple threads:

```rust
// In any thread
profiler::start_multi("thread_task");

// ... your code here ...

// In any thread
profiler::note_str_multi("thread_task", "thread_info", "worker_1");

// ... your code here ...

// In any thread
profiler::end_multi("thread_task");

// ... your code here ...

// Export results
profiler::dump_json("output.json");
// or
profiler::dump_zip("output");
```

### Output Format

The profiler generates a JSON structure containing:
- Timing information for each task
- Thread identification
- Custom annotations and notes
- Transaction and commit event details

An example of the output JSON is as follows:

```json
{
  "details": [
    {
      "detail": {
        "hash": "0x26b7c694ff75f0b4ee85b5ca2e3cc1c332b41a64982c2b454e0493497b8e76b9",
        "type": "transaction"
      },
      "end": 212387237,
      "runtime": 31286,
      "start": 212355951,
      "status": "success",
      "tx": "125",
      "type": "transaction"
    },
    {
      "detail": {
        "hash": "0xbc3d47d6c7df3430c8c88e0e6b28204185d3a7aab0fb7f8464e2b28b0d79d1bd",
        "type": "transaction"
      },
      "end": 232170705,
      "runtime": 163541,
      "start": 232007164,
      "status": "success",
      "tx": "125",
      "type": "transaction"
    },
    {
      "detail": {
        "hash": "0x255cd19c2bad53734fc8c6df7e5b6f74a85733183b9cb9bcbf1e16de9404d87d",
        "type": "transaction"
      },
      "end": 255598060,
      "runtime": 28209,
      "start": 255569851,
      "status": "revert",
      "tx": "125",
      "type": "transaction"
    },
    { "...": "..." }
  ],
  [ "..." ]
}
```

<br>

## 2. Transaction Generator

This tool generates a JSON file containing a list of transactions and a pre-state of the blockchain.

### Usage

1. Run `cargo build --release` to build the project.
2. Run `./target/release/generate --help` to see the available options.

### ETH-transfer Usage

After building the project, you can use the following commands to generate test cases.

- Generate a JSON file with 100 ETH-transfer transactions in 10 groups, using the `one-to-many` pattern, and save it to `./test-case.json`.
  ```bash
  ./target/release/generate pattern -y o2m -t 100 -g 10 -o ./test-case.json
  # or
  ./target/release/generate pattern -y one-to-many -t 100 -g 10 -o ./test-case.json
  ```

- Generate a JSON file with 200 ETH-transfer transactions in 5 groups, using the `chained` pattern, and save it to `./test-case.json`.
  ```bash
  ./target/release/generate pattern -y chained -t 200 -g 5 -o ./test-case.json
  # or
  ./target/release/generate pattern -y ring -t 200 -g 5 -o ./test-case.json
  # or
  ./target/release/generate pattern -y chain -t 200 -g 5 -o ./test-case.json
  ```

- Generate a JSON file with 100 ETH-transfer transactions with 60% conflict rate, and save it to `./test-case.json`.
  ```bash
  ./target/release/generate pattern -y m2m -t 100 -c 0.6 -o ./test-case.json
  # or
  ./target/release/generate pattern -y many-to-many -t 100 -c 0.6 -o ./test-case.json
  ```

### ERC20-transfer Usage

Directly use the `erc20` flag to generate ERC20-transfer transactions. Other options are the same as ETH-transfer.

- Generate a JSON file with 100 ERC20-transfer transactions in 10 groups, using the `one-to-many` pattern, and save it to `./test-case.json`.
  ```bash
  ./target/release/generate pattern -y o2m -t 100 -g 10 -o ./test-case.json --erc20
  # or
  ./target/release/generate pattern -y one-to-many -t 100 -g 10 -o ./test-case.json --erc20
  ```

- Other options are the same as ETH-transfer.

### Output Format

The output JSON file is a list of transactions and a pre-state of the blockchain.

An example of the output JSON is as follows:
```json
{
  "just-test": {
    "_info": { "...": "..." },
    "env": { "...": "..." },
    "post": {
      "Cancun": { "...": "..." }
    },
    "pre": {
      "0xcc2564c36a3440e7d6dd4c67b50f885edbfa5141": {
        "balance": "0x056bc75e2d63100000",
        "code": "0x",
        "nonce": "0x00",
        "storage": {}
      }
    },
    "transaction": [
      {
        "data": "0x",
        "gasLimit": "0x0f4240",
        "gasPrice": "0x0a",
        "nonce": "0x00",
        "secretKey": "0xa119adadef6246ab1780711938aa3b73f86ca408fc2fbbb2fa69135e3ae65c72",
        "sender": "0xcc2564c36a3440e7d6dd4c67b50f885edbfa5141",
        "to": "0xfa3d1fa8d995c05e9fbea98b0f2242391c738625",
        "value": "0x02b5e3af16b1880000"
      },
      {
        "data": "0x",
        "gasLimit": "0x0f4240",
        "gasPrice": "0x0a",
        "nonce": "0x00",
        "secretKey": "0x5d5baf05f2df8d5974daae1ff6848fceff6f4b0b781df360b8a0d6f9b68f96c6",
        "sender": "0xfa3d1fa8d995c05e9fbea98b0f2242391c738625",
        "to": "0x3d8b1f10cda76db2f9f5132b8250786bd4fd1f7a",
        "value": "0x02b5e3a5fe63156000"
      },
      {
        "data": "0x",
        "gasLimit": "0x0f4240",
        "gasPrice": "0x0a",
        "nonce": "0x00",
        "secretKey": "0x0d87dd2aba604787e47bd5ae0233c16db952478fa08eb77d373b1fc807c0ee11",
        "sender": "0x3d8b1f10cda76db2f9f5132b8250786bd4fd1f7a",
        "to": "0xa6a410156ec7b055ac4b5f89a812944bf47ad6de",
        "value": "0x02b5e39ce614a2c000"
      }
    ]
  }
}
```

<br>

## How To Contribute

We welcome contributions to the Altius Benchtools project! Here's how you can contribute:

### Reporting Issues

If you encounter any bugs or have feature requests:

1. Check the [Issues](https://github.com/anz-devin/toolbench/issues) page to see if your issue has already been reported.
2. If not, create a new issue with a clear description and steps to reproduce.

### Contributing Code

1. Fork the repository on GitHub.
2. Clone your fork locally: `git clone https://github.com/YOUR-USERNAME/toolbench.git`
3. Create a new branch for your feature or bugfix: `git checkout -b feature/your-feature-name`
4. Make your changes, ensuring you follow the code style of the project.
5. Run tests and linting to ensure your changes don't break existing functionality:
   ```bash
   cargo fmt
   cargo clippy
   ```
6. Commit your changes with a descriptive commit message.
7. Push your branch to your fork: `git push origin feature/your-feature-name`
8. Create a Pull Request from your fork to the main repository.

### Pull Request Guidelines

- Never commit directly to the `develop` branch. All changes must go through pull requests.
- Ensure your code is properly formatted with `cargo fmt`.
- Make sure all linting checks pass with `cargo clippy`.
- Write clear commit messages that explain the purpose of your changes.
- Update documentation if your changes affect the public API or user-facing features.
