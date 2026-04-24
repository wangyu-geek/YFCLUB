import fs from "node:fs";
import path from "node:path";

function parseArgs(argv) {
  const args = {
    "--store-name": "永丰文体",
    "--default-operator": "系统导入",
    "--legacy-jpdj": "10",
    "--source-version": "legacy-yfwt-txt-1"
  };

  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (!token.startsWith("--")) {
      continue;
    }
    const next = argv[i + 1];
    if (!next || next.startsWith("--")) {
      args[token] = "true";
      continue;
    }
    args[token] = next;
    i += 1;
  }

  return args;
}

function printUsage() {
  console.log(`Usage:
  node scripts/convert-legacy-yfwt-txt.mjs \\
    --members <cluber.txt> \\
    --redemptions <cluberjp.txt> \\
    --gifts <jp.txt> \\
    --out <output.json> \\
    [--gift-map <gift-map.json>] \\
    [--store-name <name>] \\
    [--default-operator <name>] \\
    [--legacy-jpdj <value>] \\
    [--source-version <value>]

Optional gift map JSON format:
{
  "16": {
    "giftName": "荧光笔",
    "pointsCost": 10,
    "pointsUsed": 10,
    "status": "ACTIVE",
    "uniquePerMember": true,
    "remark": "manual mapping"
  }
}`);
}

function ensureRequired(args, keys) {
  const missing = keys.filter((key) => !args[key]);
  if (missing.length > 0) {
    throw new Error(`Missing required arguments: ${missing.join(", ")}`);
  }
}

function readText(filePath) {
  return fs.readFileSync(filePath, "utf8").replace(/^\uFEFF/, "");
}

function parseCsv(text) {
  const rows = [];
  let currentField = "";
  let currentRow = [];
  let inQuotes = false;
  const normalized = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");

  for (let i = 0; i < normalized.length; i += 1) {
    const char = normalized[i];

    if (inQuotes) {
      if (char === "\"") {
        if (normalized[i + 1] === "\"") {
          currentField += "\"";
          i += 1;
        } else {
          inQuotes = false;
        }
      } else {
        currentField += char;
      }
      continue;
    }

    if (char === "\"") {
      inQuotes = true;
      continue;
    }

    if (char === ",") {
      currentRow.push(currentField);
      currentField = "";
      continue;
    }

    if (char === "\n") {
      currentRow.push(currentField);
      currentField = "";
      if (currentRow.some((value) => value.trim() !== "")) {
        rows.push(currentRow);
      }
      currentRow = [];
      continue;
    }

    currentField += char;
  }

  if (currentField !== "" || currentRow.length > 0) {
    currentRow.push(currentField);
    if (currentRow.some((value) => value.trim() !== "")) {
      rows.push(currentRow);
    }
  }

  if (rows.length === 0) {
    return [];
  }

  const headers = rows[0].map((header) => header.trim());
  return rows.slice(1).map((row) => {
    const record = {};
    headers.forEach((header, index) => {
      record[header] = (row[index] ?? "").trim();
    });
    return record;
  });
}

function parseNumber(value) {
  if (value == null || value === "") {
    return null;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function parseInteger(value) {
  const parsed = parseNumber(value);
  return parsed == null ? null : Math.trunc(parsed);
}

function parseBirthday(value) {
  if (!value) {
    return { birthMonth: null, birthDay: null };
  }
  const match = value.match(/^(\d{1,2})月(\d{1,2})日$/);
  if (!match) {
    return { birthMonth: null, birthDay: null };
  }
  return {
    birthMonth: match[1].padStart(2, "0"),
    birthDay: match[2].padStart(2, "0")
  };
}

function buildMemberRemark(row) {
  const parts = [];
  if (row["查询"]) {
    parts.push(`查询码=${row["查询"]}`);
  }

  const giftColumns = ["四等奖", "三等奖", "二等奖", "一等奖"]
    .map((key) => [key, row[key]])
    .filter(([, value]) => value);
  if (giftColumns.length > 0) {
    parts.push(
      `原始奖项列=${giftColumns.map(([key, value]) => `${key}:${value}`).join(" / ")}`
    );
  }

  if (parts.length === 0) {
    return "旧系统 TXT 导入";
  }
  return `旧系统 TXT 导入；${parts.join("；")}`;
}

function loadGiftOverrides(filePath) {
  if (!filePath) {
    return {};
  }
  const content = readText(filePath);
  const parsed = JSON.parse(content);
  return parsed && typeof parsed === "object" ? parsed : {};
}

function toGiftLegacyPk(oldGiftId) {
  return `gift::${oldGiftId}`;
}

function toMemberLegacyPk(memberNo) {
  return `member::${memberNo}`;
}

function inferKnownGiftName(jpRow, override) {
  if (override?.giftName) {
    return String(override.giftName).trim();
  }
  if (jpRow?.["奖品名称"]) {
    return String(jpRow["奖品名称"]).trim();
  }
  return "";
}

function buildGiftCatalog(jpRows, redemptionRows, overrides, warnings) {
  const catalog = new Map();

  for (const row of jpRows) {
    const oldGiftId = row["奖品编号"];
    if (!oldGiftId) {
      continue;
    }
    const override = overrides[oldGiftId];
    const giftName = inferKnownGiftName(row, override);
    if (!giftName) {
      warnings.push(`奖品 ${oldGiftId} 在 jp.txt 中缺少名称，已跳过。`);
      continue;
    }
    catalog.set(oldGiftId, {
      oldGiftId,
      legacyPk: toGiftLegacyPk(oldGiftId),
      giftName,
      pointsCost: parseInteger(override?.pointsCost) ?? 0,
      pointsUsed: parseInteger(override?.pointsUsed) ?? parseInteger(override?.pointsCost) ?? 0,
      status: override?.status ? String(override.status) : "ACTIVE",
      uniquePerMember: Boolean(override?.uniquePerMember ?? false),
      remark:
        typeof override?.remark === "string" && override.remark.trim()
          ? override.remark.trim()
          : `旧系统 jp.txt 导入；原始奖品编号=${oldGiftId}`
    });
  }

  for (const row of redemptionRows) {
    const oldGiftId = row["奖品编号"];
    if (!oldGiftId || catalog.has(oldGiftId)) {
      continue;
    }
    const override = overrides[oldGiftId];
    const giftName =
      (typeof override?.giftName === "string" && override.giftName.trim()) ||
      `历史奖品#${oldGiftId}`;
    warnings.push(`奖品编号 ${oldGiftId} 未在 jp.txt 中找到，已使用占位礼品 ${giftName}。`);
    catalog.set(oldGiftId, {
      oldGiftId,
      legacyPk: toGiftLegacyPk(oldGiftId),
      giftName,
      pointsCost: parseInteger(override?.pointsCost) ?? 0,
      pointsUsed: parseInteger(override?.pointsUsed) ?? parseInteger(override?.pointsCost) ?? 0,
      status: override?.status ? String(override.status) : "ACTIVE",
      uniquePerMember: Boolean(override?.uniquePerMember ?? false),
      remark:
        typeof override?.remark === "string" && override.remark.trim()
          ? override.remark.trim()
          : `旧系统 cluberjp.txt 导入；原始奖品编号=${oldGiftId}；jp.txt 中缺少该奖品定义`
    });
  }

  return catalog;
}

function buildMembers(memberRows, warnings) {
  const members = [];
  const memberPkByNo = new Map();

  for (const row of memberRows) {
    const memberNo = parseInteger(row["会员编号"]);
    const name = (row["姓名"] ?? "").trim();
    if (!memberNo || !name) {
      warnings.push(`跳过无效会员记录：会员编号=${row["会员编号"] ?? ""} 姓名=${row["姓名"] ?? ""}`);
      continue;
    }

    const { birthMonth, birthDay } = parseBirthday(row["生日"]);
    const legacyPk = toMemberLegacyPk(memberNo);
    const member = {
      legacyPk,
      memberNo,
      name,
      gender: row["性别"] ? row["性别"].trim() : null,
      birthMonth,
      birthDay,
      mobile: null,
      pointsBalance: parseInteger(row["积分"]) ?? 0,
      totalSpent: 0,
      lastConsumeAt: null,
      remark: buildMemberRemark(row)
    };

    members.push(member);
    memberPkByNo.set(String(memberNo), legacyPk);
  }

  return { members, memberPkByNo };
}

function buildRedemptions(redemptionRows, memberPkByNo, giftCatalog, warnings) {
  const redemptions = [];
  let sequence = 1;
  const missingMembers = new Map();

  for (const row of redemptionRows) {
    const memberNo = (row["会员编号"] ?? "").trim();
    const oldGiftId = (row["奖品编号"] ?? "").trim();
    const memberLegacyPk = memberPkByNo.get(memberNo);
    if (!memberLegacyPk) {
      missingMembers.set(memberNo, (missingMembers.get(memberNo) ?? 0) + 1);
      continue;
    }

    const gift = giftCatalog.get(oldGiftId);
    if (!gift) {
      warnings.push(`兑换记录已跳过：奖品编号 ${oldGiftId} 未能建立礼品映射。`);
      continue;
    }

    const unresolvedPoints = gift.pointsUsed === 0;
    const remarkParts = [`旧系统 cluberjp.txt 导入；原始奖品编号=${oldGiftId}`];
    if (unresolvedPoints) {
      remarkParts.push("未提供积分成本映射，本条 pointsUsed 按 0 导入，最终余额将由系统自动校正");
    }

    redemptions.push({
      legacyPk: `redeem::${memberNo}::${oldGiftId}::${sequence}`,
      memberLegacyPk,
      giftLegacyPk: gift.legacyPk,
      giftName: gift.giftName,
      qty: 1,
      pointsUsed: gift.pointsUsed,
      operatorName: "系统导入",
      remark: remarkParts.join("；"),
      createdAt: null
    });
    sequence += 1;
  }

  for (const [memberNo, count] of Array.from(missingMembers.entries()).sort((a, b) => Number(a[0]) - Number(b[0]))) {
    warnings.push(`兑换记录已跳过：会员编号 ${memberNo} 不存在于 cluber.txt（共 ${count} 条）。`);
  }

  return redemptions;
}

function ensureDirFor(filePath) {
  const dir = path.dirname(filePath);
  fs.mkdirSync(dir, { recursive: true });
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args["--help"] === "true") {
    printUsage();
    return;
  }

  ensureRequired(args, ["--members", "--redemptions", "--gifts", "--out"]);

  const warnings = [];
  const memberRows = parseCsv(readText(args["--members"]));
  const redemptionRows = parseCsv(readText(args["--redemptions"]));
  const jpRows = parseCsv(readText(args["--gifts"]));
  const giftOverrides = loadGiftOverrides(args["--gift-map"]);

  const { members, memberPkByNo } = buildMembers(memberRows, warnings);
  const giftCatalog = buildGiftCatalog(jpRows, redemptionRows, giftOverrides, warnings);
  const redemptions = buildRedemptions(redemptionRows, memberPkByNo, giftCatalog, warnings);

  const gifts = Array.from(giftCatalog.values()).map((gift) => ({
    legacyPk: gift.legacyPk,
    giftName: gift.giftName,
    pointsCost: gift.pointsCost,
    status: gift.status,
    uniquePerMember: gift.uniquePerMember,
    remark: gift.remark
  }));

  const bundle = {
    sourceVersion: args["--source-version"],
    members,
    consumptions: [],
    gifts,
    redemptions,
    settings: {
      storeName: args["--store-name"],
      pointsRuleAmount: parseInteger(args["--legacy-jpdj"]) ?? 10,
      legacyJpdj: args["--legacy-jpdj"],
      defaultOperator: args["--default-operator"]
    }
  };

  ensureDirFor(args["--out"]);
  fs.writeFileSync(args["--out"], `${JSON.stringify(bundle, null, 2)}\n`, "utf8");

  const unresolvedGiftIds = Array.from(giftCatalog.values())
    .filter((gift) => gift.pointsUsed === 0)
    .map((gift) => gift.oldGiftId);

  console.log(`Converted legacy TXT files to ${args["--out"]}`);
  console.log(`members=${members.length} gifts=${gifts.length} redemptions=${redemptions.length}`);

  if (warnings.length > 0) {
    console.log("\nWarnings:");
    for (const warning of warnings) {
      console.log(`- ${warning}`);
    }
  }

  if (unresolvedGiftIds.length > 0) {
    console.log("\nGift IDs without points mapping:");
    console.log(`- ${Array.from(new Set(unresolvedGiftIds)).sort((a, b) => Number(a) - Number(b)).join(", ")}`);
    console.log("- You can rerun with --gift-map <file> to provide giftName / pointsCost / pointsUsed overrides.");
  }
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  printUsage();
  process.exitCode = 1;
}
