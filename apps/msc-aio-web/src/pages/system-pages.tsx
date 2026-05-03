"use client";

import { useEffect, useState } from "react";

import type {
  DepartmentDto,
  DepartmentUpsertDto,
  DictGroupDto,
  DictGroupUpsertDto,
  DictItemDto,
  DictItemUpsertDto,
  MenuDto,
  MenuUpsertDto,
  RoleDto,
  RoleUpsertDto,
  UserDto,
  UserUpsertDto,
} from "@addzero/api-client";

import { api } from "@/lib/api";
import { Button, Callout, Card, DataTable, Field, Input, PageHeader, SectionTitle, Select } from "@/components/ui";

function useFeedback() {
  const [feedback, setFeedback] = useState<string | null>(null);
  return { feedback, setFeedback };
}

export function SystemUsersPage() {
  const { feedback, setFeedback } = useFeedback();
  const [items, setItems] = useState<UserDto[]>([]);
  const [form, setForm] = useState<UserUpsertDto>({
    username: "",
    password: "",
    nickname: "",
    status: "enabled",
  });
  const [editingId, setEditingId] = useState<number | null>(null);

  const refresh = async () => {
    try {
      setItems(await api.listUsers());
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <EntityPage
      title="用户"
      subtitle="用户管理保持独立模块，浏览器端只走 system user REST。"
      feedback={feedback}
      form={
        <div className="grid gap-4 md:grid-cols-2">
          <Field label="username">
            <Input value={form.username} onChange={(event) => setForm({ ...form, username: event.target.value })} />
          </Field>
          <Field label="nickname">
            <Input value={form.nickname} onChange={(event) => setForm({ ...form, nickname: event.target.value })} />
          </Field>
          <Field label="password">
            <Input type="password" value={form.password} onChange={(event) => setForm({ ...form, password: event.target.value })} />
          </Field>
          <Field label="status">
            <Select value={form.status} onChange={(event) => setForm({ ...form, status: event.target.value })}>
              <option value="enabled">enabled</option>
              <option value="disabled">disabled</option>
              <option value="locked">locked</option>
            </Select>
          </Field>
        </div>
      }
      onSave={async () => {
        if (editingId) {
          await api.updateUser(editingId, form);
        } else {
          await api.createUser(form);
        }
        setForm({ username: "", password: "", nickname: "", status: "enabled" });
        setEditingId(null);
        await refresh();
      }}
      table={
        <DataTable
          columns={["Username", "Nickname", "Status", "Action"]}
          rows={items.map((item) => [
            item.username,
            item.nickname,
            item.status,
            <div key={item.id} className="flex gap-2">
              <Button
                onClick={async () => {
                  const detail = await api.getUser(item.id);
                  setEditingId(item.id);
                  setForm({
                    username: detail.user.username,
                    password: "",
                    nickname: detail.user.nickname,
                    status: detail.user.status,
                  });
                }}
              >
                编辑
              </Button>
              <Button tone="danger" onClick={async () => { await api.deleteUser(item.id); await refresh(); }}>
                删除
              </Button>
            </div>,
          ])}
        />
      }
    />
  );
}

export function SystemMenusPage() {
  const { feedback, setFeedback } = useFeedback();
  const [items, setItems] = useState<MenuDto[]>([]);
  const [form, setForm] = useState<MenuUpsertDto>({
    parent_id: null,
    name: "",
    code: "",
    path: "",
    sort: 0,
    status: "enabled",
  });
  const [editingId, setEditingId] = useState<number | null>(null);

  const refresh = async () => {
    try {
      setItems(await api.listMenus());
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <EntityPage
      title="菜单"
      subtitle="菜单模型保持显式树结构，不再放在壳子里硬编码。"
      feedback={feedback}
      form={
        <div className="grid gap-4 md:grid-cols-2">
          <Field label="name">
            <Input value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} />
          </Field>
          <Field label="code">
            <Input value={form.code} onChange={(event) => setForm({ ...form, code: event.target.value })} />
          </Field>
          <Field label="path">
            <Input value={form.path} onChange={(event) => setForm({ ...form, path: event.target.value })} />
          </Field>
          <Field label="sort">
            <Input type="number" value={String(form.sort)} onChange={(event) => setForm({ ...form, sort: Number(event.target.value) })} />
          </Field>
        </div>
      }
      onSave={async () => {
        if (editingId) {
          await api.updateMenu(editingId, form);
        } else {
          await api.createMenu(form);
        }
        setForm({ parent_id: null, name: "", code: "", path: "", sort: 0, status: "enabled" });
        setEditingId(null);
        await refresh();
      }}
      table={
        <DataTable
          columns={["Name", "Code", "Path", "Sort", "Action"]}
          rows={items.map((item) => [
            item.name,
            item.code,
            item.path,
            String(item.sort),
            <div key={item.id} className="flex gap-2">
              <Button
                onClick={() => {
                  setEditingId(item.id);
                  setForm({
                    parent_id: item.parent_id,
                    name: item.name,
                    code: item.code,
                    path: item.path,
                    sort: item.sort,
                    status: item.status,
                  });
                }}
              >
                编辑
              </Button>
              <Button tone="danger" onClick={async () => { await api.deleteMenu(item.id); await refresh(); }}>
                删除
              </Button>
            </div>,
          ])}
        />
      }
    />
  );
}

export function SystemRolesPage() {
  const { feedback, setFeedback } = useFeedback();
  const [items, setItems] = useState<RoleDto[]>([]);
  const [form, setForm] = useState<RoleUpsertDto>({ name: "", code: "", status: "enabled" });
  const [editingId, setEditingId] = useState<number | null>(null);

  const refresh = async () => {
    try {
      setItems(await api.listRoles());
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <EntityPage
      title="角色"
      subtitle="角色页先保留角色 CRUD；菜单授权继续通过独立接口补齐。"
      feedback={feedback}
      form={
        <div className="grid gap-4 md:grid-cols-2">
          <Field label="name">
            <Input value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} />
          </Field>
          <Field label="code">
            <Input value={form.code} onChange={(event) => setForm({ ...form, code: event.target.value })} />
          </Field>
        </div>
      }
      onSave={async () => {
        if (editingId) {
          await api.updateRole(editingId, form);
        } else {
          await api.createRole(form);
        }
        setForm({ name: "", code: "", status: "enabled" });
        setEditingId(null);
        await refresh();
      }}
      table={
        <DataTable
          columns={["Name", "Code", "Status", "Action"]}
          rows={items.map((item) => [
            item.name,
            item.code,
            item.status,
            <div key={item.id} className="flex gap-2">
              <Button
                onClick={() => {
                  setEditingId(item.id);
                  setForm({ name: item.name, code: item.code, status: item.status });
                }}
              >
                编辑
              </Button>
              <Button tone="danger" onClick={async () => { await api.deleteRole(item.id); await refresh(); }}>
                删除
              </Button>
            </div>,
          ])}
        />
      }
    />
  );
}

export function SystemDepartmentsPage() {
  const { feedback, setFeedback } = useFeedback();
  const [items, setItems] = useState<DepartmentDto[]>([]);
  const [form, setForm] = useState<DepartmentUpsertDto>({
    parent_id: null,
    name: "",
    code: "",
    leader: "",
    sort: 0,
    status: "enabled",
  });
  const [editingId, setEditingId] = useState<number | null>(null);

  const refresh = async () => {
    try {
      setItems(await api.listDepartments());
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <EntityPage
      title="部门"
      subtitle="部门结构独立建模，支撑系统用户归属。"
      feedback={feedback}
      form={
        <div className="grid gap-4 md:grid-cols-2">
          <Field label="name">
            <Input value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} />
          </Field>
          <Field label="code">
            <Input value={form.code} onChange={(event) => setForm({ ...form, code: event.target.value })} />
          </Field>
          <Field label="leader">
            <Input
              value={form.leader}
              onChange={(event) => setForm({ ...form, leader: event.target.value })}
            />
          </Field>
          <Field label="sort">
            <Input type="number" value={String(form.sort)} onChange={(event) => setForm({ ...form, sort: Number(event.target.value) })} />
          </Field>
        </div>
      }
      onSave={async () => {
        if (editingId) {
          await api.updateDepartment(editingId, form);
        } else {
          await api.createDepartment(form);
        }
        setForm({ parent_id: null, name: "", code: "", leader: "", sort: 0, status: "enabled" });
        setEditingId(null);
        await refresh();
      }}
      table={
        <DataTable
          columns={["Name", "Code", "Leader", "Sort", "Action"]}
          rows={items.map((item) => [
            item.name,
            item.code,
            item.leader,
            String(item.sort),
            <div key={item.id} className="flex gap-2">
              <Button
                onClick={() => {
                  setEditingId(item.id);
                  setForm({
                    parent_id: item.parent_id,
                    name: item.name,
                    code: item.code,
                    leader: item.leader,
                    sort: item.sort,
                    status: item.status,
                  });
                }}
              >
                编辑
              </Button>
              <Button tone="danger" onClick={async () => { await api.deleteDepartment(item.id); await refresh(); }}>
                删除
              </Button>
            </div>,
          ])}
        />
      }
    />
  );
}

export function SystemDictionariesPage() {
  const { feedback, setFeedback } = useFeedback();
  const [groups, setGroups] = useState<DictGroupDto[]>([]);
  const [items, setItems] = useState<DictItemDto[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<number | null>(null);
  const [groupForm, setGroupForm] = useState<DictGroupUpsertDto>({ name: "", code: "", status: "enabled" });
  const [itemForm, setItemForm] = useState<DictItemUpsertDto>({ group_id: 0, label: "", value: "", sort: 0, status: "enabled" });

  const refresh = async (groupId?: number | null) => {
    try {
      const nextGroups = await api.listDictGroups();
      setGroups(nextGroups);
      const activeGroup = groupId ?? selectedGroup ?? nextGroups[0]?.id ?? null;
      setSelectedGroup(activeGroup);
      if (activeGroup) {
        setItems(await api.listDictItems(activeGroup));
        setItemForm((current) => ({ ...current, group_id: activeGroup }));
      } else {
        setItems([]);
      }
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "加载失败");
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <div className="space-y-6">
      <PageHeader title="字典管理" subtitle="字典分组与字典项拆成两个面板，保持原有系统域结构。" />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <div className="grid gap-6 xl:grid-cols-2">
        <Card>
          <SectionTitle title="字典分组" />
          <div className="grid gap-4 md:grid-cols-2">
            <Field label="name">
              <Input
                value={groupForm.name}
                onChange={(event) => setGroupForm({ ...groupForm, name: event.target.value })}
              />
            </Field>
            <Field label="code">
              <Input
                value={groupForm.code}
                onChange={(event) => setGroupForm({ ...groupForm, code: event.target.value })}
              />
            </Field>
          </div>
          <div className="mt-4">
            <Button
              tone="accent"
              onClick={async () => {
                await api.createDictGroup(groupForm);
                setGroupForm({ name: "", code: "", status: "enabled" });
                await refresh();
              }}
            >
              新建分组
            </Button>
          </div>
          <div className="mt-4 space-y-2">
            {groups.map((group) => (
              <button
                key={group.id}
                type="button"
                onClick={() => void refresh(group.id)}
                className={`w-full rounded-lg border px-3 py-3 text-left ${
                  group.id === selectedGroup
                    ? "border-emerald-400/40 bg-emerald-500/10"
                    : "border-white/10 bg-black/20"
                }`}
              >
                <div className="font-medium text-white">{group.name}</div>
                <div className="mt-1 text-xs text-zinc-500">{group.code}</div>
              </button>
            ))}
          </div>
        </Card>
        <Card>
          <SectionTitle title="字典项" detail={selectedGroup ? `group_id=${selectedGroup}` : "先选择分组"} />
          <div className="grid gap-4 md:grid-cols-2">
            <Field label="label">
              <Input
                value={itemForm.label}
                onChange={(event) => setItemForm({ ...itemForm, label: event.target.value })}
              />
            </Field>
            <Field label="value">
              <Input
                value={itemForm.value}
                onChange={(event) => setItemForm({ ...itemForm, value: event.target.value })}
              />
            </Field>
          </div>
          <div className="mt-4">
            <Button
              tone="accent"
              disabled={!selectedGroup}
              onClick={async () => {
                await api.createDictItem(itemForm);
                setItemForm((current) => ({ ...current, label: "", value: "", sort: 0 }));
                await refresh(selectedGroup);
              }}
            >
              新建字典项
            </Button>
          </div>
          <div className="mt-4">
            <DataTable
              columns={["Label", "Value", "Sort", "Action"]}
              rows={items.map((item) => [
                item.label,
                item.value,
                String(item.sort),
                <Button key={item.id} tone="danger" onClick={async () => { await api.deleteDictItem(item.id); await refresh(selectedGroup); }}>
                  删除
                </Button>,
              ])}
            />
          </div>
        </Card>
      </div>
    </div>
  );
}

function EntityPage({
  title,
  subtitle,
  feedback,
  form,
  onSave,
  table,
}: {
  title: string;
  subtitle: string;
  feedback: string | null;
  form: React.ReactNode;
  onSave: () => Promise<void>;
  table: React.ReactNode;
}) {
  return (
    <div className="space-y-6">
      <PageHeader title={title} subtitle={subtitle} />
      {feedback ? <Callout>{feedback}</Callout> : null}
      <Card>
        <SectionTitle title={`编辑 ${title}`} actions={<Button tone="accent" onClick={() => void onSave()}>保存</Button>} />
        {form}
      </Card>
      <Card>
        <SectionTitle title={`${title}列表`} />
        {table}
      </Card>
    </div>
  );
}
