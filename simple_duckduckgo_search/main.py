import argparse
from ddgs import DDGS
import sys
sys.stdout.reconfigure(encoding='utf-8')

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--query", required=True, type=lambda s: s.strip('"'))
    parser.add_argument("--max", type=int, default=3)
    args = parser.parse_args()

    results = DDGS().text(args.query, max_results=args.max)
    parts = []
    for i, r in enumerate(results, 1):
        parts.append(f"[{i}] Title: {r['title']}\nURL: {r['href']}\nContent: {r['body']}")
    print("\n\n".join(parts))


if __name__ == "__main__":
    main()
