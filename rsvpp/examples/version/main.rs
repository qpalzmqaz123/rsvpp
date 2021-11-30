mod out;

use rsvpp::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = Client::connect_unix("/tmp/vpp-api.sock___").await?;

    let rep: out::memclnt::ApiVersionsReply = client
        .recv_msg(client.send_msg(out::memclnt::ApiVersions::new()).await?)
        .await?;

    println!("Version: {:#?}", rep);

    Ok(())
}
