use crate::constants::ERC20_USDC_DEPLOYED_BYTECODE;
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::{rand as ethers_rand, LocalWallet, Signer},
    signers::Wallet,
    types::Address,
    utils::{hex, keccak256},
};
use rand::Rng;
use serde_json::{json, Map, Value};
use std::io::{Error, ErrorKind};

const DEFAULT_BALANCE_HEX: &str = "0x056bc75e2d63100000";
const DEFAULT_BALANCE: u128 = 100 * ONE_ETHER;
const ONE_ETHER: u128 = 1_000_000_000_000_000_000;

trait ToHex {
    fn to_hex(&self) -> String;
}

impl ToHex for u128 {
    fn to_hex(&self) -> String {
        let pure_hex = format!("{:x}", self);
        if pure_hex.len() % 2 == 0 {
            format!("0x{}", pure_hex)
        } else {
            format!("0x0{}", pure_hex)
        }
    }
}

impl ToHex for Address {
    fn to_hex(&self) -> String {
        format!("{:#x}", self)
    }
}

impl ToHex for Wallet<SigningKey> {
    fn to_hex(&self) -> String {
        format!("0x{:x}", self.signer().to_bytes())
    }
}

fn random_near(num: u64, rng: &mut rand::rngs::ThreadRng) -> u64 {
    let random_small = ((rng.random_range(0..num) * num) as f64).sqrt();
    num - 1 - random_small as u64
}

fn get_storage_slot_balance(address: Address) -> String {
    let slot_part_0 =
        "000000000000000000000000".to_owned() + address.to_hex().strip_prefix("0x").unwrap();
    let slot_part_1 = "0000000000000000000000000000000000000000000000000000000000000000";
    let slot = keccak256(hex::decode(slot_part_0 + slot_part_1).unwrap());
    "0x".to_owned() + &hex::encode(slot)
}

fn add_erc20_balance_prestate(pre: &mut Map<String, Value>, erc20: Address, account: Address) {
    pre.get_mut(&erc20.to_hex())
        .unwrap()
        .get_mut("storage")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert(
            get_storage_slot_balance(account),
            Value::from(DEFAULT_BALANCE_HEX),
        );
}

fn erc20_transfer(to: Address, value: u128) -> String {
    let to_string = to.to_hex();
    let to_without_hex = to_string.strip_prefix("0x").unwrap();
    format!(
        "0xa9059cbb000000000000000000000000{}{:064x}",
        to_without_hex, value
    )
}

pub struct TransactionGenerator {
    ethers_rng: ethers_rand::rngs::ThreadRng,
    rng: rand::rngs::ThreadRng,
    pre: Map<String, Value>,
    transactions: Vec<Value>,
}

impl TransactionGenerator {
    pub fn new() -> Self {
        Self {
            ethers_rng: ethers_rand::thread_rng(),
            rng: rand::rng(),
            pre: Map::new(),
            transactions: Vec::new(),
        }
    }

    fn _check_tx_num(&self, num_transactions: u128, num_groups: u128) -> Result<(), Error> {
        if num_transactions < num_groups {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "number of transactions must be greater than number of groups",
            ))
        } else {
            Ok(())
        }
    }

    fn _check_m2m_params(&self, num_transactions: u128, conflict_rate: f64) -> Result<(), Error> {
        if num_transactions <= 4 {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "number of transactions must be greater than 4",
            ))
        } else if conflict_rate < 0.0 || conflict_rate > 1.0 {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "conflict rate must be between 0.0 and 1.0",
            ))
        } else {
            Ok(())
        }
    }

    fn deploy_erc20(&mut self) -> Result<Address, Error> {
        let bytecode = ERC20_USDC_DEPLOYED_BYTECODE.to_string();
        let contract_address = Address::from_slice(&[
            179, 13, 249, 43, 177, 7, 230, 241, 228, 111, 125, 244, 253, 49, 163, 22, 206, 180,
            231, 217,
        ]);
        self.pre.insert(
            contract_address.to_hex(),
            json!({
                "balance": "0x00",
                "code": bytecode,
                "nonce": "0x00",
                "storage": {}
            }),
        );
        Ok(contract_address)
    }

    fn generate_pattern_o2m(
        &mut self,
        num_transactions: u128,
        num_groups: u128,
        is_erc20: bool,
    ) -> Result<(), Error> {
        self._check_tx_num(num_transactions, num_groups)?;
        let erc20_address = if is_erc20 {
            Some(self.deploy_erc20()?)
        } else {
            None
        };

        for group_idx in 0..num_groups {
            let padding = group_idx < num_transactions % num_groups;
            let tx_num = num_transactions / num_groups + padding as u128;
            let sender = LocalWallet::new(&mut self.ethers_rng);
            let value_per_tx = 50 * ONE_ETHER / tx_num;
            let mut nonce = 0;

            self.pre.insert(
                sender.address().to_hex(),
                json!({
                    "balance": DEFAULT_BALANCE_HEX,
                    "code": "0x",
                    "nonce": nonce.to_hex(),
                    "storage": {}
                }),
            );

            if is_erc20 {
                add_erc20_balance_prestate(&mut self.pre, erc20_address.unwrap(), sender.address());
            }

            for _ in 0..tx_num {
                let receiver = LocalWallet::new(&mut self.ethers_rng);
                self.transactions.push(json!({
                    "data": if is_erc20 { erc20_transfer(receiver.address(), value_per_tx) } else { "0x".to_string() }, 
                    "gasLimit": "0x0f4240",
                    "gasPrice": "0x0a",
                    "nonce": nonce.to_hex(),
                    "secretKey": sender.to_hex(),
                    "sender": sender.address().to_hex(),
                    "to": if is_erc20 { erc20_address.unwrap().to_hex() } else { receiver.address().to_hex() },
                    "value": if is_erc20 { "0x00".to_string() } else { value_per_tx.to_hex() }
                }));
                nonce += 1;
            }
        }

        Ok(())
    }

    fn generate_pattern_m2o(
        &mut self,
        num_transactions: u128,
        num_groups: u128,
        is_erc20: bool,
    ) -> Result<(), Error> {
        self._check_tx_num(num_transactions, num_groups)?;
        let erc20_address = if is_erc20 {
            Some(self.deploy_erc20()?)
        } else {
            None
        };

        for group_idx in 0..num_groups {
            let padding = group_idx < num_transactions % num_groups;
            let tx_num = num_transactions / num_groups + padding as u128;

            let receiver = LocalWallet::new(&mut self.ethers_rng);

            for _ in 0..tx_num {
                let sender = LocalWallet::new(&mut self.ethers_rng);
                self.pre.insert(
                    sender.address().to_hex(),
                    json!({
                        "balance": DEFAULT_BALANCE_HEX,
                        "code": "0x",
                        "nonce": "0x00",
                        "storage": {}
                    }),
                );
                if is_erc20 {
                    add_erc20_balance_prestate(
                        &mut self.pre,
                        erc20_address.unwrap(),
                        sender.address(),
                    );
                    self.transactions.push(json!({
                        "data": erc20_transfer(receiver.address(), 50 * ONE_ETHER),
                        "gasLimit": "0x0f4240",
                        "gasPrice": "0x0a",
                        "nonce": "0x00",
                        "secretKey": sender.to_hex(),
                        "sender": sender.address().to_hex(),
                        "to": erc20_address.unwrap().to_hex(),
                        "value": "0x00",
                    }));
                } else {
                    self.transactions.push(json!({
                        "data": "0x",
                        "gasLimit": "0x0f4240",
                        "gasPrice": "0x0a",
                        "nonce": "0x00",
                        "secretKey": sender.to_hex(),
                        "sender": sender.address().to_hex(),
                        "to": receiver.address().to_hex(),
                        "value": (50 * ONE_ETHER).to_hex(),
                    }));
                }
            }
        }

        Ok(())
    }

    fn generate_pattern_chained(
        &mut self,
        num_transactions: u128,
        num_groups: u128,
        is_erc20: bool,
    ) -> Result<(), Error> {
        self._check_tx_num(num_transactions, num_groups)?;
        let erc20_address = if is_erc20 {
            Some(self.deploy_erc20()?)
        } else {
            None
        };

        for group_idx in 0..num_groups {
            let padding = group_idx < num_transactions % num_groups;
            let tx_num = num_transactions / num_groups + padding as u128;

            let mut sender;
            let mut receiver = LocalWallet::new(&mut self.ethers_rng);

            self.pre.insert(
                receiver.address().to_hex(),
                json!({
                    "balance": DEFAULT_BALANCE_HEX,
                    "code": "0x",
                    "nonce": "0x00",
                    "storage": {}
                }),
            );
            let mut value = 50 * ONE_ETHER;

            if is_erc20 {
                add_erc20_balance_prestate(
                    &mut self.pre,
                    erc20_address.unwrap(),
                    receiver.address(),
                );
            }

            for _ in 0..tx_num {
                sender = receiver;
                receiver = LocalWallet::new(&mut self.ethers_rng);

                if is_erc20 {
                    self.pre.insert(
                        receiver.address().to_hex(),
                        json!({
                            "balance": DEFAULT_BALANCE_HEX,
                            "code": "0x",
                            "nonce": "0x00",
                            "storage": {}
                        }),
                    );
                    self.transactions.push(json!({
                        "data": erc20_transfer(receiver.address(), value),
                        "gasLimit": "0x0f4240",
                        "gasPrice": "0x0a",
                        "nonce": "0x00",
                        "secretKey": sender.to_hex(),
                        "sender": sender.address().to_hex(),
                        "to": erc20_address.unwrap().to_hex(),
                        "value": "0x00",
                    }));
                } else {
                    self.transactions.push(json!({
                        "data": "0x",
                        "gasLimit": "0x0f4240",
                        "gasPrice": "0x0a",
                        "nonce": "0x00",
                        "secretKey": sender.to_hex(),
                        "sender": sender.address().to_hex(),
                        "to": receiver.address().to_hex(),
                        "value": value.to_hex(),
                    }));
                    value -= ONE_ETHER / 100_000;
                }
            }
        }

        Ok(())
    }

    fn generate_pattern_m2m(
        &mut self,
        num_transactions: u128,
        conflict_rate: f64,
        is_erc20: bool,
    ) -> Result<(), Error> {
        self._check_m2m_params(num_transactions, conflict_rate)?;
        let erc20_address = if is_erc20 {
            Some(self.deploy_erc20()?)
        } else {
            None
        };

        let mut senders_idxs = vec![0, 1, 2];
        let mut receivers_idxs = vec![0, 1, 2];
        let mut senders_num = 3;
        let mut receivers_num = 3;

        for _ in 3..num_transactions {
            if self.rng.random::<f64>() < conflict_rate {
                let random_pattern = self.rng.random::<f64>();
                let selected_sender = random_near(senders_num, &mut self.rng);
                let selected_receiver = random_near(receivers_num, &mut self.rng);
                if random_pattern < 0.33 {
                    senders_idxs.push(selected_sender);
                    receivers_idxs.push(receivers_num);
                    receivers_num += 1;
                } else if random_pattern < 0.66 {
                    senders_idxs.push(senders_num);
                    receivers_idxs.push(selected_receiver);
                    senders_num += 1;
                } else {
                    senders_idxs.push(selected_sender);
                    receivers_idxs.push(selected_receiver);
                }
            } else {
                senders_idxs.push(senders_num);
                receivers_idxs.push(receivers_num);
                senders_num += 1;
                receivers_num += 1;
            }
        }

        let mut senders_wallets = Vec::new();
        let mut receivers_wallets = Vec::new();
        let mut senders_nonce = vec![0; senders_num as usize];
        let value_hex = (DEFAULT_BALANCE / (num_transactions + 1)).to_hex();

        for _ in 0..senders_num {
            let sender = LocalWallet::new(&mut self.ethers_rng);
            self.pre.insert(
                sender.address().to_hex(),
                json!({
                    "balance": DEFAULT_BALANCE_HEX,
                    "code": "0x",
                    "nonce": "0x00",
                    "storage": {}
                }),
            );
            if is_erc20 {
                add_erc20_balance_prestate(&mut self.pre, erc20_address.unwrap(), sender.address());
            }
            senders_wallets.push(sender);
        }
        for _ in 0..receivers_num {
            receivers_wallets.push(LocalWallet::new(&mut self.ethers_rng));
        }

        for i in 0..num_transactions {
            let sender_idx = senders_idxs[i as usize];
            let receiver_idx = receivers_idxs[i as usize];
            if is_erc20 {
                self.transactions.push(json!({
                    "data": erc20_transfer(receivers_wallets[receiver_idx as usize].address(), 1 * ONE_ETHER),
                    "gasLimit": "0x0f4240",
                    "gasPrice": "0x0a",
                    "nonce": senders_nonce[sender_idx as usize].to_hex(),
                    "secretKey": senders_wallets[sender_idx as usize].to_hex(),
                    "sender": senders_wallets[sender_idx as usize].address().to_hex(),
                    "to": erc20_address.unwrap().to_hex(),
                    "value": "0x00",
                }));
            } else {
                self.transactions.push(json!({
                    "data": "0x",
                    "gasLimit": "0x0f4240",
                    "gasPrice": "0x0a",
                    "nonce": senders_nonce[sender_idx as usize].to_hex(),
                    "secretKey": senders_wallets[sender_idx as usize].to_hex(),
                    "sender": senders_wallets[sender_idx as usize].address().to_hex(),
                    "to": receivers_wallets[receiver_idx as usize].address().to_hex(),
                    "value": value_hex,
                }));
            }
            senders_nonce[sender_idx as usize] += 1;
        }

        Ok(())
    }

    pub fn generate_pattern_transactions(
        &mut self,
        pattern_type: &str,
        num_transactions: u128,
        num_groups: u128,
        conflict_rate: f64,
        is_erc20: bool,
    ) -> Result<(), Error> {
        match pattern_type {
            "many-to-many" | "m2m" => {
                self.generate_pattern_m2m(num_transactions, conflict_rate, is_erc20)
            }
            "ring" | "chained" | "chain" => {
                self.generate_pattern_chained(num_transactions, num_groups, is_erc20)
            }
            "one-to-many" | "o2m" => {
                self.generate_pattern_o2m(num_transactions, num_groups, is_erc20)
            }
            "many-to-one" | "m2o" => {
                self.generate_pattern_m2o(num_transactions, num_groups, is_erc20)
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                "Invalid pattern type. Available patterns are: 'many-to-many' (or 'm2m'), \
                'chained' (or 'ring', 'chain'), 'one-to-many' (or 'o2m'), 'many-to-one' (or 'm2o')."
            ),
            )),
        }
    }

    pub fn get_data(&self) -> (Map<String, Value>, Vec<Value>) {
        (self.pre.clone(), self.transactions.clone())
    }
}
