#!/bin/bash
# 等待无运行中任务后自动部署：build → 重启 → 冒烟验证 codex 实时用量旁路。
# 输出全部写入 /tmp/agent-hub-deploy.log。
set -u
LOG=/tmp/agent-hub-deploy.log
CARGO=/c/Users/martin/.cargo/bin/cargo.exe
BASE=http://127.0.0.1:8721
cd /d/project/agent-hub || exit 1
: > "$LOG"
log() { echo "[$(date +%H:%M:%S)] $*" >> "$LOG"; }

log "开始等待空闲（最长 40 分钟，每 20 秒轮询）"
for i in $(seq 1 120); do
  n=$(curl -s "$BASE/api/runs" | grep -c '"running":true')
  if [ "$n" = "0" ]; then
    log "无运行中任务，开始部署"
    break
  fi
  if [ "$i" = "120" ]; then
    log "超时：仍有 $n 个任务在运行，放弃部署"
    exit 2
  fi
  sleep 20
done

taskkill //IM agent-hub.exe //F >> "$LOG" 2>&1
sleep 1
"$CARGO" build --release >> "$LOG" 2>&1 || { log "构建失败"; exit 3; }
(./target/release/agent-hub.exe >> /tmp/agent-hub.log 2>&1 &)
sleep 2
curl -s "$BASE/api/models" > /dev/null || { log "服务未响应"; exit 4; }
log "服务已重启"

# 冒烟：codex 只读小任务，验证流中出现 scope=session 的 usage 事件（含 context/window）
curl -s -N -X POST "$BASE/api/chat" -H 'Content-Type: application/json' \
  -d '{"agent":"codex","project":"D:\\project\\agent-hub","prompt":"reply with exactly: ok","session_id":null,"model":null,"permission":"read-only","effort":"low","fast":false}' \
  > /tmp/agent-hub-smoke.ndjson 2>>"$LOG"
u=$(grep -c '"scope":"session"' /tmp/agent-hub-smoke.ndjson)
c=$(grep -c '"context":' /tmp/agent-hub-smoke.ndjson)
d=$(grep -c '"t":"done"' /tmp/agent-hub-smoke.ndjson)
log "冒烟结果：session-scope usage 事件 $u 条，含 context $c 条，done $d"
if [ "$u" -ge 1 ] && [ "$d" -ge 1 ]; then log "部署验证通过"; else log "验证未达预期，看 /tmp/agent-hub-smoke.ndjson"; fi
