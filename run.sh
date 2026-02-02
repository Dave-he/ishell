#!/bin/bash

# iShell 快速启动脚本

cd /Users/hyx/codespace/ishell

echo "🚀 Starting iShell..."
echo ""
echo "📦 Project: iShell - AI-Powered SSH Manager"
echo "📍 Location: $(pwd)"
echo "📝 Version: 0.1.0-mvp"
echo ""

# 检查是否已编译
if [ ! -f "target/debug/ishell" ]; then
    echo "⚙️  First run - compiling project..."
    cargo build
    echo ""
fi

echo "▶️  Launching application..."
echo ""
echo "💡 Tips:"
echo "   - Click '➕ New Connection' to add servers"
echo "   - Try terminal commands: help, ls, date, whoami"
echo "   - Use AI assistant for command suggestions"
echo "   - Press Ctrl+C to stop"
echo ""

# 运行应用
RUST_LOG=info cargo run
