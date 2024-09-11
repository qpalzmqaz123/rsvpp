#[rustfmt::skip]
mod vpp_api;

use std::sync::Arc;

use rsvpp::Client;
use vpp_api::interface::{self, SwInterfaceDump};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(Client::connect_unix("/var/run/vpp/api.sock").await?);
    let interface_service = interface::InterfaceService::new(client);

    let list = interface_service
        .sw_interface_dump(SwInterfaceDump::new().set_sw_if_index(u32::MAX))
        .await?;
    println!("interface list: {list:?}");

    Ok(())
}
