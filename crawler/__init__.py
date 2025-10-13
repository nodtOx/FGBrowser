"""FitGirl Repacks Crawler - Extract game repack information from fitgirl-repacks.site."""

from .crawler import FitGirlCrawler, MAX_PAGES, CRAWL_DELAY
from .database import RepackDatabase, import_from_json
from .blacklist_manager import BlacklistManager

__version__ = "1.0.0"
__all__ = [
    "FitGirlCrawler",
    "RepackDatabase",
    "BlacklistManager",
    "import_from_json",
    "MAX_PAGES",
    "CRAWL_DELAY",
]

