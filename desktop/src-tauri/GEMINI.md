# Project Overview

This is a Tauri-based desktop application named "FGBrowser" for browsing "FitGirl Repacks". The application is written in Rust and uses a JavaScript frontend.

**Key Technologies:**

*   **Backend:** Rust
*   **Frontend:** JavaScript (likely a framework like React, Vue, or Svelte, managed with npm)
*   **Framework:** Tauri
*   **Database:** Rusqlite
*   **HTTP Client:** Reqwest
*   **HTML Parser:** Scraper

**Architecture:**

The application consists of a Rust backend that provides the core functionalities, including:

*   A web crawler to fetch data from the FitGirl Repacks website.
*   A database to store the crawled data.
*   An API for the frontend to interact with the backend.

The frontend is a web application that runs in a Tauri webview and provides the user interface for the application.

# Building and Running

**Development:**

To run the application in development mode, you need to have Node.js and Rust installed.

1.  Install the frontend dependencies: `npm install`
2.  Run the frontend development server: `npm run dev`
3.  Run the backend: `cargo tauri dev`

**Production:**

To build the application for production, run the following command:

```bash
cargo tauri build
```

This will create a standalone executable file in the `target/release/bundle` directory.

# Development Conventions

*   The Rust code is located in the `src` directory.
*   The frontend code is located in the parent directory (`../`).
*   The `Cargo.toml` file manages the Rust dependencies.
*   The `tauri.conf.json` file configures the Tauri application.
*   The `build.rs` file contains the build script for the application.
