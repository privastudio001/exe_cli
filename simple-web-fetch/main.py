import sys
import logging
sys.stdout.reconfigure(encoding='utf-8')
logging.disable(logging.CRITICAL)

import argparse
import trafilatura
from scrapling import Fetcher

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", required=True)
    parser.add_argument("--max", type=int, default=3000)
    args = parser.parse_args()

    page = Fetcher.get(args.url)
    if page.status != 200:
        print(f"Error: HTTP {page.status} {page.reason} for {args.url}", flush=True)
        return
    text = trafilatura.extract(
        page.html_content,
        output_format="markdown",
        favor_precision=True,
        include_comments=False,
        include_tables=False,
        no_fallback=False,
    ) or ""
    print(text[:args.max])

if __name__ == "__main__":
    main()