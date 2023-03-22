use rand::seq::SliceRandom;
use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
struct BeaconResponse<T> {
    data: T,
}

#[derive(Deserialize)]
struct Validator {
    index: String,
}

#[derive(Deserialize)]
pub struct SyncStatus {
    pub is_syncing: bool,
}

#[derive(Deserialize)]
pub struct ExecutionPayload {
    block_hash: String,
}

#[derive(Deserialize)]
pub struct BlockBody {
    execution_payload: ExecutionPayload,
}

#[derive(Deserialize)]
pub struct BlockMessage {
    body: BlockBody,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct BlockResponse {
    message: BlockMessage,
}

#[derive(Clone)]
pub struct BeaconApi {
    nodes: Vec<Url>,
    client: reqwest::Client,
}

impl BeaconApi {
    pub fn new(nodes: &Vec<Url>) -> Self {
        if nodes.len() > 0 {
            Self {
                nodes: nodes.clone(),
                client: reqwest::Client::new(),
            }
        } else {
            panic!("tried to instantiate BeaconAPI without at least one url");
        }
    }

    // poor mans load balancer, get random node from list
    fn get_node(&self) -> &Url {
        self.nodes.choose(&mut rand::thread_rng()).unwrap()
    }

    pub async fn get_validator_index(&self, pubkey: &String) -> reqwest::Result<String> {
        let url = format!(
            "{}eth/v1/beacon/states/head/validators/{}",
            self.get_node(),
            pubkey
        );
        self.client
            .get(url)
            .send()
            .await?
            .json::<BeaconResponse<Validator>>()
            .await
            .map(|body| body.data.index)
    }

    pub async fn get_block_hash(&self, slot: &i64) -> reqwest::Result<String> {
        let url = format!("{}eth/v2/beacon/blocks/{}", self.get_node(), slot);
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<BeaconResponse<BlockResponse>>()
            .await
            .map(|body| body.data.message.body.execution_payload.block_hash)
    }

    pub async fn get_sync_status(&self, node_url: &Url) -> reqwest::Result<SyncStatus> {
        let url = format!("{}eth/v1/node/syncing", node_url);
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<BeaconResponse<SyncStatus>>()
            .await
            .map(|body| body.data)
    }
}
