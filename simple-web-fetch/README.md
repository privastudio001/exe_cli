## run
```
uv run python main.py --url https://www.bbc.com/zhongwen/articles/cn0vrr5kezpo/trad
```
## build
macOS:
```
uv run pyinstaller --onefile --name simple-web-fetch --specpath build $(cat pyinstaller.args) main.py
```
Windows (PowerShell):
```
uv run pyinstaller --onefile --name simple-web-fetch --specpath build $((Get-Content pyinstaller.args) -split ' ') main.py
```
## exe test
macOS:
```
./dist/simple-web-fetch --url https://www.bbc.com/zhongwen/articles/cn0vrr5kezpo/trad
```
Windows:
```
.\dist\simple-web-fetch.exe --url https://www.bbc.com/zhongwen/articles/cn0vrr5kezpo/trad
```