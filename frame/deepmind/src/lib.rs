use ethereum::{PartialHeader, TransactionAction, TransactionV2 as Transaction};
use ethereum_types::{Address, H160, H256, U256};
use fp_ethereum::TransactionData;
use frame_support::dispatch::fmt;
use scale_info::prelude::format;
use sp_std::sync::Arc;
// use serde_json_core::ser;

pub static PLATFORM: &str = "Frontier";
pub static FORK: &str = "vanilla";

#[derive(Debug, PartialEq, Clone)]
pub enum Instrumentation {
	Full,
	BlockProgress,
	None,
}
pub struct Context {
	instrumentation: Instrumentation,
	printer: Arc<Box<dyn Printer>>,
}

pub trait ContextTrait {
    fn new(instrumentation: Instrumentation) -> Context;
    fn noop() -> Context;
    fn is_enabled(&self) -> bool;
    fn is_finalize_block_enabled(&self) -> bool;
    fn init(&self, engine: String);
    fn block_context(&self) -> BlockContextType;
    fn block_tracer(&self) -> BlockTracer;
}

impl ContextTrait for Context {
	fn new(instrumentation: Instrumentation) -> Context {
		Context {
			instrumentation,
			printer: Arc::new(Box::new(IoPrinter {})),
		}
	}
    fn noop() -> Context {
        Context {
            instrumentation: Instrumentation::None,
            printer: Arc::new(Box::new(DiscardPrinter{})),
        }
    }

    fn block_context(&self) -> BlockContextType {
        BlockContextType {
            context: *self,
            is_enabled: self.is_enabled(),
        }
    }

    fn block_tracer(&self) -> BlockTracer {
        BlockTracer{printer: self.printer.clone()}
    }

    fn is_enabled(&self) -> bool {
        return self.instrumentation == Instrumentation::Full;
    }

    fn is_finalize_block_enabled(&self) -> bool {
        return self.instrumentation == Instrumentation::Full || self.instrumentation == Instrumentation::BlockProgress;
    }

	fn init(&self, engine: String) {
		self.printer.print(format!("INIT {protocol_major} {protocol_minor} {platform} {fork} {platform_major} {platform_minor} {platform_patch} {engine}",
			protocol_major = 0,
			protocol_minor = 1,
			platform_major = 5,
			platform_minor = 11,
			platform_patch = 0,
			platform = PLATFORM,
			fork = FORK,
			engine = engine,
		).as_ref())
	}
}

pub struct BlockContextType {
	context: Context,
	is_enabled: bool,
	// is_finalize_block_enabled: bool,
	// cumulative_gas_used: u64,
	// log_index_at_block: u64,
}

trait BlockContextTrait {
    type BlockContext;

    fn is_enabled() -> bool;
	fn end_block( num: U256, size: u64, header: PartialHeader);
    fn start_transaction(trx: Transaction);
}

impl BlockContextTrait for BlockContextType{
    type BlockContext = BlockContextType;

	fn is_enabled() -> bool {
		Self::is_enabled()
	}

	fn end_block(num: U256, size: u64, header: PartialHeader) {
		Self::context.printer.print(
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

	// let tx = TransactionV0 {
	// 	nonce: 12.into(),
	// 	gas_price: 20_000_000_000_u64.into(),
	// 	gas_limit: 21000.into(),
	// 	action: TransactionAction::Call(
	// 		hex!("727fc6a68321b754475c668a6abfb6e9e71c169a").into(),
	// 	),
	// 	value: U256::from(10) * 1_000_000_000 * 1_000_000_000,
	// 	input: hex!("a9059cbb000000000213ed0f886efd100b67c7e4ec0a85a7d20dc971600000000000000000000015af1d78b58c4000").into(),
	// 	signature: TransactionSignature::new(38, hex!("be67e0a07db67da8d446f76add590e54b6e92cb6b8f9835aeb67540579a27717").into(), hex!("2d690516512020171c1ec870f6ff45398cc8609250326be89915fb538e7bd718").into()).unwrap(),
	// };

	// pub struct EIP2930TransactionMessage {
	// 	pub chain_id: u64,
	// 	pub nonce: U256,
	// 	pub gas_price: U256,
	// 	pub gas_limit: U256,
	// 	pub action: TransactionAction,
	// 	pub value: U256,
	// 	pub input: Bytes,
	// 	pub access_list: Vec<AccessListItem>,
	// }

	// pub struct EIP1559TransactionMessage {
	// 	pub chain_id: u64,
	// 	pub nonce: U256,
	// 	pub max_priority_fee_per_gas: U256,
	// 	pub max_fee_per_gas: U256,
	// 	pub gas_limit: U256,
	// 	pub action: TransactionAction,
	// 	pub value: U256,
	// 	pub input: Bytes,
	// 	pub access_list: Vec<AccessListItem>,
	// }

	// pub struct TransactionData {
	// 	pub action: TransactionAction,
	// 	pub input: Vec<u8>,
	// 	pub nonce: U256,
	// 	pub gas_limit: U256,
	// 	pub gas_price: Option<U256>,
	// 	pub max_fee_per_gas: Option<U256>,
	// 	pub max_priority_fee_per_gas: Option<U256>,
	// 	pub value: U256,
	// 	pub chain_id: Option<u64>,
	// 	pub access_list: Vec<(H160, Vec<H256>)>,
	// }

	fn start_transaction( trx: &Transaction) {
		let tx_data: TransactionData = (trx).into();
		Self::BlockContext.context.printer.print(
            format!("BEGIN_TRX {hash:x} {to} {value:x} {v:x} {r:x} {s:x} {gas_limit} {gas_price:x} {nonce} {data:x}",
            hash = 0,
            to = 0,
            value = tx_data.value,
            v = 0,
            r = 0,
            s = 0,
            gas_limit = tx_data.gas_limit,
            gas_price = tx_data.gas_price.unwrap_or_default(),
            nonce = tx_data.nonce,
            data = 0,
        ).as_ref());
	}
}
pub trait Printer: Send + Sync {
	fn print(&self, _input: &str) {}

	fn debug(&self, _input: &str) {}
}

pub struct DiscardPrinter {}

impl Printer for DiscardPrinter {}

pub struct IoPrinter {}

impl Printer for IoPrinter {
	fn print(&self, input: &str) {
		sp_std::if_std! {
		println!("DMLOG {}", input);
		}
	}

	fn debug(&self, input: &str) {
		println!("DMDEBUG {}", input);
	}
}

/// BlockTracer is responsible of recording single tracing elements (like Balance or Gas change)
/// that happens outside of any transactions on a block.
pub struct BlockTracer {
    printer: Arc<Box<dyn Printer>>,
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
