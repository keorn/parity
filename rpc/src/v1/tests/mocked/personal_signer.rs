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
use ethcore::client::TestBlockChainClient;
use ethcore::transaction::{Transaction, Action};
use v1::{SignerClient, PersonalSigner};
use v1::tests::helpers::TestMinerService;
use v1::helpers::{SigningQueue, ConfirmationsQueue};
use v1::types::TransactionRequest;


struct PersonalSignerTester {
	queue: Arc<ConfirmationsQueue>,
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

fn signer_tester() -> PersonalSignerTester {
	let queue = Arc::new(ConfirmationsQueue::default());
	let accounts = accounts_provider();
	let client = blockchain_client();
	let miner = miner_service();

	let io = IoHandler::new();
	io.add_delegate(SignerClient::new(&accounts, &client, &miner, &queue).to_delegate());

	PersonalSignerTester {
		queue: queue,
		accounts: accounts,
		io: io,
		miner: miner,
		_client: client,
	}
}


#[test]
fn should_return_list_of_transactions_in_queue() {
	// given
	let tester = signer_tester();
	tester.queue.add_request(TransactionRequest {
		from: Address::from(1),
		to: Some(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		gas_price: Some(U256::from(10_000)),
		gas: Some(U256::from(10_000_000)),
		value: Some(U256::from(1)),
		data: None,
		nonce: None,
	});

	// when
	let request = r#"{"jsonrpc":"2.0","method":"personal_transactionsToConfirm","params":[],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[{"id":"0x01","transaction":{"data":null,"from":"0x0000000000000000000000000000000000000001","gas":"0x989680","gasPrice":"0x2710","nonce":null,"to":"0xd46e8dd67c5d32be8058bb8eb970870f07244567","value":"0x01"}}],"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request(&request), Some(response.to_owned()));
}


#[test]
fn should_reject_transaction_from_queue_without_dispatching() {
	// given
	let tester = signer_tester();
	tester.queue.add_request(TransactionRequest {
		from: Address::from(1),
		to: Some(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		gas_price: Some(U256::from(10_000)),
		gas: Some(U256::from(10_000_000)),
		value: Some(U256::from(1)),
		data: None,
		nonce: None,
	});
	assert_eq!(tester.queue.requests().len(), 1);

	// when
	let request = r#"{"jsonrpc":"2.0","method":"personal_rejectTransaction","params":["0x01"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request(&request), Some(response.to_owned()));
	assert_eq!(tester.queue.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().unwrap().len(), 0);
}

#[test]
fn should_not_remove_transaction_if_password_is_invalid() {
	// given
	let tester = signer_tester();
	tester.queue.add_request(TransactionRequest {
		from: Address::from(1),
		to: Some(Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap()),
		gas_price: Some(U256::from(10_000)),
		gas: Some(U256::from(10_000_000)),
		value: Some(U256::from(1)),
		data: None,
		nonce: None,
	});
	assert_eq!(tester.queue.requests().len(), 1);

	// when
	let request = r#"{"jsonrpc":"2.0","method":"personal_confirmTransaction","params":["0x01",{},"xxx"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":false,"id":1}"#;

	// then
	assert_eq!(tester.io.handle_request(&request), Some(response.to_owned()));
	assert_eq!(tester.queue.requests().len(), 1);
}

#[test]
fn should_confirm_transaction_and_dispatch() {
	//// given
	let tester = signer_tester();
	let address = tester.accounts.new_account("test").unwrap();
	let recipient = Address::from_str("d46e8dd67c5d32be8058bb8eb970870f07244567").unwrap();
	tester.queue.add_request(TransactionRequest {
		from: address,
		to: Some(recipient),
		gas_price: Some(U256::from(10_000)),
		gas: Some(U256::from(10_000_000)),
		value: Some(U256::from(1)),
		data: None,
		nonce: None,
	});

	let t = Transaction {
		nonce: U256::zero(),
		gas_price: U256::from(0x1000),
		gas: U256::from(10_000_000),
		action: Action::Call(recipient),
		value: U256::from(0x1),
		data: vec![]
	};
	tester.accounts.unlock_account_temporarily(address, "test".into()).unwrap();
	let signature = tester.accounts.sign(address, t.hash()).unwrap();
	let t = t.with_signature(signature);

	assert_eq!(tester.queue.requests().len(), 1);

	// when
	let request = r#"{
		"jsonrpc":"2.0",
		"method":"personal_confirmTransaction",
		"params":["0x01", {"gasPrice":"0x1000"}, "test"],
		"id":1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":""#.to_owned() + format!("0x{:?}", t.hash()).as_ref() + r#"","id":1}"#;

	// then
	assert_eq!(tester.io.handle_request(&request), Some(response.to_owned()));
	assert_eq!(tester.queue.requests().len(), 0);
	assert_eq!(tester.miner.imported_transactions.lock().unwrap().len(), 1);
}

