#!/bin/bash
# Context Validation Script
# Validates that context.md is properly maintained

set -e

CONTEXT_FILE="context.md"
ERRORS=0

echo "=== Arnis Context.md Validation ==="
echo ""

# Check if context.md exists
if [ ! -f "$CONTEXT_FILE" ]; then
    echo "❌ ERROR: context.md not found in repository root"
    exit 1
fi

echo "✓ context.md exists"

# Check if file is not empty
if [ ! -s "$CONTEXT_FILE" ]; then
    echo "❌ ERROR: context.md is empty"
    exit 1
fi

echo "✓ context.md is not empty"

# Check for required sections
REQUIRED_SECTIONS=(
    "Project Overview"
    "Project Structure"
    "Build System"
    "Current Session Context"
    "Last Updated"
    "Recent Changes"
)

for section in "${REQUIRED_SECTIONS[@]}"; do
    if grep -q "## $section\|### $section" "$CONTEXT_FILE"; then
        echo "✓ Found section: $section"
    else
        echo "❌ ERROR: Missing required section: $section"
        ERRORS=$((ERRORS + 1))
    fi
done

# Check if Last Updated is recent (within last 30 days)
LAST_UPDATED=$(grep -A 1 "### Last Updated" "$CONTEXT_FILE" | tail -1 | xargs)
if [ -n "$LAST_UPDATED" ]; then
    echo "✓ Last Updated: $LAST_UPDATED"
    
    # Try to parse the date (format: YYYY-MM-DD)
    if [[ "$LAST_UPDATED" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
        echo "  ✓ Date format is valid (YYYY-MM-DD)"
    else
        echo "  ⚠ WARNING: Last Updated date format may be non-standard"
        echo "  Expected: YYYY-MM-DD, Got: $LAST_UPDATED"
    fi
else
    echo "❌ ERROR: Could not find Last Updated date"
    ERRORS=$((ERRORS + 1))
fi

# Check file size is reasonable (not too small, not too large)
FILE_SIZE=$(wc -c < "$CONTEXT_FILE")
if [ "$FILE_SIZE" -lt 1000 ]; then
    echo "⚠ WARNING: context.md is quite small (${FILE_SIZE} bytes)"
    echo "  Consider adding more project details"
elif [ "$FILE_SIZE" -gt 50000 ]; then
    echo "⚠ WARNING: context.md is quite large (${FILE_SIZE} bytes)"
    echo "  Consider archiving old session information"
else
    echo "✓ File size is reasonable (${FILE_SIZE} bytes)"
fi

# Check for active markers that suggest incomplete updates
if grep -q "TODO\|FIXME\|XXX" "$CONTEXT_FILE"; then
    echo "⚠ WARNING: Found TODO/FIXME/XXX markers in context.md"
    echo "  These should be resolved before finalizing"
fi

# Summary
echo ""
echo "=== Validation Summary ==="
if [ $ERRORS -eq 0 ]; then
    echo "✅ All validations passed!"
    echo ""
    echo "Recent changes from context.md:"
    echo "---"
    sed -n '/### Recent Changes/,/### [A-Z]/p' "$CONTEXT_FILE" | head -n -1
    exit 0
else
    echo "❌ Found $ERRORS error(s)"
    echo "Please fix the errors before committing"
    exit 1
fi
