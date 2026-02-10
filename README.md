skillforge/

# SkillForge

SkillForge is a fullstack, AI-powered platform for generating, tracking, and personalizing learning roadmaps. It combines a modern Rust/Dioxus web application with robust data collection, enrichment, and processing pipelines in both Rust and Python. The system is designed for extensibility, automation, and high-quality learning resource curation.

---

## Table of Contents
- [SkillForge](#skillforge)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Architecture Overview](#architecture-overview)
  - [Setup \& Installation](#setup--installation)
    - [1. Prerequisites](#1-prerequisites)
    - [2. Data Collection \& Processing](#2-data-collection--processing)
      - [a. YouTube Data Collection](#a-youtube-data-collection)
      - [b. Data Processing](#b-data-processing)
      - [c. Data Merging \& Cleaning](#c-data-merging--cleaning)
    - [3. Database Enrichment](#3-database-enrichment)
    - [4. Running the Web App](#4-running-the-web-app)
  - [Helper Modules \& Pipelines](#helper-modules--pipelines)
    - [YouTube Data Collector](#youtube-data-collector)
    - [Data Processor](#data-processor)
    - [Data Merger \& Cleaner](#data-merger--cleaner)
    - [Database URL Enricher](#database-url-enricher)
  - [Usage](#usage)
  - [Project Structure](#project-structure)
  - [Troubleshooting](#troubleshooting)

---

## Features
- Personalized, AI-generated skill roadmaps with progress tracking
- User authentication (sign up, login, profile management)
- Dashboard with learning analytics and visualizations
- AI-powered roadmap and question generation
- Automated data collection from YouTube and other sources
- LLM-based topic and prerequisite extraction
- Modern, responsive UI (Dioxus + Tailwind CSS)
- Modular, extensible data processing pipeline (Rust & Python)

---

## Architecture Overview

- **app/**: Main Rust/Dioxus web application (frontend + backend, fullstack)
- **database_url_enricher/**: Rust tool for enriching course/resource URLs and storing them in SurrealDB
- **data_collection_and_processing/**: Python & Rust scripts for collecting, processing, merging, and cleaning educational data
   - **yt_data_collector/**: Python YouTube scraper with LLM enhancement
   - **data_processor/**: Rust/Python scripts for transforming and enhancing raw data
   - **data_merger_and_cleaner/**: Rust/Python scripts for merging, deduplicating, and cleaning datasets

---

## Setup & Installation

### 1. Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (for Tailwind CSS, optional)
- [Python 3.9+](https://www.python.org/) (for data collection/processing)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) (for web build)
- [SurrealDB](https://surrealdb.com/) (local database, optional)
- [Dioxus CLI](https://dioxuslabs.com/learn/0.4/getting_started/installation/) (`cargo install dioxus-cli`)

### 2. Data Collection & Processing
#### a. YouTube Data Collection
1. **Install Python dependencies:**
    ```bash
    cd data_collection_and_processing/yt_data_collector
    pip install -r requirements.txt
    ```
2. **Collect YouTube data:**
    - Edit and run `integrated_scraper_with_llm.py` with your YouTube API key (and OpenRouter API key for LLM enhancement).
    - Example usage:
       ```bash
       python integrated_scraper_with_llm.py --api-key <YOUTUBE_API_KEY> --openrouter-key <OPENROUTER_API_KEY>
       ```
    - The script will fetch video metadata, transcripts, and optionally enhance topics/prerequisites using LLMs.

#### b. Data Processing
1. **Process raw data:**
    - Use the Rust or Python scripts in `data_processor/` to transform, enhance, and structure the collected data.
    - Example (Rust):
       ```bash
       cd ../data_processor
       cargo run --release
       ```
    - The processor will output cleaned and LLM-enhanced JSON datasets.

#### c. Data Merging & Cleaning
1. **Merge and deduplicate datasets:**
    - Use the scripts in `data_merger_and_cleaner/` to combine multiple processed datasets, remove duplicates, and perform final cleaning.
    - Example (Rust):
       ```bash
       cd ../data_merger_and_cleaner
       cargo run --release
       ```
    - The output will be a unified, high-quality dataset (e.g., `final_data.json`).

### 3. Database Enrichment
1. **Build and run the Rust enrichment tool:**
    ```bash
    cd database_url_enricher
    cargo build --release
    cargo run --release
    ```
    - This tool enriches your course/resource data with URLs and stores them in SurrealDB.

### 4. Running the Web App
1. **Build frontend assets (optional):**
    ```bash
    cd app
    # If you want to rebuild Tailwind CSS
    # npx tailwindcss -i ./tailwind.css -o ./assets/main.css --watch
    ```
2. **Run the app (recommended):**
    - **Web (WASM, hot reload):**
       ```bash
       dx serve
       ```
    - **Other targets:**
       ```bash
       dx serve --platform desktop
       dx serve --platform web
       ```
    - The app will be available at http://localhost:8080 by default.

---

## Helper Modules & Pipelines

### YouTube Data Collector
- **Location:** `data_collection_and_processing/yt_data_collector/`
- **Language:** Python
- **Purpose:** Scrapes YouTube for educational videos, collects metadata and transcripts, and (optionally) enhances data using LLMs via OpenRouter.
- **Key scripts:**
   - `integrated_scraper_with_llm.py`: Main entry point for scraping and LLM enhancement.
   - `llm_enhanced_processor.py`: LLM-powered topic/prerequisite extraction.
- **How to use:**
   - Install requirements, provide API keys, and run the main script as described above.

### Data Processor
- **Location:** `data_collection_and_processing/data_processor/`
- **Language:** Rust (and optionally Python)
- **Purpose:** Transforms raw scraped data into a structured, LLM-enhanced format suitable for merging and ingestion.
- **Key script:**
   - `src/main.rs`: Reads raw CSV/JSON, enhances with LLM, outputs processed JSON.
- **How to use:**
   - Configure input/output paths and API keys as needed, then run with `cargo run --release`.

### Data Merger & Cleaner
- **Location:** `data_collection_and_processing/data_merger_and_cleaner/`
- **Language:** Rust (and optionally Python)
- **Purpose:** Merges multiple processed datasets, deduplicates entries, and performs final cleaning and validation.
- **Key script:**
   - `src/main.rs`: Main merging/cleaning logic.
- **How to use:**
   - Adjust input/output paths as needed, then run with `cargo run --release`.

### Database URL Enricher
- **Location:** `database_url_enricher/`
- **Language:** Rust
- **Purpose:** Enriches course/resource data with URLs and stores them in SurrealDB for use by the main app.
- **How to use:**
   - Build and run as described above in [Database Enrichment](#3-database-enrichment).

---

## Usage
- Open the app in your browser (default: http://localhost:8080)
- Sign up, log in, and start creating your learning roadmap!
- Use the dashboard to track your progress and explore recommended resources.

---

## Project Structure
```text
skillforge/
├── app/                        # Dioxus web app (Rust)
├── database_url_enricher/      # Rust enrichment tool
├── data_collection_and_processing/
│   ├── yt_data_collector/      # Python YouTube scraper + LLM
│   ├── data_processor/         # Rust/Python data processing
│   └── data_merger_and_cleaner/ # Rust/Python data merging/cleaning
├── dataset_test.json           # Example/test dataset
├── final_data.json             # Final merged dataset
└── README.md                   # This file
```

---

## Troubleshooting
- **Build errors:** Ensure all dependencies are installed and correct Rust toolchain is active.
- **WASM issues:** Make sure `wasm-pack` and `dioxus-cli` are installed and up to date.
- **Python errors:** Check Python version and required packages.
- **Database issues:** Ensure SurrealDB is running if using server features.
- **API key issues:** Double-check your YouTube and OpenRouter API keys for data/LLM scripts.

---

