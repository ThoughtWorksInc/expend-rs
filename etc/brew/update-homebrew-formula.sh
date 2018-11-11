#!/bin/bash
set -eu -o pipefail

[[ $# != 3 ]] && {
  echo 1>&2 "USAGE: $0 <tag> <homebrew-template> <homebrew-file>"
  exit 2
}

VERSION="${1:?}"
TEMPLATE_FILE="${2:?}"
HOMEBREW_FILE="${3:?}"

OSX_FILE=expend-${VERSION}-x86_64-apple-darwin.tar.gz
URL_PREFIX=https://github.com/Byron-TW/expend-rs/releases/download/${VERSION}

#shellcheck disable=2064
trap "rm -f $OSX_FILE; exit 1" INT

SLEEP_INTERVAL=5
ROUND=0
while ! [[ -f $OSX_FILE ]]; do
  [[ $ROUND == 0 ]] && {
    echo 1>&2 "Waiting for '$OSX_FILE' to become available... (Ctrl+C to interrupt)"
  }
  ROUND=$((ROUND + 1))
  
  { curl --fail -sLo "$OSX_FILE" "$URL_PREFIX/$OSX_FILE" \
      && echo 1>&2 "Downloaded '$OSX_FILE'"; } || true
  echo 1>&2 -n '.'
  sleep $SLEEP_INTERVAL
done

SHA_SUM=$(
  command -v sha256sum 2>/dev/null \
  || command -v gsha256sum 2>/dev/null \
  || { echo 1>&2 "sha256 program not found"; false; } \
)

OSX_SHA256="$($SHA_SUM "$OSX_FILE" | awk '{print $1}')"
TEMPLATE_NOTE="---> DO NOT EDIT <--- (this file was generated from $TEMPLATE_FILE"
export VERSION OSX_SHA256 TEMPLATE_NOTE

envsubst < "$TEMPLATE_FILE" > "$HOMEBREW_FILE" && {
  echo 1>&2 'homebrew update finished'
}
