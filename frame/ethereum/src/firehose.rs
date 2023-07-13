use ethereum::{PartialHeader, TransactionV2 as Transaction};
use ethereum_types::{H160, U256};
use fp_ethereum::TransactionData;
use frame_support::dispatch::fmt;
use scale_info::prelude::format;
// use serde_json_core::ser;

pub struct BlockContext;

pub trait BlockTrait {
	// fn new() -> BlockContext;
	// fn is_enabled(&self) -> bool;
	// fn is_finalize_block_enabled(&self) -> bool;
	// fn start_block(num: u64);
	// fn transaction_tracer(&self, hash: eth::H256) -> TransactionTracer;
	fn start_transaction(trx: &Transaction, from: &H160, to: Option<H160>);
	// fn record_log_count(&mut self, count: u64);
	// fn get_cumulative_gas_used(&mut self) -> u64;
	// fn set_cumulative_gas_used(&mut self, gas_used: u64);
	// fn end_transaction(&mut self, receipt: TransactionReceipt);
	fn finalize_block(num: u64);
	// fn end_block(&self, num: u64, size: u64, header:  Header, uncles: Vec<Header>);
	fn end_block(num: U256, size: u64, header: PartialHeader);
}

impl BlockTrait for BlockContext {
	fn end_block(num: U256, size: u64, header: PartialHeader) {
		print(
			format!(
			"END_BLOCK {num} {size} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", 
			&header.parent_hash,
			&header.beneficiary,
			&header.state_root,
			&header.receipts_root,
			&header.logs_bloom,
			&header.difficulty,
			&header.number,
			&header.gas_limit,
			&header.gas_used,
			&header.timestamp,
			&header.difficulty,
			&header.extra_data,
			&header.mix_hash,
			&header.nonce,
		)
			.as_ref(),
		);
	}

	fn start_transaction(trx: &Transaction, from: &H160, to: Option<H160>) {
		let tx_data: TransactionData = (trx).into();
		print(format!("BEGIN_APPLY_TRX {hash:x} {to} {value:x} {v:x} {r:x} {s:x} {gas_limit} {gas_price:x} {nonce} {data:x}",
		hash = trx.hash(),
		to = to.unwrap_or(H160::zero()),
		value = tx_data.value,
		v = 0,
		r = 0,
		s = 0,
		gas_limit = tx_data.gas_limit,
		gas_price = tx_data.gas_price.unwrap_or_default(),
		nonce = tx_data.nonce,
		data = 0,
	).as_ref());
		print(format!("TRX_FROM {from:x}", from = from).as_ref());
	}

	fn finalize_block(num: u64) {
		print(format!("FINALIZE_BLOCK {num}", num = num).as_ref())
	}

}

pub fn print(input: &str) {
	sp_std::if_std! {
	println!("DMLOG {}", input);
	}
}

struct UU256<'a>(&'a U256);

impl fmt::LowerHex for UU256<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0.is_zero() {
			true => f.write_str("."),
			_ => fmt::LowerHex::fmt(self.0, f),
		}
	}
}
