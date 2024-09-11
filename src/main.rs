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
    let client = ClientBuilder::native().connect("http://localhost:9515").await.expect("failed to connect to webdriver.");

    let mut board = Board::new(true, true, client).await?;

    board.play().await;

    Ok(())
}

