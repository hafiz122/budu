#!/bin/bash
# Download and extract Visual C++ Redistributables for use in Wine bottles.
# These are placed in ~/.gamerunner/runtimes/vcrun<year>/ for the
# runtime manager to symlink into bottles.

set -euo pipefail

RUNTIMES_DIR="$HOME/.gamerunner/runtimes"

echo "==> VC++ Runtime Downloader"

# In production, these would be fetched from trusted Microsoft CDN URLs.
# For now, this is a placeholder that documents the expected layout.

echo "Place the following DLL files in the appropriate directories:"

cat <<EOF

  ~/.gamerunner/runtimes/vcrun2022/
    ├── vcruntime140.dll
    ├── vcruntime140_1.dll
    ├── msvcp140.dll
    ├── msvcp140_1.dll
    ├── msvcp140_2.dll
    └── concrt140.dll

  ~/.gamerunner/runtimes/vcrun2019/
    ├── vcruntime140.dll
    ├── msvcp140.dll
    └── concrt140.dll

These can be extracted from a Visual Studio 2022 installation
or from Microsoft's official VC++ redistributable packages.

EOF

echo "Run 'winetricks vcrun2022' inside a Wine bottle to install them automatically."
