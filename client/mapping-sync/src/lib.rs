// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020-2022 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![allow(clippy::too_many_arguments)]

mod worker;

pub use worker::{MappingSyncWorker, SyncStrategy};

// Substrate
use sc_client_api::BlockOf;
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT, Zero},
};
// Frontier
use fp_consensus::FindLogError;
use fp_rpc::EthereumRuntimeRPCApi;

pub fn sync_block<Block: BlockT>(
	backend: &fc_db::Backend<Block>,
	header: &Block::Header,
) -> Result<(), String> {
	match fp_consensus::find_log(header.digest()) {
		Ok(log) => {
			let post_hashes = log.into_hashes();

			let mapping_commitment = fc_db::MappingCommitment {
				block_hash: header.hash(),
				ethereum_block_hash: post_hashes.block_hash,
				ethereum_transaction_hashes: post_hashes.transaction_hashes,
			};
			backend.mapping().write_hashes(mapping_commitment)?;

			Ok(())
		}
		Err(FindLogError::NotFound) => {
			backend.mapping().write_none(header.hash())?;

			Ok(())
		}
		Err(FindLogError::MultipleLogs) => Err("Multiple logs found".to_string()),
	}
}

pub fn sync_genesis_block<Block: BlockT, C>(
	client: &C,
	backend: &fc_db::Backend<Block>,
	header: &Block::Header,
) -> Result<(), String>
where
	C: ProvideRuntimeApi<Block> + Send + Sync + HeaderBackend<Block> + BlockOf,
	C::Api: EthereumRuntimeRPCApi<Block>,
{
	log::debug!(target: "mapping-sync-sg", "Sync genesis started");

	let id = BlockId::Hash(header.hash());

	if let Some(api_version) = client
		.runtime_api()
		.api_version::<dyn EthereumRuntimeRPCApi<Block>>(&id)
		.map_err(|e| format!("{:?}", e))?
	{
		log::debug!(target: "mapping-sync-sg", "Sync genesis passed 1, API_VERSION {:?}", api_version);
		let block = client
			.runtime_api()
			.current_block(&id)
			.map_err(|e| format!("{:?}", e))?;
		log::debug!(target: "mapping-sync-sg", "Sync genesis passed 2");
		let block_hash = block
			.ok_or_else(|| "Ethereum genesis block not found".to_string())?
			.header
			.hash();
		log::debug!(target: "mapping-sync-sg", "Sync genesis passed 3");
		let mapping_commitment = fc_db::MappingCommitment::<Block> {
			block_hash: header.hash(),
			ethereum_block_hash: block_hash,
			ethereum_transaction_hashes: Vec::new(),
		};
		log::debug!(target: "mapping-sync-sg", "Sync genesis passed 4");
		backend.mapping().write_hashes(mapping_commitment)?;
		log::debug!(target: "mapping-sync-sg", "Sync genesis passed 5");
	} else {
		log::debug!(target: "mapping-sync-sg", "Sync genesis passed 1");
		backend.mapping().write_none(header.hash())?;
		log::debug!(target: "mapping-sync-sg", "Sync genesis passed 2");
	};

	Ok(())
}

pub fn sync_one_block<Block: BlockT, C, B>(
	client: &C,
	substrate_backend: &B,
	frontier_backend: &fc_db::Backend<Block>,
	sync_from: <Block::Header as HeaderT>::Number,
	strategy: SyncStrategy,
) -> Result<bool, String>
where
	C: ProvideRuntimeApi<Block> + Send + Sync + HeaderBackend<Block> + BlockOf,
	C::Api: EthereumRuntimeRPCApi<Block>,
	B: sp_blockchain::HeaderBackend<Block> + sp_blockchain::Backend<Block>,
{
	let mut current_syncing_tips = frontier_backend.meta().current_syncing_tips()?;

	if current_syncing_tips.is_empty() {
		let mut leaves = substrate_backend.leaves().map_err(|e| format!("{:?}", e))?;
		if leaves.is_empty() {
			return Ok(false);
		}
		current_syncing_tips.append(&mut leaves);
	}

	let mut operating_header = None;
	while let Some(checking_tip) = current_syncing_tips.pop() {
		log::debug!(target: "mapping-sync-ct", "Checking tip {:?}", checking_tip);

		if let Some(checking_header) =
			fetch_header(substrate_backend, frontier_backend, checking_tip, sync_from)?
		{
			log::debug!(target: "mapping-sync-ct", "fetch_header Some {:?} - {:?}", checking_tip, &checking_header.number());
			operating_header = Some(checking_header);
			break;
		} else {
			log::debug!(target: "mapping-sync-ct", "fetch_header None {:?}", checking_tip);
		}
	}
	let operating_header = match operating_header {
		Some(operating_header) => operating_header,
		None => {
			frontier_backend
				.meta()
				.write_current_syncing_tips(current_syncing_tips)?;
			return Ok(false);
		}
	};

	log::debug!(target: "mapping-sync-oh", "Operationing header {:?}", operating_header.number());

	if operating_header.number() == &Zero::zero() {
		if let Err(e) = sync_genesis_block(client, frontier_backend, &operating_header) {
			log::debug!(target: "mapping-sync-sg", "Error {:?}", e);
			return Err(e)
		};

		frontier_backend
			.meta()
			.write_current_syncing_tips(current_syncing_tips)?;
		Ok(true)
	} else {
		if SyncStrategy::Parachain == strategy
			&& operating_header.number() > &client.info().best_number
		{
			return Ok(false);
		}
		sync_block(frontier_backend, &operating_header)?;

		current_syncing_tips.push(*operating_header.parent_hash());
		frontier_backend
			.meta()
			.write_current_syncing_tips(current_syncing_tips)?;
		Ok(true)
	}
}

pub fn sync_blocks<Block: BlockT, C, B>(
	client: &C,
	substrate_backend: &B,
	frontier_backend: &fc_db::Backend<Block>,
	limit: usize,
	sync_from: <Block::Header as HeaderT>::Number,
	strategy: SyncStrategy,
) -> Result<bool, String>
where
	C: ProvideRuntimeApi<Block> + Send + Sync + HeaderBackend<Block> + BlockOf,
	C::Api: EthereumRuntimeRPCApi<Block>,
	B: sp_blockchain::HeaderBackend<Block> + sp_blockchain::Backend<Block>,
{
	let mut synced_any = false;

	for _ in 0..limit {
		synced_any = synced_any
			|| sync_one_block(
				client,
				substrate_backend,
				frontier_backend,
				sync_from,
				strategy,
			)?;
	}

	Ok(synced_any)
}

pub fn fetch_header<Block: BlockT, B>(
	substrate_backend: &B,
	frontier_backend: &fc_db::Backend<Block>,
	checking_tip: Block::Hash,
	sync_from: <Block::Header as HeaderT>::Number,
) -> Result<Option<Block::Header>, String>
where
	B: sp_blockchain::HeaderBackend<Block> + sp_blockchain::Backend<Block>,
{
	if frontier_backend.mapping().is_synced(&checking_tip)? {
		log::debug!(target: "mapping-sync-fh", "fetch_header is_synced false {:?}", checking_tip);
		return Ok(None);
	}

	match substrate_backend.header(BlockId::Hash(checking_tip.clone())) {
		Ok(Some(checking_header)) if checking_header.number() >= &sync_from => {
			Ok(Some(checking_header))
		}
		Ok(Some(_)) => {
			log::debug!(target: "mapping-sync-fh", "here {:?}", checking_tip);
			Ok(None)
		},
		Ok(None) | Err(_) => Err("Header not found".to_string()),
	}
}
