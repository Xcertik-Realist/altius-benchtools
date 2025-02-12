# Altius Benchtools

This is a collection of tools for Altius benchmarking, featuring a `profiler` for RPC server execution tracing and a `transaction generator` for Ethereum test cases.

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
    // ...
  ], 
  // ...
}
```

## 2. Transaction Generator

This tool generates a JSON file containing a list of transactions and a pre-state of the blockchain.

### Usage

1. Run `cargo build --release` to build the project.
2. Run `./target/release/generate --help` to see the available options.

#### ETH-transfer

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

#### ERC20-transfer

Directly use the `erc20` flag to generate ERC20-transfer transactions. Other options are the same as ETH-transfer.

- Generate a JSON file with 100 ERC20-transfer transactions in 10 groups, using the `one-to-many` pattern, and save it to `./test-case.json`.
  ```bash
  ./target/release/generate pattern -y o2m -t 100 -g 10 -o ./test-case.json --erc20
  # or
  ./target/release/generate pattern -y one-to-many -t 100 -g 10 -o ./test-case.json --erc20
  ```

- Other options are the same as ETH-transfer.
