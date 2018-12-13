extern crate bitcoin_rpc_client;
extern crate pretty_env_logger;
extern crate spectral;
extern crate testcontainers;
extern crate web3;
extern crate rusoto_core;
extern crate rusoto_dynamodb;
extern crate rusoto_credential;

use spectral::prelude::*;
use testcontainers::*;

use bitcoin_rpc_client::BitcoinRpcApi;
use web3::futures::Future;
use web3::transports::Http;
use web3::Web3;
use rusoto_core::Region;
use rusoto_dynamodb::{
    DynamoDb,
    DynamoDbClient,
    CreateTableInput,
    KeySchemaElement,
    AttributeDefinition,
    ProvisionedThroughput
};
use rusoto_credential::StaticProvider;
use rusoto_core::HttpClient;

#[test]
fn coblox_bitcoincore_getnewaddress() {
    let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::coblox_bitcoincore::BitcoinCore::default());

    let client = {
        let host_port = node.get_host_port(18443).unwrap();

        let url = format!("http://localhost:{}", host_port);

        let auth = node.image().auth();

        bitcoin_rpc_client::BitcoinCoreClient::new(url.as_str(), auth.username(), auth.password())
    };

    assert_that(&client.get_new_address()).is_ok().is_ok();
}

#[test]
fn parity_parity_listaccounts() {
    let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::parity_parity::ParityEthereum::default());

    let (_event_loop, web3) = {
        let host_port = node.get_host_port(8545).unwrap();

        let url = format!("http://localhost:{}", host_port);

        let (_event_loop, transport) = Http::new(&url).unwrap();
        let web3 = Web3::new(transport);

        (_event_loop, web3)
    };

    let accounts = web3.eth().accounts().wait();

    assert_that(&accounts).is_ok();
}

#[test]
fn trufflesuite_ganachecli_listaccounts() {
    let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::trufflesuite_ganachecli::GanacheCli::default());

    let (_event_loop, web3) = {
        let host_port = node.get_host_port(8545).unwrap();

        let url = format!("http://localhost:{}", host_port);

        let (_event_loop, transport) = Http::new(&url).unwrap();
        let web3 = Web3::new(transport);

        (_event_loop, web3)
    };

    let accounts = web3.eth().accounts().wait();

    assert_that(&accounts).is_ok();
}

#[test]
fn dynamodb_local_create_table() {
    let _ = pretty_env_logger::try_init();
    let docker = clients::Cli::default();
    let node = docker.run(images::dynamodb_local::DynamoDb::default());
    let host_port = node.get_host_port(8000).unwrap();

    let mut create_tables_input = CreateTableInput::default();
    create_tables_input.table_name = "books".to_string();

    let mut key_schema_input = KeySchemaElement::default();
    key_schema_input.key_type = "HASH".to_string();
    key_schema_input.attribute_name = "title".to_string();
    create_tables_input.key_schema = vec![key_schema_input];

    let mut att0 = AttributeDefinition::default();
    att0.attribute_name = "title".to_string();
    att0.attribute_type = "S".to_string();

    create_tables_input.attribute_definitions = vec![att0];

    let mut provisioned_throughput = ProvisionedThroughput::default();
    provisioned_throughput.read_capacity_units = 5;
    provisioned_throughput.write_capacity_units = 5;
    create_tables_input.provisioned_throughput = provisioned_throughput;


    let dynamodb = build_dynamodb_client(host_port);
    let result = dynamodb.create_table(create_tables_input).sync();
    assert_that(&result).is_ok();
}

fn build_dynamodb_client(host_port: u32) -> DynamoDbClient {
    let credentials_provider = StaticProvider::new(
        "fakeKey".to_string(),
        "fakeSecret".to_string(),
        None,
        None);

    let dispatcher = HttpClient::new()
        .expect("could not create http client");

    let region = Region::Custom {
        name: "dynamodb-local".to_string(),
        endpoint: format!("http://localhost:{}", host_port)
    };

    DynamoDbClient::new_with(dispatcher, credentials_provider, region)
}
