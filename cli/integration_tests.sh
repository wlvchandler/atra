#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to verify a sequence of trades
verify_trade_sequence() {
    local order_cmd=$1
    shift
    local expected_trades=("$@")
    local num_expected=$((${#expected_trades[@]} / 3))

    # Get initial trades for comparison
    local initial_trade
    initial_trade=$(./atra trades 1)

    # Execute the order
    echo "Executing order: $order_cmd"
    local result
    result=$(eval "$order_cmd")
    echo "Order result: $result"

    # Small delay to ensure trade processing
    sleep 0.1

    # Get recent trades
    local trades
    trades=$(./atra trades "$((num_expected + 1))")
    echo "Recent trades:"
    echo "$trades"

    # Skip the first trade if it matches our initial trade
    if [[ "$(echo "$trades" | head -n1)" == "$initial_trade" ]]; then
        trades=$(echo "$trades" | tail -n +2)
    fi

    local failed=0
    local trade_index=0

    while IFS=',' read -r type timestamp price quantity side maker_id taker_id; do
        if [[ $trade_index -ge $num_expected ]]; then
            break
        fi

        local expect_qty=${expected_trades[$((trade_index * 3))]}
        local expect_price=${expected_trades[$((trade_index * 3 + 1))]}
        local expect_side=${expected_trades[$((trade_index * 3 + 2))]}

        echo "Verifying trade $((trade_index + 1)):"
        echo "Expected: $expect_qty@$expect_price $expect_side"
        echo "Got: $quantity@$price $side"

        if [[ "$type" != "trade" ]]; then
            echo -e "${RED}✗ Not a trade record: $type${NC}"
            failed=1
        fi

        if [[ "$quantity" != "$expect_qty" ]]; then
            echo -e "${RED}✗ Quantity mismatch: expected $expect_qty, got $quantity${NC}"
            failed=1
        fi

        if [[ "$price" != "$expect_price" ]]; then
            echo -e "${RED}✗ Price mismatch: expected $expect_price, got $price${NC}"
            failed=1
        fi

        if [[ "$side" != "$expect_side" ]]; then
            echo -e "${RED}✗ Side mismatch: expected $expect_side, got $side${NC}"
            failed=1
        fi

        trade_index=$((trade_index + 1))
    done <<< "$trades"

    if [ $trade_index -lt $num_expected ]; then
        echo -e "${RED}✗ Expected $num_expected trades, but only got $trade_index${NC}"
        failed=1
    fi

    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}✓ All trades verified successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ Trade verification failed${NC}"
        return 1
    fi
}

# Function to clear order book
clear_book() {
    echo "Clearing order book..."
    # Match any existing orders by placing aggressive orders
    book_data=$(./atra book 9999)
    while IFS=',' read -r type side price quantity; do
        if [[ "$type" == "level" ]]; then
            if [[ "$side" == "ask" ]]; then
                ./atra buy "$quantity@$price"
            else
                ./atra sell "$quantity@$price"
            fi
        fi
    done <<< "$book_data"

    # Verify book is clear
    if [[ -n $(./atra book 10) ]]; then
        echo "Warning: Order book not fully cleared"
        ./atra book 10
    else
        echo "Order book cleared successfully"
    fi
}

# Function to display order book
show_book() {
    echo -e "${YELLOW}Current order book:${NC}"
    ./atra book 10
}

echo -e "${BLUE}Starting matching tests...${NC}"

# Test Suite 1: Simple Matches
echo -e "\n${BLUE}Test Suite 1: Simple Matches${NC}"

clear_book

# Test 1.1: Basic match at same price
echo -e "\n${YELLOW}Test 1.1: Basic match at same price${NC}"
echo "Placing sell order..."
./atra sell 10@100
show_book
verify_trade_sequence "./atra buy 10@100" \
    "10" "100" "bid"
show_book

clear_book

# Test 1.2: Partial match
echo -e "\n${YELLOW}Test 1.2: Partial match${NC}"
echo "Placing sell order..."
./atra sell 20@101
show_book
verify_trade_sequence "./atra buy 10@101" \
    "10" "101" "bid"
show_book

# Test Suite 2: Price Improvement
echo -e "\n${BLUE}Test Suite 2: Price Improvement${NC}"

clear_book

# Test 2.1: Buy price improvement - verify trades in reverse order
echo -e "\n${YELLOW}Test 2.1: Buy price improvement${NC}"
echo "Building fresh order book..."
./atra sell 5@102
./atra sell 5@103
./atra sell 5@104
show_book
verify_trade_sequence "./atra buy 15@104" \
    "5" "104" "bid" \
    "5" "103" "bid" \
    "5" "102" "bid"
show_book

clear_book

# Test 2.2: Multiple fills at same price
echo -e "\n${YELLOW}Test 2.2: Multiple fills at same price${NC}"
echo "Building order book..."
./atra sell 5@105
./atra sell 5@105
show_book
verify_trade_sequence "./atra buy 10@105" \
    "5" "105" "bid" \
    "5" "105" "bid"
show_book

# Test Suite 3: Edge Cases
echo -e "\n${BLUE}Test Suite 3: Edge Cases${NC}"

clear_book

# Test 3.1: Single unit trades
echo -e "\n${YELLOW}Test 3.1: Single unit trades${NC}"
echo "Testing minimum quantity..."
./atra sell 1@106
show_book
verify_trade_sequence "./atra buy 1@106" \
    "1" "106" "bid"
show_book

echo -e "\n${GREEN}All matching tests completed!${NC}"
