import { useEffect, useMemo, useState } from "react";
import {
    Bot,
    Loader2,
    Palette,
    Save,
    Settings2,
    ShieldCheck,
} from "lucide-react";
import {
    getApiBaseUrl,
    type BrandingLogoSource,
    type BrandingSettingsDto,
} from "@addzero/api-client";
import {
    Button,
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
    Input,
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
    Textarea,
} from "@addzero/ui";

interface OpenAiChatConfigDto {
    base_url: string;
    api_key: string;
    model: string;
}

const CONFIG_FILE_ITEMS = [
    {
        title: "数据库连接",
        detail: "~/.config/aio/aio.env 中的 MSC_AIO_DATABASE_URL / DATABASE_URL",
        status: "文件入口",
    },
    {
        title: "对象存储",
        detail: "AIO_MINIO_* 环境变量目前仍通过运行环境注入",
        status: "待前台化",
    },
    {
        title: "任意软件配置文件",
        detail: "json / toml / yaml 通用文件编辑器后续接入资源层",
        status: "待实现",
    },
];

export default function EnvPage() {
    const baseUrl = useMemo(() => getApiBaseUrl(), []);
    const [branding, setBranding] = useState<BrandingSettingsDto | null>(null);
    const [openAi, setOpenAi] = useState<OpenAiChatConfigDto | null>(null);
    const [loading, setLoading] = useState(true);
    const [savingBranding, setSavingBranding] = useState(false);
    const [savingOpenAi, setSavingOpenAi] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [message, setMessage] = useState<string | null>(null);

    useEffect(() => {
        let cancelled = false;

        async function load() {
            setLoading(true);
            setError(null);
            try {
                const [brandingData, openAiData] = await Promise.all([
                    fetch(`${baseUrl}/api/admin/settings/branding`, {
                        credentials: "include",
                    }).then((res) => {
                        if (!res.ok) throw new Error(`HTTP ${res.status}`);
                        return res.json() as Promise<BrandingSettingsDto>;
                    }),
                    fetch(`${baseUrl}/api/openai-chat/config`, {
                        credentials: "include",
                    }).then((res) => {
                        if (!res.ok) throw new Error(`HTTP ${res.status}`);
                        return res.json() as Promise<OpenAiChatConfigDto>;
                    }),
                ]);
                if (cancelled) return;
                setBranding(brandingData);
                setOpenAi(openAiData);
            } catch (err) {
                if (!cancelled) {
                    setError(err instanceof Error ? err.message : "加载配置失败");
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        load();
        return () => {
            cancelled = true;
        };
    }, [baseUrl]);

    async function saveBranding() {
        if (!branding) return;
        setSavingBranding(true);
        setError(null);
        setMessage(null);
        try {
            const saved = await fetch(`${baseUrl}/api/admin/settings/branding`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(branding),
            }).then((res) => {
                if (!res.ok) throw new Error("保存品牌配置失败");
                return res.json() as Promise<BrandingSettingsDto>;
            });
            setBranding(saved);
            setMessage("品牌配置已保存");
        } catch (err) {
            setError(err instanceof Error ? err.message : "保存品牌配置失败");
        } finally {
            setSavingBranding(false);
        }
    }

    async function saveOpenAi() {
        if (!openAi) return;
        setSavingOpenAi(true);
        setError(null);
        setMessage(null);
        try {
            const saved = await fetch(`${baseUrl}/api/openai-chat/config`, {
                method: "POST",
                credentials: "include",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(openAi),
            }).then((res) => {
                if (!res.ok) throw new Error("保存模型配置失败");
                return res.json() as Promise<OpenAiChatConfigDto>;
            });
            setOpenAi(saved);
            setMessage("模型配置已保存");
        } catch (err) {
            setError(err instanceof Error ? err.message : "保存模型配置失败");
        } finally {
            setSavingOpenAi(false);
        }
    }

    return (
        <div className="space-y-8">
            <Card>
                <CardHeader className="border-b">
                    <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                        <Settings2 className="h-3.5 w-3.5" />
                        Configuration Workbench
                    </div>
                    <CardTitle className="mt-3 text-3xl tracking-tight">
                        配置工作台
                    </CardTitle>
                    <CardDescription className="mt-2 max-w-3xl text-sm">
                        软件配置不能只靠环境变量和手改文件。前台必须能设置正式配置项。
                        这一页先接通已经有后端能力的品牌配置和 OpenAI 配置，并给其余配置文件留出统一入口。
                    </CardDescription>
                </CardHeader>

                <CardContent className="grid gap-0 p-0 md:grid-cols-3">
                    <TopSignal
                        title="品牌配置"
                        detail="站点名、品牌文案、顶部徽标来源"
                        icon={<Palette className="h-4 w-4" />}
                    />
                    <TopSignal
                        title="模型配置"
                        detail="base_url、api_key、model 直接前台维护"
                        icon={<Bot className="h-4 w-4" />}
                    />
                    <TopSignal
                        title="文件配置"
                        detail="逐步把 env / json / toml / yaml 收进统一配置面板"
                        icon={<ShieldCheck className="h-4 w-4" />}
                    />
                </CardContent>
            </Card>

            {loading ? (
                <div className="flex items-center justify-center py-20">
                    <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
            ) : (
                <section className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
                    <div className="space-y-6">
                        <ConfigPanel
                            title="品牌配置"
                            description="这部分已经有正式 API，可直接在前台维护。"
                            action={
                                <Button
                                    type="button"
                                    onClick={saveBranding}
                                    disabled={!branding || savingBranding}
                                    size="sm"
                                >
                                    {savingBranding ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <Save className="h-4 w-4" />
                                    )}
                                    保存
                                </Button>
                            }
                        >
                            {branding ? (
                                <div className="grid gap-4">
                                    <Field label="站点名称">
                                        <Input
                                            value={branding.site_name}
                                            onChange={(event) =>
                                                setBranding({
                                                    ...branding,
                                                    site_name: event.target.value,
                                                })
                                            }
                                        />
                                    </Field>

                                    <Field label="Logo 来源">
                                        <Select
                                            value={branding.logo_source}
                                            onValueChange={(value) =>
                                                setBranding({
                                                    ...branding,
                                                    logo_source: value as BrandingLogoSource,
                                                })
                                            }
                                        >
                                            <SelectTrigger>
                                                <SelectValue placeholder="选择 Logo 来源" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="app_icon">App 图标</SelectItem>
                                                <SelectItem value="custom_upload">自定义上传</SelectItem>
                                                <SelectItem value="text_only">仅文字</SelectItem>
                                            </SelectContent>
                                        </Select>
                                    </Field>

                                    <Field label="品牌文案">
                                        <Textarea
                                            className="min-h-24"
                                            value={branding.brand_copy}
                                            onChange={(event) =>
                                                setBranding({
                                                    ...branding,
                                                    brand_copy: event.target.value,
                                                })
                                            }
                                        />
                                    </Field>

                                    <Field label="顶部徽标文案">
                                        <Input
                                            value={branding.header_badge}
                                            onChange={(event) =>
                                                setBranding({
                                                    ...branding,
                                                    header_badge: event.target.value,
                                                })
                                            }
                                        />
                                    </Field>
                                </div>
                            ) : null}
                        </ConfigPanel>

                        <ConfigPanel
                            title="模型配置"
                            description="OpenAI 兼容接口配置已经支持读写，不需要再靠手动编辑 json 文件。"
                            action={
                                <Button
                                    type="button"
                                    onClick={saveOpenAi}
                                    disabled={!openAi || savingOpenAi}
                                    size="sm"
                                >
                                    {savingOpenAi ? (
                                        <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                        <Save className="h-4 w-4" />
                                    )}
                                    保存
                                </Button>
                            }
                        >
                            {openAi ? (
                                <div className="grid gap-4">
                                    <Field label="Base URL">
                                        <Input
                                            value={openAi.base_url}
                                            onChange={(event) =>
                                                setOpenAi({
                                                    ...openAi,
                                                    base_url: event.target.value,
                                                })
                                            }
                                        />
                                    </Field>
                                    <Field label="API Key">
                                        <Input
                                            type="password"
                                            value={openAi.api_key}
                                            onChange={(event) =>
                                                setOpenAi({
                                                    ...openAi,
                                                    api_key: event.target.value,
                                                })
                                            }
                                        />
                                    </Field>
                                    <Field label="Model">
                                        <Input
                                            value={openAi.model}
                                            onChange={(event) =>
                                                setOpenAi({
                                                    ...openAi,
                                                    model: event.target.value,
                                                })
                                            }
                                        />
                                    </Field>
                                </div>
                            ) : null}
                        </ConfigPanel>
                    </div>

                    <div className="space-y-6">
                        <ConfigPanel
                            title="配置文件前台化路线"
                            description="不是所有软件配置都已经接通，但边界要明确。"
                        >
                            <div className="space-y-3">
                                {CONFIG_FILE_ITEMS.map((item) => (
                                    <div
                                        key={item.title}
                                        className="rounded-lg border px-4 py-3"
                                    >
                                        <div className="flex items-center justify-between gap-3">
                                            <div className="text-sm font-medium">
                                                {item.title}
                                            </div>
                                            <span className="rounded-md bg-muted px-2 py-1 text-[11px] font-medium text-muted-foreground">
                                                {item.status}
                                            </span>
                                        </div>
                                        <p className="mt-2 text-sm text-muted-foreground">
                                            {item.detail}
                                        </p>
                                    </div>
                                ))}
                            </div>
                        </ConfigPanel>

                        {(error || message) && (
                            <div
                                className={`rounded-lg border px-4 py-3 text-sm ${
                                    error
                                        ? "border-destructive/30 bg-destructive/10 text-destructive"
                                        : "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-400"
                                }`}
                            >
                                {error ?? message}
                            </div>
                        )}
                    </div>
                </section>
            )}
        </div>
    );
}

function ConfigPanel({
    title,
    description,
    action,
    children,
}: {
    title: string;
    description: string;
    action?: React.ReactNode;
    children: React.ReactNode;
}) {
    return (
        <Card>
            <CardHeader className="flex flex-row items-start justify-between gap-4 border-b space-y-0">
                <div>
                    <CardTitle className="text-base">{title}</CardTitle>
                    <CardDescription className="mt-1 text-sm">
                        {description}
                    </CardDescription>
                </div>
                {action}
            </CardHeader>
            <CardContent className="p-5">{children}</CardContent>
        </Card>
    );
}

function Field({
    label,
    children,
}: {
    label: string;
    children: React.ReactNode;
}) {
    return (
        <label className="block">
            <div className="mb-2 text-sm font-medium">{label}</div>
            {children}
        </label>
    );
}

function TopSignal({
    title,
    detail,
    icon,
}: {
    title: string;
    detail: string;
    icon: React.ReactNode;
}) {
    return (
        <div className="border-t px-5 py-4 first:border-t-0 md:border-t-0 md:border-l first:md:border-l-0">
            <div className="flex items-center gap-2 text-sm font-medium">
                <span className="text-muted-foreground">{icon}</span>
                {title}
            </div>
            <p className="mt-2 text-sm text-muted-foreground">{detail}</p>
        </div>
    );
}
