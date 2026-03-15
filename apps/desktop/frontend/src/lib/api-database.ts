import { tauriInvoke } from './invoke';

export interface DbTable {
  name: string;
  kind: string;
}

export interface DbColumn {
  cid: number;
  name: string;
  dataType: string;
  notNull: boolean;
  defaultValue: string | null;
  primaryKey: boolean;
}

export interface DbQueryResult {
  columns: string[];
  rows: unknown[][];
  rowsAffected: number | null;
  durationMs: number;
}

export type SqliteTable = DbTable;
export type SqliteColumn = DbColumn;
export type SqliteQueryResult = DbQueryResult;

export interface MysqlConnection {
  host: string;
  port: number;
  username: string;
  password?: string;
  encrypted_password?: string;
  database: string;
}

export interface PostgresConnection {
  host: string;
  port: number;
  username: string;
  password?: string;
  encrypted_password?: string;
  database: string;
  schema?: string;
}

export interface MssqlConnection {
  host: string;
  port: number;
  username: string;
  password?: string;
  encrypted_password?: string;
  database: string;
  schema?: string;
  trustServerCertificate?: boolean;
  encrypt?: boolean;
}

export interface ClickHouseConnection {
  host: string;
  port: number;
  username?: string;
  password?: string;
  encrypted_password?: string;
  database: string;
  https?: boolean;
}

export interface OracleConnection {
  connectionString: string;
  username?: string;
  password?: string;
  encrypted_password?: string;
}

export interface RedisConnection {
  host: string;
  port: number;
  username?: string;
  password?: string;
  encrypted_password?: string;
  db?: number;
}

export interface RedisKeyInfo {
  key: string;
  keyType: string;
}

export interface RedisKeyDetail {
  key: string;
  keyType: string;
  ttlSeconds: number | null;
  encoding?: string | null;
  memoryUsageBytes?: number | null;
  length?: number | null;
  metaError?: string | null;
  value: unknown;
}

export interface RedisCommandResult {
  result: unknown;
  durationMs: number;
}

export const databaseApi = {
  // SQLite
  sqliteListTables: (dbPath: string) =>
    tauriInvoke<SqliteTable[]>('db_sqlite_list_tables', { dbPath }),
  sqliteListColumns: (dbPath: string, table: string) =>
    tauriInvoke<SqliteColumn[]>('db_sqlite_list_columns', { dbPath, tableName: table }),
  sqliteQuery: (dbPath: string, sql: string) =>
    tauriInvoke<SqliteQueryResult>('db_sqlite_query', { dbPath, sql }),

  // MySQL / MariaDB
  mysqlListTables: (conn: MysqlConnection) =>
    tauriInvoke<DbTable[]>('db_mysql_list_tables', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
      },
    }),
  mysqlListColumns: (conn: MysqlConnection, tableName: string) =>
    tauriInvoke<DbColumn[]>('db_mysql_list_columns', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        table_name: tableName,
      },
    }),
  mysqlQuery: (conn: MysqlConnection, sql: string) =>
    tauriInvoke<DbQueryResult>('db_mysql_query', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        sql,
      },
    }),

  // PostgreSQL
  postgresListTables: (conn: PostgresConnection) =>
    tauriInvoke<DbTable[]>('db_postgres_list_tables', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        schema: conn.schema ?? null,
      },
    }),
  postgresListColumns: (conn: PostgresConnection, tableName: string) =>
    tauriInvoke<DbColumn[]>('db_postgres_list_columns', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        schema: conn.schema ?? null,
        table_name: tableName,
      },
    }),
  postgresQuery: (conn: PostgresConnection, sql: string) =>
    tauriInvoke<DbQueryResult>('db_postgres_query', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        schema: conn.schema ?? null,
        sql,
      },
    }),

  // SQL Server
  mssqlListTables: (conn: MssqlConnection) =>
    tauriInvoke<DbTable[]>('db_mssql_list_tables', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        schema: conn.schema ?? null,
        trust_server_certificate: conn.trustServerCertificate ?? null,
        encrypt: conn.encrypt ?? null,
      },
    }),
  mssqlListColumns: (conn: MssqlConnection, tableName: string) =>
    tauriInvoke<DbColumn[]>('db_mssql_list_columns', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        schema: conn.schema ?? null,
        trust_server_certificate: conn.trustServerCertificate ?? null,
        encrypt: conn.encrypt ?? null,
        table_name: tableName,
      },
    }),
  mssqlQuery: (conn: MssqlConnection, sql: string) =>
    tauriInvoke<DbQueryResult>('db_mssql_query', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        schema: conn.schema ?? null,
        trust_server_certificate: conn.trustServerCertificate ?? null,
        encrypt: conn.encrypt ?? null,
        sql,
      },
    }),

  // ClickHouse
  clickhouseListTables: (conn: ClickHouseConnection) =>
    tauriInvoke<DbTable[]>('db_clickhouse_list_tables', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        https: conn.https ?? null,
      },
    }),
  clickhouseListColumns: (conn: ClickHouseConnection, tableName: string) =>
    tauriInvoke<DbColumn[]>('db_clickhouse_list_columns', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        https: conn.https ?? null,
        table_name: tableName,
      },
    }),
  clickhouseQuery: (conn: ClickHouseConnection, sql: string) =>
    tauriInvoke<DbQueryResult>('db_clickhouse_query', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        database: conn.database,
        https: conn.https ?? null,
        sql,
      },
    }),

  // Oracle (ODBC)
  oracleListTables: (conn: OracleConnection) =>
    tauriInvoke<DbTable[]>('db_oracle_list_tables', {
      req: {
        connection_string: conn.connectionString,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
      },
    }),
  oracleListColumns: (conn: OracleConnection, tableName: string) =>
    tauriInvoke<DbColumn[]>('db_oracle_list_columns', {
      req: {
        connection_string: conn.connectionString,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        table_name: tableName,
      },
    }),
  oracleQuery: (conn: OracleConnection, sql: string) =>
    tauriInvoke<DbQueryResult>('db_oracle_query', {
      req: {
        connection_string: conn.connectionString,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        sql,
      },
    }),

  // Redis
  redisScanKeys: (conn: RedisConnection, opts?: { pattern?: string; limit?: number }) =>
    tauriInvoke<RedisKeyInfo[]>('db_redis_scan_keys', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        db: conn.db ?? null,
        pattern: opts?.pattern ?? null,
        limit: opts?.limit ?? null,
      },
    }),
  redisGetKey: (conn: RedisConnection, key: string, previewLimit?: number) =>
    tauriInvoke<RedisKeyDetail>('db_redis_get_key', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        db: conn.db ?? null,
        key,
        preview_limit: previewLimit ?? null,
      },
    }),
  redisCommand: (conn: RedisConnection, args: string[]) =>
    tauriInvoke<RedisCommandResult>('db_redis_command', {
      req: {
        host: conn.host,
        port: conn.port,
        username: conn.username ?? null,
        password: conn.password ?? null,
        encrypted_password: conn.encrypted_password ?? null,
        db: conn.db ?? null,
        args,
      },
    }),
};
