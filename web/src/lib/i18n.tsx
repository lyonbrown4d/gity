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
  Repository: "仓库",
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
  Open: "打开",
  Code: "代码",
  Commits: "提交",
  Branches: "分支",
  Settings: "设置",
  "Back to repositories": "返回仓库列表",
  "No description provided.": "暂无描述。",
  "Repository source view and clone information.": "仓库源码视图与克隆信息。",
  "Quick clone": "快速克隆",
  "File tree preview is not available yet. You can clone the repository locally.":
    "暂不支持在线文件树预览，你可以先将仓库克隆到本地。",
  Up: "返回上级",
  "Loading files...": "文件加载中...",
  "No files found.": "没有文件。",
  "README Preview": "README 预览",
  "View README source": "查看 README 源码",
  "Create file and commit": "创建文件并提交",
  "Commit and create file": "提交并创建文件",
  "Committing...": "提交中...",
  "File path": "文件路径",
  "File content": "文件内容",
  "Commit message": "提交信息",
  "Add new file": "新增文件",
  "Branch is required": "分支不能为空",
  "File path is required": "文件路径不能为空",
  "Commit message is required": "提交信息不能为空",
  "README not found. Select a source file from the left to open in Monaco.":
    "未找到 README。可从左侧选择源码文件并在 Monaco 中查看。",
  "README is binary and cannot be rendered.": "README 是二进制内容，无法渲染。",
  "This file is binary and cannot be shown in Monaco.": "该文件是二进制内容，无法在 Monaco 中展示。",
  Languages: "语言统计",
  Refresh: "刷新",
  "Loading language statistics...": "语言统计加载中...",
  "Language statistics are being computed...": "语言统计计算中...",
  "No language statistics yet.": "暂无语言统计。",
  "Total size": "总大小",
  "Last analyzed": "最近分析",
  Issues: "议题",
  "Track bugs, tasks, and discussions for this repository.": "跟踪该仓库中的缺陷、任务和讨论。",
  Status: "状态",
  "Open issues": "打开的议题",
  "Closed issues": "已关闭的议题",
  "All issues": "全部议题",
  "Search issues": "搜索议题",
  "Recently updated": "最近更新",
  "Recently created": "最近创建",
  "Newest number": "编号从新到旧",
  "Oldest number": "编号从旧到新",
  Reload: "重新加载",
  Total: "总数",
  "New issue": "新建议题",
  "Hide new issue form": "收起新建议题表单",
  "Issue title": "议题标题",
  "Describe the issue (optional)": "议题描述（可选）",
  "Assignee user ID (optional)": "指派用户 ID（可选）",
  "Creating issue...": "创建议题中...",
  "Create issue": "创建议题",
  "Loading issues...": "议题加载中...",
  "No issues found.": "暂无议题。",
  Closed: "已关闭",
  Author: "作者",
  Assignee: "指派给",
  "Close issue": "关闭议题",
  "Reopen issue": "重新打开议题",
  Comments: "评论",
  "Loading comments...": "评论加载中...",
  "No comments yet.": "暂无评论。",
  "Add a comment": "添加评论",
  "Comment with markdown, mention #123, or upload files...":
    "使用 Markdown 发表评论，可引用 #123，或上传文件…",
  "Commenting...": "评论提交中...",
  Comment: "评论",
  Discussion: "讨论",
  Back: "返回",
  "Loading issue...": "议题加载中...",
  "Invalid issue number": "议题编号无效",
  Write: "编写",
  Preview: "预览",
  "Upload files": "上传文件",
  "Uploading...": "上传中...",
  "Rendering preview...": "渲染预览中...",
  "Select an issue to view details and comments.": "选择一个议题以查看详情与评论。",
  "Issue title is required": "议题标题不能为空",
  "Comment content is required": "评论内容不能为空",
  "Recent commit activity in this repository.": "该仓库最近的提交活动。",
  Branch: "分支",
  "All branches": "所有分支",
  "Loading commits...": "提交加载中...",
  "No commits found.": "暂无提交。",
  "Manage repository branches and protections.": "管理仓库分支与保护策略。",
  "New branch name": "新分支名称",
  "Create branch": "创建分支",
  "Loading branches...": "分支加载中...",
  "Last commit": "最近提交",
  Protected: "已保护",
  Unprotected: "未保护",
  Unprotect: "取消保护",
  Protect: "保护",
  "No branches found.": "暂无分支。",
  "Repository metadata and danger zone.": "仓库元数据与危险操作。",
  "Danger zone": "危险区域",
  "Deleting a repository is irreversible.": "删除仓库后不可恢复。",
  "Repository not found in selected organization.": "在当前组织下未找到该仓库。",
  "Failed to load branches": "加载分支失败",
  "Failed to load commits": "加载提交失败",
  "Failed to create branch": "创建分支失败",
  "Failed to update branch protection": "更新分支保护失败",
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
