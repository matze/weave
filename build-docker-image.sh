#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

IMAGE="${IMAGE:-quxfoo/weave}"
ARCHES=(amd64 arm64)

# A clean checkout of a tagged commit is a release: it is published as that
# version and moves `latest`. Every other state is a development build,
# published as `<nearest tag>-<commit>[-dirty]` and leaving `latest` alone, so
# work in progress never replaces a released image.
DIRTY=
[ -z "$(git status --porcelain --untracked-files=no)" ] || DIRTY=1

if [ -z "${DIRTY}" ] && git describe --tags --exact-match >/dev/null 2>&1; then
    VERSION="$(git describe --tags --exact-match)"
    TAGS=("${VERSION}" latest)
else
    VERSION="$(git describe --tags --abbrev=0 2>/dev/null || echo untagged)"
    VERSION="${VERSION}-$(git rev-parse --short HEAD)"
    [ -z "${DIRTY}" ] || VERSION="${VERSION}-dirty"
    TAGS=("${VERSION}")
fi

echo "publishing ${IMAGE}:${VERSION} (tags: ${TAGS[*]})"

for arch in "${ARCHES[@]}"; do
    docker build --target "${arch}" -t "${IMAGE}:${VERSION}-${arch}" .
    docker push "${IMAGE}:${VERSION}-${arch}"
done

manifests=()
for arch in "${ARCHES[@]}"; do
    manifests+=("${IMAGE}:${VERSION}-${arch}")
done

for tag in "${TAGS[@]}"; do
    docker manifest rm "${IMAGE}:${tag}" 2>/dev/null || true
    docker manifest create "${IMAGE}:${tag}" "${manifests[@]}"
    docker manifest annotate "${IMAGE}:${tag}" "${IMAGE}:${VERSION}-arm64" \
        --os linux \
        --arch arm64
done

docker manifest inspect "${IMAGE}:${VERSION}"

for tag in "${TAGS[@]}"; do
    docker manifest push "${IMAGE}:${tag}"
done
