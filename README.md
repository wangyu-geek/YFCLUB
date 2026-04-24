# 永丰文体会员管理系统

基于 `Tauri 2 + Rust + Vue 3 + SQLite` 的单机绿色版会员管理系统，实现了以下核心能力：

- 会员新增、编辑、停用与统一搜索
- 消费登记、积分累计与积分流水
- 礼品维护与积分兑换
- JSON 迁移文件预检、正式导入、批次幂等保护与导入报告
- 本地数据库备份、恢复前保护性备份
- 门店参数设置、基础报表与操作日志

## 本地运行

```bash
npm install
npm run tauri:dev
```

## 打包

```bash
npm run tauri:build
```

## 目录说明

- `src/`：Vue 前端页面与交互逻辑
- `src-tauri/`：Rust 后端、SQLite 事务与桌面配置
- `docs/`：补充说明文档

## 迁移文件

当前首版实现支持通过 JSON 中间文件导入旧数据，格式说明见：

- [docs/迁移文件格式.md](/d:/projects/windows/YFCLUB/docs/迁移文件格式.md)
