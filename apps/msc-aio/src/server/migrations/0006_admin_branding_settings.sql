CREATE TABLE IF NOT EXISTS admin_branding_settings (
    singleton_key TEXT PRIMARY KEY DEFAULT 'default',
    site_name TEXT NOT NULL DEFAULT 'MSC_AIO',
    logo_source TEXT NOT NULL DEFAULT 'app_icon',
    logo_object_key TEXT,
    logo_relative_path TEXT,
    logo_file_name TEXT,
    logo_content_type TEXT,
    logo_backend_label TEXT,
    brand_copy TEXT NOT NULL DEFAULT '顶部品牌区默认使用 App 图标，可切换为上传品牌资产。',
    header_badge TEXT NOT NULL DEFAULT 'Knowledge Workspace',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT admin_branding_settings_singleton_key
        CHECK (singleton_key = 'default'),
    CONSTRAINT admin_branding_settings_logo_source
        CHECK (logo_source IN ('app_icon', 'custom_upload', 'text_only'))
);

INSERT INTO admin_branding_settings (
    singleton_key,
    site_name,
    logo_source,
    brand_copy,
    header_badge
)
VALUES (
    'default',
    'MSC_AIO',
    'app_icon',
    '顶部品牌区默认使用 App 图标，可切换为上传品牌资产。',
    'Knowledge Workspace'
)
ON CONFLICT (singleton_key) DO NOTHING;
