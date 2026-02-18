#!/bin/bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}"
cat << "EOF"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║        ___                             _                  ║
║       / _ \ _ __   ___ _ __   ___ ___ | |__   __ _       ║
║      | | | | '_ \ / _ \ '_ \ / __/ _ \| '_ \ / _` |      ║
║      | |_| | |_) |  __/ | | | (_| (_) | | | | (_| |      ║
║       \___/| .__/ \___|_| |_|\___\___/|_| |_|\__,_|      ║
║            |_|                                            ║
║                   _ _      _                              ║
║        ___  __ _ | | | ___| |                            ║
║       / _ \/ _` || | |/ _ \ |                            ║
║      |  __/ (_| || | |  __/ |                            ║
║       \___|\__,_||_|_|\___|_|                            ║
║                                                           ║
║          Multiple AI Agents in Parallel                  ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

echo ""
echo -e "${YELLOW}🚀 OpenCode Parallel Demo${NC}"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust/Cargo is not installed${NC}"
    echo "Please run ./setup.sh first"
    exit 1
fi

echo -e "${GREEN}✓ Rust/Cargo detected${NC}"
echo ""

# Build the project
echo -e "${BLUE}📦 Building opencode-parallel...${NC}"
cargo build --release --quiet

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${CYAN}Available Demo Commands:${NC}"
echo ""
echo -e "  ${GREEN}1.${NC} Show Help"
echo -e "     ${YELLOW}./target/release/opencode-parallel --help${NC}"
echo ""
echo -e "  ${GREEN}2.${NC} List Configured Providers"
echo -e "     ${YELLOW}./target/release/opencode-parallel providers${NC}"
echo ""
echo -e "  ${GREEN}3.${NC} Configure Authentication (Example)"
echo -e "     ${YELLOW}./target/release/opencode-parallel auth anthropic --key sk-ant-...${NC}"
echo ""
echo -e "  ${GREEN}4.${NC} Run Interactive TUI"
echo -e "     ${YELLOW}./target/release/opencode-parallel tui --agents 4${NC}"
echo ""
echo -e "  ${GREEN}5.${NC} Run Batch Mode"
echo -e "     ${YELLOW}./target/release/opencode-parallel run --config tasks.example.json${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Interactive menu
echo -e "${CYAN}Choose a demo:${NC}"
echo ""
echo "  1) Show help"
echo "  2) Show version"
echo "  3) List providers"
echo "  4) Run TUI (4 agents)"
echo "  5) Run TUI (8 agents)"
echo "  6) Run batch mode with example config"
echo "  7) Exit"
echo ""

read -p "Enter choice [1-7]: " choice

case $choice in
    1)
        echo ""
        ./target/release/opencode-parallel --help
        ;;
    2)
        echo ""
        ./target/release/opencode-parallel --version
        ;;
    3)
        echo ""
        ./target/release/opencode-parallel providers
        ;;
    4)
        echo ""
        echo -e "${CYAN}Starting TUI with 4 agents...${NC}"
        echo -e "${YELLOW}Controls: q=quit, ↑/k=up, ↓/j=down, s=start, c=cancel${NC}"
        echo ""
        ./target/release/opencode-parallel tui --agents 4
        ;;
    5)
        echo ""
        echo -e "${CYAN}Starting TUI with 8 agents...${NC}"
        echo -e "${YELLOW}Controls: q=quit, ↑/k=up, ↓/j=down, s=start, c=cancel${NC}"
        echo ""
        ./target/release/opencode-parallel tui --agents 8
        ;;
    6)
        echo ""
        echo -e "${CYAN}Running batch mode with example config...${NC}"
        echo ""
        ./target/release/opencode-parallel run --config tasks.example.json --parallel 4
        ;;
    7)
        echo ""
        echo -e "${GREEN}Thanks for trying opencode-parallel!${NC}"
        exit 0
        ;;
    *)
        echo ""
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}✨ Demo completed!${NC}"
echo ""
echo -e "For more information, see ${CYAN}README.md${NC}"
echo ""
