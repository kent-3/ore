#!/bin/bash
# Script to switch between Solana versions

SOLANA_INSTALL_DIR="$HOME/.local/share/solana/install"

# Available versions
V1_17_22="$SOLANA_INSTALL_DIR/releases/stable-dbf06e258ae418097049e845035d7d5502fe1327/solana-release"
V3_0_10="$SOLANA_INSTALL_DIR/releases/stable-96c3a8519a3bac8c7e7dd49b6d6aefcfeba09d90/solana-release"

if [ "$1" == "1.17.22" ] || [ "$1" == "work" ]; then
    echo "Switching to Solana 1.17.22 (work version)..."
    rm -f "$SOLANA_INSTALL_DIR/active_release"
    ln -s "$V1_17_22" "$SOLANA_INSTALL_DIR/active_release"
    hash -r
    echo "Done! Run: solana --version"
elif [ "$1" == "3.0.10" ] || [ "$1" == "latest" ] || [ "$1" == "ore" ]; then
    echo "Switching to Solana 3.0.10 (for ore project)..."
    rm -f "$SOLANA_INSTALL_DIR/active_release"
    ln -s "$V3_0_10" "$SOLANA_INSTALL_DIR/active_release"
    hash -r
    echo "Done! Run: solana --version"
else
    echo "Usage: switch-solana [1.17.22|work|3.0.10|latest|ore]"
    echo ""
    echo "Current version:"
    solana --version
    echo ""
    echo "Available versions:"
    echo "  1.17.22 (work)  - For your work projects"
    echo "  3.0.10 (latest) - For ore project"
fi
