#!/usr/bin/env bash
# Warn when a feat or fix PR neither touches a LeanSpec spec nor names one.
#
# usage: spec-link-check.sh <base-sha> [--strict]
# env:   PR_TITLE, PR_BODY (from the pull_request event)
#
# A PR is linked when its diff against <base-sha> touches specs/ or .lean-spec,
# or its title or body names a spec ("SPEC-NNNN", "spec NNNN", "specs/", "REQ-",
# "LeanSpec spec"). gs names specs specs/SPEC-NNNN-*.md. Only feat and fix titles are checked; other conventional
# types are exempt. Warning mode exits 0; --strict exits 1 on an unlinked PR,
# for when the check becomes required (CER-2890 standard, step 3).
set -euo pipefail

base="${1:?usage: $0 <base-sha> [--strict]}"
strict="${2:-}"
title="${PR_TITLE:-}"
body="${PR_BODY:-}"

if ! [[ "$title" =~ ^(feat|fix)(\([^\)]*\))?!?: ]]; then
  echo "spec-link: exempt (title is not feat or fix): $title"
  exit 0
fi

if git diff --name-only "$base"...HEAD | grep -qE '^(specs/|\.lean-spec$)'; then
  echo "spec-link: linked (diff touches specs/ or .lean-spec)"
  exit 0
fi

if printf '%s\n%s\n' "$title" "$body" | grep -qiE 'spec[ -][0-9]{4}|specs/|REQ-|LeanSpec spec'; then
  echo "spec-link: linked (title or body names a spec)"
  exit 0
fi

msg="feat/fix PR touches no spec under specs/ and names none; create or update a LeanSpec spec, or name it in the PR body (SPEC-NNNN)"
if [ "$strict" = "--strict" ]; then
  echo "::error title=spec-link::$msg"
  exit 1
fi
echo "::warning title=spec-link::$msg"
