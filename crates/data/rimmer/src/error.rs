use thiserror::Error;

/// ORM API 的统一错误类型。
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum OrmError {
    /// 查询尚未指定 select。
    #[error("query selection is required")]
    MissingSelection,

    /// 保存命令没有可写字段。
    #[error("save command has no writable fields")]
    NoWritableFields,

    /// 保存命令缺少主键字段。
    #[error("save command requires id field for update")]
    MissingId,

    /// 实体没有声明主键字段。
    #[error("entity '{entity}' has no id field")]
    EntityHasNoId { entity: &'static str },

    /// Fetcher JSON 序列化失败。
    #[error("failed to serialize fetcher shape: {message}")]
    FetcherJsonSerialize { message: String },

    /// Fetcher JSON 反序列化失败。
    #[error("failed to deserialize fetcher shape: {message}")]
    FetcherJsonDeserialize { message: String },

    /// Fetcher JSON 与目标实体不匹配。
    #[error("fetcher shape targets '{actual}', expected '{expected}'")]
    FetcherEntityMismatch { expected: String, actual: String },

    /// Fetcher 关联元数据不完整。
    #[error("invalid fetcher relation: {message}")]
    InvalidFetcherRelation { message: String },

    /// 图保存缺少父对象关联值。
    #[error("graph save requires parent value for entity '{entity}', column '{column}', collection '{collection}'")]
    GraphSaveMissingParentValue {
        entity: String,
        column: String,
        collection: String,
    },

    /// 数据库执行失败。
    #[error("database execution failed: {message}")]
    Database { message: String },

    /// 数据库行解码失败。
    #[error("database row decode failed: {message}")]
    RowDecode { message: String },

    /// 数据库 URL 方言不受支持。
    #[error("unsupported database dialect for url: {database_url}")]
    UnsupportedDialect { database_url: String },
}

/// ORM API 的 Result 别名。
pub type OrmResult<T> = Result<T, OrmError>;
