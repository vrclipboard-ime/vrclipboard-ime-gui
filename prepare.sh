#!/usr/bin/env sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
azookey_root=${AZOOKEY_KKC_ROOT:-"$repo_root/../azookey-kkc-rs"}
source_dir="$azookey_root/native/dist"
resource_root="$repo_root/src-tauri/resources"
destination="$resource_root/azookey-native"

"$azookey_root/native/build.sh" release

case "$destination" in
  "$resource_root"/*) ;;
  *) echo "invalid native resource destination: $destination" >&2; exit 1 ;;
esac
rm -rf -- "$destination"
mkdir -p -- "$destination"
cp -R -- "$source_dir"/. "$destination"/
chmod -R u+w "$destination"

npm install
