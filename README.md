# Minesweeper Bot

This Rust project automates playing MinesweeperOnline using the Fantoccini crate.

## Setup

- Clone the repository
- Run `cargo build` to build the project
- Run `chromedriver --port=9515` to start chromedriver instance
- Run `cargo run` to start the bot

## Features

- Automatically solves MinesweeperOnline
- Uses WebDriver for browser interaction

## TODO:

- Currently when met with 50/50 mine scenario, will just reset and try again on new board
- Should be updated to choose a random blank square and click it
