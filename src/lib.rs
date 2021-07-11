use std::{error::Error, net::SocketAddr, sync::Arc};

use quinn::{Certificate, CertificateChain, ClientConfig, ClientConfigBuilder, Endpoint, Incoming, PrivateKey, ServerConfig, ServerConfigBuilder, TransportConfig};

pub fn server(addr: SocketAddr) -> Result<(Incoming, Vec<u8>), Box<dyn Error>> {
    let (server_config, server_cert) = configure_server()?;
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.listen(server_config);
    let (_endpoint, incoming) = endpoint_builder.bind(&addr)?;

    Ok((incoming, server_cert))
}

fn configure_server() -> Result<(ServerConfig, Vec<u8>), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = PrivateKey::from_der(&priv_key)?;

    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_uni_streams(0).unwrap();
    let mut server_config = ServerConfig::default();
    server_config.transport = Arc::new(transport_config);
    let mut cfg_builder = ServerConfigBuilder::new(server_config);
    let cert = Certificate::from_der(&cert_der)?;
    cfg_builder.certificate(CertificateChain::from_certs(vec![cert]), priv_key)?;

    Ok((cfg_builder.build(), cert_der))
}

pub fn client(addr: SocketAddr, server_certs: &[&[u8]]) -> Result<Endpoint, Box<dyn Error>> {
    let client_cfg = configure_client(server_certs)?;
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.default_client_config(client_cfg);
    let (endpoint, _incoming) = endpoint_builder.bind(&addr)?;

    Ok(endpoint)
}

fn configure_client(server_certs: &[&[u8]]) -> Result<ClientConfig, Box<dyn Error>> {
    let mut cfg_builder = ClientConfigBuilder::default();
    for cert in server_certs {
        cfg_builder.add_certificate_authority(Certificate::from_der(&cert)?)?;
    }

    Ok(cfg_builder.build())
}