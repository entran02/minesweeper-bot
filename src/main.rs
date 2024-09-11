use fantoccini::{ClientBuilder, Locator};
use fantoccini::actions::{Actions, MouseActions, PointerAction, InputSource, MOUSE_BUTTON_RIGHT};
use tokio::time::Duration;
use tokio;
mod info;
mod posn;
mod cell;
mod board;
use board::Board;
mod cell_wrapper;

#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {

    // Connect to the WebDriver.
    let client = ClientBuilder::native().connect("http://localhost:9515").await.expect("failed to connect to webdriver.");

    let mut board = Board::new(true, true, client).await?;

    board.play().await;

    // Navigate to Google Minesweeper.
    //client.goto("https://minesweeperonline.com/").await?;
    //let url = client.current_url().await?;
    //assert_eq!(url.as_ref(), "https://minesweeperonline.com/");

    //client.wait().for_element(Locator::Css(".square.blank")).await?;

    //let element = client.find(Locator::Css(r#"#\31_1"#)).await?;
    //println!("{:?}", element);
    
    //client.find(Locator::Css(r#"#\31_1"#)).await?.click().await?;

    // Here you would add more logic to interact with the game.

    // Close the browser.
    //client.close().await?;

    Ok(())
}

