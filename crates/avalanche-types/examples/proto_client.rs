use avalanche_types::proto::pb::helloworld::{greeter_client::GreeterClient, HelloRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(HelloRequest {
        name: "Avalanche Rustaceans!".into(),
    });
    let resp = GreeterClient::connect("http://[::1]:50051")
        .await?
        .say_hello(request)
        .await?;

    println!("SUCCESSFUL response: {resp:?}");

    Ok(())
}
