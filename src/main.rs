use fantoccini::{ClientBuilder, Locator};
use tokio;

#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {
    // Connect to the WebDriver.
    let client = ClientBuilder::native().connect("http://localhost:9515").await.expect("failed to connect to webdriver.");

    // Navigate to Google Minesweeper.
    client.goto("https://minesweeperonline.com/").await?;
    let url = client.current_url().await?;
    assert_eq!(url.as_ref(), "https://minesweeperonline.com/");

    client.wait().for_element(Locator::Css(".square.blank")).await?;
    
    client.find(Locator::Css(r#"#\31_1"#)).await?.click().await?;

    // Here you would add more logic to interact with the game.

    // Close the browser.
    //client.close().await?;

    Ok(())
}

