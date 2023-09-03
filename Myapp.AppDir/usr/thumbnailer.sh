#!/bin/bash
# runtime="././runtime-x86_64"
APPIMAGE="$1"
OUT="$2"
SIZE="$3"
APPIMAGES_PATH="$4"

export TARGET_APPIMAGE="$APPIMAGE"
icon_path=".DirIcon"
pushd $APPIMAGES_PATH
while : ; do
    # appimageruntime is the runtime downloaded directly from AppImageKit
    # to avoid executing an untrusted runtime
    $APPDIR/usr/runtime-x86_64 --appimage-extract "$icon_path"
    icon_path="$(readlink "squashfs-root/$icon_path")"
    echo $(pwd)
    [ -z "$icon_path"  ] && break
done
convert squashfs-root/.DirIcon -resize "$SIZE" "$OUT"
rm -r squashfs-root




