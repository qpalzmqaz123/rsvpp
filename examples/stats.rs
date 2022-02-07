use rsvpp::Stats;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stats = Stats::connect("/run/vpp/stats.sock").await?;

    let ifaces = stats.interface();
    println!("{:#?}", ifaces);

    let acls = stats.acl();
    println!("acls{:#?}", acls);

    Ok(())
}
