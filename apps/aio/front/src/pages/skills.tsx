import { useEffect, useState } from "react";
import { Brain, Search, Loader2 } from "lucide-react";
import { getApiBaseUrl } from "@addzero/api-client";
import {
  Badge,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  ScrollArea,
} from "@addzero/ui";

interface Skill {
  name: string;
  keywords: string[];
  description: string;
  body: string;
  source: string;
  updated_at: string;
}

export default function SkillsPage() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<Skill | null>(null);

  useEffect(() => {
    const baseUrl = getApiBaseUrl();
    fetch(`${baseUrl}/api/skills`, { credentials: "include" })
      .then((r) => r.json())
      .then((data) => setSkills(data))
      .finally(() => setLoading(false));
  }, []);

  const filtered = skills.filter(
    (s) =>
      !search ||
      s.name.toLowerCase().includes(search.toLowerCase()) ||
      s.description.toLowerCase().includes(search.toLowerCase()) ||
      s.keywords.some((k) => k.toLowerCase().includes(search.toLowerCase())),
  );

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Skills</h1>
        <p className="mt-1 text-muted-foreground">
          {skills.length} 个技能已加载
        </p>
      </div>

      <div className="flex gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜索技能..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-10"
          />
        </div>
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-20">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {filtered.map((skill) => (
            <Card
              key={skill.name}
              className="cursor-pointer transition hover:border-primary/50"
              onClick={() => setSelected(skill === selected ? null : skill)}
            >
                <CardHeader>
                <CardTitle className="flex items-center gap-2 text-lg">
                  <Brain className="h-4 w-4" />
                  {skill.name}
                </CardTitle>
                <CardDescription className="line-clamp-2">
                  {skill.description}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-1.5">
                  {skill.keywords.map((kw) => (
                    <Badge key={kw} variant="secondary">
                      {kw}
                    </Badge>
                  ))}
                  <Badge variant="outline">{skill.source}</Badge>
                </div>
                {selected?.name === skill.name && (
                  <ScrollArea className="mt-4 h-60 rounded-lg border bg-muted/40 p-4">
                    <pre className="text-xs whitespace-pre-wrap">{skill.body}</pre>
                  </ScrollArea>
                )}
              </CardContent>
            </Card>
          ))}
          {filtered.length === 0 && (
            <p className="text-muted-foreground col-span-2 py-10 text-center">
              无匹配技能
            </p>
          )}
        </div>
      )}
    </div>
  );
}
