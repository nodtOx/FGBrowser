from typing import List, Set
import re


class BlacklistManager:
    """Manages blacklist for pages to skip during crawling."""

    def __init__(self, blacklist_file: str = "config/blacklist.txt"):
        self.blacklist_file = blacklist_file
        self.patterns: Set[str] = set()
        self.load_blacklist()

    def load_blacklist(self):
        """Load blacklist patterns from file."""
        try:
            with open(self.blacklist_file, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    # Skip empty lines and comments
                    if line and not line.startswith("#"):
                        self.patterns.add(line.lower())
            print(f"Loaded {len(self.patterns)} blacklist patterns")
        except FileNotFoundError:
            print(f"Blacklist file not found: {self.blacklist_file}")
            print("Creating empty blacklist...")
            self.save_blacklist()

    def save_blacklist(self):
        """Save blacklist patterns to file."""
        with open(self.blacklist_file, "w", encoding="utf-8") as f:
            f.write("# Blacklist for pages to skip during crawling\n")
            f.write("# Add one URL pattern or title per line\n")
            f.write("# Lines starting with # are comments\n\n")
            for pattern in sorted(self.patterns):
                f.write(f"{pattern}\n")
        print(f"Saved {len(self.patterns)} patterns to {self.blacklist_file}")

    def is_blacklisted(self, url: str = None, title: str = None) -> bool:
        """
        Check if a URL or title matches any blacklist pattern.

        Args:
            url: The page URL to check
            title: The page title to check

        Returns:
            True if blacklisted, False otherwise
        """
        if not self.patterns:
            return False

        # Check URL
        if url:
            url_lower = url.lower()
            for pattern in self.patterns:
                if pattern in url_lower:
                    return True

        # Check title
        if title:
            title_lower = title.lower()
            for pattern in self.patterns:
                if pattern in title_lower:
                    return True

        return False

    def add_pattern(self, pattern: str):
        """Add a pattern to the blacklist."""
        pattern = pattern.strip().lower()
        if pattern and not pattern.startswith("#"):
            self.patterns.add(pattern)
            print(f"Added pattern: {pattern}")

    def remove_pattern(self, pattern: str):
        """Remove a pattern from the blacklist."""
        pattern = pattern.strip().lower()
        if pattern in self.patterns:
            self.patterns.remove(pattern)
            print(f"Removed pattern: {pattern}")
            return True
        else:
            print(f"Pattern not found: {pattern}")
            return False

    def list_patterns(self):
        """List all blacklist patterns."""
        if not self.patterns:
            print("Blacklist is empty")
            return

        print(f"\nBlacklist patterns ({len(self.patterns)}):")
        print("=" * 60)
        for idx, pattern in enumerate(sorted(self.patterns), 1):
            print(f"{idx}. {pattern}")

    def clear(self):
        """Clear all patterns from blacklist."""
        count = len(self.patterns)
        self.patterns.clear()
        print(f"Cleared {count} patterns from blacklist")


def main():
    """Example usage and CLI interface."""
    import sys

    if len(sys.argv) < 2:
        print("\nUsage: python blacklist_manager.py <command> [args]")
        print("\nCommands:")
        print("  list                - List all blacklist patterns")
        print("  add <pattern>       - Add a pattern to blacklist")
        print("  remove <pattern>    - Remove a pattern from blacklist")
        print("  check <url/title>   - Check if URL or title is blacklisted")
        print("  clear               - Clear all patterns")
        print("\nExamples:")
        print('  python blacklist_manager.py add "upcoming-repacks"')
        print(
            '  python blacklist_manager.py check "https://fitgirl-repacks.site/upcoming-repacks-9/"'
        )
        print("  python blacklist_manager.py list")
        return

    manager = BlacklistManager()
    command = sys.argv[1].lower()

    if command == "list":
        manager.list_patterns()

    elif command == "add":
        if len(sys.argv) < 3:
            print("Error: Please provide a pattern to add")
            return
        pattern = " ".join(sys.argv[2:])
        manager.add_pattern(pattern)
        manager.save_blacklist()

    elif command == "remove":
        if len(sys.argv) < 3:
            print("Error: Please provide a pattern to remove")
            return
        pattern = " ".join(sys.argv[2:])
        if manager.remove_pattern(pattern):
            manager.save_blacklist()

    elif command == "check":
        if len(sys.argv) < 3:
            print("Error: Please provide a URL or title to check")
            return
        text = " ".join(sys.argv[2:])
        # Determine if it's a URL or title
        if text.startswith("http"):
            is_blacklisted = manager.is_blacklisted(url=text)
            print(f"\nURL: {text}")
        else:
            is_blacklisted = manager.is_blacklisted(title=text)
            print(f"\nTitle: {text}")

        if is_blacklisted:
            print("Status: BLACKLISTED")
        else:
            print("Status: OK")

    elif command == "clear":
        confirm = input("Are you sure you want to clear all patterns? (yes/no): ")
        if confirm.lower() == "yes":
            manager.clear()
            manager.save_blacklist()
        else:
            print("Cancelled")

    else:
        print(f"Unknown command: {command}")


if __name__ == "__main__":
    main()
