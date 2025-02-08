#!/usr/bin/env bash

set -Eeuo pipefail

source .env

curl "https://api.telegram.org/bot${TELOXIDE_TOKEN}/getUpdates"
