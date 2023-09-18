use namada::ledger::queries::{Client as ClientTrait, EncodedResponseQuery, Error};
use namada::tendermint::abci::Code;
use namada::types::storage::BlockHeight;
use namada::{tendermint, tendermint_rpc as tm_rpc, tendermint_rpc::Response};
use reqwest::{Client, Response as ClientResponse};

#[derive(Clone)]
pub struct SdkClient {
    url: String,
    client: Client,
}

impl SdkClient {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }

    pub async fn post(&self, body: String) -> Result<ClientResponse, reqwest::Error> {
        self.client
            .post(format!("http://{}", &self.url))
            .body(body)
            .send()
            .await
    }
}

#[async_trait::async_trait]
impl ClientTrait for SdkClient {
    type Error = Error;

    async fn request(
        &self,
        path: String,
        data: Option<Vec<u8>>,
        height: Option<BlockHeight>,
        prove: bool,
    ) -> Result<EncodedResponseQuery, Self::Error> {
        let data = data.unwrap_or_default();
        let height = height
            .map(|height| {
                tendermint::block::Height::try_from(height.0)
                    .map_err(|_err| Error::InvalidHeight(height))
            })
            .transpose()?;
        let response = self
            .abci_query(
                Some(std::str::FromStr::from_str(&path).unwrap()),
                data,
                height,
                prove,
            )
            .await?;

        match response.code {
            Code::Ok => Ok(EncodedResponseQuery {
                data: response.value,
                info: response.info,
                proof: response.proof,
            }),
            Code::Err(code) => Err(Error::Query(response.info, code)),
        }
    }

    async fn perform<R>(&self, request: R) -> Result<R::Response, tm_rpc::Error>
    where
        R: tm_rpc::SimpleRequest,
    {
        let request_body = request.into_json();
        let response = self.post(request_body).await;

        match response {
            Ok(response) => {
                let response_json = response.text().await.unwrap();
                R::Response::from_string(response_json)
            }
            Err(e) => {
                let error_msg = e.to_string();
                Err(tm_rpc::Error::server(error_msg))
            }
        }
    }
}
