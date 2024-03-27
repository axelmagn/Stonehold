set -exuo pipefail

readonly PACKAGE_DIR="target/package/stonehold"

cargo build --release

rm -rf "${PACKAGE_DIR}"
mkdir -p "${PACKAGE_DIR}"

while read f; do
  mkdir -p "${PACKAGE_DIR}/$(dirname $f)"
  cp "$f" "${PACKAGE_DIR}/$f"
done < manifest.txt

zip -d "${PACKAGE_DIR}.zip" "${PACKAGE_DIR}"