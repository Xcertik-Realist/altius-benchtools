use clap::{Arg, Command};
use serde_json::{json, Map, Value};
use std::{fs::File, io::Write};
use transaction_generator::TransactionGenerator;

pub fn build_json_output(
    pre: Map<String, Value>,
    transactions: Vec<Value>,
    info: serde_json::Value,
    env: serde_json::Value,
    name: String,
) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(json!({
        name: {
            "_info": info,
            "env": env,
            "pre": pre,
            "transaction": transactions,
            "post": {
                "Cancun": {
                    "hash": "",
                    "indexes": {},
                    "logs": "",
                    "txbytes": ""
                }
            }
        }
    }))
}

fn get_info() -> serde_json::Value {
    json!({
        "comment": "altius transfer",
        "filling-rpc-server": "evm version 1.13.11-unstable-765f2904-20240124",
        "filling-tool-version": "retesteth-0.3.2-cancun+commit.ea13235b.Linux.g++",
        "generatedTestHash": "7e42d931d6e7a1e308874fd21b386d86daf6af0a52be5c5a1f52f89423b2c28b",
        "lllcversion": "Version: 0.5.14-develop.2023.7.11+commit.c58ab2c6.mod.Linux.g++",
        "solidity": "Version: 0.8.21+commit.d9974bed.Linux.g++",
        "source": "",
        "sourceHash": "5138279197c12e7d349cb50a0f3d9c8ceaef4310463fb11af9b6a346ad5a5918"
    })
}

fn gen_env() -> serde_json::Value {
    json!({
        "currentBaseFee": "0x0a",
        "currentCoinbase": "0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
        "currentDifficulty": "0x020000",
        "currentExcessBlobGas": "0x00",
        "currentGasLimit": "0x05f5e100",
        "currentNumber": "0x01",
        "currentRandom": "0x0000000000000000000000000000000000000000000000000000000000020000",
        "currentTimestamp": "0x03e8"
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Ethereum Transaction Generator")
        .version("1.0")
        .author("PlanD")
        .about("Generates Ethereum transaction test cases")
        .long_about("A tool for generating Ethereum transaction test cases with various patterns and configurations. It can create pattern-based random transactions.")
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("FILE")
            .help("Output JSON file path")
            .global(true)
            .default_value("./data/my_test_case.json"))
        .subcommand(Command::new("pattern")
            .about("Generate transactions based on a pattern")
            .long_about("Generates transactions following a specific pattern. Available patterns are:\n\n\
                         - one-to-many: One sender, multiple receivers\n\
                         \t            o                              \n\
                         \t    ┌---┬---┼---┬---┐                      \n\
                         \t    ↓   ↓   ↓   ↓   ↓                      \n\
                         \t    o   o   o   o   o                      \n\n\
                         - many-to-many: Multiple senders, multiple receivers\n\
                         \t    o   o   o   o   o                      \n\
                         \t    ↓   ↓   ↓   ↓   ↓                      \n\
                         \t    o   o   o   o   o                      \n\n\
                         - many-to-one: Multiple senders, one receiver\n\
                         \t    o   o   o   o   o                      \n\
                         \t    ↓   ↓   ↓   ↓   ↓                      \n\
                         \t    └---┴---┼---┴---┘                      \n\
                         \t            o                              \n\n\
                         - chained: Transactions form a circular pattern\n\
                         \t    o → o → o → o → o                      \n\
                         \t    ↑               ↓                      \n\
                         \t    o ← o ← o ← o ← o                      \n\
                         ")
            .arg(Arg::new("type")
                .short('y')
                .long("type")
                .value_name("TYPE")
                .help("Pattern type (one-to-many, many-to-many, many-to-one, chained)")
                .required(true))
            .arg(Arg::new("num_transactions")
                .short('t')
                .long("num-transactions")
                .value_name("NUM")
                .help("Number of transactions to generate")
                .default_value("20")
                .required(false))
            .arg(Arg::new("num_groups")
                .short('g')
                .long("num-groups")
                .value_name("NUM")
                .help("Number of groups to generate")
                .default_value("4")
                .required(false))
            .arg(Arg::new("conflict_rate")
                .short('c')
                .long("conflict-rate")
                .value_name("RATE")
                .help("Conflict rate (0.0 to 1.0)")
                .default_value("0.5")
                .required(false))
            .arg(Arg::new("erc20")
                .long("erc20")
                .help("Whether to generate ERC20 transactions")
                .action(clap::ArgAction::SetTrue)))
        .after_help("Examples:\n\
                     Generate 50 transactions in a chained pattern:\n\
                     $ ethereum-tx-gen pattern -y chained -t 50\n\n\
                     Specify output file:\n\
                     $ ethereum-tx-gen pattern -y chained -t 50 -o ./my_test_case.json")
        .get_matches();

    let (pre, transactions) = match matches.subcommand() {
        Some(("pattern", sub_m)) => {
            let pattern_type = sub_m.get_one::<String>("type").unwrap();
            let num_transactions = sub_m
                .get_one::<String>("num_transactions")
                .unwrap()
                .parse()?;
            let num_groups = sub_m.get_one::<String>("num_groups").unwrap().parse()?;
            let conflict_rate = sub_m.get_one::<String>("conflict_rate").unwrap().parse()?;
            let is_erc20 = *sub_m.get_one::<bool>("erc20").unwrap_or(&false);

            let mut tx_gen = TransactionGenerator::new();
            tx_gen.generate_pattern_transactions(
                pattern_type,
                num_transactions,
                num_groups,
                conflict_rate,
                is_erc20,
            )?;
            tx_gen.get_data()
        }
        _ => return Err("Invalid subcommand".into()),
    };

    let json_output =
        build_json_output(pre, transactions, get_info(), gen_env(), "just-test".into())?;

    let file_path = matches.get_one::<String>("output").unwrap();
    let mut file = File::create(file_path)?;
    file.write_all(serde_json::to_string_pretty(&json_output)?.as_bytes())?;

    println!("Test cases written to {}", file_path);

    Ok(())
}
