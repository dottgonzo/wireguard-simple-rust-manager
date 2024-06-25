#[cfg(test)]
mod tests;

use core::time;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use defguard_wireguard_rs::{
    host::Peer, key::Key, net::IpAddrMask, InterfaceConfiguration, WGApi, WireguardInterfaceApi,
};

fn get_network_address(ip: Ipv4Addr, prefix: u8) -> Ipv4Addr {
    let mask: u32 = !0 << (32 - prefix);
    let ip_u32: u32 = u32::from(ip);
    Ipv4Addr::from(ip_u32 & mask)
}

fn get_first_ip(network_address: Ipv4Addr) -> Ipv4Addr {
    let ip_u32: u32 = u32::from(network_address);
    Ipv4Addr::from(ip_u32 + 1)
}

pub async fn connect_to_wireguard(
    server_endpoint: SocketAddr,
    server_public_key: String,
    client_private_key: String,
    client_address: String,
    client_port: Option<u32>,
    client_addresses_maks: Option<Vec<String>>,
    network_prefix: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create new API object for interface

    let ifname: String = if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
        "wg0".into()
    } else {
        "utun3".into()
    };

    // Create new API object for interface

    let wgapi = WGApi::new(ifname.clone(), false)?;

    // Check if the interface is just created

    let wire_data = wgapi.read_interface_data();
    let ip: Ipv4Addr = client_address.parse().expect("Invalid IP address");
    let network_prefix = 16;

    let network_address = get_network_address(ip, network_prefix);
    if wire_data.is_ok() {
        // Interface already exists

        let first_ip = get_first_ip(network_address);
        let ip_ping: IpAddr = IpAddr::V4(first_ip);

        let pinged = rust_simple_ping::ping(Some(ip_ping), None).await;

        match pinged {
            Ok(_pinged) => {
                return Ok(());
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }

        // check if the interface is active
    }

    // create interface
    wgapi.create_interface()?;

    // Peer configuration
    let peer_public_key: Key = server_public_key.parse()?;
    let mut peer = Peer::new(peer_public_key);
    // Your WireGuard server endpoint which client connects to
    // let endpoint: SocketAddr = "10.10.10.10:55001".parse().unwrap();
    // Peer endpoint and interval
    peer.endpoint = Some(server_endpoint);
    peer.persistent_keepalive_interval = Some(25);

    // Allowed IPs

    if let Some(client_addresses_maks) = client_addresses_maks {
        for addr in client_addresses_maks {
            // peer.allowed_ips.push(IpAddrMask::from_str("10.6.0.0/24")?);

            peer.allowed_ips.push(IpAddrMask::from_str(&addr)?);
        }
    } else {
        peer.allowed_ips.push(
            IpAddrMask::from_str(
                (network_address.to_string().as_str().to_owned()
                    + "/"
                    + network_prefix.to_string().as_str())
                .as_str(),
            )
            .unwrap(),
        );
    }

    let mut default_client_port: u32 = 12345;

    // Peer port
    if let Some(client_port) = client_port {
        default_client_port = client_port;
    }

    // let client_address= "10.6.0.30".to_string();
    // let client_private_key= "AAECAwQFBgcICQoLDA0OD/Dh0sO0pZaHeGlaSzwtHg8=".to_string();

    // interface configuration
    let interface_config = InterfaceConfiguration {
        name: ifname.clone(),
        prvkey: client_private_key,
        address: client_address,
        port: default_client_port,
        peers: vec![peer],
    };

    #[cfg(not(windows))]
    wgapi.configure_interface(&interface_config)?;
    #[cfg(windows)]
    wgapi.configure_interface(&interface_config, &[])?;
    wgapi.configure_peer_routing(&interface_config.peers)?;
    eprintln!("Interface created");

    Ok(())
}

pub async fn routine_connect_to_wireguard(
    server_endpoint: SocketAddr,
    server_public_key: String,
    client_private_key: String,
    client_address: String,
    client_port: Option<u32>,
    client_addresses_maks: Option<Vec<String>>,
    network_prefix: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let connect = connect_to_wireguard(
        server_endpoint,
        server_public_key,
        client_private_key,
        client_address,
        client_port,
        client_addresses_maks,
        network_prefix,
    )
    .await;
    loop {
        if let Err(e) = connect {
            eprintln!("Error: {:?}", e);
            break;
        }
        std::thread::sleep(time::Duration::from_secs(5));
    }
    Ok(())
}
