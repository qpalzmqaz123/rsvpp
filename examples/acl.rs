#[rustfmt::skip]
mod vpp_api;

use std::sync::Arc;

use rsvpp::Client;
use vpp_api::acl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(Client::connect_unix("/var/run/vpp/api.sock").await?);
    let acl_service = acl::AclService::new(client);

    // Add acl
    let rule = acl::AclRule::new()
        // permit
        .set_is_permit(acl::AclAction::AclActionApiPermit)
        // src 192.168.1.0/24
        .set_src_prefix(
            acl::Prefix::new()
                .set_address(
                    acl::Address::new()
                        .set_af(acl::AddressFamily::AddressIp4)
                        .set_un(acl::AddressUnion::from_ip4([192, 168, 1, 0])),
                )
                .set_len(24),
        )
        // dst 0.0.0.0/0
        .set_dst_prefix(
            acl::Prefix::new()
                .set_address(
                    acl::Address::new()
                        .set_af(acl::AddressFamily::AddressIp4)
                        .set_un(acl::AddressUnion::from_ip4([0, 0, 0, 0])),
                )
                .set_len(0),
        )
        // proto tcp
        .set_proto(acl::IpProto::IpApiProtoTcp);
    let acl = acl::AclAddReplace::new()
        .set_acl_index(0xffffffff)
        .set_r(vec![rule]);
    let rep = acl_service.acl_add_replace(acl).await?;
    println!("Acl index is: {}", rep.acl_index());

    // Get acl list
    let list = acl_service
        .acl_dump(acl::AclDump::new().set_acl_index(0xffffffff))
        .await?;
    println!("Acl list is: {:?}", list);

    Ok(())
}
