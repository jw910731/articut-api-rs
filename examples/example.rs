use articut_api::articut::Articut;
use env_file_reader::read_file;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let env_variables = read_file(".env")?;
    let articut = Articut::new(&env_variables["USERNAME"], &env_variables["API_KEY"]);
    println!(
        "{:?}",
        articut.parse("我想過過過兒過過的日子。").await.unwrap()
    );
    Ok(())
}
