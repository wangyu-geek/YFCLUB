# 旧版 TXT 迁移脚本

项目已新增脚本：`scripts/convert-legacy-yfwt-txt.mjs`

用途：

- 读取旧版 `cluber.txt`、`cluberjp.txt`、`jp.txt`
- 转换为当前系统可导入的 JSON 迁移文件
- 后续再在 YFCLUB 的“数据迁移”页面执行预检和正式导入

## 用法

```bash
node scripts/convert-legacy-yfwt-txt.mjs \
  --members "D:\projects\yfwt\cluber.txt" \
  --redemptions "D:\projects\yfwt\cluberjp.txt" \
  --gifts "D:\projects\yfwt\jp.txt" \
  --out "D:\projects\yfwt\legacy-yfwt-import.json"
```

## 可选参数

```bash
--gift-map <file>
--store-name <name>
--default-operator <name>
--legacy-jpdj <value>
--source-version <value>
```

## gift-map 文件

由于 `cluberjp.txt` 中部分旧奖品编号并不出现在 `jp.txt` 中，脚本会为这些奖品生成占位礼品。

如果已经确认旧奖品编号和积分成本，可提供一个 JSON 文件覆盖：

```json
{
  "16": {
    "giftName": "荧光笔",
    "pointsCost": 10,
    "pointsUsed": 10,
    "status": "ACTIVE",
    "uniquePerMember": true,
    "remark": "人工确认映射"
  }
}
```

然后执行：

```bash
node scripts/convert-legacy-yfwt-txt.mjs \
  --members "D:\projects\yfwt\cluber.txt" \
  --redemptions "D:\projects\yfwt\cluberjp.txt" \
  --gifts "D:\projects\yfwt\jp.txt" \
  --gift-map "D:\projects\yfwt\gift-map.json" \
  --out "D:\projects\yfwt\legacy-yfwt-import.json"
```

## 说明

- `cluber.txt` 作为会员主数据来源
- `cluberjp.txt` 作为历史兑换记录来源
- `jp.txt` 作为旧奖品主数据来源
- 当前脚本不生成消费记录，输出文件中的 `consumptions` 为空
- 当旧兑换记录缺少可确认的积分成本时，脚本会将 `pointsUsed` 设为 `0`，最终会员积分会在导入时通过“余额校正”恢复到 `cluber.txt` 中的积分值
