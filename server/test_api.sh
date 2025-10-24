#!/bin/bash

# PC情報収集システム サーバーAPIテストスクリプト
# チケット #9: サーバー統合テスト

API_URL="http://localhost:8080/api/pc-info"

echo "================================"
echo "サーバー統合テスト開始"
echo "================================"
echo ""

# テストケース1: 新規登録（正常系）
echo "【テストケース1】新規登録（正常系）"
echo "UUID: test-uuid-001"
curl -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "uuid": "test-uuid-001",
    "mac_address": "00:11:22:33:44:55",
    "network_type": "Ethernet",
    "user_name": "testuser",
    "ip_address": "192.168.1.100",
    "os": "Windows 11 Pro",
    "os_version": "10.0.22631",
    "model_name": "Test Model 001"
  }'
echo -e "\n"

# テストケース2: 更新（正常系）
echo "【テストケース2】更新（正常系）"
echo "UUID: test-uuid-001（同じUUIDで更新）"
curl -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "uuid": "test-uuid-001",
    "mac_address": "00:11:22:33:44:66",
    "network_type": "Wi-Fi",
    "user_name": "testuser",
    "ip_address": "192.168.1.101",
    "os": "Windows 11 Pro",
    "os_version": "10.0.22631",
    "model_name": "Test Model 001 Updated"
  }'
echo -e "\n"

# テストケース3: 別のPC新規登録（正常系）
echo "【テストケース3】別のPC新規登録（正常系）"
echo "UUID: test-uuid-002"
curl -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "uuid": "test-uuid-002",
    "mac_address": "AA:BB:CC:DD:EE:FF",
    "network_type": "Ethernet",
    "user_name": "testuser2",
    "ip_address": "192.168.1.200",
    "os": "Windows 10 Pro",
    "os_version": "10.0.19045",
    "model_name": "Test Model 002"
  }'
echo -e "\n"

# テストケース4: 必須項目欠損（異常系）
echo "【テストケース4】必須項目欠損（異常系）"
echo "UUIDが空文字列"
curl -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "uuid": "",
    "mac_address": "00:11:22:33:44:55",
    "network_type": "Ethernet",
    "user_name": "testuser",
    "ip_address": "192.168.1.100",
    "os": "Windows 11 Pro",
    "os_version": "10.0.22631",
    "model_name": "Test Model"
  }'
echo -e "\n"

# テストケース5: 不正なJSON（異常系）
echo "【テストケース5】不正なJSON（異常系）"
curl -X POST "$API_URL" \
  -H "Content-Type: application/json" \
  -d '{invalid json}'
echo -e "\n"

echo "================================"
echo "テスト完了"
echo "================================"
echo ""
echo "ログファイル確認: server/server.log"
