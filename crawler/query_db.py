import sys
from .database import RepackDatabase


def show_stats():
    """Display database statistics."""
    with RepackDatabase() as db:
        stats = db.get_stats()
        print("\n" + "=" * 60)
        print("DATABASE STATISTICS")
        print("=" * 60)
        print(f"Total repacks: {stats['total_repacks']}")
        print(f"Total magnet links: {stats['total_magnet_links']}")
        print(f"Total companies: {stats['total_companies']}")
        print("=" * 60)


def search_games(query):
    """Search for games."""
    with RepackDatabase() as db:
        results = db.search_repacks(query)

        if not results:
            print(f"\nNo results found for '{query}'")
            return

        print(f"\nFound {len(results)} result(s) for '{query}':")
        print("=" * 60)

        for idx, repack in enumerate(results, 1):
            print(f"\n{idx}. {repack['title']}")
            print(f"   Company: {repack['company'] or 'N/A'}")
            print(f"   Genres: {repack['genres_tags'] or 'N/A'}")
            print(f"   Languages: {repack['languages'] or 'N/A'}")
            print(f"   Original Size: {repack['original_size'] or 'N/A'}")
            print(f"   Repack Size: {repack['repack_size'] or 'N/A'}")
            print(f"   Magnet Links: {len(repack['magnet_links'])}")


def show_recent(limit=10):
    """Show recent repacks."""
    with RepackDatabase() as db:
        repacks = db.get_all_repacks(limit=limit)

        print(f"\nShowing {len(repacks)} most recent repacks:")
        print("=" * 60)

        for idx, repack in enumerate(repacks, 1):
            print(f"\n{idx}. {repack['title']}")
            print(f"   Size: {repack['repack_size'] or 'N/A'}")
            print(f"   Date: {repack['date'][:10] if repack['date'] else 'N/A'}")
            print(f"   Magnet Links: {len(repack['magnet_links'])}")


def get_game_detail(title):
    """Get detailed information about a specific game."""
    with RepackDatabase() as db:
        repack = db.get_repack_by_title(title)

        if not repack:
            print(f"\nGame '{title}' not found")
            return

        print("\n" + "=" * 60)
        print(f"GAME DETAILS: {repack['title']}")
        print("=" * 60)
        print(f"Company: {repack['company'] or 'N/A'}")
        print(f"Genres/Tags: {repack['genres_tags'] or 'N/A'}")
        print(f"Languages: {repack['languages'] or 'N/A'}")
        print(f"Original Size: {repack['original_size'] or 'N/A'}")
        print(f"Repack Size: {repack['repack_size'] or 'N/A'}")
        print(f"Date: {repack['date'][:10] if repack['date'] else 'N/A'}")
        print(f"\nMagnet Links ({len(repack['magnet_links'])}):")

        for idx, magnet in enumerate(repack["magnet_links"], 1):
            print(f"\n  {idx}. {magnet['source']}")
            print(f"     {magnet['magnet'][:100]}...")


def export_json(filename="export.json"):
    """Export database to JSON."""
    with RepackDatabase() as db:
        db.export_to_json(filename)


def print_usage():
    """Print usage information."""
    print("\nUsage: python query_db.py <command> [args]")
    print("\nCommands:")
    print("  stats                    - Show database statistics")
    print("  search <query>           - Search for games")
    print("  recent [limit]           - Show recent repacks (default: 10)")
    print("  detail <title>           - Get detailed info about a game")
    print("  export [filename]        - Export database to JSON")
    print("\nExamples:")
    print("  python query_db.py stats")
    print('  python query_db.py search "Little Nightmares"')
    print("  python query_db.py recent 5")
    print('  python query_db.py detail "Arcane Path"')
    print("  python query_db.py export my_repacks.json")


def main():
    if len(sys.argv) < 2:
        print_usage()
        return

    command = sys.argv[1].lower()

    if command == "stats":
        show_stats()

    elif command == "search":
        if len(sys.argv) < 3:
            print("Error: Please provide a search query")
            return
        query = " ".join(sys.argv[2:])
        search_games(query)

    elif command == "recent":
        limit = int(sys.argv[2]) if len(sys.argv) > 2 else 10
        show_recent(limit)

    elif command == "detail":
        if len(sys.argv) < 3:
            print("Error: Please provide a game title")
            return
        title = " ".join(sys.argv[2:])
        get_game_detail(title)

    elif command == "export":
        filename = sys.argv[2] if len(sys.argv) > 2 else "export.json"
        export_json(filename)

    else:
        print(f"Unknown command: {command}")
        print_usage()


if __name__ == "__main__":
    main()
