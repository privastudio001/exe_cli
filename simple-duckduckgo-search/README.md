## run
```
uv run python main.py --query "rust programming language"
```
## build
```
uv run pyinstaller --onefile --name simple-duckduckgo-search --specpath build main.py
```
## exe test
macOS:
```
./dist/simple-duckduckgo-search --query "rust programming language"
```
Windows:
```
.\dist\simple-duckduckgo-search.exe --query "rust programming language"
```
