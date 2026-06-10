async fn async_io(url: &str) -> Result<String, Error> {
    let body = client_get(url).await?;
    println!("{}", body);
    Ok(body)
}
