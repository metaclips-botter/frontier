#![cfg_attr(not(feature = "std"), no_std)]

use ethereum::{PartialHeader, TransactionV2 as Transaction};
use ethereum_types::{H160, U256};
use fp_ethereum::TransactionData;
use frame_support::dispatch::fmt;
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::{prelude::format, TypeInfo};
use sp_runtime::RuntimeDebug;
// use serde_json_core::ser;
pub use pallet::*;
use sp_std::prelude::*;

#[derive(
	Copy,
	Clone,
	Encode,
	Decode,
	Eq,
	PartialEq,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo
)]
pub struct BlockContext {
	context: u64, // Context,
	is_enabled: bool,
	is_finalize_block_enabled: bool,
	cumulative_gas_used: u64,
	log_index_at_block: u64,
}

impl Default for BlockContext {
	fn default() -> Self {
		Self {
			context: 0,
			is_enabled: false,
			is_finalize_block_enabled: false,
			cumulative_gas_used: 0,
			log_index_at_block: 0,
		}
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	/// The lookup table for names.
	#[pallet::storage]
	#[pallet::getter(fn block_context)]
	pub(super) type BlockContextStorage<T: Config> = StorageValue<_, BlockContext, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(1000)]
		pub fn enable(origin: OriginFor<T>, enable: bool) -> DispatchResult {
			ensure_signed(origin)?;

			// set is_enabled in the BlockContext
			BlockContextStorage::<T>::mutate(|bc| bc.is_enabled = enable);

			Ok(())
		}
	}
}
pub trait Tracer: Send {
	// fn is_enabled() -> bool { false }

	// fn start_call(_call: Call) {}
	// fn reverted_call(_gas_left: &eth::U256) {}
	// fn failed_call(_gas_left_after_failure: &eth::U256, _err: String) {}
	// fn end_call(_gas_left: &eth::U256, _return_data: Option<&[u8]>) {}
	// fn seen_failed_call() -> bool { return false }
	// fn end_failed_call(_from: &str) {}

	// fn record_balance_change(_address: &eth::Address, _old: &eth::U256, _new: &eth::U256, _reason: BalanceChangeReason) {}
	// fn record_nonce_change(_address: &eth::Address, _old: &eth::U256, _new: &eth::U256) {}
	// fn record_keccak(_hash_of_data: &eth::H256, _data: &[u8]) {}
	fn record_new_account(_addr: &H160) {}
	// fn record_suicide(_addr: &eth::Address, _already_suicided: bool, _balance_before_suicide: &eth::U256) {}
	// fn record_storage_change(_addr: &eth::Address, _key: &eth::H256, _old_data: &eth::H256, _new_data: &eth::H256) {}
	// fn record_log(_log: Log) {}
	// fn record_call_without_code(&mut self) {}

	// fn record_gas_refund(_gas_old: usize, _gas_refund: usize) {}
	// fn record_gas_consume(_gas_old: usize, _gas_consumed: usize, _reason: GasChangeReason) {}
	// fn record_code_change(_addr: &eth::Address, _input_hash: &eth::H256, _code_hash: &eth::H256, _old_code: &[u8], _new_code: &[u8]) {}
	// fn record_before_call_gas_event(_gas_value: usize) {}
	// fn record_after_call_gas_event(_gas_value: usize) {}

	// Returns the number of Ethereum Log that was performed as part of this tracer
	// fn get_log_count() -> u64 { return 0 }

	// Use this to add printing statement useful for debugging, the message is printed with the current
	// tracer context like active call index and other tracer state information.
	// fn debug(_message: String) {}
}

/// BlockTracer is responsible of recording single tracing elements (like Balance or Gas change)
/// that happens outside of any transactions on a block.
pub struct BlockTracer;

impl Tracer for BlockTracer {
	fn record_new_account(address: &H160) {
		print(
			format!(
				"CREATED_ACCOUNT {call_index} {address:x}",
				call_index = 0,
				address = address,
			)
			.as_ref(),
		);
	}
}

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
