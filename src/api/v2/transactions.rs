use crate::network::rpc;

use super::types::{SubmitResponse, Transaction};
use anyhow::{Context, Result};
use async_trait::async_trait;
use avail_subxt::{api, primitives::AvailExtrinsicParams, AvailConfig};
use sp_core::sr25519::Pair;
use subxt::tx::PairSigner;

#[async_trait]
pub trait Submit {
	async fn submit(&self, transaction: Transaction) -> Result<SubmitResponse>;
}

#[derive(Clone)]
pub struct Submitter {
	pub node_client: rpc::Client,
	pub app_id: u32,
	pub pair_signer: PairSigner<AvailConfig, Pair>,
}

#[async_trait]
impl Submit for Submitter {
	async fn submit(&self, transaction: Transaction) -> Result<SubmitResponse> {
		let tx_progress = match transaction {
			Transaction::Data(data) => {
				let extrinsic = api::tx().data_availability().submit_data(data.into());
				let params = AvailExtrinsicParams::new_with_app_id(self.app_id.into());
				self.node_client
					.submit_signed_and_watch(extrinsic, self.pair_signer.clone(), params)
					.await?
			},
			Transaction::Extrinsic(extrinsic) => {
				self.node_client
					.submit_from_bytes_and_watch(extrinsic.into())
					.await?
			},
		};
		tx_progress
			.wait_for_finalized_success()
			.await
			.map(|event| SubmitResponse {
				block_hash: event.block_hash(),
				hash: event.extrinsic_hash(),
				index: event.extrinsic_index(),
			})
			.context("Cannot sign and submit transaction")
	}
}
