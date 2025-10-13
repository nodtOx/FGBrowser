import requests
from bs4 import BeautifulSoup
import time
import sys
from typing import List, Dict, Optional
from urllib.parse import urljoin
from .database import RepackDatabase
from .blacklist_manager import BlacklistManager


# Configuration
MAX_PAGES = 50  # Maximum pages to crawl (set to None for infinite crawl)
CRAWL_DELAY = 1.0  # Delay between requests in seconds


def sanitize_for_console(text: str) -> str:
    """Remove problematic Unicode characters for Windows console."""
    if not text:
        return text

    # Replace problematic characters with ASCII equivalents or remove them
    replacements = {
        "–": "-",  # en dash
        "—": "-",  # em dash
        """: "'",  # smart quote
        """: "'",  # smart quote
        '"': '"',  # smart quote
        '"': '"',  # smart quote
        "…": "...",  # ellipsis
        "™": "(TM)",
        "®": "(R)",
        "©": "(C)",
        "°": " deg",
        "×": "x",
        "÷": "/",
        "∆": "Delta",
        "�": "",  # replacement character
    }

    for old, new in replacements.items():
        text = text.replace(old, new)

    # Remove any remaining non-ASCII characters including emojis
    # Keep only printable ASCII characters (32-126)
    text = "".join(
        char if 32 <= ord(char) < 127 or char in "\n\r\t" else "" for char in text
    )

    return text


class FitGirlCrawler:
    """Crawler for fitgirl-repacks.site to extract game repack information."""

    def __init__(self, base_url: str = "https://fitgirl-repacks.site"):
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update(
            {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            }
        )
        self.blacklist = BlacklistManager()

    def get_page(self, url: str, delay: float = CRAWL_DELAY) -> Optional[BeautifulSoup]:
        """Fetch and parse a page with rate limiting."""
        try:
            time.sleep(delay)
            response = self.session.get(url, timeout=30)
            response.raise_for_status()
            return BeautifulSoup(response.content, "lxml")
        except requests.RequestException as e:
            print(f"Error fetching {url}: {e}")
            return None

    def extract_repack_from_post(self, article) -> Optional[Dict]:
        """Extract repack information from a post article element."""
        try:
            data = {}

            # Get title and link
            title_elem = article.find("h1", class_="entry-title")
            if not title_elem:
                title_elem = article.find("h2", class_="entry-title")

            if title_elem:
                link_elem = title_elem.find("a")
                if link_elem:
                    data["title"] = link_elem.get_text(strip=True)
                    data["url"] = link_elem.get("href")
                else:
                    data["title"] = title_elem.get_text(strip=True)
                    data["url"] = None
            else:
                return None

            # Get post date
            date_elem = article.find("time", class_="entry-date")
            if date_elem:
                data["date"] = date_elem.get("datetime") or date_elem.get_text(
                    strip=True
                )

            # Get content
            content_elem = article.find("div", class_="entry-content")
            if content_elem:
                # Extract game details
                data["details"] = self._extract_game_details(content_elem)

                # Extract magnet links from listing page
                data["magnet_links"] = self._extract_magnet_links(content_elem)

                # Extract description
                description_parts = []
                for elem in content_elem.find_all(["p", "div"], recursive=False):
                    text = elem.get_text(strip=True)
                    if text and not text.startswith("Download Mirrors"):
                        description_parts.append(text)
                data["description"] = "\n\n".join(
                    description_parts[:3]
                )  # First few paragraphs

            return data
        except Exception as e:
            print(f"Error extracting post: {e}")
            return None

    def _extract_game_details(self, content_elem) -> Dict:
        """Extract structured game details from content."""
        import re

        details = {}

        # Look for the game info section (usually in <h3>)
        info_section = content_elem.find("h3")
        if info_section:
            # Get all text until next heading or download section
            text_parts = []
            current = info_section.find_next_sibling()

            while current and current.name != "h3":
                text = current.get_text(strip=True)
                if text:
                    text_parts.append(text)
                current = current.find_next_sibling()
                if not current:
                    break

            # Join and parse the combined text
            full_text = " ".join(text_parts)

            # Try more flexible patterns
            # Genres/Tags
            match = re.search(
                r"Genres?[/\s]*Tags?:\s*(.+?)(?=Compan(?:y|ies):|Languages?:|$)",
                full_text,
                re.IGNORECASE | re.DOTALL,
            )
            if match:
                details["genres_tags"] = re.sub(r"\s+", " ", match.group(1).strip())

            # Company/Companies
            match = re.search(
                r"Compan(?:y|ies):\s*(.+?)(?=Languages?:|Original|Repack|$)",
                full_text,
                re.IGNORECASE | re.DOTALL,
            )
            if match:
                details["company"] = re.sub(r"\s+", " ", match.group(1).strip())

            # Languages
            match = re.search(
                r"Languages?:\s*(.+?)(?=Original|Repack|This game|$)",
                full_text,
                re.IGNORECASE | re.DOTALL,
            )
            if match:
                details["languages"] = re.sub(r"\s+", " ", match.group(1).strip())

            # Original Size
            match = re.search(
                r"Original Size:\s*(.+?)(?=Repack|$)",
                full_text,
                re.IGNORECASE | re.DOTALL,
            )
            if match:
                details["original_size"] = re.sub(r"\s+", " ", match.group(1).strip())

            # Repack Size
            match = re.search(
                r"Repack Size:\s*(.+?)(?=Download|$)",
                full_text,
                re.IGNORECASE | re.DOTALL,
            )
            if match:
                details["repack_size"] = re.sub(r"\s+", " ", match.group(1).strip())

        return details

    def _extract_magnet_links(self, content_elem) -> List[Dict]:
        """Extract magnet links from content."""
        magnet_links = []

        # Find all magnet links in the content
        for link in content_elem.find_all(
            "a", href=lambda x: x and x.startswith("magnet:")
        ):
            # Get the parent li to find the source
            parent_li = link.find_parent("li")
            if parent_li:
                # Try to find the source name (usually at the beginning of the li)
                source_text = parent_li.get_text(strip=True)
                # Extract source name (before the | or [ character)
                source = source_text.split("|")[0].split("[")[0].strip()

                magnet_links.append({"source": source, "magnet": link.get("href")})

        return magnet_links

    def crawl_page(self, page_num: int = 1) -> List[Dict]:
        """Crawl a single page and extract all repacks."""
        if page_num == 1:
            url = self.base_url
        else:
            url = f"{self.base_url}/page/{page_num}/"

        print(f"\nPage {page_num}: {url}")
        soup = self.get_page(url)

        if not soup:
            return []

        repacks = []
        articles = soup.find_all("article")

        for article in articles:
            repack = self.extract_repack_from_post(article)
            if repack:
                # Check blacklist
                if self.blacklist.is_blacklisted(
                    url=repack.get("url"), title=repack.get("title")
                ):
                    print(f"  [SKIP] {sanitize_for_console(repack.get('title'))}")
                    continue
                print(f"  [+] {sanitize_for_console(repack.get('title'))}")
                repacks.append(repack)

        return repacks

    def crawl_repack_detail(self, url: str) -> Optional[Dict]:
        """Crawl a detailed repack page for complete information."""
        soup = self.get_page(url)

        if not soup:
            return None

        article = soup.find("article")
        if not article:
            return None

        data = self.extract_repack_from_post(article)

        # Ensure URL is set to the page we're crawling
        if data:
            data["url"] = url

            # Check blacklist
            if self.blacklist.is_blacklisted(url=url, title=data.get("title")):
                print(f"  Skipping blacklisted page: {data.get('title')}")
                return None

        if data:
            # Extract additional details from full page
            content = article.find("div", class_="entry-content")
            if content:
                # Extract repack features
                features_header = content.find(string="Repack Features")
                if features_header:
                    features_list = features_header.find_next("ul")
                    if features_list:
                        data["repack_features"] = [
                            li.get_text(strip=True)
                            for li in features_list.find_all("li")
                        ]

                # Extract magnet links and download mirrors
                magnet_links = []
                download_mirrors = []

                for heading in content.find_all(["h3", "h2"]):
                    heading_text = heading.get_text()
                    if "Download Mirrors" in heading_text:
                        # Look for torrent section
                        if "Torrent" in heading_text:
                            mirror_section = heading.find_next_sibling("ul")
                            if mirror_section:
                                for li in mirror_section.find_all("li"):
                                    # Find magnet links
                                    magnet_link = li.find(
                                        "a",
                                        href=lambda x: x and x.startswith("magnet:"),
                                    )
                                    if magnet_link:
                                        magnet_links.append(
                                            {
                                                "source": li.get_text(strip=True)
                                                .split("[")[0]
                                                .strip(),
                                                "magnet": magnet_link.get("href"),
                                            }
                                        )
                        else:
                            # Regular download mirrors
                            mirror_section = heading.find_next_sibling("ul")
                            if mirror_section:
                                for li in mirror_section.find_all("li"):
                                    download_mirrors.append(li.get_text(strip=True))

                if magnet_links:
                    data["magnet_links"] = magnet_links
                if download_mirrors:
                    data["download_mirrors"] = download_mirrors

        return data

    def crawl_multiple_pages(
        self, start_page: int = 1, max_pages: Optional[int] = 5
    ) -> List[Dict]:
        """Crawl multiple pages and return all repacks.

        Args:
            start_page: Page number to start from
            max_pages: Maximum number of pages to crawl (None for infinite)
        """
        all_repacks = []

        print("\n" + "=" * 80)
        if max_pages is None:
            print(f"CRAWLING PAGES {start_page} to END (infinite mode)")
        else:
            print(f"CRAWLING PAGES {start_page} to {start_page + max_pages - 1}")
        print("=" * 80)

        page_num = start_page
        while True:
            # Check if we've reached max_pages
            if max_pages is not None and page_num >= start_page + max_pages:
                break

            repacks = self.crawl_page(page_num)

            if not repacks:
                print(f"\nNo more content found at page {page_num}")
                print("Reached end of available pages.")
                break

            all_repacks.extend(repacks)
            print(
                f"[OK] Page {page_num}: Found {len(repacks)} games (Total: {len(all_repacks)})"
            )

            page_num += 1

        print("\n" + "=" * 80)
        print(f"CRAWLING COMPLETE: {len(all_repacks)} games found")
        print("=" * 80 + "\n")

        return all_repacks

    def save_to_database(self, data: List[Dict], db_path: str = "repacks.db"):
        """Save crawled data to database."""
        with RepackDatabase(db_path) as db:
            db.create_tables()
            count = db.insert_repacks_batch(data)
            stats = db.get_stats()
            print(f"\nDatabase Statistics:")
            print(f"  Total repacks: {stats['total_repacks']}")
            print(f"  Total magnet links: {stats['total_magnet_links']}")
            print(f"  Total companies: {stats['total_companies']}")
            return count


def main():
    """Example usage of the crawler."""
    crawler = FitGirlCrawler()

    # Crawl pages based on MAX_PAGES configuration
    print("Starting crawl...")
    if MAX_PAGES is None:
        print("Infinite crawl mode - will crawl until no more pages found...")
        print("This may take 10+ minutes depending on total pages available.")
    else:
        print(f"Crawling {MAX_PAGES} pages (estimated time: 2-3 minutes)...")

    repacks = crawler.crawl_multiple_pages(start_page=1, max_pages=MAX_PAGES)

    # Format repacks from listing page data (everything is available on listing pages!)
    print("\n" + "=" * 80)
    print(f"PROCESSING {len(repacks)} GAMES FROM LISTING PAGES")
    print("=" * 80 + "\n")

    formatted_repacks = []
    for idx, repack in enumerate(repacks, 1):
        if repack.get("url"):
            formatted_repack = {
                "title": repack.get("title"),
                "genres_tags": repack.get("details", {}).get("genres_tags"),
                "company": repack.get("details", {}).get("company"),
                "languages": repack.get("details", {}).get("languages"),
                "original_size": repack.get("details", {}).get("original_size"),
                "repack_size": repack.get("details", {}).get("repack_size"),
                "magnet_links": repack.get("magnet_links", []),
                "url": repack.get("url"),
                "date": repack.get("date"),
            }
            formatted_repacks.append(formatted_repack)

            if idx % 50 == 0:
                print(f"Processed {idx}/{len(repacks)} games...")

    print(f"Processed all {len(formatted_repacks)} games!")

    # Show summary
    total_magnets = sum(len(r["magnet_links"]) for r in formatted_repacks)
    print(
        f"\nExtracted {total_magnets} total magnet links from {len(formatted_repacks)} games!"
    )

    # Save to database
    print("\nSaving to database...")
    crawler.save_to_database(formatted_repacks)
    print(f"\nSuccessfully crawled and saved {len(formatted_repacks)} repacks!")
    print(f"Database location: repacks.db")


if __name__ == "__main__":
    main()
