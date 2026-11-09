"""Package only the generated public website for the restricted receiver."""

import zipfile
from pathlib import Path

from build_website import OUTPUT
from build_website import main as build
from receive_website import unpack


def main():
    build()
    output = Path("website/site.zip")
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(OUTPUT.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(OUTPUT))
    unpack(output.read_bytes())
    print(output)


if __name__ == "__main__":
    main()
