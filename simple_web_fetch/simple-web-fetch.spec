# -*- mode: python ; coding: utf-8 -*-


a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[
        ('.venv/Lib/site-packages/apify_fingerprint_datapoints/data', 'apify_fingerprint_datapoints/data'),
        ('.venv/Lib/site-packages/browserforge/injectors/data', 'browserforge/injectors/data'),
        ('.venv/Lib/site-packages/tld/res', 'tld/res'),
        ('.venv/Lib/site-packages/trafilatura/settings.cfg', 'trafilatura'),
        ('.venv/Lib/site-packages/trafilatura/data', 'trafilatura/data'),
        ('.venv/Lib/site-packages/justext/stoplists', 'justext/stoplists'),
    ],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='simple-web-fetch',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
coll = COLLECT(
    exe,
    a.binaries,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='simple-web-fetch',
)
