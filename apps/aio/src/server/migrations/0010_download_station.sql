-- Download Station 文件索引表
CREATE TABLE IF NOT EXISTS download_station_files (
    id BIGSERIAL PRIMARY KEY,
    source VARCHAR(255) NOT NULL,  -- 来源目录名称（如 Downloads, Nextcloud）
    path VARCHAR(1024) NOT NULL,   -- 相对路径
    full_path VARCHAR(2048) NOT NULL, -- 完整路径
    name VARCHAR(512) NOT NULL,    -- 文件名
    dir VARCHAR(1024),             -- 所在目录
    size BIGINT NOT NULL,          -- 文件大小（字节）
    ext VARCHAR(32),               -- 扩展名
    category VARCHAR(64),          -- 分类（视频、音频、图片等）
    mtime TIMESTAMP NOT NULL,      -- 修改时间
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_download_station_files_source ON download_station_files(source);
CREATE INDEX IF NOT EXISTS idx_download_station_files_category ON download_station_files(category);
CREATE INDEX IF NOT EXISTS idx_download_station_files_ext ON download_station_files(ext);
CREATE INDEX IF NOT EXISTS idx_download_station_files_name ON download_station_files(name);
CREATE INDEX IF NOT EXISTS idx_download_station_files_mtime ON download_station_files(mtime DESC);

-- Download Station 分享链接表
CREATE TABLE IF NOT EXISTS download_station_shares (
    id BIGSERIAL PRIMARY KEY,
    token VARCHAR(32) UNIQUE NOT NULL,  -- 分享令牌
    source VARCHAR(255) NOT NULL,       -- 来源目录
    path VARCHAR(1024) NOT NULL,        -- 文件路径
    file_name VARCHAR(512) NOT NULL,    -- 文件名
    expires_at TIMESTAMP NOT NULL,      -- 过期时间
    created_at TIMESTAMP DEFAULT NOW(),
    created_by VARCHAR(255)             -- 创建者
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_download_station_shares_token ON download_station_shares(token);
CREATE INDEX IF NOT EXISTS idx_download_station_shares_expires ON download_station_shares(expires_at);

-- Download Station 配置表
CREATE TABLE IF NOT EXISTS download_station_config (
    id SERIAL PRIMARY KEY,
    key VARCHAR(128) UNIQUE NOT NULL,
    value TEXT,
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 插入默认配置
INSERT INTO download_station_config (key, value) VALUES
    ('directories', '["~/Downloads", "~/Nextcloud"]'),
    ('upload_dir', '~/Downloads'),
    ('scan_interval', '300')
ON CONFLICT (key) DO NOTHING;
