use sp_core::{H160};
use scale_info::prelude::format;
// use serde_json_core::ser;


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
        print(format!("CREATED_ACCOUNT {call_index} {address:x}",
            call_index = 0,
            address = address,
        ).as_ref());
    }
}

pub fn print(input: &str) {
	sp_std::if_std! {
	println!("DMLOG {}", input);
	}
}