#!/bin/sh
cd "$(dirname "$0")" || exit 1
exec ./bin/linux/run-carter "$@"
