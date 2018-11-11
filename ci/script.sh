#!/usr/bin/env sh
# This script takes care of testing your crate

set -ex

main() {
    cross build --target "$TARGET"

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target "$TARGET"

    bin=expend
    ./tests/stateless-journey.sh "target/$TARGET/debug/$bin"
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
