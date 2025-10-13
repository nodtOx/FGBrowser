"""Main entry point for running the FitGirl Repacks crawler.

Usage:
    python run_crawler.py

Configuration:
    Edit MAX_PAGES and CRAWL_DELAY in crawler/crawler.py
"""

import sys
from crawler.crawler import main

if __name__ == "__main__":
    main()
