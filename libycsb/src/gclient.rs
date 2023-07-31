
// mod common;
use crate::common::ycsb_server_client::YcsbServerClient;
use crate::common::{ExecuteRequest, CmdsReqeust};
use log::info;
// pub mod common {
//     tonic::include_proto!("common");
// }
pub struct GClient {

}
impl GClient {
    pub fn new() -> Self {
        Self {}
    }
    // #[tokio::main]
    pub async fn generate_cmds(&mut self, id: i32) -> Vec<u8> {
        let mut client = YcsbServerClient::connect("http://127.0.0.1:8000").await.unwrap();

        // let client = gclient
        let request = tonic::Request::new(CmdsReqeust {
            id: id,
        });

        let response = client.get_cmds(request).await.unwrap();
        // info!("RESPONSE={:?}", response);

        response.get_ref().payload.clone()

        // Ok(())
    }

    pub async fn execute_cmds(&mut self, payload: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = YcsbServerClient::connect("http://127.0.0.1:8000").await?;

        let request = tonic::Request::new(ExecuteRequest {
            payload: payload,
        });

        let response = client.execute_cmds(request).await?;

        info!("RESPONSE={:?}", response);

        Ok(())
    }
}

