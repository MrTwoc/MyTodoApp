#!/bin/bash
set -e

# 启动后端进程
cd /root/myTodoProject/MyTodoApp/backend
cargo run --release &
BACKEND_PID=$!

# 等待服务器启动（最大10秒）
for i in {1..10}; do
    if ss -tlnp | grep :8698 > /dev/null; then
        echo "Server is up"
        break
    fi
    sleep 1
done

# 发送注册请求
curl -X POST http://localhost:8698/api/users/register \
    -H "Content-Type: application/json" \
    -d '{"username":"testuser","password":"Testpass123","email":"test@example.com","phone":"1234567890"}' \
    -v

# 杀死后端
kill $BACKEND_PID 2>/dev/null || true
wait $BACKEND_PID 2>/dev/null || true