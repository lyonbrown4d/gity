import { createContext, useContext, useEffect, useMemo, useState } from "react";

export type Locale = "en" | "zh";

interface I18nContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  toggleLocale: () => void;
  t: (text: string) => string;
}

const STORAGE_KEY = "gity.locale";

const zhDict: Record<string, string> = {
  "Welcome back": "欢迎回来",
  "Sign in to manage repositories and organizations.": "登录后可管理仓库与组织。",
  "Username / Email": "用户名 / 邮箱",
  Password: "密码",
  "Sign in": "登录",
  "Signing in...": "登录中...",
  "Gity Standalone": "Gity 独立版",
  "Unified workspace for auth, organization management, and repository operations.":
    "统一的认证、组织管理与仓库操作工作台。",
  "Refine Admin": "Refine 管理端",
  "Gity Console": "Gity 控制台",
  Overview: "概览",
  Organizations: "组织",
  Repositories: "仓库",
  Users: "用户",
  "Admin Side": "管理侧",
  "Refine-driven routing and auth flow": "基于 refine 的路由与认证流程",
  "Super Admin": "超级管理员",
  Logout: "退出登录",
  "User Workspace": "用户工作台",
  Workspace: "工作台",
  Dashboard: "仪表盘",
  "My Repositories": "我的仓库",
  Profile: "个人资料",
  "Admin Dashboard": "管理面板",
  "User Portal": "用户门户",
  "Switch theme": "切换主题",
  "Switch language": "切换语言",
  "Loading...": "加载中...",
  "Login failed": "登录失败",
  Organization: "组织",
  "New Repository": "新建仓库",
  "Account Status": "账号状态",
  User: "用户",
  "organization selected": "已选择组织",
  "no organization": "未选择组织",
  "Loading repositories...": "仓库加载中...",
  "User ID": "用户 ID",
  "Admin Overview": "管理概览",
  "Monitor organizations, repositories, and current control-plane baseline.":
    "监控组织、仓库以及当前控制平面基线。",
  Baseline: "基线",
  "System Composition": "系统组成",
  "Current baseline aligns route auth with role-scoped layouts and CRUD pages.":
    "当前基线已实现路由鉴权与按角色划分的 CRUD 页面。",
  "Organization Snapshot": "组织快照",
  "Quick view of organizations and role mapping.": "快速查看组织及角色映射。",
  "No organizations available.": "暂无组织。",
  "Total organizations visible to admin.": "当前管理员可见的组织总数。",
  "Repositories under the primary organization.": "主组织下的仓库数量。",
  "Organizations visible to current user": "当前用户可见组织",
  "User accounts in current admin scope.": "当前管理员范围内的用户账号。",
  "User Directory": "用户目录",
  "Identity, status, and privilege level.": "身份、状态与权限级别。",
  total: "总计",
  active: "激活",
  "No users found.": "暂无用户。",
  "New User": "新建用户",
  "Create User": "创建用户",
  "Failed to create user": "创建用户失败",
  "super-admin": "超级管理员",
  "Unknown User": "未知用户",
  "Unknown Email": "未知邮箱",
  Username: "用户名",
  Email: "邮箱",
  Cancel: "取消",
  Create: "创建",
  "Creating...": "创建中...",
  "Save Changes": "保存修改",
  "Edit Profile": "编辑个人资料",
  "Current account information and access status.": "当前账号信息与访问状态。",
  "Update username, email, and optionally change your password.":
    "更新用户名、邮箱，并可选修改密码。",
  "New Password (Optional)": "新密码（可选）",
  Save: "保存",
  "Profile updated.": "资料更新成功。",
  "My Account": "我的账号",
  "My Organizations": "我的组织",
  "Overview of your account, organizations, and repository workspace.":
    "账号、组织和仓库工作区概览。",
  "Open Repositories": "打开仓库",
  "Go Admin": "进入管理端",
  "Memberships available in your scope.": "你当前权限范围内的组织成员关系。",
  "Total repositories visible to you.": "你当前可见的仓库总数。",
  "Access role": "访问角色",
  "Identity and quick navigation.": "身份信息与快捷入口。",
  "Organizations you can operate in.": "你可操作的组织列表。",
  "Create, clone, and manage repositories in your organizations.":
    "在组织内创建、克隆并管理仓库。",
  "Repository List": "仓库列表",
  "Repositories under the selected organization.": "当前组织下的仓库列表。",
  "Select an organization to view repositories.": "请选择组织查看仓库。",
  "Clone URL": "克隆地址",
  "Copy Clone URL": "复制克隆地址",
  Delete: "删除",
  "No repositories available.": "暂无仓库。",
  repos: "个仓库",
  "N/A": "无",
  "Please select an organization first.": "请先选择一个组织。",
  "Failed to create repository": "创建仓库失败",
  "Failed to delete repository": "删除仓库失败",
  "Failed to copy clone URL": "复制克隆地址失败",
  "Delete repository \"{name}\"?": "确认删除仓库“{name}”？",
  "Delete organization \"{name}\"?": "确认删除组织“{name}”？",
  "Create Repository": "创建仓库",
  Owner: "所属者",
  "Repository key": "仓库标识",
  "Repository name": "仓库名称",
  "Repository Name": "仓库名称",
  Description: "描述",
  "Description (optional)": "描述（可选）",
  "Default branch": "默认分支",
  Visibility: "可见性",
  "Add .gitignore": "添加 .gitignore",
  "Add license": "添加 license",
  None: "不添加",
  Rust: "Rust",
  Node: "Node",
  Python: "Python",
  Go: "Go",
  Java: "Java",
  "MIT License": "MIT 许可证",
  "Apache License 2.0": "Apache 2.0 许可证",
  "GNU GPLv3": "GNU GPLv3 许可证",
  private: "私有",
  internal: "内部",
  public: "公开",
  "Admin can view and delete repositories. New repository creation is handled in user workspace.":
    "管理员可查看与删除仓库，仓库创建由用户侧完成。",
  "No repositories in this organization.": "该组织下暂无仓库。",
  "Failed to create organization": "创建组织失败",
  "Failed to update organization": "更新组织失败",
  "Failed to delete organization": "删除组织失败",
  "Failed to add member": "添加成员失败",
  "Full CRUD for organizations and member management entry.":
    "组织完整 CRUD 与成员管理入口。",
  "New Organization": "新建组织",
  "Selected Organization": "当前组织",
  "Edit Organization": "编辑组织",
  "Delete Organization": "删除组织",
  "Add Member": "添加成员",
  Members: "成员",
  "Members of the selected organization.": "当前组织的成员列表。",
  "No members found.": "暂无成员。",
  "No repositories found.": "暂无仓库。",
  "Create Organization": "创建组织",
  "Organization key": "组织标识",
  "Organization name": "组织名称",
  "Add Organization Member": "添加组织成员",
  Role: "角色",
  member: "成员",
  owner: "所有者",
  "default:": "默认分支:",
  "default branch:": "默认分支:",
  "Update failed": "更新失败",
  Platform: "平台",
  Projects: "项目",
  More: "更多",
  "View Project": "查看项目",
  "Share Project": "分享项目",
  "Delete Project": "删除项目",
  Toggle: "切换",
  "Toggle Sidebar": "切换侧边栏",
  Sidebar: "侧边栏",
  "Displays the mobile sidebar.": "显示移动端侧边栏。",
  Close: "关闭",
  "split user/admin": "用户/管理分离",
  "Adding...": "添加中...",
};

const I18nContext = createContext<I18nContextValue | null>(null);

export function resolveStoredLocale(): Locale {
  if (typeof window === "undefined") {
    return "en";
  }
  const stored = window.localStorage.getItem(STORAGE_KEY);
  if (stored === "en" || stored === "zh") {
    return stored;
  }
  return "en";
}

export function translateText(text: string, locale: Locale): string {
  return locale === "zh" ? (zhDict[text] ?? text) : text;
}

export function translateForCurrentLocale(text: string): string {
  return translateText(text, resolveStoredLocale());
}

export function I18nProvider({ children }: { children: React.ReactNode }): JSX.Element {
  const [locale, setLocaleState] = useState<Locale>(() => resolveStoredLocale());

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }
    window.localStorage.setItem(STORAGE_KEY, locale);
    document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
  }, [locale]);

  const value = useMemo<I18nContextValue>(
    () => ({
      locale,
      setLocale: (next) => {
        setLocaleState(next);
      },
      toggleLocale: () => {
        const next = locale === "en" ? "zh" : "en";
        setLocaleState(next);
      },
      t: (text) => translateText(text, locale),
    }),
    [locale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within I18nProvider");
  }
  return context;
}
