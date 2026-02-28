#!/usr/bin/env bash
# update-vendor.sh
# Pulls the latest WebUI upstream into the vendor folder.
#
# Usage:
#   ./update-vendor.sh              # update to latest commit on tracked branch
#   ./update-vendor.sh v2.5.0       # update to a specific tag
#   ./update-vendor.sh main         # update to tip of a specific branch

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SUBMODULE_DIR="$SCRIPT_DIR/webui-src"
VENDOR_DIR="$SCRIPT_DIR/vendor/webui"

# ---------------------------------------------------------------------------
# Colors
# ---------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info()    { echo -e "${CYAN}[info]${NC}  $*"; }
success() { echo -e "${GREEN}[ok]${NC}    $*"; }
warn()    { echo -e "${YELLOW}[warn]${NC}  $*"; }
die()     { echo -e "${RED}[error]${NC} $*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Sanity checks
# ---------------------------------------------------------------------------
[[ -d "$SUBMODULE_DIR" ]] || die "webui-src directory not found. Is the submodule initialised?\n       Run: git submodule update --init --recursive"
[[ -d "$VENDOR_DIR" ]]    || die "vendor/webui directory not found. Expected it at $VENDOR_DIR"

# ---------------------------------------------------------------------------
# Optional: check out a specific ref
# ---------------------------------------------------------------------------
if [[ $# -ge 1 ]]; then
    REF="$1"
    info "Checking out ref '$REF' in submodule..."
    git -C "$SUBMODULE_DIR" fetch --tags origin
    git -C "$SUBMODULE_DIR" checkout "$REF"
else
    info "Updating submodule to latest commit on tracked branch..."
    git submodule update --remote webui-src
fi

# ---------------------------------------------------------------------------
# Print the version we are vendoring
# ---------------------------------------------------------------------------
UPSTREAM_COMMIT=$(git -C "$SUBMODULE_DIR" rev-parse --short HEAD)
UPSTREAM_TAG=$(git -C "$SUBMODULE_DIR" describe --tags --exact-match 2>/dev/null || echo "(no tag)")
info "Vendoring commit $UPSTREAM_COMMIT $UPSTREAM_TAG"

# ---------------------------------------------------------------------------
# Verify the source files exist before copying
# ---------------------------------------------------------------------------
SRC_WEBUI_C="$SUBMODULE_DIR/src/webui.c"
SRC_CIVETWEB_C="$SUBMODULE_DIR/src/civetweb/civetweb.c"
SRC_WEBUI_H="$SUBMODULE_DIR/include/webui.h"

[[ -f "$SRC_WEBUI_C" ]]    || die "webui.c not found at $SRC_WEBUI_C"
[[ -f "$SRC_CIVETWEB_C" ]] || die "civetweb.c not found at $SRC_CIVETWEB_C — did the upstream directory structure change?"
[[ -f "$SRC_WEBUI_H" ]]    || die "webui.h not found at $SRC_WEBUI_H"

# ---------------------------------------------------------------------------
# Copy into vendor
# ---------------------------------------------------------------------------
info "Copying sources into vendor/webui/..."

mkdir -p "$VENDOR_DIR/src/civetweb"
mkdir -p "$VENDOR_DIR/include"

cp "$SRC_WEBUI_C"    "$VENDOR_DIR/src/webui.c"
cp "$SRC_CIVETWEB_C" "$VENDOR_DIR/src/civetweb/civetweb.c"
cp "$SRC_WEBUI_H"    "$VENDOR_DIR/include/webui.h"

success "vendor/webui/src/webui.c"
success "vendor/webui/src/civetweb/civetweb.c"
success "vendor/webui/include/webui.h"

# ---------------------------------------------------------------------------
# Warn if the header changed — that likely means bindings need updating
# ---------------------------------------------------------------------------
if git diff --quiet "$VENDOR_DIR/include/webui.h" 2>/dev/null; then
    info "webui.h is unchanged — no binding updates needed."
else
    warn "webui.h has changed. Review the diff and update src/ffi.rs if new functions were added:"
    echo ""
    git diff "$VENDOR_DIR/include/webui.h" | head -80
    echo ""
    warn "Run 'git diff vendor/webui/include/webui.h' to see the full diff."
fi

# ---------------------------------------------------------------------------
# Stage the vendor changes
# ---------------------------------------------------------------------------
git add "$VENDOR_DIR"

# ---------------------------------------------------------------------------
# Show summary and suggested commit message
# ---------------------------------------------------------------------------
echo ""
info "Staged changes:"
git diff --cached --stat

echo ""
info "Suggested commit command:"
echo -e "  ${CYAN}git commit -m \"vendor: update WebUI to $UPSTREAM_COMMIT $UPSTREAM_TAG\"${NC}"
echo ""
success "Done. Review the changes above then commit."