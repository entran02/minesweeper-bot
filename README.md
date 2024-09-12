# Minesweeper Bot

This Rust project automates playing Google Minesweeper using the Fantoccini crate.

## Setup

- Clone the repository
- Run `cargo build` to build the project
- Run `chromedriver --port=9515` to start chromedriver instance
- Run `cargo run` to start the bot

## Features

- Automatically solves Google Minesweeper
- Uses WebDriver for browser interaction

## TODO:

- Currently when met with 50/50 mine scenario, will just reset and try again on new board
- Should be updated to choose and random blank square and click it
