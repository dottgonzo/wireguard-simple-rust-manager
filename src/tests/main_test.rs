#[cfg(test)]
mod tests {
    use core::net::SocketAddr;

    use crate::connect_to_wireguard;

    #[tokio::test]
    async fn test_connection() {
        let server_endpoint: SocketAddr = "XX.XX.XX.XX:51820".parse().unwrap();
        let server_public_key = "N9ZPcCtSJJQIp/GtfD5+EAiNQlyABe06GPEaibKtmws=".to_string();
        let client_private_key = "0PPBFCQ+p2OwJBPbw+OrYecb6pKp4DqIDT0GP4EIsF4=".to_string();
        let client_address = "10.33.0.33".to_string();
        let client_port = Some(12345);
        let client_addresses_maks = Some(vec!["10.33.0.0/16".to_string()]);

        let result = connect_to_wireguard(
            server_endpoint,
            server_public_key,
            client_private_key,
            client_address,
            client_port,
            client_addresses_maks,
        )
        .await;

        assert!(result.is_ok());
    }
}
