#!/bin/bash

runtime="./runtime-x86_64"

APPIMAGE="$1"
OUT="$2"
SIZE="$3"

export TARGET_APPIMAGE="$APPIMAGE"

icon_path=".DirIcon"

while : ; do
    # appimageruntime is the runtime downloaded directly from AppImageKit
    # to avoid executing an untrusted runtime
    $runtime --appimage-extract "$icon_path"
    icon_path="$(readlink "squashfs-root/$icon_path")"
    [ -z "$icon_path"  ] && break
done

convert squashfs-root/.DirIcon -resize "$SIZE" "$OUT"
rm -r squashfs-root


