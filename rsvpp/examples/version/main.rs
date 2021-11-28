mod out;

use rsvpp::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut client = Client::connect_unix("/tmp/vpp-api.sock___").await?;

    let rep: out::memclnt::VlApiApiVersionsReplyT = client
        .recv_msg(
            client
                .send_msg(out::memclnt::VlApiApiVersionsT::default())
                .await?,
        )
        .await?;

    println!("Version: {:#?}", rep);

    Ok(())
}
