CREATE TABLE IF NOT EXISTS schema_versions (
  version_no INTEGER PRIMARY KEY,
  description TEXT NOT NULL,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS members (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  member_no INTEGER NOT NULL UNIQUE,
  name TEXT NOT NULL,
  gender TEXT,
  birth_month TEXT,
  birth_day TEXT,
  mobile TEXT,
  name_pinyin TEXT,
  name_initials TEXT,
  points_balance INTEGER NOT NULL DEFAULT 0,
  total_spent NUMERIC NOT NULL DEFAULT 0,
  last_consume_at TEXT,
  status TEXT NOT NULL DEFAULT 'ACTIVE',
  remark TEXT,
  legacy_member_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_members_mobile ON members(mobile);
CREATE INDEX IF NOT EXISTS idx_members_name ON members(name);
CREATE INDEX IF NOT EXISTS idx_members_initials ON members(name_initials);
CREATE INDEX IF NOT EXISTS idx_members_legacy_member_id ON members(legacy_member_id);

CREATE TABLE IF NOT EXISTS consumption_records (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  record_no TEXT NOT NULL UNIQUE,
  member_id INTEGER NOT NULL,
  amount NUMERIC NOT NULL,
  points_added INTEGER NOT NULL DEFAULT 0,
  operator_name TEXT NOT NULL,
  remark TEXT,
  legacy_record_id TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(member_id) REFERENCES members(id)
);

CREATE INDEX IF NOT EXISTS idx_consumption_member_id ON consumption_records(member_id);
CREATE INDEX IF NOT EXISTS idx_consumption_created_at ON consumption_records(created_at);
CREATE INDEX IF NOT EXISTS idx_consumption_legacy_record_id ON consumption_records(legacy_record_id);

CREATE TABLE IF NOT EXISTS points_ledger (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  member_id INTEGER NOT NULL,
  change_type TEXT NOT NULL,
  points_delta INTEGER NOT NULL,
  balance_after INTEGER NOT NULL,
  source_type TEXT NOT NULL,
  source_id TEXT,
  operator_name TEXT NOT NULL,
  remark TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(member_id) REFERENCES members(id)
);

CREATE INDEX IF NOT EXISTS idx_points_ledger_member_id ON points_ledger(member_id);
CREATE INDEX IF NOT EXISTS idx_points_ledger_source ON points_ledger(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_points_ledger_created_at ON points_ledger(created_at);

CREATE TABLE IF NOT EXISTS gifts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  gift_name TEXT NOT NULL,
  points_cost INTEGER NOT NULL,
  stock_qty INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'ACTIVE',
  remark TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_gifts_status ON gifts(status);
CREATE INDEX IF NOT EXISTS idx_gifts_name ON gifts(gift_name);

CREATE TABLE IF NOT EXISTS gift_redemptions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  redeem_no TEXT NOT NULL UNIQUE,
  member_id INTEGER NOT NULL,
  gift_id INTEGER,
  gift_name_snapshot TEXT NOT NULL,
  qty INTEGER NOT NULL DEFAULT 1,
  points_used INTEGER NOT NULL,
  operator_name TEXT NOT NULL,
  remark TEXT,
  legacy_redemption_id TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(member_id) REFERENCES members(id),
  FOREIGN KEY(gift_id) REFERENCES gifts(id)
);

CREATE INDEX IF NOT EXISTS idx_redemptions_member_id ON gift_redemptions(member_id);
CREATE INDEX IF NOT EXISTS idx_redemptions_created_at ON gift_redemptions(created_at);
CREATE INDEX IF NOT EXISTS idx_redemptions_legacy_redemption_id ON gift_redemptions(legacy_redemption_id);

CREATE TABLE IF NOT EXISTS sys_settings (
  setting_key TEXT PRIMARY KEY,
  setting_value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS operators (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  login_name TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  password_hash TEXT,
  role_code TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'ACTIVE',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS operation_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  operator_name TEXT NOT NULL,
  module_name TEXT NOT NULL,
  action_name TEXT NOT NULL,
  target_type TEXT,
  target_id TEXT,
  request_summary TEXT,
  result_status TEXT NOT NULL,
  error_message TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_operation_logs_created_at ON operation_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_operation_logs_module ON operation_logs(module_name);

CREATE TABLE IF NOT EXISTS migration_batches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  batch_no TEXT NOT NULL UNIQUE,
  source_file TEXT NOT NULL,
  source_file_hash TEXT NOT NULL,
  source_version TEXT,
  import_scope TEXT NOT NULL,
  batch_fingerprint TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL,
  success_count INTEGER NOT NULL DEFAULT 0,
  failed_count INTEGER NOT NULL DEFAULT 0,
  error_message TEXT,
  created_at TEXT NOT NULL,
  completed_at TEXT
);

CREATE TABLE IF NOT EXISTS migration_entity_map (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  batch_no TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  legacy_pk TEXT NOT NULL,
  target_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(entity_type, legacy_pk)
);

CREATE INDEX IF NOT EXISTS idx_migration_entity_batch ON migration_entity_map(batch_no);

CREATE TABLE IF NOT EXISTS migration_errors (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  batch_no TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  legacy_pk TEXT,
  error_code TEXT NOT NULL,
  error_message TEXT NOT NULL,
  raw_payload TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_migration_errors_batch ON migration_errors(batch_no);
