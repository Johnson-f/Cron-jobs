#!/bin/bash

# Script to start the Leptos app with cargo watch and Tailwind CSS

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to cleanup background processes on exit
cleanup() {
    echo -e "\n${RED}Stopping processes...${NC}"
    kill $LEPTOS_PID $TAILWIND_PID 2>/dev/null
    exit 0
}

# Set up trap to cleanup on script exit
trap cleanup SIGINT SIGTERM EXIT

echo -e "${BLUE}Starting Leptos development server...${NC}"
cargo leptos watch &
LEPTOS_PID=$!

echo -e "${BLUE}Starting Tailwind CSS watcher...${NC}"
npm run tailwind:watch &
TAILWIND_PID=$!

echo -e "${GREEN}Both processes started!${NC}"
echo -e "${GREEN}Leptos PID: $LEPTOS_PID${NC}"
echo -e "${GREEN}Tailwind PID: $TAILWIND_PID${NC}"
echo -e "${BLUE}Press Ctrl+C to stop both processes${NC}"

# Wait for both processes
wait

