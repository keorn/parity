// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use std::str::FromStr;
use jsonrpc_core::IoHandler;
use util::numbers::*;
use ethcore::account_provider::AccountProvider;
use v1::{PersonalClient, Personal};
use v1::tests::helpers::TestMinerService;
use ethcore::client::TestBlockChainClient;
use ethcore::transaction::{Action, Transaction};

struct PersonalTester {
	accounts: Arc<AccountProvider>,
	io: IoHandler,
	miner: Arc<TestMinerService>,
	// these unused fields are necessary to keep the data alive
	// as the handler has only weak pointers.
	_client: Arc<TestBlockChainClient>,
}

fn blockchain_client() -> Arc<TestBlockChainClient> {
	let client = TestBlockChainClient::new();
	Arc::new(client)
}

fn accounts_provider() -> Arc<AccountProvider> {
	Arc::new(AccountProvider::transient_provider())
}

fn miner_service() -> Arc<TestMinerService> {
	Arc::new(TestMinerService::default())
}

fn setup(signer: Option<u16>) -> PersonalTester {
	let accounts = accounts_provider();
	let client = blockchain_client();
	let miner = miner_service();
	let personal = PersonalClient::new(&accounts, &client, &miner, signer);

	let io = IoHandler::new();
	io.add_delegate(personal.to_delegate());

	let tester = PersonalTester {
		accounts: accounts,
		io: io,
		miner: miner,
		_client: client,
	};

	tester
}

#[test]
fn should_return_false_if_signer_is_disabled() {
	// given
	let tester = setup(None);

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "personal_signerEnabled", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":false,"id":1}"#;


	// then
	assert_eq!(tester.io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn should_return_port_number_if_signer_is_enabled() {
	// given
	let tester = setup(Some(8180));

	// when
	let request = r#"{"jsonrpc": "2.0", "method": "personal_signerEnabled", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":8180,"id":1}"#;


	// then
	assert_eq!(tester.io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn accounts() {
	let tester = setup(None);
	let address = tester.accounts.new_account("").unwrap();
	let request = r#"{"jsonrpc": "2.0", "method": "personal_listAccounts", "params": [], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[""#.to_owned() + &format!("0x{:?}", address) + r#""],"id":1}"#;

	assert_eq!(tester.io.handle_request(request), Some(response.to_owned()));
}

#[test]
fn new_account() {
	let tester = setup(None);
	let request = r#"{"jsonrpc": "2.0", "method": "personal_newAccount", "params": ["pass"], "id": 1}"#;

	let res = tester.io.handle_request(request);

	let accounts = tester.accounts.accounts();
	assert_eq!(accounts.len(), 1);
	let address = accounts[0];
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"","id":1}"#;

	assert_eq!(res, Some(response));
}

#[test]
fn sign_and_send_transaction_with_invalid_password() {
	let tester = setup(None);
	let address = tester.accounts.new_account("password123").unwrap();
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_signAndSendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}, "password321"],
		"id": 1
	}"#;

	let response = r#"{"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000","id":1}"#;

	assert_eq!(tester.io.handle_request(request.as_ref()), Some(response.into()));
}

#[test]
fn sign_and_send_transaction() {
	let tester = setup(None);
	let address = tester.accounts.new_account("password123").unwrap();

	let request = r#"{
		"jsonrpc": "2.0",
		"method": "personal_signAndSendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}, "password123"],
		"id": 1
	}"#;

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	tester.accounts.unlock_account_temporarily(address, "password123".into()).unwrap();
	let signature = tester.accounts.sign(address, t.hash()).unwrap();
	let t = t.with_signature(signature);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request(request.as_ref()), Some(response));

	tester.miner.last_nonces.write().unwrap().insert(address.clone(), U256::zero());

	let t = Transaction {
		nonce: U256::one(),
		gas_price: U256::from(0x9184e72a000u64),
		gas: U256::from(0x76c0),
		action: Action::Call(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		value: U256::from(0x9184e72au64),
		data: vec![]
	};
	tester.accounts.unlock_account_temporarily(address, "password123".into()).unwrap();
	let signature = tester.accounts.sign(address, t.hash()).unwrap();
	let t = t.with_signature(signature);

	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":1}"#;

	assert_eq!(tester.io.handle_request(request.as_ref()), Some(response));
}
