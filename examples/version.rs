#[rustfmt::skip]
mod vpp_api;

use rsvpp::Client;
use vpp_api::memclnt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect_unix("/var/run/vpp/api.sock").await?;
    let ctx = client.send_msg(memclnt::ApiVersions::new()).await?;
    let rep = client.recv_msg::<memclnt::ApiVersionsReply>(ctx).await?;

    println!("Versions: {:#?}", rep);

    Ok(())
}
